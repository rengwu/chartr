package terminal

import (
	"encoding/base64"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/terminal/detect"
)

// recordingsDir holds the real PTY captures taken while this map was designed.
// The engine is tested against recorded agent bytes rather than hand-written
// strings, because hand-written strings encode what we *think* an agent draws;
// these are what Claude Code actually emitted. Ticket 02 extends the same set.
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
	cols, rows := recordingGeometry(t, name)
	chunks := loadRecording(t, name)

	g := newGrid(cols, rows)
	defer g.close()

	var scanner oscScanner
	var title, progress string
	store := func(dst *string) func(string) { return func(v string) { *dst = v } }

	pub := newPublisher(time.Time{})
	var out []transition
	next := 0
	end := chunks[len(chunks)-1].at

	for now := time.Duration(0); now <= end+sampleInterval; now += sampleInterval {
		for next < len(chunks) && chunks[next].at <= now {
			scanner.scan(chunks[next].data, store(&title), store(&progress))
			g.write(chunks[next].data)
			next++
		}
		ev := detect.Evidence{Title: title, Progress: progress, Screen: g.text()}
		res := agentEngine.Evaluate(agent, ev)
		if state, changed := pub.update(res, time.Time{}.Add(now)); changed {
			out = append(out, transition{at: now, state: state, positive: res.State != ""})
		}
	}
	return out
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
