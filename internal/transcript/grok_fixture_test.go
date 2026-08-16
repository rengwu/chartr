package transcript

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/proc"
)

// Synthetic Grok fixtures: a state root, a percent-encoded working-directory
// bucket, and one session directory with its summary, its update log, and the
// raw model wire beside it that must never be read.
//
// Every byte here was written for this test. Nothing is copied from a real
// session — no personal conversation, no credentials, no reasoning, no real tool
// output.

const (
	// grokFixtureFormat and grokFixtureVersion record what these fixtures
	// represent: Grok's append-only updates.jsonl of ACP session-update events
	// plus the rewritten summary.json beside it, chat_format_version 1, as
	// written by grok 1.0.0.
	grokFixtureFormat  = "grok updates.jsonl (chat_format_version 1) + summary.json"
	grokFixtureVersion = "1.0.0"
)

type grokStore struct {
	t       *testing.T
	root    string
	dir     string
	pid     int
	started time.Time
	id      string

	session string
	file    string
	summary string
	partial string
	seq     int
}

func newGrokStore(t *testing.T) contractStore { return newGrokFixture(t) }

func newGrokFixture(t *testing.T) *grokStore {
	t.Helper()
	base := t.TempDir()
	s := &grokStore{
		t:       t,
		root:    filepath.Join(base, "state"),
		dir:     filepath.Join(base, "work"),
		pid:     8181,
		started: time.Now().Add(-time.Minute),
		id:      "019fc31f-2f33-77d0-bf18-66040f754f98",
	}
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		t.Fatalf("work directory: %v", err)
	}
	s.seat()
	return s
}

// seat points the fixture at the paths its session lives under. The bucket is a
// percent-encoding of the absolute working directory; the adapter confirms a
// candidate against the summary's own cwd rather than trusting the name.
func (s *grokStore) seat() {
	bucket := strings.ReplaceAll(s.dir, "/", "%2F")
	s.session = filepath.Join(s.root, grokSessions, bucket, s.id)
	s.file = filepath.Join(s.session, grokUpdates)
	s.summary = filepath.Join(s.session, grokSummary)
}

func (s *grokStore) Format() string { return grokFixtureFormat + " " + grokFixtureVersion }

func (s *grokStore) Agent() proc.Agent {
	return proc.Agent{
		Adapter:   "grok",
		PID:       s.pid,
		PGID:      s.pid,
		Started:   s.started,
		Dir:       s.dir,
		StateRoot: s.root,
	}
}

// Peer is a second Grok tab in the same space. Grok has no process-to-session
// registry — the active_sessions.json that looks like one is a leftover from an
// earlier build — so two sessions in one working directory are indistinguishable
// and both tabs stay untitled.
func (s *grokStore) Peer() (contractStore, bool) {
	peer := &grokStore{
		t: s.t, root: s.root, dir: s.dir,
		pid: s.pid + 1, started: s.started, id: "019fc31f-9999-77d0-bf18-66040f754f98",
	}
	peer.seat()
	return peer, false
}

// Start opens the session the way the provider does: the summary with no
// generated title yet, and an update log carrying the session's opening
// bookkeeping and no conversation.
func (s *grokStore) Start() {
	s.writeSummary(nil)
	s.append(s.line(map[string]any{
		"timestamp": s.stamp(), "method": "session/update",
		"params": map[string]any{
			"sessionId": s.id,
			"update":    map[string]any{"sessionUpdate": "available_commands_update", "availableCommands": []any{}},
		},
	}))
}

func (s *grokStore) Turn(prompt, response string) { s.append(s.records(prompt, response, true)...) }

func (s *grokStore) PartialTurn(prompt, response string) {
	records := s.records(prompt, response, true)
	last := records[len(records)-1]
	cut := len(last) / 2
	s.appendRaw(strings.Join(records[:len(records)-1], "") + last[:cut])
	s.partial = last[cut:]
}

func (s *grokStore) Complete() {
	s.appendRaw(s.partial)
	s.partial = ""
}

// Title rewrites the summary with a generated title, which is what Grok's own
// titling — and an operator's /rename — both produce.
func (s *grokStore) Title(title string) bool {
	s.writeSummary(&title)
	return true
}

// Ignored writes the machinery: hidden reasoning, tool traffic, plan and mode
// updates, and a vendor-extension update of a kind this adapter does not read.
func (s *grokStore) Ignored() {
	s.append(
		s.update("agent_thought_chunk", map[string]any{
			"content": map[string]any{"type": "text", "text": "invented reasoning the operator never sees"},
		}, ""),
		s.update("tool_call", map[string]any{
			"toolCallId": s.next(), "title": "read", "status": "pending",
			"rawInput": map[string]any{"path": "/invented/path"},
		}, ""),
		s.update("tool_call_update", map[string]any{
			"toolCallId": s.next(), "status": "completed",
			"content": []map[string]any{{"type": "content", "content": map[string]any{
				"type": "text", "text": "invented tool output",
			}}},
		}, ""),
		s.update("plan", map[string]any{"entries": []any{}}, ""),
		s.update("current_mode_update", map[string]any{"currentModeId": "default"}, ""),
		s.update("token_usage", map[string]any{"totalTokens": 1}, "_x.ai/session/update"),
	)
}

// SyntheticUser writes the user-shaped material Grok keeps outside the update
// log: the raw model wire, which carries the system prompt and every user-role
// envelope, and the telemetry stream. Both live in this session's own directory,
// and neither may become a turn. An answer and a completion follow in the update
// log, so if the adapter had read a prompt out of either file, this is where it
// would show.
func (s *grokStore) SyntheticUser(text string) {
	s.aside("chat_history.jsonl",
		s.line(map[string]any{"role": "system", "content": "invented system instructions"}),
		s.line(map[string]any{"role": "user", "content": text}),
		s.line(map[string]any{"role": "assistant", "content": "an answer to nobody"}),
	)
	s.aside("events.jsonl",
		s.line(map[string]any{"event": "turn_started", "timestamp": s.stamp()}),
		s.line(map[string]any{"event": "permission_requested", "timestamp": s.stamp()}),
	)
	s.append(
		s.update("agent_message_chunk", map[string]any{
			"content": map[string]any{"type": "text", "text": "an answer to nobody"},
		}, ""),
		s.update("turn_completed", map[string]any{
			"prompt_id": s.next(), "stop_reason": "end_turn",
		}, "_x.ai/session/update"),
	)
}

func (s *grokStore) Malformed() {
	s.appendRaw(`{"method":"session/update","params":{"update":{"sessionUpdate":` + "\n")
}

// Drift writes an update of a kind the adapter interprets, in a shape it has
// never seen: a message chunk whose content is a bare string rather than the
// typed object every chunk carries.
func (s *grokStore) Drift() {
	s.append(s.update("user_message_chunk", map[string]any{
		"content": "a shape from a later version",
	}, ""))
}

// Replace rewrites the update log in place, shorter than it was, keeping a turn
// of history so that a reader which re-seated naively would emit it.
func (s *grokStore) Replace() {
	before := s.size()
	records := s.records("a question from before the rewrite", "an answer from before the rewrite", false)
	if err := os.WriteFile(s.file, []byte(strings.Join(records, "")), 0o600); err != nil {
		s.t.Fatalf("replace store: %v", err)
	}
	if after := s.size(); after >= before {
		s.t.Fatalf("replacement fixture is %d bytes, not shorter than the %d it replaced", after, before)
	}
}

func (s *grokStore) Remove() {
	if err := os.Remove(s.file); err != nil && !os.IsNotExist(err) {
		s.t.Fatalf("remove store: %v", err)
	}
}

// records renders one complete turn. Both sides arrive in chunks, exactly as the
// provider streams them, so a reader that took only one chunk would show half a
// sentence.
func (s *grokStore) records(prompt, response string, machinery bool) []string {
	index := s.seq

	var out []string
	for _, chunk := range halves(prompt) {
		out = append(out, s.update("user_message_chunk", map[string]any{
			"content": map[string]any{"type": "text", "text": chunk},
			"_meta":   map[string]any{"modelId": "grok-fixture-1", "promptIndex": index},
		}, ""))
	}
	if machinery {
		out = append(out,
			s.update("agent_thought_chunk", map[string]any{
				"content": map[string]any{"type": "text", "text": "invented reasoning the operator never sees"},
			}, ""),
			s.update("tool_call", map[string]any{
				"toolCallId": s.next(), "title": "read", "status": "pending",
			}, ""),
			s.update("tool_call_update", map[string]any{
				"toolCallId": s.next(), "status": "completed",
			}, ""),
		)
	}
	for _, chunk := range halves(response) {
		out = append(out, s.update("agent_message_chunk", map[string]any{
			"content": map[string]any{"type": "text", "text": chunk},
		}, ""))
	}
	return append(out, s.update("turn_completed", map[string]any{
		"prompt_id": s.next(), "stop_reason": "end_turn",
		"usage": map[string]any{"totalTokens": 1},
	}, "_x.ai/session/update"))
}

// halves splits text into two chunks on a rune boundary, so a multibyte glyph is
// never cut in half by the fixture itself.
func halves(text string) []string {
	r := []rune(text)
	if len(r) < 2 {
		return []string{text}
	}
	return []string{string(r[:len(r)/2]), string(r[len(r)/2:])}
}

// update renders one session-update record. method names the vendor extension
// for the updates that arrive on it; the ACP method is the default.
func (s *grokStore) update(kind string, fields map[string]any, method string) string {
	if method == "" {
		method = "session/update"
	}
	update := map[string]any{"sessionUpdate": kind}
	for k, v := range fields {
		update[k] = v
	}
	return s.line(map[string]any{
		"timestamp": s.stamp(), "method": method,
		"params": map[string]any{
			"sessionId": s.id,
			"update":    update,
			"_meta":     map[string]any{"eventId": s.next()},
		},
	})
}

// writeSummary rewrites summary.json. A nil title is the shape before Grok has
// generated one: the key is absent rather than empty.
func (s *grokStore) writeSummary(title *string) {
	s.t.Helper()
	if err := os.MkdirAll(s.session, 0o755); err != nil {
		s.t.Fatalf("session directory: %v", err)
	}
	summary := map[string]any{
		"info":                map[string]any{"id": s.id, "cwd": s.dir},
		"created_at":          time.Now().UTC().Format(time.RFC3339Nano),
		"updated_at":          time.Now().UTC().Format(time.RFC3339Nano),
		"num_chat_messages":   2,
		"current_model_id":    "grok-fixture-1",
		"chat_format_version": grokFormat,
		"grok_home":           s.root,
	}
	if title != nil {
		summary["generated_title"] = *title
		summary["session_summary"] = *title
	}
	data, err := json.Marshal(summary)
	if err != nil {
		s.t.Fatalf("marshal summary: %v", err)
	}
	if err := os.WriteFile(s.summary, data, 0o600); err != nil {
		s.t.Fatalf("write summary: %v", err)
	}
	// A sidecar rewritten within the same filesystem timestamp tick must still
	// read as a change, so the poll that re-checks it sees the new title.
	s.seq++
	touch(s.t, s.summary, time.Now().Add(time.Duration(s.seq)*time.Second))
}

// aside writes one of the files that sit beside the update log and must never be
// read: the raw model wire, and the telemetry stream.
func (s *grokStore) aside(name string, records ...string) {
	s.t.Helper()
	if err := os.MkdirAll(s.session, 0o755); err != nil {
		s.t.Fatalf("session directory: %v", err)
	}
	path := filepath.Join(s.session, name)
	f, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o600)
	if err != nil {
		s.t.Fatalf("open %s: %v", name, err)
	}
	defer f.Close()
	if _, err := f.WriteString(strings.Join(records, "")); err != nil {
		s.t.Fatalf("write %s: %v", name, err)
	}
}

func (s *grokStore) line(rec map[string]any) string {
	s.t.Helper()
	data, err := json.Marshal(rec)
	if err != nil {
		s.t.Fatalf("marshal fixture record: %v", err)
	}
	return string(data) + "\n"
}

func (s *grokStore) append(records ...string) { s.appendRaw(strings.Join(records, "")) }

func (s *grokStore) appendRaw(text string) {
	s.t.Helper()
	if err := os.MkdirAll(s.session, 0o755); err != nil {
		s.t.Fatalf("session directory: %v", err)
	}
	f, err := os.OpenFile(s.file, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o600)
	if err != nil {
		s.t.Fatalf("open fixture: %v", err)
	}
	defer f.Close()
	if _, err := f.WriteString(text); err != nil {
		s.t.Fatalf("write fixture: %v", err)
	}
}

func (s *grokStore) size() int64 {
	s.t.Helper()
	info, err := os.Stat(s.file)
	if err != nil {
		s.t.Fatalf("stat fixture: %v", err)
	}
	return info.Size()
}

func (s *grokStore) stamp() int64 { return time.Now().Unix() }

func (s *grokStore) next() string {
	s.seq++
	return fmt.Sprintf("%s-%d", s.id, s.seq)
}
