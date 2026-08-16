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

// Synthetic Codex fixtures: a state root and one rollout JSONL, built record by
// record in the shapes the real store uses.
//
// Every byte here was written for this test. Nothing is copied from a real
// rollout — no personal conversation, no credentials, no reasoning, no real tool
// output — and the record shapes are the *envelopes* the adapter sniffs, filled
// with invented content.
//
// The fixture is built twice, once per history mode, because those are two
// incompatible record families that one Codex build writes both of. Both go
// through the whole contract.

const (
	// codexFixtureFormat and codexFixtureVersion record what these fixtures
	// represent: Codex's per-thread append-only rollout JSONL under CODEX_HOME,
	// as written by codex-cli 0.147.0, which writes both history modes.
	codexFixtureFormat  = "codex rollout JSONL, history_mode "
	codexFixtureVersion = "0.147.0"
)

type codexStore struct {
	t       *testing.T
	root    string
	dir     string
	pid     int
	started time.Time
	id      string
	mode    string

	file    string
	partial string
	seq     int
	turn    int
}

func newCodexPaginatedStore(t *testing.T) contractStore {
	return newCodexFixture(t, codexPaginated)
}

func newCodexLegacyStore(t *testing.T) contractStore {
	return newCodexFixture(t, codexLegacy)
}

func newCodexFixture(t *testing.T, mode string) *codexStore {
	t.Helper()
	base := t.TempDir()
	s := &codexStore{
		t:       t,
		root:    filepath.Join(base, "state"),
		dir:     filepath.Join(base, "work"),
		pid:     5151,
		started: time.Now().Add(-time.Minute),
		id:      "019ff156-1a13-7e01-9576-ee3212e96f9e",
		mode:    mode,
	}
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		t.Fatalf("work directory: %v", err)
	}
	s.seat()
	return s
}

// seat points the fixture at the shard its rollout belongs in. The date shard
// and the filename stamp are local time, exactly as the provider writes them.
func (s *codexStore) seat() {
	when := s.started
	s.file = filepath.Join(
		s.root, codexSessions,
		when.Format("2006"), when.Format("01"), when.Format("02"),
		codexPrefix+when.Format("2006-01-02T15-04-05")+"-"+s.id+jsonlSuffix,
	)
}

func (s *codexStore) Format() string { return codexFixtureFormat + s.mode + " " + codexFixtureVersion }

func (s *codexStore) Agent() proc.Agent {
	return proc.Agent{
		Adapter:   "codex",
		PID:       s.pid,
		PGID:      s.pid,
		Started:   s.started,
		Dir:       s.dir,
		StateRoot: s.root,
	}
}

// Peer is a second Codex tab in the same space. Codex has no process-to-session
// registry — its thread-writer locks name threads rather than their holders — so
// two rollouts in one working directory are indistinguishable, and both tabs
// stay untitled rather than one of them being handed the other's conversation.
func (s *codexStore) Peer() (contractStore, bool) {
	peer := &codexStore{
		t: s.t, root: s.root, dir: s.dir, mode: s.mode,
		pid: s.pid + 1, started: s.started, id: "019ff168-3950-7430-9a3c-207a24d41180",
	}
	peer.seat()
	return peer, false
}

// Start opens the thread the way the provider does: the session_meta head record
// and the turn context it writes before the operator has said anything.
func (s *codexStore) Start() {
	s.append(
		s.line(map[string]any{
			"timestamp": s.stamp(), "ordinal": 0, "type": "session_meta",
			"payload": map[string]any{
				"session_id":    s.id,
				"id":            s.id,
				"timestamp":     s.stamp(),
				"cwd":           s.dir,
				"originator":    "codex-tui",
				"cli_version":   codexFixtureVersion,
				"source":        "cli",
				"thread_source": "user",
				"history_mode":  s.mode,
				"instructions":  "invented instructions the operator never sees",
			},
		}),
		s.event(map[string]any{
			"type": "turn_context", "cwd": s.dir, "model": "codex-fixture-1",
		}, "turn_context"),
	)
}

func (s *codexStore) Turn(prompt, response string) { s.append(s.records(prompt, response, true)...) }

func (s *codexStore) PartialTurn(prompt, response string) {
	records := s.records(prompt, response, true)
	last := records[len(records)-1]
	cut := len(last) / 2
	s.appendRaw(strings.Join(records[:len(records)-1], "") + last[:cut])
	s.partial = last[cut:]
}

func (s *codexStore) Complete() {
	s.appendRaw(s.partial)
	s.partial = ""
}

// Title says what Codex's store says: there is no title record in a rollout at
// all. The sidecar database's `title` column is a verbatim copy of the first user
// message, which would pin the tab to a whole prompt and block paid generation
// for the life of the session, so this adapter exposes no native title.
func (s *codexStore) Title(string) bool { return false }

// Ignored writes the machinery: the whole model-wire stream, world state,
// compaction, subagent bookkeeping, token counts, and the item kinds that are
// reasoning, commands, file changes and intermediate progress rather than an
// answer.
func (s *codexStore) Ignored() {
	s.append(
		s.wire(map[string]any{
			"type": "reasoning", "summary": []any{},
			"content": []map[string]any{{"type": "reasoning_text", "text": "invented reasoning the operator never sees"}},
		}),
		s.wire(map[string]any{
			"type": "function_call", "name": "shell", "call_id": s.next(),
			"arguments": `{"command":["ls"]}`,
		}),
		s.wire(map[string]any{
			"type": "function_call_output", "call_id": s.next(),
			"output": "invented tool output",
		}),
		s.wire(map[string]any{
			"type": "message", "role": "assistant",
			"content": []map[string]any{{"type": "output_text", "text": "an intermediate assistant message"}},
		}),
		s.line(map[string]any{
			"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": "world_state",
			"payload": map[string]any{"files": []any{}},
		}),
		s.line(map[string]any{
			"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": "compacted",
			"payload": map[string]any{"message": "a summary of earlier work"},
		}),
		s.line(map[string]any{
			"timestamp": s.stamp(), "ordinal": s.ordinal(),
			"type":    "inter_agent_communication_metadata",
			"payload": map[string]any{"agent": "subagent"},
		}),
		s.event(map[string]any{"type": "task_started", "turn_id": s.next(), "model_context_window": 400000}, ""),
		s.event(map[string]any{"type": "token_count", "info": map[string]any{"total_tokens": 1}}, ""),
		s.event(map[string]any{"type": "thread_settings_applied", "settings": map[string]any{}}, ""),
	)
	if s.mode == codexPaginated {
		s.append(
			s.item("Reasoning", map[string]any{
				"content": []map[string]any{{"type": "reasoning_text", "text": "invented reasoning"}},
			}, s.next()),
			s.item("CommandExecution", map[string]any{
				"command": "ls", "aggregated_output": "invented tool output", "exit_code": 0,
			}, s.next()),
			s.item("FileChange", map[string]any{"changes": []any{}}, s.next()),
			s.item("McpToolCall", map[string]any{"server": "invented", "tool": "invented"}, s.next()),
			// An intermediate AgentMessage: progress, not the answer.
			s.item("AgentMessage", map[string]any{
				"content": []map[string]any{{"type": "text", "text": "still working on it"}},
				"phase":   "commentary",
			}, s.next()),
		)
		return
	}
	s.append(s.event(map[string]any{
		"type": "agent_message", "message": "still working on it", "phase": "commentary",
	}, ""))
}

// SyntheticUser writes the user-shaped records the provider itself authors, all
// of which live in the model-wire stream: an <environment_context> envelope in
// the user role, and a developer-role instruction. A task_complete follows, so
// if either of them could open a turn, this is where the adapter would invent
// one.
func (s *codexStore) SyntheticUser(text string) {
	s.append(
		s.wire(map[string]any{
			"type": "message", "role": "user",
			"content": []map[string]any{{
				"type": "input_text",
				"text": "<environment_context>\n  <cwd>" + s.dir + "</cwd>\n</environment_context>",
			}},
		}),
		s.wire(map[string]any{
			"type": "message", "role": "developer",
			"content": []map[string]any{{"type": "input_text", "text": text}},
		}),
		s.event(map[string]any{
			"type": "task_complete", "turn_id": s.next(),
			"last_agent_message": "an answer to nobody",
			"duration_ms":        10,
		}, ""),
	)
}

func (s *codexStore) Malformed() {
	s.appendRaw(`{"type":"event_msg","payload":{"type":"user_message",` + "\n")
}

// Drift writes a record of a kind the adapter interprets, in a shape it has
// never seen: a user message whose text is an object rather than the typed list
// or the plain string its family uses.
func (s *codexStore) Drift() {
	if s.mode == codexPaginated {
		s.append(s.line(map[string]any{
			"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": "event_msg",
			"payload": map[string]any{
				"type": "item_completed", "thread_id": s.id, "turn_id": s.next(),
				"item": map[string]any{
					"type": "UserMessage", "id": s.next(),
					"content": map[string]any{"blocks": []string{"a shape from a later version"}},
				},
			},
		}))
		return
	}
	s.append(s.event(map[string]any{
		"type":    "user_message",
		"message": map[string]any{"blocks": []string{"a shape from a later version"}},
	}, ""))
}

// Replace rewrites the rollout in place, shorter than it was, keeping a turn of
// history so that a reader which re-seated naively would emit it.
func (s *codexStore) Replace() {
	before := s.size()
	head := s.line(map[string]any{
		"timestamp": s.stamp(), "ordinal": 0, "type": "session_meta",
		"payload": map[string]any{
			"session_id": s.id, "id": s.id, "timestamp": s.stamp(), "cwd": s.dir,
			"originator": "codex-tui", "cli_version": codexFixtureVersion,
			"source": "cli", "thread_source": "user", "history_mode": s.mode,
		},
	})
	records := append([]string{head},
		s.records("a question from before the rewrite", "an answer from before the rewrite", false)...)
	if err := os.WriteFile(s.file, []byte(strings.Join(records, "")), 0o600); err != nil {
		s.t.Fatalf("replace store: %v", err)
	}
	if after := s.size(); after >= before {
		s.t.Fatalf("replacement fixture is %d bytes, not shorter than the %d it replaced", after, before)
	}
}

func (s *codexStore) Remove() {
	if err := os.Remove(s.file); err != nil && !os.IsNotExist(err) {
		s.t.Fatalf("remove store: %v", err)
	}
}

// records renders one complete turn in whichever family this fixture writes.
// machinery adds the reasoning, tool calls and intermediate progress the real
// store interleaves; without it the same turn is two records, which is what makes
// a replacement shorter than what it replaced.
func (s *codexStore) records(prompt, response string, machinery bool) []string {
	s.turn++
	turnID := fmt.Sprintf("019ff156-ff8a-7ab1-a381-%012d", s.turn)

	var out []string
	if s.mode == codexPaginated {
		out = append(out, s.line(map[string]any{
			"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": "event_msg",
			"payload": map[string]any{
				"type": "item_completed", "thread_id": s.id, "turn_id": turnID,
				"item": map[string]any{
					"type": "UserMessage", "id": s.next(),
					"content": []map[string]any{{"type": "text", "text": prompt, "text_elements": []any{}}},
				},
				"started_at_ms": 1, "completed_at_ms": 2,
			},
		}))
	} else {
		out = append(out, s.event(map[string]any{
			"type": "user_message", "message": prompt,
			"images": []any{}, "local_images": []any{}, "audio": []any{}, "local_audio": []any{},
			"text_elements": []any{},
		}, ""))
	}
	if machinery {
		out = append(out,
			s.event(map[string]any{"type": "task_started", "turn_id": turnID}, ""),
			s.wire(map[string]any{
				"type": "reasoning",
				"content": []map[string]any{
					{"type": "reasoning_text", "text": "invented reasoning the operator never sees"},
				},
			}),
			s.wire(map[string]any{
				"type": "function_call", "name": "shell", "call_id": s.next(),
				"arguments": `{"command":["ls"]}`,
			}),
			s.wire(map[string]any{
				"type": "function_call_output", "call_id": s.next(), "output": "invented tool output",
			}),
		)
		// The visible answer also lands as an item or an event of its own; the
		// adapter takes it from task_complete, and this is here so that a reader
		// which took it from the wrong record would show it.
		if s.mode == codexPaginated {
			out = append(out, s.item("AgentMessage", map[string]any{
				"content": []map[string]any{{"type": "text", "text": response}},
				"phase":   "final_answer",
			}, turnID))
		} else {
			out = append(out, s.event(map[string]any{
				"type": "agent_message", "message": response, "phase": "final_answer",
			}, ""))
		}
	}
	return append(out, s.event(map[string]any{
		"type": "task_complete", "turn_id": turnID, "last_agent_message": response,
		"started_at": 1, "completed_at": 2, "duration_ms": 1,
	}, ""))
}

// item renders one paginated item_completed record.
func (s *codexStore) item(kind string, fields map[string]any, turnID string) string {
	item := map[string]any{"type": kind, "id": s.next()}
	for k, v := range fields {
		item[k] = v
	}
	return s.line(map[string]any{
		"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": "event_msg",
		"payload": map[string]any{
			"type": "item_completed", "thread_id": s.id, "turn_id": turnID, "item": item,
			"started_at_ms": 1, "completed_at_ms": 2,
		},
	})
}

// event renders one event_msg record; kind names a different top-level type for
// the records that are not event_msg at all.
func (s *codexStore) event(payload map[string]any, kind string) string {
	if kind == "" {
		kind = "event_msg"
	}
	return s.line(map[string]any{
		"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": kind, "payload": payload,
	})
}

// wire renders one response_item record — the model wire, which this adapter
// must never read a turn out of.
func (s *codexStore) wire(payload map[string]any) string {
	return s.line(map[string]any{
		"timestamp": s.stamp(), "ordinal": s.ordinal(), "type": "response_item", "payload": payload,
	})
}

func (s *codexStore) line(rec map[string]any) string {
	s.t.Helper()
	data, err := json.Marshal(rec)
	if err != nil {
		s.t.Fatalf("marshal fixture record: %v", err)
	}
	return string(data) + "\n"
}

func (s *codexStore) append(records ...string) { s.appendRaw(strings.Join(records, "")) }

func (s *codexStore) appendRaw(text string) {
	s.t.Helper()
	if err := os.MkdirAll(filepath.Dir(s.file), 0o755); err != nil {
		s.t.Fatalf("session shard: %v", err)
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

func (s *codexStore) size() int64 {
	s.t.Helper()
	info, err := os.Stat(s.file)
	if err != nil {
		s.t.Fatalf("stat fixture: %v", err)
	}
	return info.Size()
}

func (s *codexStore) stamp() string { return time.Now().UTC().Format(time.RFC3339Nano) }

func (s *codexStore) ordinal() int {
	s.seq++
	return s.seq
}

func (s *codexStore) next() string {
	s.seq++
	return fmt.Sprintf("019ff156-0000-4000-8000-%012d", s.seq)
}
