package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/proc"
)

// This file is the Codex adapter.
//
// # The store, as measured
//
// Under the resolved state root (CODEX_HOME, default ~/.codex):
//
//	sessions/<YYYY>/<MM>/<DD>/rollout-<local-timestamp>-<session-uuid>.jsonl
//
// One append-only JSONL file per thread, every line `{timestamp, ordinal, type,
// payload}`. The date shard and the filename stamp are local time while the
// records inside are UTC, which is why discovery walks the shards rather than
// computing today's.
//
// Codex also keeps SQLite sidecars (state_5.sqlite, thread_history_1.sqlite)
// under CODEX_SQLITE_HOME. They are a *projection* of the rollout, not a second
// source of truth, and this adapter does not read them.
//
// # Two record families, chosen by a field that is not a version
//
// `session_meta.history_mode` selects between two incompatible shapes, and it is
// not a function of the version — one Codex build writes both:
//
//	paginated: event_msg/item_completed with item.type UserMessage
//	legacy:    event_msg/user_message
//
// So it is sniffed exactly as a schema version is, and a rollout that names
// neither — an older build wrote one with no history_mode at all — is not a
// store this adapter reads.
//
// # The model wire is not the conversation
//
// Every line is one of two streams. `response_item` is what was sent to the
// model: it carries user-role records wrapped in an <environment_context>
// envelope, developer-role instructions, reasoning, tool calls and tool results.
// `event_msg` is the UI item stream, and of every user message in it, not one
// carried a synthetic marker.
//
// This adapter reads `event_msg` and ignores `response_item` without exception.
// That single structural rule is what keeps the agent's own machinery out of a
// turn, and it is cheaper and far more reliable than a predicate over content
// that tries to recognise an envelope.
//
// # Where a turn begins and ends
//
// A turn opens on a text-only user message and closes on `task_complete`, which
// carries both the turn's identity and `last_agent_message` — the final visible
// answer, byte-identical to the last AgentMessage of that turn in every turn
// measured. An interrupted turn writes `turn_aborted` and never a
// `task_complete`, so it simply never completes. Intermediate AgentMessage items
// (`phase: "commentary"`) are progress, not the answer, and are never read.
//
// # No native title
//
// Codex writes no title record anywhere in the rollout. Its sidecar database has
// a `threads.title`, but that column is a verbatim copy of the first user message
// — up to tens of thousands of characters — and a native title blocks paid
// generation for the life of the session, so publishing it would pin every Codex
// tab to a whole prompt forever. Codex tabs take the paid path.
type codex struct{}

const (
	codexSessions = "sessions"
	codexPrefix   = "rollout-"

	// The two history modes, which are the whole of what this adapter has to
	// tell apart.
	codexPaginated = "paginated"
	codexLegacy    = "legacy"
)

// Bind matches a live Codex process to its persisted thread. There is no
// process-to-session registry — the thread-writer locks name threads rather than
// their holders — so this is the specification's other route: a rollout written
// since the agent started, whose head record names this working directory and
// this process's own kind of thread.
func (codex) Bind(agent proc.Agent) (Session, bool) {
	if agent.Adapter != "codex" || agent.StateRoot == "" || agent.PID <= 0 {
		return nil, false
	}
	path, meta, ok := codexSearch(agent)
	if !ok {
		return nil, false
	}
	s := &codexSession{id: meta.session, mode: meta.mode}
	s.store.path = path
	if !s.store.seat(s.fold) {
		return nil, false
	}
	s.forget()
	return s, true
}

// codexSession is one bound thread: which rollout, which record family, how far
// it has been read, and the one prompt held while its answer is still being
// written.
type codexSession struct {
	store tail
	id    string
	mode  string

	// pending is the operator's text for the turn now under way, and turn the
	// identity that turn opened with — empty in legacy mode, where the opening
	// record carries none and the closing record's identity is all there is.
	pending string
	turn    string

	out []Event
}

func (s *codexSession) ID() string { return s.id }

// forget drops the turn the fold was holding open: a turn already under way when
// the cursor was seated stays behind it whole.
func (s *codexSession) forget() { s.pending, s.turn = "", "" }

// Poll reads whatever has been appended since the last call.
func (s *codexSession) Poll() ([]Event, bool) {
	reseated, ok := s.store.advance(s.fold)
	if reseated {
		s.forget()
	}
	out := s.out
	s.out = nil
	return out, ok
}

// codexRecord is the envelope every line shares, narrowed to what this adapter
// reads. The payload is kept raw because which shape it has depends on its own
// type, which is the next thing read.
type codexRecord struct {
	Type    string          `json:"type"`
	Payload json.RawMessage `json:"payload"`
}

// codexPayload is the event_msg payload, narrowed the same way. Every field here
// is part of the sniff: a type change in any of them makes the store unreadable
// rather than reinterpreted.
type codexPayload struct {
	Type string `json:"type"`
	// TurnID identifies a turn on the records that carry one — the paginated
	// user message and every task_complete.
	TurnID string `json:"turn_id"`
	// Item is the paginated stream's completed item.
	Item *codexItem `json:"item"`
	// Message is the legacy stream's user text, with the attachment arrays that
	// have to be empty for the turn to be text-only.
	Message     string            `json:"message"`
	Images      []json.RawMessage `json:"images"`
	LocalImages []json.RawMessage `json:"local_images"`
	Audio       []json.RawMessage `json:"audio"`
	LocalAudio  []json.RawMessage `json:"local_audio"`
	// LastAgentMessage is the final visible answer, carried by task_complete
	// together with the identity of the turn it closes.
	LastAgentMessage string `json:"last_agent_message"`
}

type codexItem struct {
	Type    string          `json:"type"`
	Content json.RawMessage `json:"content"`
}

// fold folds one complete record in. false ends the binding.
func (s *codexSession) fold(line []byte, history bool) bool {
	var rec codexRecord
	if err := json.Unmarshal(line, &rec); err != nil {
		return false // malformed, or a field whose type this adapter relies on changed
	}
	if rec.Type == "" {
		return false // every record in this format is typed
	}
	// response_item is the model wire, turn_context and world_state are
	// bookkeeping, session_meta was read at binding, compacted and
	// inter_agent_communication_metadata are machinery. Only the item stream is
	// conversation.
	if rec.Type != "event_msg" || len(rec.Payload) == 0 {
		return true
	}

	var p codexPayload
	if err := json.Unmarshal(rec.Payload, &p); err != nil {
		return false
	}
	if p.Type == "" {
		return false // an event with no type is not a shape this adapter knows
	}

	switch p.Type {
	case "item_completed":
		if s.mode != codexPaginated || p.Item == nil {
			return true
		}
		if p.Item.Type != "UserMessage" {
			// Reasoning, command executions, file changes, MCP calls and the
			// intermediate AgentMessage items are all read past. The answer
			// comes from task_complete, so nothing here is the visible text.
			return true
		}
		text, only, ok := codexItemText(p.Item.Content)
		if !ok {
			return false // the shape a human turn is read out of has drifted
		}
		s.open(text, p.TurnID, only)
		return true

	case "user_message":
		if s.mode != codexLegacy {
			return true
		}
		// A legacy user message carries its attachments beside the text, and
		// this record has no identity of its own — the turn it opens is named
		// only by the task_complete that closes it.
		textOnly := len(p.Images) == 0 && len(p.LocalImages) == 0 &&
			len(p.Audio) == 0 && len(p.LocalAudio) == 0
		s.open(p.Message, "", textOnly)
		return true

	case "task_complete":
		prompt, turn := s.pending, s.turn
		s.forget()
		// An answer to a turn this adapter is not holding the prompt of, a turn
		// whose identity does not match the one that opened, and a turn that
		// produced no visible text at all: all three end quietly.
		if history || prompt == "" || (turn != "" && turn != p.TurnID) {
			return true
		}
		answer := head(p.LastAgentMessage, textCap)
		if answer == "" {
			return true
		}
		s.out = append(s.out, Event{Kind: HumanTurn, Prompt: prompt, Response: answer})
		return true

	case "turn_aborted":
		// Interrupted: no task_complete will follow, and the prompt it was
		// holding is not a completed turn.
		s.forget()
		return true
	}
	// New event types are ordinary — Codex adds them often, and this corpus
	// alone spans three versions. Reading past a kind is not a guess about it.
	return true
}

// open starts a turn on the operator's own text. Whatever was pending is not
// going to be answered: a new user message is a new act by whoever is at the
// keyboard, however the previous one ended.
func (s *codexSession) open(text, turn string, textOnly bool) {
	s.forget()
	if !textOnly {
		return // an image or an audio clip: nothing this adapter may summarise from
	}
	if text = head(text, textCap); text == "" {
		return
	}
	s.pending, s.turn = text, turn
}

// codexItemText reads a paginated user message's visible text and reports
// whether the message was text and nothing else.
//
// ok=false means the content is not the shape this adapter reads turns out of —
// not a list, or an entry with no type — which is drift rather than a record to
// skip.
func codexItemText(content json.RawMessage) (string, bool, bool) {
	if len(content) == 0 || content[0] != '[' {
		return "", false, false
	}
	var entries []struct {
		Type string `json:"type"`
		Text string `json:"text"`
	}
	if json.Unmarshal(content, &entries) != nil {
		return "", false, false
	}
	var (
		text []string
		only = len(entries) > 0
	)
	for _, e := range entries {
		if e.Type == "" {
			return "", false, false
		}
		if e.Type != "text" {
			// A local_image entry, and whatever else Codex grows: the turn is
			// no longer text-only and cannot be summarised from its text alone.
			only = false
			continue
		}
		if strings.TrimSpace(e.Text) != "" {
			text = append(text, e.Text)
		}
	}
	return strings.Join(text, "\n\n"), only, true
}

// codexMeta is what one rollout's head record says about itself: enough to
// accept or reject it as a candidate without reading a line of conversation.
type codexMeta struct {
	session string
	cwd     string
	mode    string
}

// codexSearch finds the one rollout that belongs to this live agent.
//
// A candidate is a rollout written since the agent started — an idle store
// belongs to nobody live — whose head record names this working directory, is an
// interactive thread rather than a subagent's, and was written by the TUI rather
// than by `codex exec`. That last filter is what makes chartr's own title
// generations invisible to binding: they are ordinary rollouts in the same tree
// with the same working directory, and they are excluded by the same predicate
// subagents are.
//
// One candidate is a binding; none or several is silence, re-asked on the next
// poll.
func codexSearch(agent proc.Agent) (string, codexMeta, bool) {
	var (
		found codexMeta
		path  string
		hits  int
	)
	// sessions/<YYYY>/<MM>/<DD>/. The shards are shallow and the filter that
	// matters is the file's own head record, so the walk is a directory listing
	// rather than a date computation — a rollout resumed across a midnight, or a
	// host whose local time moved, is still found where it actually is.
	for _, day := range codexDays(filepath.Join(agent.StateRoot, codexSessions)) {
		entries, err := os.ReadDir(day)
		if err != nil {
			continue
		}
		for _, entry := range entries {
			name := entry.Name()
			if entry.IsDir() || !strings.HasPrefix(name, codexPrefix) || !strings.HasSuffix(name, jsonlSuffix) {
				continue
			}
			info, err := entry.Info()
			if err != nil {
				continue
			}
			if !agent.Started.IsZero() && info.ModTime().Before(agent.Started) {
				continue
			}
			full := filepath.Join(day, name)
			meta, ok := codexHead(full)
			if !ok || !sameDir(meta.cwd, agent.Dir) {
				continue
			}
			found, path, hits = meta, full, hits+1
		}
	}
	if hits != 1 {
		return "", codexMeta{}, false
	}
	return path, found, true
}

// codexDays lists the day directories under sessions/, three levels down.
func codexDays(root string) []string {
	var days []string
	for _, year := range readSubdirs(root) {
		for _, month := range readSubdirs(year) {
			days = append(days, readSubdirs(month)...)
		}
	}
	return days
}

// readSubdirs lists a directory's subdirectories by full path, quietly returning
// nothing for a directory that is not there or not readable.
func readSubdirs(dir string) []string {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil
	}
	var out []string
	for _, entry := range entries {
		if entry.IsDir() {
			out = append(out, filepath.Join(dir, entry.Name()))
		}
	}
	return out
}

// codexHead reads a candidate's session_meta record — the first line of the file
// — and reports it when the rollout is one this adapter can read and one this
// tab could own.
//
// The sniff is here: a head record that is not session_meta, that names no
// working directory or session, or whose history_mode is absent or unknown, is
// not a store to parse on a guess. An older build wrote rollouts with no
// history_mode at all, and those fail closed rather than defaulting to either
// family.
func codexHead(path string) (codexMeta, bool) {
	var (
		meta codexMeta
		ok   bool
	)
	if !peek(path, func(line []byte) bool {
		var rec struct {
			Type    string `json:"type"`
			Payload struct {
				SessionID    string `json:"session_id"`
				Cwd          string `json:"cwd"`
				CliVersion   string `json:"cli_version"`
				HistoryMode  string `json:"history_mode"`
				Originator   string `json:"originator"`
				ThreadSource string `json:"thread_source"`
			} `json:"payload"`
		}
		if json.Unmarshal(line, &rec) != nil || rec.Type != "session_meta" {
			return false
		}
		p := rec.Payload
		if p.SessionID == "" || p.Cwd == "" || p.CliVersion == "" {
			return false
		}
		if p.HistoryMode != codexPaginated && p.HistoryMode != codexLegacy {
			return false
		}
		// A subagent thread is an ordinary rollout in the same tree with the
		// same working directory, and `codex exec` — which is how chartr's own
		// title generation runs — writes one too. Both are excluded here, by
		// the metadata rather than by anything in the conversation.
		if p.ThreadSource != "user" || p.Originator != "codex-tui" {
			return false
		}
		meta = codexMeta{session: p.SessionID, cwd: p.Cwd, mode: p.HistoryMode}
		ok = true
		return false
	}) {
		return codexMeta{}, false
	}
	return meta, ok
}
