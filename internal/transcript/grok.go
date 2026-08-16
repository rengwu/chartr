package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/proc"
)

// This file is the Grok adapter.
//
// # The store, as measured
//
// Under the resolved state root (GROK_HOME, default ~/.grok):
//
//	sessions/<percent-encoded-cwd>/<session-uuid>/
//	    summary.json    — session metadata, and the generated title
//	    updates.jsonl   — the conversation, as ACP session-update events
//	    chat_history.jsonl — the raw model wire, never read
//	    subagents/      — child sessions, never read
//
// The same two-cursor shape as Kimi: a byte offset over the append-only log, and
// a stat over the rewritten JSON beside it. Grok's own documentation is explicit
// that "JSONL is the source of truth for session content"; chat_history.jsonl is
// the direct analogue of Codex's model-wire stream and events.jsonl is telemetry,
// so neither is a place a turn may be read out of.
//
// Keying on updates.jsonl has a useful side effect: a headless `grok -p` run —
// which is how chartr's own title generation runs — writes chat_history.jsonl and
// no updates.jsonl at all, so a generation can never become a binding candidate.
//
// # Turns arrive in chunks
//
// A user message and an assistant answer are each streamed as a run of
// `*_message_chunk` updates that have to be concatenated, and the closing
// `turn_completed` names its own prompt while the user chunks name a prompt
// index. The two are different keys, so the linkage is positional — which an
// in-order tail of an append-only log makes unambiguous — and the reader tracks
// where in a turn it is rather than trying to join on an identity that is not
// shared.
//
// # A real native title
//
// summary.json's `generated_title` is a model-generated title, and Grok writes
// one reliably once there is a conversation to title: every measured session with
// seven or more messages had one, and every session without one had two or fewer.
// `/rename` sets it by hand as well, so it can change and each change is
// published. The key is *absent* before generation rather than empty, and the
// older summary shape does not carry it at all — both are "no title yet" rather
// than a failure.
type grok struct{}

const (
	grokSessions = "sessions"
	grokUpdates  = "updates.jsonl"
	grokSummary  = "summary.json"

	// grokSummaryCap bounds the summary.json read. The real ones are a couple of
	// kilobytes; anything of a different order is not this file.
	grokSummaryCap = 1 << 20

	// grokFormat is the chat format version this adapter reads. Grok stamps
	// every session with one, so it is used exactly as a schema version should
	// be: an unknown value ends the binding.
	grokFormat = 1
)

// Bind matches a live Grok process to its persisted session. Grok exposes no
// process-to-session registry — the active_sessions.json that looks like one is a
// leftover from an earlier build and its name does not appear in this one — so
// the route is working directory plus writes observed since the agent started.
func (grok) Bind(agent proc.Agent) (Session, bool) {
	if agent.Adapter != "grok" || agent.StateRoot == "" || agent.PID <= 0 {
		return nil, false
	}
	dir, id, ok := grokSearch(agent)
	if !ok {
		return nil, false
	}
	s := &grokSession{id: id}
	s.store.path = filepath.Join(dir, grokUpdates)
	s.summary.path = filepath.Join(dir, grokSummary)
	if !s.store.seat(s.fold) {
		return nil, false
	}
	s.forget()
	return s, true
}

// grokPhase is where in a turn the reader is. It exists because Grok's records
// carry no shared turn identity: what separates one turn from the next is that
// the operator's chunks stop and the assistant's begin.
type grokPhase int

const (
	grokIdle grokPhase = iota
	grokPrompting
	grokAnswering
)

// grokSession is one bound conversation: the update log's cursor, the summary
// sidecar beside it, and the turn now being assembled out of chunks.
type grokSession struct {
	store   tail
	summary sidecar
	id      string

	native string

	phase  grokPhase
	index  int
	seen   bool // whether index holds a prompt index the provider actually sent
	prompt []string
	answer []string
	// text is false once a chunk arrives that is not text: an image or an
	// attachment, which disqualifies the turn.
	text bool

	out []Event
}

func (s *grokSession) ID() string { return s.id }

func (s *grokSession) forget() {
	s.phase, s.index, s.seen, s.prompt, s.answer, s.text = grokIdle, 0, false, nil, nil, true
}

func (s *grokSession) Poll() ([]Event, bool) {
	s.title()
	reseated, ok := s.store.advance(s.fold)
	if reseated {
		s.forget()
	}
	out := s.out
	s.out = nil
	return out, ok
}

// title publishes the session's own generated title, and again whenever it
// changes — a rename during the session is as free as the first one.
func (s *grokSession) title() {
	if !s.summary.changed() {
		return
	}
	data, ok := s.summary.read(grokSummaryCap)
	if !ok {
		return
	}
	var summary struct {
		GeneratedTitle string `json:"generated_title"`
	}
	if json.Unmarshal(data, &summary) != nil {
		return
	}
	// Absent before generation, and absent entirely in the older summary shape.
	// Both are "no title yet", not a failure.
	title := head(oneLine(summary.GeneratedTitle), textCap)
	if title == "" || title == s.native {
		return
	}
	s.native = title
	s.out = append(s.out, Event{Kind: NativeTitle, Title: title})
}

// grokRecord is one line of the update log, narrowed to what this adapter reads.
type grokRecord struct {
	Method string      `json:"method"`
	Params *grokParams `json:"params"`
}

type grokParams struct {
	SessionID string      `json:"sessionId"`
	Update    *grokUpdate `json:"update"`
}

type grokUpdate struct {
	SessionUpdate string       `json:"sessionUpdate"`
	Content       *grokContent `json:"content"`
	Meta          *struct {
		PromptIndex *int `json:"promptIndex"`
	} `json:"_meta"`
	StopReason string `json:"stop_reason"`
	PromptID   string `json:"prompt_id"`
}

type grokContent struct {
	Type string `json:"type"`
	Text string `json:"text"`
}

// fold folds one complete update in. false ends the binding.
func (s *grokSession) fold(line []byte, history bool) bool {
	var rec grokRecord
	if err := json.Unmarshal(line, &rec); err != nil {
		return false // malformed, or a field whose type this adapter relies on changed
	}
	if rec.Method == "" {
		return false // every record in this format names a method
	}
	// `session/update` is the ACP stream and `_x.ai/session/update` its vendor
	// extension; anything else is not conversation.
	if !strings.HasSuffix(rec.Method, "session/update") {
		return true
	}
	if rec.Params == nil || rec.Params.Update == nil || rec.Params.Update.SessionUpdate == "" {
		return false // the shape a turn is read out of has drifted
	}
	u := rec.Params.Update

	switch u.SessionUpdate {
	case "user_message_chunk":
		index, seen := 0, false
		if u.Meta != nil && u.Meta.PromptIndex != nil {
			index, seen = *u.Meta.PromptIndex, true
		}
		// A user chunk that is not continuing the prompt already being read
		// starts a new turn, and whatever was being assembled is abandoned: the
		// operator has moved on.
		if s.phase != grokPrompting || (seen && s.seen && index != s.index) {
			s.forget()
			s.phase, s.index, s.seen = grokPrompting, index, seen
		}
		text, ok := grokChunk(u.Content)
		if !ok {
			return false
		}
		if text == "" {
			// A chunk that is not text at all: an image or an attachment, and
			// the turn is no longer something to summarise from text alone.
			s.text = false
			return true
		}
		s.prompt = append(s.prompt, text)
		return true

	case "agent_message_chunk":
		if s.phase == grokIdle {
			return true // an answer to a turn that began before the cursor
		}
		s.phase = grokAnswering
		text, ok := grokChunk(u.Content)
		if !ok {
			return false
		}
		s.answer = append(s.answer, text)
		return true

	case "turn_completed":
		prompt, answer, textOnly := strings.Join(s.prompt, ""), strings.Join(s.answer, ""), s.text
		phase := s.phase
		s.forget()
		if history || phase != grokAnswering || u.StopReason != "end_turn" || !textOnly {
			return true
		}
		prompt, answer = head(prompt, textCap), head(answer, textCap)
		if prompt == "" || answer == "" {
			return true
		}
		s.out = append(s.out, Event{Kind: HumanTurn, Prompt: prompt, Response: answer})
		return true
	}
	// agent_thought_chunk is reasoning, tool_call and tool_call_update are tool
	// traffic, and new update kinds are ordinary. Reading past a kind is not a
	// guess about it.
	return true
}

// grokChunk reads one chunk's visible text. An empty string with ok=true means
// the chunk carried something that is not text; ok=false means the chunk is not
// the shape this adapter reads content out of.
func grokChunk(c *grokContent) (string, bool) {
	if c == nil || c.Type == "" {
		return "", false
	}
	if c.Type != "text" {
		return "", true
	}
	return c.Text, true
}

// grokSearch finds the one session that belongs to this live agent: a session
// directory under any working-directory bucket whose update log was written since
// the agent started, and whose summary names this working directory and a chat
// format this adapter reads.
//
// A session with no updates.jsonl is not a candidate at all, which is what
// excludes both `grok -p` runs and the stub sessions that make up the bulk of a
// real store.
func grokSearch(agent proc.Agent) (string, string, bool) {
	var (
		dir  string
		id   string
		hits int
	)
	for _, bucket := range readSubdirs(filepath.Join(agent.StateRoot, grokSessions)) {
		for _, session := range readSubdirs(bucket) {
			info, err := os.Stat(filepath.Join(session, grokUpdates))
			if err != nil || info.IsDir() {
				continue
			}
			if !agent.Started.IsZero() && info.ModTime().Before(agent.Started) {
				continue // an idle store belongs to nobody live
			}
			cwd, found, ok := grokHead(filepath.Join(session, grokSummary))
			if !ok || !sameDir(cwd, agent.Dir) {
				continue
			}
			dir, id, hits = session, found, hits+1
		}
	}
	if hits != 1 {
		return "", "", false
	}
	return dir, id, true
}

// grokHead reads a candidate's summary for the working directory and identity
// discovery needs. It is also the sniff: an unknown chat format version, a
// missing identity, and a session that is a fork or a subagent's child are all
// refused.
func grokHead(path string) (string, string, bool) {
	file := sidecar{path: path}
	data, ok := file.read(grokSummaryCap)
	if !ok {
		return "", "", false
	}
	var summary struct {
		Info struct {
			ID  string `json:"id"`
			Cwd string `json:"cwd"`
		} `json:"info"`
		ChatFormatVersion *int `json:"chat_format_version"`
	}
	if json.Unmarshal(data, &summary) != nil {
		return "", "", false
	}
	if summary.ChatFormatVersion == nil || *summary.ChatFormatVersion != grokFormat {
		return "", "", false
	}
	if summary.Info.ID == "" || summary.Info.Cwd == "" {
		return "", "", false
	}
	return summary.Info.Cwd, summary.Info.ID, true
}
