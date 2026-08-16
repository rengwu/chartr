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

// Synthetic Kimi Code fixtures: a state root, the session index, a session
// directory with its state.json, and the main agent's wire log.
//
// Every byte here was written for this test. Nothing is copied from a real
// session — no personal conversation, no credentials, no reasoning, no real tool
// output.

const (
	// kimiFixtureFormat and kimiFixtureVersion record what these fixtures
	// represent: Kimi Code's append-only agents/main/wire.jsonl plus the
	// rewritten state.json beside it, at wire protocol 1.5, as written by Kimi
	// Code 0.36.1.
	kimiFixtureFormat  = "kimi-code wire.jsonl (protocol 1.5) + state.json"
	kimiFixtureVersion = "0.36.1"
)

type kimiStore struct {
	t       *testing.T
	root    string
	dir     string
	pid     int
	started time.Time
	id      string
	bucket  string

	file    string
	state   string
	partial string
	seq     int
	turn    int
}

func newKimiStore(t *testing.T) contractStore { return newKimiFixture(t) }

func newKimiFixture(t *testing.T) *kimiStore {
	t.Helper()
	base := t.TempDir()
	s := &kimiStore{
		t:       t,
		root:    filepath.Join(base, "state"),
		dir:     filepath.Join(base, "work"),
		pid:     7171,
		started: time.Now().Add(-time.Minute),
		id:      "session_3beb89ca-f311-46b7-8bcf-85b23336aec3",
		bucket:  "wd_work_328473b6eb1a",
	}
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		t.Fatalf("work directory: %v", err)
	}
	s.seat()
	return s
}

// seat points the fixture at the paths its session lives under. The bucket name
// is a slug plus a hash in the real store and is not reversible, which is why the
// adapter reads the index rather than computing it.
func (s *kimiStore) seat() {
	dir := filepath.Join(s.root, kimiSessions, s.bucket, s.id)
	s.file = filepath.Join(dir, kimiAgents, kimiMain, kimiWire)
	s.state = filepath.Join(dir, kimiState)
}

func (s *kimiStore) sessionDir() string { return filepath.Dir(filepath.Dir(filepath.Dir(s.file))) }

func (s *kimiStore) Format() string { return kimiFixtureFormat + " " + kimiFixtureVersion }

func (s *kimiStore) Agent() proc.Agent {
	return proc.Agent{
		Adapter:   "kimi",
		PID:       s.pid,
		PGID:      s.pid,
		Started:   s.started,
		Dir:       s.dir,
		StateRoot: s.root,
	}
}

// Peer is a second Kimi tab in the same space. Kimi has no process-to-session
// registry — no pid appears anywhere under its root — so two sessions in one
// working directory are indistinguishable and both tabs stay untitled.
func (s *kimiStore) Peer() (contractStore, bool) {
	peer := &kimiStore{
		t: s.t, root: s.root, dir: s.dir, bucket: s.bucket,
		pid: s.pid + 1, started: s.started, id: "session_9999aaaa-f311-46b7-8bcf-85b23336aec3",
	}
	peer.seat()
	return peer, false
}

// Start opens the session the way the provider does: an index entry naming it,
// the state file with no title yet, and the wire log's protocol header.
func (s *kimiStore) Start() {
	s.index()
	s.writeState(map[string]any{
		"id": s.id, "version": 2, "cwd": s.dir,
		"createdAt": time.Now().UnixMilli(), "updatedAt": time.Now().UnixMilli(),
		"archived": false, "isCustomTitle": false,
	})
	s.append(s.line(map[string]any{
		"type": "metadata", "protocol_version": "1.5", "created_at": time.Now().UnixMilli(),
	}))
}

func (s *kimiStore) Turn(prompt, response string) { s.append(s.records(prompt, response, true)...) }

func (s *kimiStore) PartialTurn(prompt, response string) {
	records := s.records(prompt, response, true)
	last := records[len(records)-1]
	cut := len(last) / 2
	s.appendRaw(strings.Join(records[:len(records)-1], "") + last[:cut])
	s.partial = last[cut:]
}

func (s *kimiStore) Complete() {
	s.appendRaw(s.partial)
	s.partial = ""
}

// Title rewrites state.json with a title an operator set. Only `custom` and
// `generated` are titles here; the default `replaceable` kind is the prompt
// itself, and the adapter must refuse it.
func (s *kimiStore) Title(title string) bool {
	s.writeState(map[string]any{
		"id": s.id, "version": 2, "cwd": s.dir,
		"updatedAt": time.Now().UnixMilli(), "archived": false,
		"title": title, "titleKind": "custom", "isCustomTitle": true,
	})
	return true
}

// replaceable rewrites state.json with the kind Kimi sets by default: the prompt
// text, capped, wearing the title field. Publishing it would pin the tab to a
// prompt and block paid generation for the life of the session.
func (s *kimiStore) replaceable(text string) {
	s.writeState(map[string]any{
		"id": s.id, "version": 2, "cwd": s.dir,
		"updatedAt": time.Now().UnixMilli(), "archived": false,
		"title": text, "titleKind": "replaceable", "isCustomTitle": false,
	})
}

// Ignored writes the machinery: model requests, usage records, permission and
// interaction bookkeeping, compaction, plugin and task events, and the mirrored
// context.append_message stream that carries the same prompts again beside
// user-role machinery.
func (s *kimiStore) Ignored() {
	s.append(
		s.line(map[string]any{"type": "llm.request", "messages": []any{}, "time": s.stamp()}),
		s.line(map[string]any{"type": "usage.record", "usage": map[string]any{"input": 1}, "time": s.stamp()}),
		s.line(map[string]any{
			"type": "context.append_message", "time": s.stamp(),
			"message": map[string]any{"role": "user", "content": "the same prompt, mirrored into the model context"},
		}),
		s.line(map[string]any{"type": "permission.set_mode", "mode": "default", "time": s.stamp()}),
		s.line(map[string]any{"type": "interaction.request", "id": s.next(), "time": s.stamp()}),
		s.line(map[string]any{"type": "interaction.resolved", "id": s.next(), "time": s.stamp()}),
		s.line(map[string]any{"type": "plugin.session_start", "plugin": "invented", "time": s.stamp()}),
		s.line(map[string]any{"type": "task.started", "task": s.next(), "time": s.stamp()}),
		s.line(map[string]any{"type": "full_compaction.begin", "time": s.stamp()}),
		s.line(map[string]any{"type": "full_compaction.complete", "time": s.stamp()}),
		s.line(map[string]any{"type": "tools.update_store", "tools": []any{}, "time": s.stamp()}),
		s.loop(map[string]any{
			"type": "tool.call", "uuid": s.next(), "turnId": "99", "step": 1,
			"toolCallId": s.next(), "name": "read", "args": map[string]any{"path": "/invented/path"},
		}),
		s.loop(map[string]any{
			"type": "tool.result", "parentUuid": s.next(), "toolCallId": s.next(),
			"result": "invented tool output",
		}),
		s.loop(map[string]any{
			"type": "content.part", "uuid": s.next(), "turnId": "99", "step": 1,
			"part": map[string]any{"type": "think", "text": "invented reasoning the operator never sees"},
		}),
	)
}

// SyntheticUser writes the submissions Kimi itself authors on the same record a
// person's arrives on — a skill activation, a mid-turn steer — plus the mirrored
// user-role context message. A complete step and turn.ended follow, so if any of
// them could open a turn, this is where the adapter would invent one.
func (s *kimiStore) SyntheticUser(text string) {
	s.turn++
	id := fmt.Sprint(s.turn)
	s.append(
		s.line(map[string]any{
			"type":   "turn.prompt",
			"input":  []map[string]any{{"type": "text", "text": text}},
			"origin": map[string]any{"kind": "skill_activation"},
			"time":   s.stamp(),
		}),
		s.line(map[string]any{
			"type": "turn.steer", "text": text,
			"origin": map[string]any{"kind": "skill_activation"}, "time": s.stamp(),
		}),
		s.line(map[string]any{
			"type": "context.append_message", "time": s.stamp(),
			"message": map[string]any{"role": "user", "content": text},
		}),
		s.loop(map[string]any{"type": "step.begin", "uuid": s.next(), "turnId": id, "step": 1}),
		s.loop(map[string]any{
			"type": "content.part", "uuid": s.next(), "turnId": id, "step": 1,
			"part": map[string]any{"type": "text", "text": "an answer to nobody"},
		}),
		s.loop(map[string]any{
			"type": "step.end", "uuid": s.next(), "turnId": id, "step": 1, "finishReason": "end_turn",
		}),
		s.line(map[string]any{
			"type": "turn.ended", "turnId": s.turn, "reason": "completed",
			"durationMs": 1, "time": s.stamp(),
		}),
	)
}

func (s *kimiStore) Malformed() {
	s.appendRaw(`{"type":"turn.prompt","input":[{"type":"text",` + "\n")
}

// Drift writes a record of a kind the adapter interprets, in a shape it has
// never seen: a prompt whose input is an object rather than a list of typed
// parts.
func (s *kimiStore) Drift() {
	s.append(s.line(map[string]any{
		"type":   "turn.prompt",
		"input":  map[string]any{"blocks": []string{"a shape from a later version"}},
		"origin": map[string]any{"kind": "user"},
		"time":   s.stamp(),
	}))
}

// Replace rewrites the wire log in place, shorter than it was, keeping a turn of
// history so that a reader which re-seated naively would emit it.
func (s *kimiStore) Replace() {
	before := s.size()
	records := append([]string{s.line(map[string]any{
		"type": "metadata", "protocol_version": "1.5", "created_at": time.Now().UnixMilli(),
	})}, s.records("a question from before the rewrite", "an answer from before the rewrite", false)...)
	if err := os.WriteFile(s.file, []byte(strings.Join(records, "")), 0o600); err != nil {
		s.t.Fatalf("replace store: %v", err)
	}
	if after := s.size(); after >= before {
		s.t.Fatalf("replacement fixture is %d bytes, not shorter than the %d it replaced", after, before)
	}
}

func (s *kimiStore) Remove() {
	if err := os.Remove(s.file); err != nil && !os.IsNotExist(err) {
		s.t.Fatalf("remove store: %v", err)
	}
}

// records renders one complete turn: the operator's submission, a step that
// thinks and calls a tool, and the step that produces the visible answer and ends
// the turn. Note the identity Kimi writes as a string on the steps and as an
// integer on turn.ended.
func (s *kimiStore) records(prompt, response string, machinery bool) []string {
	s.turn++
	id := fmt.Sprint(s.turn)

	out := []string{s.line(map[string]any{
		"type":   "turn.prompt",
		"input":  []map[string]any{{"type": "text", "text": prompt}},
		"origin": map[string]any{"kind": "user"},
		"time":   s.stamp(),
	})}
	if machinery {
		out = append(out,
			s.loop(map[string]any{"type": "step.begin", "uuid": s.next(), "turnId": id, "step": 1}),
			s.loop(map[string]any{
				"type": "content.part", "uuid": s.next(), "turnId": id, "step": 1,
				"part": map[string]any{"type": "think", "text": "invented reasoning the operator never sees"},
			}),
			s.loop(map[string]any{
				"type": "content.part", "uuid": s.next(), "turnId": id, "step": 1,
				"part": map[string]any{"type": "text", "text": "still working on it"},
			}),
			s.loop(map[string]any{
				"type": "tool.call", "uuid": s.next(), "turnId": id, "step": 1,
				"toolCallId": s.next(), "name": "read", "args": map[string]any{"path": "/invented/path"},
			}),
			s.loop(map[string]any{
				"type": "tool.result", "parentUuid": s.next(), "toolCallId": s.next(),
				"result": "invented tool output",
			}),
			s.loop(map[string]any{
				"type": "step.end", "uuid": s.next(), "turnId": id, "step": 1, "finishReason": "tool_use",
			}),
		)
	}
	return append(out,
		s.loop(map[string]any{"type": "step.begin", "uuid": s.next(), "turnId": id, "step": 2}),
		s.loop(map[string]any{
			"type": "content.part", "uuid": s.next(), "turnId": id, "step": 2,
			"part": map[string]any{"type": "text", "text": response},
		}),
		s.loop(map[string]any{
			"type": "step.end", "uuid": s.next(), "turnId": id, "step": 2, "finishReason": "end_turn",
		}),
		s.line(map[string]any{
			"type": "turn.ended", "turnId": s.turn, "reason": "completed",
			"durationMs": 1, "time": s.stamp(),
		}),
	)
}

// loop renders one context.append_loop_event record.
func (s *kimiStore) loop(event map[string]any) string {
	return s.line(map[string]any{
		"type": "context.append_loop_event", "event": event, "time": s.stamp(),
	})
}

// index appends this session to the index Kimi keeps of every session it opens.
func (s *kimiStore) index() {
	s.t.Helper()
	if err := os.MkdirAll(s.root, 0o755); err != nil {
		s.t.Fatalf("state root: %v", err)
	}
	entry := s.line(map[string]any{
		"sessionId": s.id, "sessionDir": s.sessionDir(), "workDir": s.dir,
	})
	f, err := os.OpenFile(filepath.Join(s.root, kimiIndex), os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o600)
	if err != nil {
		s.t.Fatalf("open index: %v", err)
	}
	defer f.Close()
	if _, err := f.WriteString(entry); err != nil {
		s.t.Fatalf("write index: %v", err)
	}
}

func (s *kimiStore) writeState(state map[string]any) {
	s.t.Helper()
	if err := os.MkdirAll(filepath.Dir(s.state), 0o755); err != nil {
		s.t.Fatalf("session directory: %v", err)
	}
	data, err := json.Marshal(state)
	if err != nil {
		s.t.Fatalf("marshal state: %v", err)
	}
	if err := os.WriteFile(s.state, data, 0o600); err != nil {
		s.t.Fatalf("write state: %v", err)
	}
	// A sidecar rewritten within the same filesystem timestamp tick must still
	// read as a change, so the poll that re-checks it sees the new title.
	touch(s.t, s.state, time.Now().Add(time.Duration(s.seq)*time.Second))
}

func (s *kimiStore) line(rec map[string]any) string {
	s.t.Helper()
	data, err := json.Marshal(rec)
	if err != nil {
		s.t.Fatalf("marshal fixture record: %v", err)
	}
	return string(data) + "\n"
}

func (s *kimiStore) append(records ...string) { s.appendRaw(strings.Join(records, "")) }

func (s *kimiStore) appendRaw(text string) {
	s.t.Helper()
	if err := os.MkdirAll(filepath.Dir(s.file), 0o755); err != nil {
		s.t.Fatalf("agent directory: %v", err)
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

func (s *kimiStore) size() int64 {
	s.t.Helper()
	info, err := os.Stat(s.file)
	if err != nil {
		s.t.Fatalf("stat fixture: %v", err)
	}
	return info.Size()
}

func (s *kimiStore) stamp() int64 { return time.Now().UnixMilli() }

func (s *kimiStore) next() string {
	s.seq++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", s.seq)
}
