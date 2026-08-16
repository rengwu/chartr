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

// Synthetic Pi fixtures: a sessions root and one session JSONL, built entry by
// entry in the shapes the real store uses.
//
// Every byte here was written for this test. Nothing is copied from a real
// session — no personal conversation, no credentials, no reasoning, no real tool
// output.

const (
	// piFixtureFormat and piFixtureVersion record what these fixtures
	// represent: Pi's per-session append-only JSONL under the resolved sessions
	// directory, session format version 3, as written by pi 0.78.0.
	piFixtureFormat  = "pi session JSONL v3"
	piFixtureVersion = "0.78.0"
)

type piStore struct {
	t       *testing.T
	root    string
	dir     string
	pid     int
	started time.Time
	id      string

	file    string
	partial string
	seq     int
}

func newPiStore(t *testing.T) contractStore { return newPiFixture(t) }

func newPiFixture(t *testing.T) *piStore {
	t.Helper()
	base := t.TempDir()
	s := &piStore{
		t:       t,
		root:    filepath.Join(base, "sessions"),
		dir:     filepath.Join(base, "work"),
		pid:     6161,
		started: time.Now().Add(-time.Minute),
		id:      "019fe024-850d-7c27-9b8f-2bf1b6d9ebe5",
	}
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		t.Fatalf("work directory: %v", err)
	}
	s.seat()
	return s
}

// seat points the fixture at the file its session belongs in. The bucket name
// mimics Pi's lossy encoding of the working directory closely enough for a
// fixture; the adapter under test must never depend on it, and this exists only
// so the fixture sits where a real store's does.
func (s *piStore) seat() {
	bucket := "--" + strings.NewReplacer("/", "-", "\\", "-", ":", "-").Replace(strings.TrimPrefix(s.dir, "/")) + "--"
	s.file = filepath.Join(s.root, bucket, s.started.UTC().Format("2006-01-02T15-04-05.000Z")+"_"+s.id+jsonlSuffix)
}

func (s *piStore) Format() string { return piFixtureFormat + " " + piFixtureVersion }

func (s *piStore) Agent() proc.Agent {
	return proc.Agent{
		Adapter:   "pi",
		PID:       s.pid,
		PGID:      s.pid,
		Started:   s.started,
		Dir:       s.dir,
		StateRoot: s.root,
	}
}

// Peer is a second Pi tab in the same space. Pi has no process-to-session
// registry, so two session files in one working directory are indistinguishable
// and both tabs stay untitled.
func (s *piStore) Peer() (contractStore, bool) {
	peer := &piStore{
		t: s.t, root: s.root, dir: s.dir,
		pid: s.pid + 1, started: s.started, id: "019fe024-9999-7c27-9b8f-2bf1b6d9ebe5",
	}
	peer.seat()
	return peer, false
}

// Start creates the session in the earliest state it can exist on disk in.
//
// Pi buffers everything in memory and does not create the file until the first
// assistant message lands, at which point it writes the header, the opening
// prompt and that answer together. There is no header-only Pi file, so the
// truthful fixture for "the session exists" already carries one completed turn —
// and since binding seats at the end of the store, that turn is history a tab
// can never be charged for. It is why a Pi tab's first turn is never titled.
func (s *piStore) Start() {
	s.append(s.header())
	s.append(s.records("the prompt the session was created with", "the answer that created the file", true)...)
}

func (s *piStore) Turn(prompt, response string) { s.append(s.records(prompt, response, true)...) }

func (s *piStore) PartialTurn(prompt, response string) {
	records := s.records(prompt, response, true)
	last := records[len(records)-1]
	cut := len(last) / 2
	s.appendRaw(strings.Join(records[:len(records)-1], "") + last[:cut])
	s.partial = last[cut:]
}

func (s *piStore) Complete() {
	s.appendRaw(s.partial)
	s.partial = ""
}

// Title writes the session_info entry an operator's /name produces. Pi never
// generates one itself, but the entry is real and the latest one wins.
func (s *piStore) Title(title string) bool {
	s.append(s.entry(map[string]any{"type": "session_info", "name": title}))
	return true
}

// Ignored writes the machinery: model and thinking-level changes, compaction and
// its summary, a branch summary, a label, extension state, a tool result and a
// shell execution in their own roles, and an assistant turn that is nothing but
// hidden reasoning and a tool call.
func (s *piStore) Ignored() {
	s.append(
		s.entry(map[string]any{"type": "model_change", "model": "pi-fixture-1"}),
		s.entry(map[string]any{"type": "thinking_level_change", "level": "medium"}),
		s.entry(map[string]any{"type": "compaction", "summary": "a summary of earlier work"}),
		s.entry(map[string]any{"type": "branch_summary", "summary": "a summary of a branch"}),
		s.entry(map[string]any{"type": "label", "targetId": s.next(), "label": "a label"}),
		s.entry(map[string]any{"type": "custom", "state": map[string]any{"invented": true}}),
		s.message("toolResult", []map[string]any{{"type": "text", "text": "invented tool output"}}, ""),
		s.message("bashExecution", []map[string]any{{"type": "text", "text": "invented shell output"}}, ""),
		s.message("compactionSummary", []map[string]any{{"type": "text", "text": "a summary of earlier work"}}, ""),
		s.message("branchSummary", []map[string]any{{"type": "text", "text": "a summary of a branch"}}, ""),
		s.message("assistant", []map[string]any{
			{"type": "thinking", "thinking": "invented reasoning the operator never sees"},
			{"type": "toolCall", "id": s.next(), "name": "read", "arguments": map[string]any{"path": "/invented/path"}},
		}, "toolUse"),
	)
}

// SyntheticUser writes the user-shaped records Pi itself authors: an
// extension-injected custom_message, which enters the model's context looking
// like a person, and a custom-role message. An ordinary finished answer follows,
// so if either of them could open a turn, this is where the adapter would invent
// one.
func (s *piStore) SyntheticUser(text string) {
	s.append(
		s.entry(map[string]any{
			"type": "custom_message", "role": "user",
			"message": map[string]any{"role": "user", "content": []map[string]any{{"type": "text", "text": text}}},
		}),
		s.message("custom", []map[string]any{{"type": "text", "text": text}}, ""),
		s.message("assistant", []map[string]any{{"type": "text", "text": "an answer to nobody"}}, "stop"),
	)
}

func (s *piStore) Malformed() {
	s.appendRaw(`{"type":"message","id":"abcd1234","message":{"role":"user",` + "\n")
}

// Drift writes an entry of a kind the adapter interprets, in a shape it has
// never seen: a user message whose content is an object rather than a string or
// a list of typed blocks.
func (s *piStore) Drift() {
	s.append(s.entry(map[string]any{
		"type": "message", "id": s.next(), "parentId": s.next(),
		"message": map[string]any{
			"role":      "user",
			"content":   map[string]any{"blocks": []string{"a shape from a later version"}},
			"timestamp": s.stamp(),
		},
	}))
}

// Replace rewrites the store in place, shorter than it was, keeping a turn of
// history so that a reader which re-seated naively would emit it.
func (s *piStore) Replace() {
	before := s.size()
	records := append([]string{s.header()},
		s.records("a question from before the rewrite", "an answer from before the rewrite", false)...)
	if err := os.WriteFile(s.file, []byte(strings.Join(records, "")), 0o600); err != nil {
		s.t.Fatalf("replace store: %v", err)
	}
	if after := s.size(); after >= before {
		s.t.Fatalf("replacement fixture is %d bytes, not shorter than the %d it replaced", after, before)
	}
}

func (s *piStore) Remove() {
	if err := os.Remove(s.file); err != nil && !os.IsNotExist(err) {
		s.t.Fatalf("remove store: %v", err)
	}
}

func (s *piStore) header() string {
	return s.line(map[string]any{
		"type": "session", "version": piVersion, "id": s.id,
		"timestamp": s.stamp(), "cwd": s.dir,
	})
}

// records renders one complete turn: the operator's text, an assistant that
// thinks and calls a tool, the tool's result in its own role, and the finished
// answer. machinery adds the middle, so a replacement can be shorter than what
// it replaced.
func (s *piStore) records(prompt, response string, machinery bool) []string {
	out := []string{s.message("user", []map[string]any{{"type": "text", "text": prompt}}, "")}
	if machinery {
		out = append(out,
			s.message("assistant", []map[string]any{
				{"type": "thinking", "thinking": "invented reasoning the operator never sees"},
				{"type": "toolCall", "id": s.next(), "name": "read", "arguments": map[string]any{"path": "/invented/path"}},
			}, "toolUse"),
			s.message("toolResult", []map[string]any{{"type": "text", "text": "invented tool output"}}, ""),
		)
	}
	return append(out, s.message("assistant", []map[string]any{{"type": "text", "text": response}}, "stop"))
}

// message renders one message entry. stopReason is set only on the assistant
// role, which is the only role that carries one.
func (s *piStore) message(role string, content []map[string]any, stop string) string {
	msg := map[string]any{"role": role, "content": content, "timestamp": s.stamp()}
	if role == "assistant" {
		msg["stopReason"] = stop
		msg["api"] = "invented"
		msg["provider"] = "invented"
		msg["model"] = "pi-fixture-1"
		msg["usage"] = map[string]any{"input": 1, "output": 1}
	}
	return s.entry(map[string]any{"type": "message", "message": msg})
}

// entry fills in the fields every entry carries.
func (s *piStore) entry(rec map[string]any) string {
	base := map[string]any{"id": s.next(), "parentId": s.next(), "timestamp": s.stamp()}
	for k, v := range rec {
		base[k] = v
	}
	return s.line(base)
}

func (s *piStore) line(rec map[string]any) string {
	s.t.Helper()
	data, err := json.Marshal(rec)
	if err != nil {
		s.t.Fatalf("marshal fixture entry: %v", err)
	}
	return string(data) + "\n"
}

func (s *piStore) append(records ...string) { s.appendRaw(strings.Join(records, "")) }

func (s *piStore) appendRaw(text string) {
	s.t.Helper()
	if err := os.MkdirAll(filepath.Dir(s.file), 0o755); err != nil {
		s.t.Fatalf("session bucket: %v", err)
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

func (s *piStore) size() int64 {
	s.t.Helper()
	info, err := os.Stat(s.file)
	if err != nil {
		s.t.Fatalf("stat fixture: %v", err)
	}
	return info.Size()
}

func (s *piStore) stamp() string { return time.Now().UTC().Format(time.RFC3339Nano) }

// next is a fixture identifier generator: Pi's 8-char hex entry ids, obviously
// invented and deterministic.
func (s *piStore) next() string {
	s.seq++
	return fmt.Sprintf("%08x", s.seq)
}
