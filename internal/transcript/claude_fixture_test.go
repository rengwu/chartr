package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/proc"
)

// Synthetic Claude Code fixtures: a state root, a registry entry and a session
// JSONL file, built record by record in the shapes the real store uses.
//
// Every byte here was written for this test. Nothing is copied from a real
// transcript — no personal conversation, no credentials, no hidden reasoning, no
// real tool output — and the record shapes are the *envelopes* the adapter
// sniffs, filled with invented content. What makes them a fixture rather than a
// guess is claudeFixtureVersion: they were written against a store observed
// first-hand at that version, so when a future Claude changes the format, the
// deliberate act is editing this file to say so.

const (
	// claudeFixtureFormat and claudeFixtureVersion record what these fixtures
	// represent: Claude Code's per-session append-only JSONL under
	// CLAUDE_CONFIG_DIR, as written by the 2.1.22x–2.1.23x line, together with
	// the sessions/<pid>.json process registry it keeps beside them.
	claudeFixtureFormat  = "claude-code session JSONL + sessions/<pid>.json registry"
	claudeFixtureVersion = "2.1.233"
)

// claudeStore writes those fixtures for one session.
type claudeStore struct {
	t       *testing.T
	root    string
	dir     string
	pid     int
	started time.Time
	id      string

	file    string
	partial string // the tail of a record deliberately left unwritten
	seq     int
}

func newClaudeStore(t *testing.T) contractStore { return newClaudeFixture(t) }

func newClaudeFixture(t *testing.T) *claudeStore {
	t.Helper()
	base := t.TempDir()
	s := &claudeStore{
		t:       t,
		root:    filepath.Join(base, "state"),
		dir:     filepath.Join(base, "work"),
		pid:     4242,
		started: time.Now().Add(-time.Minute),
		id:      "11111111-2222-4333-8444-555555555555",
	}
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		t.Fatalf("work directory: %v", err)
	}
	s.seat()
	return s
}

// seat points the fixture at the file its current session belongs in.
func (s *claudeStore) seat() {
	s.file = filepath.Join(s.root, claudeProjects, encodeProject(s.dir), s.id+claudeSuffix)
}

// The helpers below exist for the binding tests, which need to build the
// situations discovery has to tell apart: a second agent in the same directory,
// a session known to the registry before it has written anything, a prompt
// already persisted when chartr arrives, a session the process walked away from,
// and a store with no registry at all.

// peer is a second agent in the same space: the same state root and working
// directory, its own pid and its own session.
func (s *claudeStore) peer(pid int, id string) *claudeStore {
	peer := &claudeStore{
		t: s.t, root: s.root, dir: s.dir,
		pid: pid, started: s.started, id: id,
	}
	peer.seat()
	return peer
}

// announce writes only the registry entry: the process has said which session it
// is driving, and has not written a line of it yet. This is where a
// chartr-launched tab is bound.
func (s *claudeStore) announce() { s.registry(s.id, s.pid, s.dir, s.started.Add(time.Second)) }

// startBare opens the session without a registry entry, as an older provider
// build does — the case discovery has to solve from the working directory and
// observed writes alone.
func (s *claudeStore) startBare() {
	if err := os.MkdirAll(filepath.Dir(s.file), 0o755); err != nil {
		s.t.Fatalf("project tree: %v", err)
	}
	s.append(s.boot()...)
}

// prompt writes a human turn's opening record with no answer behind it yet, and
// answer writes the final visible response that completes it.
func (s *claudeStore) prompt(text string) {
	s.append(s.line(s.envelope(map[string]any{
		"type":         "user",
		"message":      map[string]any{"role": "user", "content": text},
		"origin":       map[string]any{"kind": "human"},
		"promptSource": "typed",
		"promptId":     s.next(),
	})))
}

func (s *claudeStore) answer(text string) {
	s.append(s.line(s.assistant([]map[string]any{{"type": "text", "text": text}}, "end_turn", false)))
}

// clearTo is what /clear looks like from outside: the same process, now driving
// a different session, with the registry rewritten to say so.
func (s *claudeStore) clearTo(id string) {
	s.id = id
	s.seat()
	s.registry(id, s.pid, s.dir, s.started.Add(2*time.Second))
}

// stale writes a registry entry left behind by whoever held this pid before —
// same pid, same directory, but a session that began before this process did.
func (s *claudeStore) stale(id string) {
	s.registry(id, s.pid, s.dir, s.started.Add(-time.Hour))
}

// idle backdates the session file, so it no longer counts as a store written to
// since the agent started.
func (s *claudeStore) idle() { touch(s.t, s.file, s.started.Add(-time.Hour)) }

func (s *claudeStore) Format() string { return claudeFixtureFormat + " " + claudeFixtureVersion }

func (s *claudeStore) Agent() proc.Agent {
	return proc.Agent{
		Adapter:   "claude",
		PID:       s.pid,
		PGID:      s.pid,
		Started:   s.started,
		Dir:       s.dir,
		StateRoot: s.root,
	}
}

// Start opens the session the way the provider does: the registry entry that
// maps this process to it, then the handful of boot records it writes before the
// operator has said anything. None of those carry a working directory — which is
// exactly why registry-less discovery has to read a little way into a candidate.
func (s *claudeStore) Start() {
	s.registry(s.id, s.pid, s.dir, s.started.Add(time.Second))
	if err := os.MkdirAll(filepath.Dir(s.file), 0o755); err != nil {
		s.t.Fatalf("project tree: %v", err)
	}
	s.append(s.boot()...)
}

// Turn writes one complete turn as the provider records it: the operator's text,
// an assistant that thinks and calls a tool, the tool's result coming back in
// the user role, and finally the visible answer.
func (s *claudeStore) Turn(prompt, response string) {
	s.append(s.turn(prompt, response, true)...)
}

// PartialTurn writes the same, with the last record cut mid-write — a reader
// arriving between the provider's write and its newline.
func (s *claudeStore) PartialTurn(prompt, response string) {
	records := s.turn(prompt, response, true)
	last := records[len(records)-1]
	cut := len(last) / 2
	s.appendRaw(strings.Join(records[:len(records)-1], "") + last[:cut])
	s.partial = last[cut:]
}

func (s *claudeStore) Complete() {
	s.appendRaw(s.partial)
	s.partial = ""
}

func (s *claudeStore) Title(title string) {
	s.append(s.line(map[string]any{
		"type":      "ai-title",
		"aiTitle":   title,
		"sessionId": s.id,
	}))
}

// Ignored writes the machinery: modes, permission choices, bridge and queue
// bookkeeping, file snapshots, an attachment, a system note, a turn that is all
// hidden reasoning and no visible text, and a subagent's complete exchange on a
// sidechain.
func (s *claudeStore) Ignored() {
	s.append(
		s.line(map[string]any{"type": "mode", "mode": "normal", "sessionId": s.id}),
		s.line(map[string]any{"type": "permission-mode", "permissionMode": "default", "sessionId": s.id}),
		s.line(map[string]any{"type": "queue-operation", "sessionId": s.id, "operation": "drain"}),
		s.line(map[string]any{"type": "summary", "summary": "a summary of earlier work", "leafUuid": s.next()}),
		s.line(map[string]any{"type": "last-prompt", "lastPrompt": "an earlier prompt", "sessionId": s.id, "leafUuid": s.next()}),
		s.line(map[string]any{
			"type": "file-history-snapshot", "messageId": s.next(),
			"snapshot": map[string]any{"trackedFileBackups": map[string]any{}},
		}),
		s.line(s.envelope(map[string]any{
			"type":       "attachment",
			"attachment": map[string]any{"type": "deferred_tools_delta", "addedNames": []string{"WebFetch"}},
		})),
		s.line(s.envelope(map[string]any{
			"type": "system", "subtype": "turn_duration", "durationMs": 1200, "isMeta": false,
		})),
		s.line(s.assistant([]map[string]any{
			{"type": "thinking", "thinking": "reasoning the operator never sees"},
		}, "end_turn", false)),
		s.line(s.envelope(map[string]any{
			"type":         "user",
			"isSidechain":  true,
			"message":      map[string]any{"role": "user", "content": "a subagent's own instructions"},
			"origin":       map[string]any{"kind": "human"},
			"promptSource": "typed",
		})),
		s.line(s.assistant([]map[string]any{
			{"type": "text", "text": "a subagent's own answer"},
		}, "end_turn", true)),
	)
}

// SyntheticUser writes the user-role records the provider itself authors — a
// slash-command envelope, a caveat, an interruption notice, a tool result — and
// then a perfectly ordinary final answer after them. If any of those could pass
// as a human turn, this is where the adapter would invent one.
func (s *claudeStore) SyntheticUser(text string) {
	s.append(
		s.line(s.envelope(map[string]any{
			"type":    "user",
			"message": map[string]any{"role": "user", "content": "<command-name>" + text + "</command-name>\n<command-message>" + text + "</command-message>"},
		})),
		s.line(s.envelope(map[string]any{
			"type":    "user",
			"isMeta":  true,
			"message": map[string]any{"role": "user", "content": "<local-command-caveat>Caveat: the messages below were generated by a command.</local-command-caveat>"},
		})),
		s.line(s.envelope(map[string]any{
			"type": "user",
			"message": map[string]any{"role": "user", "content": []map[string]any{
				{"type": "text", "text": "[Request interrupted by user]"},
			}},
		})),
		s.line(s.envelope(map[string]any{
			"type":          "user",
			"toolUseResult": map[string]any{"stdout": "invented tool output", "stderr": ""},
			"message": map[string]any{"role": "user", "content": []map[string]any{
				{"type": "tool_result", "tool_use_id": s.next(), "content": "invented tool output"},
			}},
		})),
		s.line(s.assistant([]map[string]any{
			{"type": "text", "text": "an answer to nobody"},
		}, "end_turn", false)),
	)
}

func (s *claudeStore) Malformed() {
	s.appendRaw(`{"type":"user","message":{"role":"user",` + "\n")
}

// Drift writes a record of a kind the adapter interprets, in a shape it has
// never seen: a message whose content is an object rather than a string or a
// list of typed blocks. This is the provider's next schema, and the adapter's
// answer to it must be "not mine to read".
func (s *claudeStore) Drift() {
	s.append(s.line(s.envelope(map[string]any{
		"type": "user",
		"message": map[string]any{
			"role":    "user",
			"content": map[string]any{"blocks": []string{"a shape from a later version"}},
		},
		"origin":       map[string]any{"kind": "human"},
		"promptSource": "typed",
	})))
}

// Replace rewrites the store in place, shorter than it was — a compaction, a
// rewrite, a truncation — keeping a turn of history so that a reader which
// re-seated naively would emit it.
func (s *claudeStore) Replace() {
	before := s.size()
	records := append(s.boot(), s.turn("a question from before the rewrite", "an answer from before the rewrite", false)...)
	if err := os.WriteFile(s.file, []byte(strings.Join(records, "")), 0o600); err != nil {
		s.t.Fatalf("replace store: %v", err)
	}
	if after := s.size(); after >= before {
		s.t.Fatalf("replacement fixture is %d bytes, not shorter than the %d it replaced", after, before)
	}
}

func (s *claudeStore) Remove() {
	if err := os.Remove(s.file); err != nil && !os.IsNotExist(err) {
		s.t.Fatalf("remove store: %v", err)
	}
}

// registry writes the process-to-session entry, which is how a live pid says
// which conversation it is driving.
func (s *claudeStore) registry(id string, pid int, cwd string, startedAt time.Time) {
	s.t.Helper()
	dir := filepath.Join(s.root, claudeSessions)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		s.t.Fatalf("registry directory: %v", err)
	}
	entry := map[string]any{
		"pid":        pid,
		"sessionId":  id,
		"cwd":        cwd,
		"startedAt":  startedAt.UnixMilli(),
		"procStart":  startedAt.Format("Mon Jan _2 15:04:05 2006"),
		"version":    claudeFixtureVersion,
		"kind":       "interactive",
		"entrypoint": "cli",
		"name":       "work-a1",
		"nameSource": "derived",
		"status":     "idle",
	}
	data, err := json.Marshal(entry)
	if err != nil {
		s.t.Fatalf("registry entry: %v", err)
	}
	path := filepath.Join(dir, strconv.Itoa(pid)+".json")
	if err := os.WriteFile(path, data, 0o600); err != nil {
		s.t.Fatalf("write registry: %v", err)
	}
	// A registry entry rewritten within the same filesystem timestamp tick must
	// still read as a change, so the poll that re-checks it sees the new
	// session rather than the cached one.
	touch(s.t, path, time.Now().Add(time.Second))
}

// boot is the run of records a session opens with, none of which name a working
// directory.
func (s *claudeStore) boot() []string {
	return []string{
		s.line(map[string]any{"type": "mode", "mode": "normal", "sessionId": s.id}),
		s.line(map[string]any{"type": "permission-mode", "permissionMode": "bypassPermissions", "sessionId": s.id}),
		s.line(map[string]any{
			"type": "bridge-session", "sessionId": s.id,
			"bridgeSessionId": "cse_" + s.next(), "lastSequenceNum": 0,
		}),
	}
}

// turn renders one complete exchange. machinery adds the reasoning, tool call
// and tool result the real store interleaves; without it the same turn is two
// records, which is what makes a replacement shorter than what it replaced.
func (s *claudeStore) turn(prompt, response string, machinery bool) []string {
	records := []string{s.line(s.envelope(map[string]any{
		"type":           "user",
		"message":        map[string]any{"role": "user", "content": prompt},
		"origin":         map[string]any{"kind": "human"},
		"promptSource":   "typed",
		"permissionMode": "bypassPermissions",
		"promptId":       s.next(),
	}))}
	if machinery {
		toolUse := s.next()
		records = append(records,
			s.line(s.assistant([]map[string]any{
				{"type": "thinking", "thinking": "invented reasoning the operator never sees"},
			}, "tool_use", false)),
			s.line(s.assistant([]map[string]any{
				{"type": "tool_use", "id": toolUse, "name": "Read", "input": map[string]any{"file_path": "/invented/path"}},
			}, "tool_use", false)),
			s.line(s.envelope(map[string]any{
				"type":          "user",
				"toolUseResult": map[string]any{"stdout": "invented tool output"},
				"message": map[string]any{"role": "user", "content": []map[string]any{
					{"type": "tool_result", "tool_use_id": toolUse, "content": "invented tool output"},
				}},
			})),
		)
	}
	return append(records, s.line(s.assistant([]map[string]any{
		{"type": "text", "text": response},
	}, "end_turn", false)))
}

// assistant renders one assistant record with the given content blocks.
func (s *claudeStore) assistant(content []map[string]any, stop string, sidechain bool) map[string]any {
	return s.envelope(map[string]any{
		"type":        "assistant",
		"isSidechain": sidechain,
		"requestId":   "req_" + s.next(),
		"message": map[string]any{
			"role":        "assistant",
			"model":       "claude-fixture-1",
			"content":     content,
			"stop_reason": stop,
			"usage":       map[string]any{"input_tokens": 1, "output_tokens": 1},
		},
	})
}

// envelope fills in the fields every message-bearing record carries.
func (s *claudeStore) envelope(rec map[string]any) map[string]any {
	base := map[string]any{
		"parentUuid":  s.next(),
		"isSidechain": false,
		"uuid":        s.next(),
		"timestamp":   time.Now().UTC().Format(time.RFC3339Nano),
		"userType":    "external",
		"entrypoint":  "cli",
		"cwd":         s.dir,
		"sessionId":   s.id,
		"version":     claudeFixtureVersion,
		"gitBranch":   "main",
	}
	for k, v := range rec {
		base[k] = v
	}
	return base
}

func (s *claudeStore) line(rec map[string]any) string {
	s.t.Helper()
	data, err := json.Marshal(rec)
	if err != nil {
		s.t.Fatalf("marshal fixture record: %v", err)
	}
	return string(data) + "\n"
}

func (s *claudeStore) append(records ...string) { s.appendRaw(strings.Join(records, "")) }

func (s *claudeStore) appendRaw(text string) {
	s.t.Helper()
	if err := os.MkdirAll(filepath.Dir(s.file), 0o755); err != nil {
		s.t.Fatalf("project tree: %v", err)
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

func (s *claudeStore) size() int64 {
	s.t.Helper()
	info, err := os.Stat(s.file)
	if err != nil {
		s.t.Fatalf("stat fixture: %v", err)
	}
	return info.Size()
}

// next is a fixture identifier generator: uuid-shaped, deterministic, and
// obviously invented.
func (s *claudeStore) next() string {
	s.seq++
	return "00000000-0000-4000-8000-" + strconv.Itoa(100000000000 + s.seq)[1:]
}

// encodeProject mimics the provider's private directory encoding closely enough
// for a fixture. The adapter under test must never depend on it — it finds a
// session file by its uuid under any project tree — and this exists only so the
// fixtures sit where a real store's do.
func encodeProject(dir string) string {
	var b strings.Builder
	for _, r := range dir {
		switch {
		case r >= 'a' && r <= 'z', r >= 'A' && r <= 'Z', r >= '0' && r <= '9':
			b.WriteRune(r)
		default:
			b.WriteByte('-')
		}
	}
	return b.String()
}

// touch moves a file's modification time, so a test can say "this was written
// again" without waiting for the filesystem's own timestamp resolution.
func touch(t *testing.T, path string, when time.Time) {
	t.Helper()
	if err := os.Chtimes(path, when, when); err != nil {
		t.Fatalf("touch %s: %v", path, err)
	}
}
