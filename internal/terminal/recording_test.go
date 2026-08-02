package terminal

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/terminal/detect"
)

// recordingsDir holds the real PTY captures of every agent in the roster. The
// engine is tested against recorded agent bytes rather than hand-written strings,
// because hand-written strings encode what we *think* an agent draws. Ticket 04 is
// the proof: three of the four manifests written from herdr's data and from the
// braille shape claude and kimi share turned out to describe signals their agents
// never emit, and every one of those bugs read as a working agent gone idle.
const recordingsDir = "../../.plan/maps/agent-state-detection/assets"

// chunk is one recorded PTY read: when it arrived, and the bytes.
type chunk struct {
	at   time.Duration
	data []byte
}

// loadRecording parses one .jsonl capture: a header line of {"cols":N,"rows":M},
// then [elapsed_seconds, "<base64>"] per PTY read, in order.
func loadRecording(t *testing.T, name string) []chunk {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join(recordingsDir, name))
	if err != nil {
		t.Fatalf("reading recording %s: %v", name, err)
	}
	lines := strings.Split(strings.TrimSpace(string(raw)), "\n")
	if len(lines) < 2 {
		t.Fatalf("recording %s has no chunks", name)
	}
	var out []chunk
	for _, line := range lines[1:] { // line 0 is the geometry header
		var rec []json.RawMessage
		if err := json.Unmarshal([]byte(line), &rec); err != nil || len(rec) != 2 {
			t.Fatalf("recording %s: bad chunk line %q: %v", name, line, err)
		}
		var secs float64
		var b64 string
		if err := json.Unmarshal(rec[0], &secs); err != nil {
			t.Fatalf("recording %s: bad timestamp: %v", name, err)
		}
		if err := json.Unmarshal(rec[1], &b64); err != nil {
			t.Fatalf("recording %s: bad payload: %v", name, err)
		}
		data, err := base64.StdEncoding.DecodeString(b64)
		if err != nil {
			t.Fatalf("recording %s: bad base64: %v", name, err)
		}
		out = append(out, chunk{at: time.Duration(secs * float64(time.Second)), data: data})
	}
	return out
}

// recordingGeometry reads the {"cols":N,"rows":M} header of a capture. The screen
// reconstruction must be sized from it — both agents lay out against the reported
// width, so replaying at a different size would not reproduce the recorded screens.
func recordingGeometry(t *testing.T, name string) (cols, rows int) {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join(recordingsDir, name))
	if err != nil {
		t.Fatalf("reading recording %s: %v", name, err)
	}
	line0 := strings.SplitN(strings.TrimSpace(string(raw)), "\n", 2)[0]
	var hdr struct {
		Cols int `json:"cols"`
		Rows int `json:"rows"`
	}
	if err := json.Unmarshal([]byte(line0), &hdr); err != nil || hdr.Cols <= 0 || hdr.Rows <= 0 {
		t.Fatalf("recording %s: bad geometry header %q: %v", name, line0, err)
	}
	return hdr.Cols, hdr.Rows
}

// transition is one published change while replaying a recording. positive records
// whether the engine matched a rule for the sample that published it, as opposed
// to publishing off a confirmed absence — the two halves of the hysteresis.
type transition struct {
	at       time.Duration
	state    string
	positive bool
}

// replay drives a recording through the real read-loop scanner, the real rule
// engine and the real hysteresis, sampling on the manager's own cadence against
// the recording's clock. It returns every state the tab would have published.
//
// This is the whole detection path end to end, minus only the PTY itself: the
// bytes are the ones an agent really wrote.
func replay(t *testing.T, agent string, chunks []chunk) []transition {
	t.Helper()
	eng := agentEngine

	var scanner oscScanner
	var title, progress string
	store := func(dst *string) func(string) { return func(v string) { *dst = v } }

	pub := newPublisher(time.Time{}) // the recording's clock starts at zero
	var out []transition
	next := 0
	end := chunks[len(chunks)-1].at

	for now := time.Duration(0); now <= end+sampleInterval; now += sampleInterval {
		// Feed everything the PTY would have delivered by now.
		for next < len(chunks) && chunks[next].at <= now {
			scanner.scan(chunks[next].data, store(&title), store(&progress))
			next++
		}
		res := eng.Evaluate(agent, detect.Evidence{Title: title, Progress: progress})
		// The publisher's clock is the recording's, so the startup grace is measured
		// in recorded time rather than wall time.
		if state, changed := pub.update(res, time.Time{}.Add(now)); changed {
			out = append(out, transition{at: now, state: state, positive: res.State != ""})
		}
	}
	return out
}

// replayScreen is replay's screen-aware sibling: it additionally feeds every chunk
// through the real grid emulator and hands the reconstructed screen to the engine
// alongside the OSC evidence. This is the whole ticket-02 detection path end to end
// — the scanner, the grid, the rule engine's screen regions, and the hysteresis —
// driven by the bytes an agent really wrote, at the size it wrote them.
func replayScreen(t *testing.T, agent, name string) []transition {
	t.Helper()
	return replayScreenTail(t, agent, name, 0)
}

// replayScreenTail is replayScreen with sampling that keeps going for tail past the
// last recorded byte. An agent that finishes its turn and falls quiet writes its
// final idle title and then stops emitting entirely, so the absence confirmation
// that publishes idle needs ticks the recording itself no longer supplies — real
// sampling does not stop when an agent stops typing. Claude's capture ends mid-
// stream and needs none; the agents captured in ticket 04 end at a settled prompt
// and do.
func replayScreenTail(t *testing.T, agent, name string, tail time.Duration) []transition {
	t.Helper()
	var out []transition
	replayPublished(t, agent, name, tail, func(now time.Duration, tr transition, changed bool) {
		if changed {
			out = append(out, tr)
		}
	})
	return out
}

// replayPublished is replayScreen's engine: it drives one recording through the
// whole path and calls visit for *every* sample tick, not only the ones that
// changed the published state. replayScreen keeps the changes; the notification
// clock (clock_test.go) needs the full per-tick history, because its rule is about
// how long a state persisted rather than when it moved.
//
// tail extends the sampling past the last recorded chunk with the evidence the
// recording ended on. Real sampling does not stop when an agent stops writing
// bytes, and a settle delay measured after the last byte needs those ticks.
func replayPublished(t *testing.T, agent, name string, tail time.Duration, visit func(now time.Duration, tr transition, changed bool)) {
	t.Helper()
	cols, rows := recordingGeometry(t, name)
	chunks := loadRecording(t, name)

	g := newGrid(cols, rows)
	defer g.close()

	var scanner oscScanner
	var title, progress string
	store := func(dst *string) func(string) { return func(v string) { *dst = v } }

	pub := newPublisher(time.Time{})
	next := 0
	end := chunks[len(chunks)-1].at

	for now := time.Duration(0); now <= end+tail+sampleInterval; now += sampleInterval {
		for next < len(chunks) && chunks[next].at <= now {
			scanner.scan(chunks[next].data, store(&title), store(&progress))
			g.write(chunks[next].data)
			next++
		}
		ev := detect.Evidence{Title: title, Progress: progress, Screen: g.text()}
		res := agentEngine.Evaluate(agent, ev)
		state, changed := pub.update(res, time.Time{}.Add(now))
		visit(now, transition{at: now, state: state, positive: res.State != ""}, changed)
	}
}

func sawState(trs []transition, state string) bool {
	for _, tr := range trs {
		if tr.state == state {
			return true
		}
	}
	return false
}

// Kimi reads its whole grammar off the screen: it writes nothing to its title, so
// working (the ⠋ thinking spinner) and blocked (the ▶ Run this command? approval
// panel) both come from the reconstructed grid. Replaying the real 319-second turn
// must surface both — the states the map says the screen is where they arrive.
func TestKimiRecordingReadsWorkingAndBlockedFromScreen(t *testing.T) {
	got := replayScreen(t, "kimi", "rec-kimi-0.29.0.jsonl")
	if len(got) == 0 {
		t.Fatal("replaying the Kimi recording with the screen published nothing at all")
	}
	if !sawState(got, model.TerminalWorking) {
		t.Errorf("never read working across the recorded turn; the ⠋ spinner did not fire. published %v", got)
	}
	if !sawState(got, model.TerminalBlocked) {
		t.Errorf("never read blocked; the ▶ Run this command? approval panel did not fire. published %v", got)
	}
	// The blocked panel is a real, discrete event, not a permanent state: kimi must
	// leave it again (it approved the command and kept working). So blocked is not
	// the last thing published.
	if last := got[len(got)-1]; last.state == model.TerminalBlocked {
		t.Errorf("settled on blocked at the end of the recording; the panel should have cleared")
	}
}

// Kimi's status bar reads "K2.7 Coding thinking  ~" on every single screen — the
// trap the region-and-anchor design exists to defuse. A screen carrying the status
// bar and the idle prompt box but *no* live spinner must not read as working; only
// a braille frame at the head of a line does. This is the regression the ticket
// names, pinned against a line lifted from the real capture.
func TestKimiStatusBarThinkingIsNotWorking(t *testing.T) {
	// The always-present status bar plus a rounded (cornered, non-flat-ruled) idle
	// input box — exactly what kimi draws while waiting, minus any spinner.
	idle := strings.Join([]string{
		"╭─────────────────────────────────────╮",
		"│ >                                   │",
		"╰─────────────────────────────────────╯",
		"K2.7 Coding thinking  ~                          /model: switch model",
		"                                       context: 15% (37.7k/256k)",
	}, "\n")
	if res := agentEngine.Evaluate("kimi", detect.Evidence{Screen: idle}); res.State != "" {
		t.Errorf("kimi idle screen with the ever-present 'thinking' status bar read as %q; want no state", res.State)
	}
}

// The reported bug, gone — read off real Claude Code bytes. The capture is an idle
// prompt, a turn, and a permission prompt left on screen; replaying it must show
// the tab working during the turn and idle at the prompt, rather than pinned to
// "working" for the agent's whole life the way the process-liveness proxy did.
func TestClaudeRecordingReadsWorkingThenIdle(t *testing.T) {
	got := replay(t, "claude", loadRecording(t, "rec-claude.jsonl"))
	if len(got) == 0 {
		t.Fatal("replaying the Claude recording published nothing at all")
	}

	var sawWorking, sawIdle bool
	for _, tr := range got {
		switch tr.state {
		case model.TerminalWorking:
			sawWorking = true
		case model.TerminalIdle:
			sawIdle = true
		}
	}
	if !sawWorking {
		t.Errorf("never read working across the recorded turn; published %v", got)
	}
	if !sawIdle {
		t.Errorf("never read idle at the recorded prompt; published %v", got)
	}

	// The turn ends with the agent back at its prompt, so the tab settles idle.
	if last := got[len(got)-1]; last.state != model.TerminalIdle {
		t.Errorf("settled on %q at the end of the recording, want %q", last.state, model.TerminalIdle)
	}

	// The boot is the flicker risk: Claude emits no title at all for its first
	// seconds, and a tab must not fall idle on that silence while an agent comes up.
	// Any idle published inside the startup grace has to be one Claude *announced* —
	// a ✳ in the title — never one inferred from an absence of evidence.
	for _, tr := range got {
		if tr.at < agentStartupGrace && tr.state == model.TerminalIdle && !tr.positive {
			t.Errorf("published an absence-derived idle at %s, inside the %s startup grace",
				tr.at, agentStartupGrace)
		}
	}
}

func TestClaudeRecordingPromptGlyphDoesNotOpenC1OSC(t *testing.T) {
	prompt := []byte("❯") // E2 9D AF: the middle byte is the C1 OSC value.
	for _, c := range loadRecording(t, "rec-claude.jsonl") {
		if start := bytes.Index(c.data, prompt); start >= 0 {
			var scanner oscScanner
			scanner.scan(c.data[:start+len(prompt)], func(string) {}, func(string) {})
			if scanner.state != oscGround {
				t.Fatalf("state after recorded Claude prompt glyph = %v, want oscGround", scanner.state)
			}
			return
		}
	}
	t.Fatal("recorded Claude turn contains no prompt glyph")
}

// Claude's blocked never reaches its title — a permission prompt paints ✳, byte-
// identical to idle — so it is the screen that has to carry it. Replaying the real
// capture with the grid, claude must read blocked while it sits on the Bash
// permission dialog and leave it once the turn moves on, which is the state the
// title alone could never see. This is the finding ticket 01 flagged, resolved by
// ticket 02's screen evidence exactly where it said it would be.
func TestClaudeRecordingReadsBlockedFromScreen(t *testing.T) {
	got := replayScreen(t, "claude", "rec-claude.jsonl")
	if len(got) == 0 {
		t.Fatal("replaying the Claude recording with the screen published nothing at all")
	}
	if !sawState(got, model.TerminalWorking) {
		t.Errorf("never read working across the recorded turn; published %v", got)
	}
	if !sawState(got, model.TerminalBlocked) {
		t.Errorf("never read blocked while sitting on the permission dialog; published %v", got)
	}
	if !sawState(got, model.TerminalIdle) {
		t.Errorf("never read idle at the recorded prompt; published %v", got)
	}
	// blocked is a discrete event, not the resting state: the capture ends past the
	// dialog, so it must not be the last thing published.
	if last := got[len(got)-1]; last.state == model.TerminalBlocked {
		t.Errorf("settled on blocked at the end of the recording, want the dialog cleared; published %v", got)
	}
	// blocked must come after working (the turn ran, then the dialog appeared), and a
	// blocked reading must never be an absence-derived guess — the screen positively
	// showed the dialog.
	for _, tr := range got {
		if tr.state == model.TerminalBlocked && !tr.positive {
			t.Errorf("published an absence-derived blocked at %s — blocked must be a positive screen match", tr.at)
		}
	}
}

// No tab flickers on a normal turn. Claude rewrites its title about once a second
// for the whole turn; if any of that reached the sidebar the indicator would
// strobe. What actually reaches it is one transition per real change of state.
func TestClaudeRecordingDoesNotFlicker(t *testing.T) {
	chunks := loadRecording(t, "rec-claude.jsonl")
	got := replay(t, "claude", chunks)

	// A frame-by-frame indicator would publish dozens of times across an 89-second
	// capture whose title updates every second. A calm one publishes a handful.
	const calm = 10
	if len(got) > calm {
		t.Errorf("published %d transitions across the recording (%v); want at most %d — the indicator is strobing",
			len(got), got, calm)
	}

	// And no two consecutive transitions may carry the same state: publish-on-change
	// is the contract sampleShell already held.
	for i := 1; i < len(got); i++ {
		if got[i].state == got[i-1].state {
			t.Errorf("published %q twice in a row at %s and %s", got[i].state, got[i-1].at, got[i].at)
		}
	}
}

// Codex is the bug ticket 04 was opened on: its shipped working rule looked for
// "Working" / "Thinking" / "Running" in the title, and the capture proves codex
// writes none of them — it prefixes a braille frame to the directory name instead,
// so the rule never fired and a working codex fell through to idle. Replaying the
// real capture must now show the full grammar: working during the turn, blocked on
// the "Would you like to run the following command?" approval (the one roster title
// that says "Action Required" outright), and idle at the prompt either side.
func TestCodexRecordingReadsWorkingBlockedAndIdle(t *testing.T) {
	got := replayScreenTail(t, "codex", "rec-codex-0.146.0.jsonl", 5*time.Second)
	if len(got) == 0 {
		t.Fatal("replaying the Codex recording published nothing at all")
	}
	if !sawState(got, model.TerminalWorking) {
		t.Errorf("never read working across the recorded turn; the braille title frame did not fire. published %v", got)
	}
	if !sawState(got, model.TerminalBlocked) {
		t.Errorf("never read blocked while sitting on the approval; published %v", got)
	}
	if !sawState(got, model.TerminalIdle) {
		t.Errorf("never read idle at the recorded prompt; published %v", got)
	}
	// The capture approves the command and runs on past it, so blocked is a discrete
	// event rather than where the tab settles.
	if last := got[len(got)-1]; last.state != model.TerminalIdle {
		t.Errorf("settled on %q at the end of the recording, want %q; published %v",
			last.state, model.TerminalIdle, got)
	}
	// blocked comes from a title codex positively broadcast, never from an absence.
	for _, tr := range got {
		if tr.state == model.TerminalBlocked && !tr.positive {
			t.Errorf("published an absence-derived blocked at %s — codex announces it in the title", tr.at)
		}
	}
}

// The dead rule, pinned. Codex's status line does read "Working (2s • esc to
// interrupt)", but it draws that on the *screen*; the title it broadcasts is the
// directory name with a braille frame. Asserting the old strings resolve to nothing
// keeps the next reader from "restoring" them from herdr's manifest.
func TestCodexTitleNeverCarriesHerdrsWorkingStrings(t *testing.T) {
	for _, title := range []string{"Working", "Thinking", "Running"} {
		if res := agentEngine.Evaluate("codex", detect.Evidence{Title: title}); res.State == model.TerminalWorking {
			t.Errorf("codex title %q read as working; the capture shows codex never writes it", title)
		}
	}
	if res := agentEngine.Evaluate("codex", detect.Evidence{Title: "⠹ agentcwd"}); res.State != model.TerminalWorking {
		t.Errorf("codex braille title read as %q, want %q", res.State, model.TerminalWorking)
	}
	if res := agentEngine.Evaluate("codex", detect.Evidence{Title: "agentcwd"}); res.State != "" {
		t.Errorf("codex idle title read as %q, want no state (idle by absence)", res.State)
	}
}

// opencode reads its whole grammar off the screen: its title is written twice a
// session and names the conversation, never the state. The capture stops on the
// "△ Permission required" panel, is approved, and runs to completion, so replaying
// it must surface blocked, working and idle from the grid alone.
func TestOpencodeRecordingReadsBlockedWorkingAndIdleFromScreen(t *testing.T) {
	got := replayScreenTail(t, "opencode", "rec-opencode-1.2.27.jsonl", 5*time.Second)
	if len(got) == 0 {
		t.Fatal("replaying the opencode recording published nothing at all")
	}
	if !sawState(got, model.TerminalBlocked) {
		t.Errorf("never read blocked; the △ Permission required panel did not fire. published %v", got)
	}
	if !sawState(got, model.TerminalWorking) {
		t.Errorf("never read working; the square-cell spinner did not fire. published %v", got)
	}
	if !sawState(got, model.TerminalIdle) {
		t.Errorf("never read idle at the recorded prompt; published %v", got)
	}
	if last := got[len(got)-1]; last.state != model.TerminalIdle {
		t.Errorf("settled on %q at the end of the recording, want %q; published %v",
			last.state, model.TerminalIdle, got)
	}
	for _, tr := range got {
		if tr.state == model.TerminalBlocked && !tr.positive {
			t.Errorf("published an absence-derived blocked at %s — the panel is a positive screen match", tr.at)
		}
	}
}

// opencode's spinner is squares, not braille — the extrapolation ticket 04 caught.
// Pinning both halves keeps a future reader from "restoring" the braille pattern
// from the shape claude and kimi share, and keeps the "esc interrupt" half of the
// working rule load-bearing rather than decorative.
func TestOpencodeSpinnerIsSquaresNotBraille(t *testing.T) {
	const footer = "  ctrl+t variants  tab agents  ctrl+p commands    • OpenCode 1.2.27"
	cases := []struct {
		name  string
		line  string
		state string
	}{
		{"squares with the interrupt hint", "   ⬝⬝⬝⬝⬝⬝⬝⬝  esc interrupt" + footer, model.TerminalWorking},
		{"half-filled squares", "   ⬝⬝⬝■■■■■  esc interrupt" + footer, model.TerminalWorking},
		{"braille, which opencode never draws", "   ⠋⠙⠹  esc interrupt" + footer, ""},
		{"the same line once the turn ends", "  " + footer, ""},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			res := agentEngine.Evaluate("opencode", detect.Evidence{Screen: tc.line})
			if res.State != tc.state {
				t.Errorf("read %q, want %q", res.State, tc.state)
			}
		})
	}
}

// Grok is the roster agent the map had written off as unmeasurable. Replaying its
// real capture must show the whole grammar off the title alone: working while the
// braille frame leads it, blocked while "⚠ Action Required" does, and idle at the
// prompt either side.
func TestGrokRecordingReadsWorkingBlockedAndIdle(t *testing.T) {
	got := replayScreenTail(t, "grok", "rec-grok-0.2.118.jsonl", 5*time.Second)
	if len(got) == 0 {
		t.Fatal("replaying the Grok recording published nothing at all")
	}
	if !sawState(got, model.TerminalWorking) {
		t.Errorf("never read working across the recorded turn; published %v", got)
	}
	if !sawState(got, model.TerminalBlocked) {
		t.Errorf("never read blocked while sitting on the approval panel; published %v", got)
	}
	if !sawState(got, model.TerminalIdle) {
		t.Errorf("never read idle at the recorded prompt; published %v", got)
	}
	if last := got[len(got)-1]; last.state != model.TerminalIdle {
		t.Errorf("settled on %q at the end of the recording, want %q; published %v",
			last.state, model.TerminalIdle, got)
	}
}

// Grok emits no OSC 9;4 progress at all — herdr's claim, carried into this manifest
// unverified, is simply false here. The capture is the evidence, and this pins it:
// were grok to start pulsing progress, this fails and the manifest is worth
// revisiting. It also pins the ordering the capture forces — grok keeps spinning
// while it waits, so a title carrying *both* a braille frame and "Action Required"
// must read blocked, not working.
func TestGrokEmitsNoProgressAndBlockedOutranksItsSpinner(t *testing.T) {
	var scanner oscScanner
	var progress string
	for _, c := range loadRecording(t, "rec-grok-0.2.118.jsonl") {
		scanner.scan(c.data, func(string) {}, func(v string) { progress = v })
	}
	if progress != "" {
		t.Errorf("Grok emitted OSC progress %q; the manifest records that it emits none", progress)
	}

	const blocked = "⚠ Action Required - ⠙ - Remove probe file as requested… - grok"
	if res := agentEngine.Evaluate("grok", detect.Evidence{Title: blocked}); res.State != model.TerminalBlocked {
		t.Errorf("grok title %q read as %q, want %q — the spinner must not outrank Action Required",
			blocked, res.State, model.TerminalBlocked)
	}
	if res := agentEngine.Evaluate("grok", detect.Evidence{Title: "⠼ - Thinking - grok"}); res.State != model.TerminalWorking {
		t.Errorf("grok working title read as %q, want %q", res.State, model.TerminalWorking)
	}
	if res := agentEngine.Evaluate("grok", detect.Evidence{Title: "Run Exact Shell rm Temp Probe File - grok"}); res.State != "" {
		t.Errorf("grok idle title read as %q, want no state (idle by absence)", res.State)
	}
}

// pi reads its grammar off the screen — its title is written once a session and
// says "π - <dirname>" forever. The capture is a working turn driving four bash
// commands and the settle back to the prompt; there is no blocked to assert,
// because pi ships no approval gate to stop at (see pi.toml).
func TestPiRecordingReadsWorkingThenIdleFromScreen(t *testing.T) {
	got := replayScreenTail(t, "pi", "rec-pi-0.78.0.jsonl", 5*time.Second)
	if len(got) == 0 {
		t.Fatal("replaying the pi recording published nothing at all")
	}
	if !sawState(got, model.TerminalWorking) {
		t.Errorf("never read working across the recorded turn; the ⠇ Working... spinner did not fire. published %v", got)
	}
	if !sawState(got, model.TerminalIdle) {
		t.Errorf("never read idle at the recorded prompt; published %v", got)
	}
	if last := got[len(got)-1]; last.state != model.TerminalIdle {
		t.Errorf("settled on %q at the end of the recording, want %q; published %v",
			last.state, model.TerminalIdle, got)
	}
	// pi's title says nothing, so every one of those states came off the grid.
	var scanner oscScanner
	titles := map[string]bool{}
	for _, c := range loadRecording(t, "rec-pi-0.78.0.jsonl") {
		scanner.scan(c.data, func(v string) { titles[v] = true }, func(string) {})
	}
	for got := range titles {
		if res := agentEngine.Evaluate("pi", detect.Evidence{Title: got}); res.State != "" {
			t.Errorf("pi title %q resolved to %q; pi's grammar is screen-only", got, res.State)
		}
	}
}

// The region, not the pattern, was pi's real defect. Its spinner sits five non-empty
// lines above the foot with an empty input box and six with one line typed into it —
// the last slot bottom_non_empty_lines(6) could see. An operator typing a two-line
// message while pi worked would have pushed the spinner out of the region entirely
// and a working pi would have read as idle, with no second rule to catch it. This
// pins the headroom the widened region buys.
func TestPiWorkingSurvivesATypedInputBox(t *testing.T) {
	const spinner = " ⠇ Working..."
	const rule = "─────────────────────────────────────────────"
	foot := []string{
		rule,
		"/private/tmp/scratchpad/agentcwd (main)",
		"↑3.1k ↓688 R4.1k $0.006 (sub) 0.8%/272k (auto)",
	}
	for _, typed := range []int{0, 1, 2, 3} {
		t.Run(fmt.Sprintf("%d typed lines", typed), func(t *testing.T) {
			lines := []string{"Took 0.0s", spinner, rule}
			for i := 0; i < typed; i++ {
				lines = append(lines, fmt.Sprintf("a follow-up line %d typed while pi works", i))
			}
			lines = append(lines, foot...)
			res := agentEngine.Evaluate("pi", detect.Evidence{Screen: strings.Join(lines, "\n")})
			if res.State != model.TerminalWorking {
				t.Errorf("read %q with %d lines in the input box, want %q — the spinner fell out of the region",
					res.State, typed, model.TerminalWorking)
			}
		})
	}
}

// Kimi signals nothing in its title — two title writes for a whole 319-second
// session — which is exactly why it gets no manifest in this ticket and is ticket
// 02's, on screen evidence. Asserted here so the claim stays true: were Kimi to
// start broadcasting state, this test fails and the manifest becomes worth writing.
//
// It also stands in for the OSC 8 flood: Kimi emits ~1000 hyperlink sequences a
// turn, and replaying the real capture through the scanner must not turn any of
// them into evidence.
func TestKimiRecordingCarriesNoTitleState(t *testing.T) {
	chunks := loadRecording(t, "rec-kimi-0.29.0.jsonl")

	var scanner oscScanner
	var title, progress string
	titles := map[string]bool{}
	for _, c := range chunks {
		scanner.scan(c.data, func(v string) { title = v; titles[v] = true }, func(v string) { progress = v })
	}

	if progress != "" {
		t.Errorf("Kimi emitted OSC progress %q; the map records that it signals nothing", progress)
	}
	// Whatever titles it wrote, none of them mean anything to the shipped rules.
	for got := range titles {
		if res := agentEngine.Evaluate("kimi", detect.Evidence{Title: got}); res.State != "" {
			t.Errorf("Kimi title %q resolved to state %q; kimi ships no manifest in this ticket", got, res.State)
		}
	}
	t.Logf("Kimi wrote %d distinct titles across the capture (last %q)", len(titles), title)
}
