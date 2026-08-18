package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/proc"
)

// This file is the Claude Code adapter, and the first proof that the contract
// above is a contract rather than a description of one provider.
//
// # The store, as measured
//
// Under the resolved state root (CLAUDE_CONFIG_DIR, default ~/.claude):
//
//   - projects/<encoded-cwd>/<session-uuid>.jsonl — one append-only JSONL file
//     per session, one JSON object per line. Records carry a "type", and the
//     ones that matter are "user" (the operator's own turns *and* every tool
//     result, which are both written as user messages), "assistant" (reasoning,
//     tool calls and visible answers), and "ai-title" (the provider's own title
//     for the conversation).
//   - sessions/<pid>.json — a live process-to-session registry: the pid, the
//     session uuid it is driving, its working directory, and when the session
//     started. This is the direct mapping the specification says to prefer, and
//     it is what lets two Claude tabs in one directory each find their own
//     conversation instead of both finding neither.
//
// The directory under projects/ encodes the working directory in a scheme this
// adapter deliberately does not reimplement — a private encoding is exactly the
// kind of thing that changes quietly. A session uuid is unique across the whole
// tree, so the file is found by looking for that uuid under any project
// directory, and confirmed against the working directory recorded inside it.
//
// # What is a turn here
//
// A top-level human turn is a "user" record that Claude itself marked as typed
// by a human (promptSource "typed", origin.kind "human"), is not a sidechain
// (subagent traffic), not meta (caveats, image placeholders, context readouts),
// carries no tool result, and whose content is text only. Everything the machine
// writes in the user role — tool results, slash-command envelopes,
// interruption notices, local-command output — fails at least one of those, and
// an adapter that instead guessed from "content that looks like a sentence"
// would hand chartr the agent's own machinery as if the operator had said it.
//
// The answer to that turn is the next assistant record that *finishes*
// (stop_reason "end_turn") and has visible text. An assistant record that stops
// to call a tool has not answered yet; an interrupted or errored turn never
// produces one, and is skipped by simply never completing.
//
// # Fail closed, quietly
//
// A complete line that is not JSON, a record with no type, or a user/assistant
// record whose message is not the shape this adapter knows all end the binding.
// Reading past them would mean guessing, and a guess here is the operator's
// money spent on their own tool output. Unknown *record kinds*, on the other
// hand, are ordinary: Claude adds them regularly, and ignoring a kind is not a
// guess about its contents.
type claude struct{}

const (
	claudeProjects = "projects"
	claudeSessions = "sessions"

	// claudeRegistryCap bounds the registry entry read into memory. The real
	// ones are a few hundred bytes; anything of a different order is not the
	// file this adapter is looking at.
	claudeRegistryCap = 64 << 10

	// claudeRegistrySlack absorbs the clock granularity between a kernel's
	// process start time and the provider's own millisecond timestamp for the
	// session it then opened.
	claudeRegistrySlack = 2 * time.Second
)

// Bind matches a live Claude process to its persisted session: by the registry
// where it exists, and otherwise by working directory and observed writes.
func (claude) Bind(agent proc.Agent) (Session, bool) {
	if agent.Adapter != "claude" || agent.StateRoot == "" || agent.PID <= 0 {
		return nil, false
	}
	s := &claudeSession{root: agent.StateRoot, dir: agent.Dir}
	if reg, id, ok := claudeRegistered(agent); ok {
		s.reg, s.id = reg, id
	} else if id, path, found := claudeSearch(agent); found {
		s.id, s.store.path = id, path
	} else {
		return nil, false
	}
	if !s.open() {
		return nil, false
	}
	s.staged = s.drain()
	return s, true
}

// claudeSession is one bound conversation: which session, which file, how far it
// has been read, and the title it last published. It holds one prompt at a time
// while the answer to it is still being written, and no other conversation
// content.
type claudeSession struct {
	root string
	dir  string
	id   string

	// store is the byte-offset cursor over the transcript. Its path is empty
	// until the file exists: a tab chartr launched is bound during the agent's
	// startup, before its session has been written to at all — the end of an
	// unwritten transcript is offset zero, which is why such a tab sees its own
	// opening turn while a resumed one does not.
	store tail

	// reg is the registry entry this binding came from, watched for the moment
	// the process moves to another session; nil when the binding was made by
	// searching instead.
	reg *claudeRegistry

	title   string
	pending string
	active  bool

	// staged holds what binding already read, until the first Poll asks for it;
	// out is what the fold has produced during the poll in progress.
	staged []Event
	out    []Event
}

func (s *claudeSession) ID() string { return s.id }

// open seats the cursor at the end of the transcript as it stands, returning
// only what history is allowed to say: the provider's own title. No historical
// turn becomes an event, which is the whole of why a resumed session cannot be
// charged for work already done.
func (s *claudeSession) open() bool {
	if s.store.path == "" {
		path, ok := claudeFile(s.root, s.id, s.dir)
		if !ok {
			return false // several files claim this session: no binding
		}
		if path == "" {
			return true // nothing written yet; the cursor stays at zero
		}
		s.store.path = path
	}
	if !s.store.seat(s.fold) {
		return false
	}
	s.forget()
	return true
}

// forget drops the turn the fold was holding open. A turn already under way when
// the cursor was seated stays behind it whole: its prompt was persisted before
// binding, so the answer that lands next is not chartr's to charge for. This is
// the rule that keeps a manually started agent's launch-command prompt untitled
// rather than rescued by comparing timestamps.
func (s *claudeSession) forget() { s.pending, s.active = "", false }

// drain takes the events the fold produced.
func (s *claudeSession) drain() []Event {
	out := s.out
	s.out = nil
	return out
}

// Poll reads whatever has been appended since the last call.
func (s *claudeSession) Poll() ([]Event, bool) {
	out := s.staged
	s.staged = nil

	// A process that moved to another session — /clear, a resume into a new
	// conversation — is no longer the session this binding names. Ending the
	// binding lets the watcher bind the new one, seated at its end.
	if s.reg != nil && s.reg.changed() {
		if id, ok := s.reg.session(); ok && id != s.id {
			return out, false
		}
	}

	if s.store.path == "" {
		path, ok := claudeFile(s.root, s.id, s.dir)
		if !ok {
			return out, false
		}
		if path == "" {
			return out, true // still nothing written
		}
		// The file appeared after this binding was seated, so everything in it
		// is this agent's own work: read it from the start.
		s.store.path = path
	}

	reseated, ok := s.store.advance(s.fold)
	if reseated {
		s.forget()
	}
	return append(out, s.drain()...), ok
}

// fold folds one complete JSONL record in, producing the events it stands for —
// almost always none. false is the fail-closed answer: this is not a store this
// adapter can read, and the binding ends.
func (s *claudeSession) fold(line []byte, history bool) bool {
	var rec claudeRecord
	if err := json.Unmarshal(line, &rec); err != nil {
		return false // malformed, or a field whose type this adapter relies on changed
	}
	if rec.Type == "" {
		return false // every record in this format is typed
	}

	switch rec.Type {
	case "ai-title":
		title := head(oneLine(rec.AITitle), textCap)
		if title == "" || title == s.title {
			return true
		}
		s.title = title
		s.out = append(s.out, Event{Kind: NativeTitle, Title: title})
		return true

	case "user":
		text, kinds, ok := claudeContent(rec.Message)
		if !ok {
			return false // the shape a human turn is read out of has drifted
		}
		if rec.IsSidechain || rec.IsMeta || kinds.has("tool_result") || rec.ToolUseResult != nil {
			// Subagent traffic, tool results, and the notes Claude writes to
			// itself in the user role — caveats, image placeholders, context
			// readouts — are not the operator speaking, and none of them
			// interrupts the turn they land inside.
			return true
		}
		// Any other user-role record is a new act by whoever is at the keyboard:
		// the next prompt, a slash command, an interruption notice. Whatever was
		// pending is not going to be answered.
		s.forget()
		if claudeTyped(rec) {
			s.active = true
			if kinds.only("text") {
				s.pending = head(text, textCap)
			}
			if !history {
				s.out = append(s.out, Event{Kind: TurnStarted, Prompt: s.pending})
			}
		}
		return true

	case "assistant":
		text, kinds, ok := claudeContent(rec.Message)
		if !ok {
			return false
		}
		if rec.IsSidechain || rec.Message.stopReason() != "end_turn" || !kinds.has("text") {
			// A subagent, a stop to call a tool, or a finished record with
			// nothing visible in it — Claude writes the reasoning it ended on as
			// its own finished record and the answer in the next one. None of
			// these ends the pending turn; only something visible does.
			return true
		}
		prompt, active := s.pending, s.active
		s.forget()
		if history || !active {
			return true
		}
		answer := ""
		if kinds.has("text") {
			answer = head(text, textCap)
		}
		s.out = append(s.out, Event{Kind: TurnFinished, Outcome: OutcomeCompleted,
			Prompt: prompt, Response: answer})
		return true
	}
	// Every other kind is machinery — modes, permission choices, attachments,
	// file snapshots, queue operations, summaries, whatever ships next. Reading
	// past a kind is not a guess about it.
	return true
}

// claudeRecord is the envelope every record shares, narrowed to the fields this
// adapter needs. Fields it does not name are ignored, so a new one is not a
// schema break; the ones it does name are its sniff, and a type change in any of
// them makes the store unreadable rather than reinterpreted.
type claudeRecord struct {
	Type         string          `json:"type"`
	AITitle      string          `json:"aiTitle"`
	IsSidechain  bool            `json:"isSidechain"`
	IsMeta       bool            `json:"isMeta"`
	PromptSource string          `json:"promptSource"`
	Origin       json.RawMessage `json:"origin"`
	// ToolUseResult marks a user record that carries a tool's output. Raw
	// because its shape is a tool's business and this adapter only asks whether
	// it is there.
	ToolUseResult json.RawMessage `json:"toolUseResult"`
	Message       *claudeMessage  `json:"message"`
}

// claudeMessage is the provider's message envelope: a role, content that is
// either a plain string or a list of typed blocks, and the reason generation
// stopped.
type claudeMessage struct {
	Role       string          `json:"role"`
	Content    json.RawMessage `json:"content"`
	StopReason *string         `json:"stop_reason"`
}

func (m *claudeMessage) stopReason() string {
	if m == nil || m.StopReason == nil {
		return ""
	}
	return *m.StopReason
}

// claudeTyped reports whether Claude itself attributed this record to a human at
// the keyboard. Both marks are required: chartr's own opening prompt arrives as
// a typed human turn (it is submitted as the process's argument, which the
// provider records exactly as it records typing), while its machinery's
// user-role records carry neither.
func claudeTyped(rec claudeRecord) bool {
	if rec.PromptSource != "typed" {
		return false
	}
	var origin struct {
		Kind string `json:"kind"`
	}
	if len(rec.Origin) == 0 || json.Unmarshal(rec.Origin, &origin) != nil {
		return false
	}
	return origin.Kind == "human"
}

// blockKinds is the set of content-block types one message carried.
type blockKinds []string

func (k blockKinds) has(kind string) bool {
	for _, got := range k {
		if got == kind {
			return true
		}
	}
	return false
}

// only reports whether the message was made of nothing but kind — the test that
// keeps an image, an attachment or an opaque block out of a turn chartr would
// otherwise summarise from its text alone.
func (k blockKinds) only(kind string) bool {
	if len(k) == 0 {
		return false
	}
	for _, got := range k {
		if got != kind {
			return false
		}
	}
	return true
}

// claudeContent reads a message's visible text and the kinds of block it was
// made of. A bare string is the provider's shorthand for one text block.
//
// ok=false means the message envelope is not the shape this adapter reads turns
// out of — missing, roleless, or content that is neither a string nor a list of
// typed blocks — which is drift rather than a record to skip.
func claudeContent(m *claudeMessage) (string, blockKinds, bool) {
	if m == nil || m.Role == "" || len(m.Content) == 0 {
		return "", nil, false
	}
	switch m.Content[0] {
	case '"':
		var text string
		if json.Unmarshal(m.Content, &text) != nil {
			return "", nil, false
		}
		return text, blockKinds{"text"}, true
	case '[':
		var blocks []struct {
			Type string `json:"type"`
			Text string `json:"text"`
		}
		if json.Unmarshal(m.Content, &blocks) != nil {
			return "", nil, false
		}
		var (
			kinds blockKinds
			text  []string
		)
		for _, b := range blocks {
			if b.Type == "" {
				return "", nil, false
			}
			kinds = append(kinds, b.Type)
			if b.Type == "text" && strings.TrimSpace(b.Text) != "" {
				text = append(text, b.Text)
			}
		}
		return strings.Join(text, "\n\n"), kinds, true
	}
	return "", nil, false
}

// claudeRegistry is the live process-to-session mapping for one pid, watched for
// the moment it names a different session. It is a rewritten JSON file like
// Kimi's and Grok's title sidecars, and is polled the same way.
type claudeRegistry struct {
	agent proc.Agent
	sidecar
}

// claudeRegistered reads the registry entry for an agent's pid and returns the
// session it is driving, when the entry is this process's and not a stale one
// left by whoever held the pid before.
func claudeRegistered(agent proc.Agent) (*claudeRegistry, string, bool) {
	reg := &claudeRegistry{agent: agent}
	reg.path = filepath.Join(agent.StateRoot, claudeSessions, strconv.Itoa(agent.PID)+".json")
	reg.changed()
	id, ok := reg.session()
	return reg, id, ok
}

// session reads the entry and returns the session uuid it names for this agent.
//
// Three things must agree before that mapping is trusted: the entry is for this
// pid, it is working in the same directory the process is, and the session it
// names began after the process did. The last is the pid-reuse guard — an entry
// left behind by an earlier holder of this pid describes a session that started
// before this process existed, and would otherwise hand a tab a stranger's
// conversation.
func (r *claudeRegistry) session() (string, bool) {
	data, ok := r.read(claudeRegistryCap)
	if !ok {
		return "", false
	}
	var entry struct {
		PID       int    `json:"pid"`
		SessionID string `json:"sessionId"`
		Cwd       string `json:"cwd"`
		StartedAt int64  `json:"startedAt"`
	}
	if json.Unmarshal(data, &entry) != nil {
		return "", false
	}
	if entry.PID != r.agent.PID || !isUUID(entry.SessionID) || entry.StartedAt <= 0 {
		return "", false
	}
	if !sameDir(entry.Cwd, r.agent.Dir) {
		return "", false
	}
	if !r.agent.Started.IsZero() &&
		time.UnixMilli(entry.StartedAt).Before(r.agent.Started.Add(-claudeRegistrySlack)) {
		return "", false
	}
	return entry.SessionID, true
}

// claudeFile finds the transcript for a session uuid. The path is built from
// validated identity — a uuid this adapter checked the shape of, under the state
// root the process itself named — never from a path read out of a record.
//
// It returns ("", true) when no file for that session exists yet, which is the
// ordinary state of a session bound during its agent's startup, and ("", false)
// when the answer is not unique.
func claudeFile(root, id, dir string) (string, bool) {
	if !isUUID(id) {
		return "", false
	}
	matches, err := filepath.Glob(filepath.Join(root, claudeProjects, "*", id+jsonlSuffix))
	if err != nil {
		return "", false
	}
	switch len(matches) {
	case 0:
		return "", true
	case 1:
		return matches[0], true
	}
	// One uuid under two project trees. Keep the one whose own records say it
	// was written in this process's directory, and bind nothing if that is still
	// not a single answer.
	var keep []string
	for _, path := range matches {
		if cwd, _, ok := claudeHead(path); ok && sameDir(cwd, dir) {
			keep = append(keep, path)
		}
	}
	if len(keep) != 1 {
		return "", false
	}
	return keep[0], true
}

// claudeSearch is binding without the registry — an older provider build, or an
// entry that does not describe this process. A candidate is a session file
// written since the agent started (an idle store belongs to nobody live) whose
// own head records name this working directory. One candidate is a binding;
// none or several is silence, re-asked on the next poll.
func claudeSearch(agent proc.Agent) (string, string, bool) {
	projects := filepath.Join(agent.StateRoot, claudeProjects)
	trees, err := os.ReadDir(projects)
	if err != nil {
		return "", "", false
	}
	var (
		id   string
		path string
		hits int
	)
	for _, tree := range trees {
		if !tree.IsDir() {
			continue
		}
		entries, err := os.ReadDir(filepath.Join(projects, tree.Name()))
		if err != nil {
			continue
		}
		for _, entry := range entries {
			name := entry.Name()
			if entry.IsDir() || !strings.HasSuffix(name, jsonlSuffix) {
				continue
			}
			candidate := strings.TrimSuffix(name, jsonlSuffix)
			if !isUUID(candidate) {
				continue
			}
			info, err := entry.Info()
			if err != nil {
				continue
			}
			if !agent.Started.IsZero() && info.ModTime().Before(agent.Started) {
				continue
			}
			full := filepath.Join(projects, tree.Name(), name)
			cwd, session, ok := claudeHead(full)
			if !ok || !sameDir(cwd, agent.Dir) || (session != "" && session != candidate) {
				continue
			}
			id, path, hits = candidate, full, hits+1
		}
	}
	if hits != 1 {
		return "", "", false
	}
	return id, path, true
}

// claudeHead peeks at a candidate's opening records for the two pieces of
// metadata discovery needs — the working directory it was written in and the
// session it belongs to — and stops as soon as it has them. A session opens with
// a few records that carry neither, so the peek reads a little way in; it never
// reads the conversation, and nothing it finds becomes an event.
func claudeHead(path string) (string, string, bool) {
	var cwd, session string
	bad := false
	ok := peek(path, func(line []byte) bool {
		var rec struct {
			Cwd       string `json:"cwd"`
			SessionID string `json:"sessionId"`
		}
		if json.Unmarshal(line, &rec) != nil {
			bad = true
			return false
		}
		if cwd == "" {
			cwd = rec.Cwd
		}
		if session == "" {
			session = rec.SessionID
		}
		return cwd == "" || session == ""
	})
	if !ok || bad {
		return "", "", false
	}
	return cwd, session, cwd != ""
}

// oneLine flattens a provider title onto the cockpit's single-line contract; the
// length contract is the title generator's own cleaning, which every displayed
// title already goes through.
func oneLine(s string) string {
	return strings.Join(strings.Fields(s), " ")
}

// isUUID reports whether s is a canonical 8-4-4-4-12 hex uuid. It is what makes
// a session identifier safe to build a path out of: nothing that passes contains
// a separator, a dot, or a glob metacharacter.
func isUUID(s string) bool {
	if len(s) != 36 {
		return false
	}
	for i := 0; i < len(s); i++ {
		c := s[i]
		if i == 8 || i == 13 || i == 18 || i == 23 {
			if c != '-' {
				return false
			}
			continue
		}
		switch {
		case c >= '0' && c <= '9', c >= 'a' && c <= 'f', c >= 'A' && c <= 'F':
		default:
			return false
		}
	}
	return true
}

// sameDir reports whether two paths name the same directory, resolving symlinks
// when the plain comparison fails — a process's own working directory is already
// resolved by the kernel, while a path a provider recorded may still run through
// one (/tmp on macOS being the everyday example).
func sameDir(a, b string) bool {
	if a == "" || b == "" {
		return false
	}
	if filepath.Clean(a) == filepath.Clean(b) {
		return true
	}
	ra, aerr := filepath.EvalSymlinks(a)
	rb, berr := filepath.EvalSymlinks(b)
	return aerr == nil && berr == nil && ra == rb
}
