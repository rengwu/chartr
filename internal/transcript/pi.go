package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/proc"
)

// This file is the Pi adapter.
//
// # The store, as measured
//
// Under the resolved state root — the sessions directory, which
// PI_CODING_AGENT_SESSION_DIR names outright, PI_CODING_AGENT_DIR names the
// parent of, and which defaults to ~/.pi/agent/sessions:
//
//	<encoded-cwd>/<ISO-timestamp>_<session-uuid>.jsonl
//
// One append-only JSONL file per session, entries forming a tree through
// id/parentId. The directory name encodes the working directory in a lossy
// private scheme, so this adapter does not reimplement it: it looks under every
// directory and confirms a candidate against the working directory the session's
// own header records.
//
// # The file appears late, and that is the whole of Pi's behaviour here
//
// Pi buffers its session in memory and creates the file only once the first
// assistant message exists, writing the header, the opening prompt and that
// answer together — its own _persist opens with "wx" and dumps every buffered
// entry. There is no header-only file. From then on each entry is appended
// singly, which is what the byte-offset cursor reads.
//
// So a Pi binding always arrives after the opening prompt is already on disk,
// and under the seat-at-end rule a Pi tab's first turn is never titled: a
// single-prompt session stays untitled. That is the settled reading of "history
// stays behind the cursor" — a missing title is the cheap failure, and rescuing
// the first turn would mean deciding that an absent store counts as a bound one.
//
// # No generated title
//
// Pi has a native title, but only an operator sets it: `/name`, --name, or an
// extension calling setSessionName, all of which write a session_info entry. Pi
// never generates one — not one session in the measured corpus carried the
// entry — so in practice Pi takes the paid path, with session_info as a free
// override when an operator has named the session. The latest entry wins, so a
// rename during the session republishes.
type pi struct{}

// piVersion is the session format this adapter reads. Versions 1 and 2 exist
// historically and Pi migrates them on load; an adapter that guessed at one
// would be reading a shape nobody writes any more, so anything but 3 is refused.
const piVersion = 3

// Bind matches a live Pi process to its persisted session. Pi exposes no
// process-to-session registry, so this is the working-directory-and-observed-
// writes route: unique or nothing.
func (pi) Bind(agent proc.Agent) (Session, bool) {
	if agent.Adapter != "pi" || agent.StateRoot == "" || agent.PID <= 0 {
		return nil, false
	}
	path, id, ok := piSearch(agent)
	if !ok {
		return nil, false
	}
	s := &piSession{id: id}
	s.store.path = path
	if !s.store.seat(s.fold) {
		return nil, false
	}
	s.forget()
	return s, true
}

// piSession is one bound conversation: which file, how far it has been read, the
// name last published, and the one prompt held while its answer is written.
type piSession struct {
	store tail
	id    string

	title   string
	pending string
	out     []Event
}

func (s *piSession) ID() string { return s.id }

func (s *piSession) forget() { s.pending = "" }

func (s *piSession) Poll() ([]Event, bool) {
	reseated, ok := s.store.advance(s.fold)
	if reseated {
		s.forget()
	}
	out := s.out
	s.out = nil
	return out, ok
}

// piEntry is one line of the session file, narrowed to what this adapter reads.
type piEntry struct {
	Type    string     `json:"type"`
	Version *int       `json:"version"`
	Name    string     `json:"name"`
	Message *piMessage `json:"message"`
}

// piMessage is the message envelope: a role, content that is either a plain
// string or a list of typed blocks, and the reason generation stopped.
type piMessage struct {
	Role       string          `json:"role"`
	Content    json.RawMessage `json:"content"`
	StopReason string          `json:"stopReason"`
}

// piRoles is the closed set of message roles Pi writes. Only two of them are
// conversation; the rest are tool results, shell output, extension state and
// summaries, which are read past. A role outside the set is a shape from a later
// version, and this adapter does not parse on a guess.
var piRoles = map[string]bool{
	"user": true, "assistant": true, "toolResult": true,
	"bashExecution": true, "custom": true,
	"branchSummary": true, "compactionSummary": true,
}

// piStopReasons is the closed union Pi documents. toolUse continues the turn;
// error, aborted and length end it with nothing to show.
var piStopReasons = map[string]bool{
	"stop": true, "length": true, "toolUse": true, "error": true, "aborted": true,
}

// fold folds one complete entry in. false ends the binding.
func (s *piSession) fold(line []byte, history bool) bool {
	var e piEntry
	if err := json.Unmarshal(line, &e); err != nil {
		return false // malformed, or a field whose type this adapter relies on changed
	}
	if e.Type == "" {
		return false // every entry in this format is typed
	}

	switch e.Type {
	case "session":
		// The header, which Pi's own loader refuses to read a file without.
		return e.Version != nil && *e.Version == piVersion

	case "session_info":
		// The operator's own name for the session. The latest entry wins, so a
		// rename during the session publishes again — for free, like any native
		// title.
		title := head(oneLine(e.Name), textCap)
		if title == "" || title == s.title {
			return true
		}
		s.title = title
		s.out = append(s.out, Event{Kind: NativeTitle, Title: title})
		return true

	case "message":
		if e.Message == nil || !piRoles[e.Message.Role] {
			return false // the shape a turn is read out of has drifted
		}
		return s.message(e.Message, history)
	}
	// model_change, thinking_level_change, compaction, branch_summary, label,
	// custom and the extension-injected custom_message are all machinery.
	// Reading past a kind is not a guess about it.
	return true
}

// message folds one message entry. It is where the two roles that are
// conversation are told apart from the five that are not.
func (s *piSession) message(m *piMessage, history bool) bool {
	switch m.Role {
	case "user":
		text, only, ok := piContent(m.Content)
		if !ok {
			return false
		}
		// Any user message is a new act by whoever is at the keyboard, so
		// whatever was pending is not going to be answered. A message carrying
		// an image is not text-only and opens nothing.
		s.forget()
		if only {
			s.pending = head(text, textCap)
		}
		return true

	case "assistant":
		if !piStopReasons[m.StopReason] {
			return false // a reason outside the documented union
		}
		text, _, ok := piContent(m.Content)
		if !ok {
			return false
		}
		if m.StopReason == "toolUse" {
			// The turn continues: the assistant stopped to call a tool, and the
			// answer is still to come.
			return true
		}
		prompt, answer := s.pending, head(text, textCap)
		s.forget()
		// length, error and aborted all end the turn with nothing an operator
		// would recognise as an answer, and a stop with no visible text is the
		// same.
		if history || m.StopReason != "stop" || prompt == "" || answer == "" {
			return true
		}
		s.out = append(s.out, Event{Kind: HumanTurn, Prompt: prompt, Response: answer})
		return true
	}
	// A tool result, shell output, extension state or a summary. None of them is
	// the operator speaking, and none interrupts the turn it lands inside.
	return true
}

// piContent reads a message's visible text and reports whether it was text and
// nothing else. A bare string is the documented shorthand for one text block;
// thinking and toolCall blocks are hidden and never cross the seam.
//
// ok=false means the content is not a shape this adapter reads turns out of.
func piContent(content json.RawMessage) (string, bool, bool) {
	if len(content) == 0 {
		return "", false, false
	}
	switch content[0] {
	case '"':
		var text string
		if json.Unmarshal(content, &text) != nil {
			return "", false, false
		}
		return text, true, true
	case '[':
		var blocks []struct {
			Type string `json:"type"`
			Text string `json:"text"`
		}
		if json.Unmarshal(content, &blocks) != nil {
			return "", false, false
		}
		var (
			text []string
			only = len(blocks) > 0
		)
		for _, b := range blocks {
			if b.Type == "" {
				return "", false, false
			}
			switch b.Type {
			case "text":
				if strings.TrimSpace(b.Text) != "" {
					text = append(text, b.Text)
				}
			case "thinking", "toolCall":
				// Hidden reasoning and tool calls do not disqualify an
				// assistant's answer — they are simply not part of it. On a user
				// message neither appears.
			default:
				// An image, an attachment, or whatever ships next: the message
				// is no longer text-only.
				only = false
			}
		}
		return strings.Join(text, "\n\n"), only, true
	}
	return "", false, false
}

// piSearch finds the one session file that belongs to this live agent: a file
// written since the agent started, under any of the encoded working-directory
// buckets, whose own header names this working directory and the format version
// this adapter reads.
func piSearch(agent proc.Agent) (string, string, bool) {
	var (
		path string
		id   string
		hits int
	)
	for _, bucket := range readSubdirs(agent.StateRoot) {
		entries, err := os.ReadDir(bucket)
		if err != nil {
			continue
		}
		for _, entry := range entries {
			name := entry.Name()
			if entry.IsDir() || !strings.HasSuffix(name, jsonlSuffix) {
				continue
			}
			info, err := entry.Info()
			if err != nil {
				continue
			}
			if !agent.Started.IsZero() && info.ModTime().Before(agent.Started) {
				continue
			}
			full := filepath.Join(bucket, name)
			cwd, session, ok := piHead(full)
			if !ok || !sameDir(cwd, agent.Dir) {
				continue
			}
			path, id, hits = full, session, hits+1
		}
	}
	if hits != 1 {
		return "", "", false
	}
	return path, id, true
}

// piHead reads a candidate's header — always the first line — for the working
// directory and session identity discovery needs. It is also the sniff: a file
// whose head is not a version-3 session header is not one this adapter reads.
func piHead(path string) (string, string, bool) {
	var (
		cwd, id string
		ok      bool
	)
	if !peek(path, func(line []byte) bool {
		var rec struct {
			Type    string `json:"type"`
			Version *int   `json:"version"`
			ID      string `json:"id"`
			Cwd     string `json:"cwd"`
		}
		if json.Unmarshal(line, &rec) != nil || rec.Type != "session" {
			return false
		}
		if rec.Version == nil || *rec.Version != piVersion || rec.ID == "" || rec.Cwd == "" {
			return false
		}
		cwd, id, ok = rec.Cwd, rec.ID, true
		return false
	}) {
		return "", "", false
	}
	return cwd, id, ok
}
