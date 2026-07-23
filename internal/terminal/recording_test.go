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
const recordingsDir = "../../.plan/agent-state-detection/assets"

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
