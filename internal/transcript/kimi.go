package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/proc"
)

// This file is the Kimi Code adapter.
//
// # The store, as measured
//
// Under the resolved state root (KIMI_CODE_HOME, default ~/.kimi-code):
//
//	session_index.jsonl                       — {sessionId, sessionDir, workDir}
//	sessions/wd_<slug>_<hash12>/session_<uuid>/
//	    state.json                            — session metadata, and the title
//	    agents/main/wire.jsonl                — the conversation
//	    agents/agent-0/wire.jsonl             — a subagent's, never read
//
// Two cursors, because there are two shapes: a byte offset over the append-only
// wire log, and a stat over the rewritten state.json beside it. The bucket name
// is a slug plus a hash and is not reversible, so this adapter never computes a
// path to a session — it reads the index, and validates every path it finds
// against the root it already resolved.
//
// Subagent traffic needs no predicate at all: it lives in a sibling agents/
// directory, and reading only agents/main excludes it structurally.
//
// # What a turn is here
//
// `turn.prompt` is the operator's own submission, and its `origin.kind` is the
// human gate — a skill activation and a task both arrive on the same record with
// a different origin. The visible answer is the text content parts of the step
// that finished with `end_turn`; the earlier steps are tool calls, and `think`
// parts are hidden reasoning. `turn.ended` closes the turn and says whether it
// completed, failed or was cancelled.
//
// The prompt record carries no turn identity and the closing record does, so the
// two are tied through the steps between them — and Kimi writes that identity as
// a string in the loop events and as an integer in `turn.ended`, so both are
// normalized before they are compared.
//
// # The title is conditional, and the condition is load-bearing
//
// state.json carries {title, titleKind, isCustomTitle}. Only `generated` and
// `custom` are titles; the default `titleKind` is `replaceable`, which is the
// prompt itself truncated at 200 characters. Publishing that would put a whole
// prompt in the tab label *and*, because a native title blocks paid generation
// for the life of the session, pin the tab to it forever. Kimi's own generation
// is behind a feature flag and an OAuth provider besides, so in practice a Kimi
// tab takes the paid path.
type kimi struct{}

const (
	kimiIndex    = "session_index.jsonl"
	kimiSessions = "sessions"
	kimiAgents   = "agents"
	kimiMain     = "main"
	kimiWire     = "wire.jsonl"
	kimiState    = "state.json"

	// kimiStateCap bounds the state.json read. The real ones are a few hundred
	// bytes; anything of a different order is not this file.
	kimiStateCap = 256 << 10

	// kimiIndexCap bounds the index read. It is one line per session ever
	// opened, so it is allowed to be large — but not unbounded.
	kimiIndexCap = 8 << 20
)

// kimiProtocols is the set of wire protocol versions this adapter reads. Kimi is
// the one provider that hands chartr an explicit, versioned protocol number, so
// it is used exactly as a schema version should be: an unrecognized one ends the
// binding rather than being parsed optimistically.
var kimiProtocols = map[string]bool{"1.4": true, "1.5": true}

// kimiReasons is the closed set of turn endings. Only completed is a turn.
var kimiReasons = map[string]bool{"completed": true, "failed": true, "cancelled": true}

// Bind matches a live Kimi process to its persisted session through the index
// Kimi keeps of them. There is no process-to-session registry, so the route is
// working directory plus writes observed since the agent started: unique or
// nothing.
func (kimi) Bind(agent proc.Agent) (Session, bool) {
	if agent.Adapter != "kimi" || agent.StateRoot == "" || agent.PID <= 0 {
		return nil, false
	}
	dir, id, ok := kimiSearch(agent)
	if !ok {
		return nil, false
	}
	s := &kimiSession{id: id}
	s.store.path = filepath.Join(dir, kimiAgents, kimiMain, kimiWire)
	s.state.path = filepath.Join(dir, kimiState)
	if !s.store.seat(s.fold) {
		return nil, false
	}
	s.forget()
	return s, true
}

// kimiSession is one bound conversation: the wire log's cursor, the state
// sidecar beside it, and the one prompt held while its answer is written.
type kimiSession struct {
	store tail
	state sidecar
	id    string

	native  string
	pending string
	// turn is the identity the pending turn's steps carry, and step the visible
	// text of the step now running — replaced at every step, so only the step
	// that finishes the turn contributes an answer.
	turn string
	step []string
	// final is the text of the step that ended the turn.
	final string

	out []Event
}

func (s *kimiSession) ID() string { return s.id }

func (s *kimiSession) forget() {
	s.pending, s.turn, s.step, s.final = "", "", nil, ""
}

func (s *kimiSession) Poll() ([]Event, bool) {
	s.title()
	reseated, ok := s.store.advance(s.fold)
	if reseated {
		s.forget()
	}
	out := s.out
	s.out = nil
	return out, ok
}

// title publishes the session's own title when it has one worth publishing. It
// is the free path, so it is gated by nothing except the one filter that decides
// whether Kimi wrote a title or a copy of the prompt.
func (s *kimiSession) title() {
	if !s.state.changed() {
		return
	}
	data, ok := s.state.read(kimiStateCap)
	if !ok {
		return
	}
	var state struct {
		Title         string `json:"title"`
		TitleKind     string `json:"titleKind"`
		IsCustomTitle *bool  `json:"isCustomTitle"`
	}
	if json.Unmarshal(data, &state) != nil {
		return
	}
	switch state.TitleKind {
	case "generated", "custom":
	case "":
		// A record from before titleKind existed: a custom title is one an
		// operator set, and anything else was the prompt.
		if state.IsCustomTitle == nil || !*state.IsCustomTitle {
			return
		}
	default:
		// replaceable — the prompt itself, capped at 200 characters — and
		// whatever kind ships next.
		return
	}
	title := head(oneLine(state.Title), textCap)
	if title == "" || title == "New Session" || title == s.native {
		return
	}
	s.native = title
	s.out = append(s.out, Event{Kind: NativeTitle, Title: title})
}

// kimiRecord is one line of the wire log, narrowed to what this adapter reads.
type kimiRecord struct {
	Type            string          `json:"type"`
	ProtocolVersion string          `json:"protocol_version"`
	Input           []kimiPart      `json:"input"`
	Origin          *kimiOrigin     `json:"origin"`
	Event           *kimiEvent      `json:"event"`
	TurnID          json.RawMessage `json:"turnId"`
	Reason          string          `json:"reason"`
}

type kimiOrigin struct {
	Kind string `json:"kind"`
}

type kimiPart struct {
	Type string `json:"type"`
	Text string `json:"text"`
}

type kimiEvent struct {
	Type         string          `json:"type"`
	TurnID       json.RawMessage `json:"turnId"`
	Part         *kimiPart       `json:"part"`
	FinishReason string          `json:"finishReason"`
}

// fold folds one complete wire record in. false ends the binding.
func (s *kimiSession) fold(line []byte, history bool) bool {
	var rec kimiRecord
	if err := json.Unmarshal(line, &rec); err != nil {
		return false // malformed, or a field whose type this adapter relies on changed
	}
	if rec.Type == "" {
		return false // every record in this format is typed
	}

	switch rec.Type {
	case "metadata":
		// The head record, and the sniff: an unknown protocol version is a
		// store this adapter will not guess at.
		return kimiProtocols[rec.ProtocolVersion]

	case "turn.prompt":
		// Any submission is a new act, so whatever was pending is not going to
		// be answered. Only a human origin opens a turn: a skill activation and
		// a task arrive on this same record.
		s.forget()
		if rec.Origin == nil {
			return false // the shape a turn is read out of has drifted
		}
		if rec.Origin.Kind != "user" || len(rec.Input) == 0 {
			return true
		}
		var text []string
		for _, part := range rec.Input {
			if part.Type == "" {
				return false
			}
			if part.Type != "text" {
				return true // an image: not text-only, so nothing opens
			}
			if strings.TrimSpace(part.Text) != "" {
				text = append(text, part.Text)
			}
		}
		s.pending = head(strings.Join(text, "\n\n"), textCap)
		return true

	case "context.append_loop_event":
		if rec.Event == nil || rec.Event.Type == "" {
			return false
		}
		s.loop(rec.Event)
		return true

	case "turn.ended":
		if !kimiReasons[rec.Reason] {
			return false // an ending outside the known set
		}
		prompt, turn, final := s.pending, s.turn, s.final
		s.forget()
		ended, ok := kimiTurnID(rec.TurnID)
		if !ok {
			return false
		}
		// A turn that failed or was cancelled, one this adapter is not holding
		// the prompt of, one whose identity does not match the steps that ran,
		// and one that produced no visible text: all four end quietly.
		if history || rec.Reason != "completed" || prompt == "" || final == "" || turn != ended {
			return true
		}
		s.out = append(s.out, Event{Kind: HumanTurn, Prompt: prompt, Response: final})
		return true
	}
	// Everything else in the census is machinery: the model requests, the usage
	// records, permission and interaction bookkeeping, compaction, plugins, and
	// the mirrored context.append_message stream that carries the same prompts
	// again alongside user-role machinery. The turn.prompt stream is the clean
	// one, and reading past a kind is not a guess about it.
	return true
}

// loop folds one loop event: where the visible half of an answer is assembled.
func (s *kimiSession) loop(e *kimiEvent) {
	switch e.Type {
	case "step.begin":
		// A new step's text replaces the last one's. Only the step that ends
		// the turn contributes an answer, so intermediate tool steps leave
		// nothing behind.
		s.step = nil
		if id, ok := kimiTurnID(e.TurnID); ok && s.turn == "" {
			s.turn = id
		}
	case "content.part":
		// think parts are hidden reasoning and never cross the seam.
		if e.Part != nil && e.Part.Type == "text" && strings.TrimSpace(e.Part.Text) != "" {
			s.step = append(s.step, e.Part.Text)
		}
	case "step.end":
		if e.FinishReason == "end_turn" {
			s.final = head(strings.Join(s.step, ""), textCap)
		}
		s.step = nil
	}
	// tool.call and tool.result are the machinery of a step, and tool.result is
	// the one loop event with no turn identity at all.
}

// kimiTurnID normalizes a turn identity, which Kimi writes as a string in the
// loop events and as an integer in turn.ended. Comparing them without this would
// silently never match, and a turn that never matches is a turn that is never
// titled.
func kimiTurnID(raw json.RawMessage) (string, bool) {
	if len(raw) == 0 {
		return "", false
	}
	if raw[0] == '"' {
		var s string
		if json.Unmarshal(raw, &s) != nil {
			return "", false
		}
		return s, s != ""
	}
	var n json.Number
	if json.Unmarshal(raw, &n) != nil {
		return "", false
	}
	return n.String(), true
}

// kimiSearch finds the one session that belongs to this live agent, through the
// index Kimi appends to as it opens them.
//
// Every path the index names is validated against the root chartr already
// resolved before anything is read from it: the session directory must sit
// directly under this root's sessions/, and its name must be the session id the
// entry claims. A path out of a file is never followed on trust.
func kimiSearch(agent proc.Agent) (string, string, bool) {
	index := sidecar{path: filepath.Join(agent.StateRoot, kimiIndex)}
	data, ok := index.read(kimiIndexCap)
	if !ok {
		return "", "", false
	}
	sessions := filepath.Join(agent.StateRoot, kimiSessions)

	var (
		dir  string
		id   string
		hits int
	)
	seen := make(map[string]bool)
	for _, line := range strings.Split(string(data), "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		var entry struct {
			SessionID  string `json:"sessionId"`
			SessionDir string `json:"sessionDir"`
			WorkDir    string `json:"workDir"`
		}
		if json.Unmarshal([]byte(line), &entry) != nil {
			continue // a line this adapter cannot read is not a candidate
		}
		if entry.SessionID == "" || !sameDir(entry.WorkDir, agent.Dir) {
			continue
		}
		candidate := filepath.Clean(entry.SessionDir)
		if filepath.Dir(filepath.Dir(candidate)) != filepath.Clean(sessions) {
			continue // not a session directory under the root this process named
		}
		if filepath.Base(candidate) != entry.SessionID {
			continue
		}
		wire := filepath.Join(candidate, kimiAgents, kimiMain, kimiWire)
		info, err := os.Stat(wire)
		if err != nil || info.IsDir() {
			continue
		}
		if !agent.Started.IsZero() && info.ModTime().Before(agent.Started) {
			continue // an idle store belongs to nobody live
		}
		if seen[candidate] {
			continue // the index is append-only and may name a session twice
		}
		seen[candidate] = true
		dir, id, hits = candidate, entry.SessionID, hits+1
	}
	if hits != 1 {
		return "", "", false
	}
	return dir, id, true
}
