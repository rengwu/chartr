package transcript

import (
	"encoding/json"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/rengwu/chartr/internal/proc"
	"github.com/rengwu/chartr/internal/sqlite"
)

// This file is the OpenCode adapter, and the one place in this package where the
// store is not a file to tail.
//
// # The store, as measured
//
// Under the resolved state root (XDG_DATA_HOME + /opencode, default
// ~/.local/share/opencode): a single SQLite database, opencode.db, under WAL,
// with the agent holding it open and writing to it while chartr reads.
//
//	session(id, project_id, parent_id, slug, directory, title, version, …)
//	message(id, session_id, time_created, time_updated, data)
//	part(id, message_id, session_id, time_created, time_updated, data)
//
// The reading is read-only and incremental, over rowid — see internal/sqlite,
// which opens the file O_RDONLY and cannot write, lock or migrate it even by
// accident. The cursor is a rowid high-water mark per table, so a poll costs the
// rows written since the last one and several open tabs never re-read history.
//
// # A row is not a record
//
// The one thing a database store does that an append-only log cannot: rows are
// *updated in place*. A part is inserted when the assistant starts emitting it
// and rewritten as the text streams, and the assistant message row is rewritten
// when it completes — all without the rowid moving. So a rowid cursor discovers
// that a row exists, and the rows of the turn being closed are read again at the
// moment of closing, which is the only moment their content is final.
//
// # Where a turn ends
//
// A turn is one user message plus every assistant message pointing at it through
// parentID — usually several, since OpenCode starts a new assistant message after
// each round of tool calls. The one that *finishes* the turn is the one carrying
// a `step-finish` part with `reason: "stop"`; the others carry `tool-calls`. That
// is the direct analogue of Claude's end_turn, and it is why this adapter never
// has to guess which assistant message was the last.
//
// # The native title, and the placeholder that must not be published
//
// session.title is the strongest native title of the six: OpenCode generates one
// during the first turn and never refreshes it. But it *starts* as a placeholder
// — "New session - <ISO8601>" — and publishing that would pin the tab to a
// timestamp and, because a native title blocks paid generation for the life of
// the session, keep it there forever. The predicate below is OpenCode's own.
type opencode struct{}

const (
	opencodeDB = "opencode.db"

	// opencodeBatch bounds one poll's read of each table. Whatever is left over
	// is read on the next poll, since the cursor only advances over rows that
	// were consumed.
	opencodeBatch = 512

	// opencodeSessions bounds the scan for a binding candidate. The session
	// table is one row per conversation ever held, which is small, but it is not
	// chartr's to assume it stays small.
	opencodeSessions = 10000
)

// opencodePlaceholder is OpenCode's own Session.isDefaultTitle, reproduced
// exactly. A title matching it is not a title.
var opencodePlaceholder = regexp.MustCompile(
	`^(New session - |Child session - )\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`)

// Bind matches a live OpenCode process to its persisted session. OpenCode
// exposes no process-to-session registry — it puts its own pid in its children's
// environment, and nothing on disk maps one to a session — so the route is
// working directory plus rows created since the agent started: unique or nothing.
func (opencode) Bind(agent proc.Agent) (Session, bool) {
	if agent.Adapter != "opencode" || agent.StateRoot == "" || agent.PID <= 0 {
		return nil, false
	}
	path := filepath.Join(agent.StateRoot, opencodeDB)
	db, err := sqlite.Open(path)
	if err != nil {
		return nil, false
	}
	s := &opencodeSession{db: db, path: path}
	if !s.tables() || !s.bind(agent) {
		db.Close()
		return nil, false
	}
	return s, true
}

// opencodeSession is one bound conversation: the database, the shape of the three
// tables it reads, the rowid cursors over two of them, and the turn being
// assembled.
type opencodeSession struct {
	db   *sqlite.DB
	path string

	sessions *sqlite.Table
	messages *sqlite.Table
	parts    *sqlite.Table
	col      opencodeColumns

	// row is the session row's own rowid, and id the identity it must still
	// carry. A rowid that now names a different session means the store was
	// rewritten under this binding, and the binding is over.
	row int64
	id  string

	native  string
	message int64 // cursor over message rowids
	part    int64 // cursor over part rowids

	turn opencodeTurn
	out  []Event
}

// opencodeColumns is where the three tables' columns are, resolved by name once
// at binding. A column that is not there is a schema this adapter does not read.
type opencodeColumns struct {
	sessionID, parent, directory, title, created int
	messageID, messageSession, messageData,
	partMessage, partSession, partData int
}

// opencodeTurn is the turn being assembled: the user message it opened with, the
// rowids of the rows that will have to be read again when it closes, and the
// assistant message that closed it.
type opencodeTurn struct {
	user   string  // the user message's id, which its assistant messages point at
	prompt []int64 // part rowids holding the operator's own text
	// answers maps an assistant message's id to the part rowids holding its
	// visible text, and rows to that message's own rowid.
	answers map[string][]int64
	rows    map[string]int64
	closer  string // the assistant message that finished the turn
}

func (s *opencodeSession) ID() string { return s.id }

func (s *opencodeSession) forget() { s.turn = opencodeTurn{} }

// tables resolves the three tables and every column this adapter reads. It is the
// sniff: a missing table or column ends the binding before a row is read.
func (s *opencodeSession) tables() bool {
	var ok bool
	if s.sessions, ok = s.db.Table("session"); !ok {
		return false
	}
	if s.messages, ok = s.db.Table("message"); !ok {
		return false
	}
	if s.parts, ok = s.db.Table("part"); !ok {
		return false
	}
	for _, resolve := range []struct {
		into  *int
		table *sqlite.Table
		name  string
	}{
		{&s.col.sessionID, s.sessions, "id"},
		{&s.col.parent, s.sessions, "parent_id"},
		{&s.col.directory, s.sessions, "directory"},
		{&s.col.title, s.sessions, "title"},
		{&s.col.created, s.sessions, "time_created"},
		{&s.col.messageID, s.messages, "id"},
		{&s.col.messageSession, s.messages, "session_id"},
		{&s.col.messageData, s.messages, "data"},
		{&s.col.partMessage, s.parts, "message_id"},
		{&s.col.partSession, s.parts, "session_id"},
		{&s.col.partData, s.parts, "data"},
	} {
		i, ok := resolve.table.Column(resolve.name)
		if !ok {
			return false
		}
		*resolve.into = i
	}
	return true
}

// bind finds the one session row that belongs to this live agent and seats the
// cursors at the end of the message and part tables, so everything already
// written stays behind them.
func (s *opencodeSession) bind(agent proc.Agent) bool {
	var (
		hits  int
		since int64
	)
	if !agent.Started.IsZero() {
		since = agent.Started.UnixMilli()
	}
	err := s.db.Rows(s.sessions, 0, opencodeSessions, func(rowid int64, row sqlite.Row) bool {
		// A child session is a subagent's: it has its own real title and its own
		// directory, and must never be a tab's binding.
		if !row.IsNull(s.col.parent) {
			return true
		}
		dir, ok := row.Text(s.col.directory)
		if !ok || !sameDir(dir, agent.Dir) {
			return true
		}
		// A session created before this process was is nobody live's, which is
		// what keeps every past conversation in this directory — chartr's own
		// `opencode run` generations among them — out of the candidate set.
		if created, ok := row.Int(s.col.created); !ok || created < since {
			return true
		}
		id, ok := row.Text(s.col.sessionID)
		if !ok || id == "" {
			return true
		}
		s.row, s.id, hits = rowid, id, hits+1
		return true
	})
	if err != nil || hits != 1 {
		return false
	}

	message, err := s.db.MaxRowID(s.messages)
	if err != nil {
		return false
	}
	part, err := s.db.MaxRowID(s.parts)
	if err != nil {
		return false
	}
	s.message, s.part = message, part
	// The session's own title is history that is allowed to speak: publishing a
	// title the provider already wrote costs nothing.
	s.title()
	return s.db.Stable()
}

// Poll reads the rows that appeared since the last call.
func (s *opencodeSession) Poll() ([]Event, bool) {
	if _, err := os.Stat(s.path); err != nil {
		return nil, false // the store went away
	}
	if err := s.db.Refresh(); err != nil {
		return nil, false
	}
	if !s.reseat() {
		return nil, false
	}
	// The cursors are only advanced once the whole batch has been read from one
	// snapshot, so a checkpoint that moves the ground mid-poll costs a beat
	// rather than a turn.
	message, part := s.message, s.part
	turn := s.turn

	native := s.native

	if !s.alive() {
		return nil, false
	}
	s.title()
	drained, ok := s.messagesAfter()
	if !ok {
		s.rewind(message, part, turn, native)
		return nil, false
	}
	if drained && (!s.partsAfter() || !s.close()) {
		s.rewind(message, part, turn, native)
		return nil, false
	}
	if !s.db.Stable() {
		// The snapshot moved while it was being read. Nothing is believed, and
		// the next poll asks again from where this one started.
		s.rewind(message, part, turn, native)
		return nil, true
	}
	out := s.out
	s.out = nil
	return out, true
}

// reseat handles a store rewritten underneath the binding: history cleared, a
// backup restored, a table emptied and refilled. A table shorter than its own
// cursor means the rows the cursor named are gone, and the only safe position is
// the new end — the same answer the byte-offset reader gives a truncated file.
func (s *opencodeSession) reseat() bool {
	message, err := s.db.MaxRowID(s.messages)
	if err != nil {
		return false
	}
	part, err := s.db.MaxRowID(s.parts)
	if err != nil {
		return false
	}
	if message < s.message || part < s.part {
		s.message, s.part = message, part
		s.forget()
	}
	return true
}

// rewind undoes a poll that could not be believed, so the next one asks the same
// question again from the same place — the published title included, which would
// otherwise be lost between the poll that read it and the poll that reports it.
func (s *opencodeSession) rewind(message, part int64, turn opencodeTurn, native string) {
	s.message, s.part, s.turn, s.native, s.out = message, part, turn, native, nil
}

// Close releases the store. A binding that is over holds nothing open on a file
// it does not own.
func (s *opencodeSession) Close() error { return s.db.Close() }

// alive reports whether the session this binding names is still the one at its
// rowid. A store rewritten underneath — cleared history, a restored backup —
// ends the binding rather than being read as if nothing happened.
func (s *opencodeSession) alive() bool {
	row, ok, err := s.db.Row(s.sessions, s.row)
	if err != nil || !ok {
		return false
	}
	id, ok := row.Text(s.col.sessionID)
	return ok && id == s.id
}

// title publishes the session's own title, and again whenever it changes —
// except while it is still the placeholder OpenCode names a new session with.
func (s *opencodeSession) title() {
	row, ok, err := s.db.Row(s.sessions, s.row)
	if err != nil || !ok {
		return
	}
	raw, ok := row.Text(s.col.title)
	if !ok || opencodePlaceholder.MatchString(raw) {
		return
	}
	title := head(oneLine(raw), textCap)
	if title == "" || title == s.native {
		return
	}
	s.native = title
	s.out = append(s.out, Event{Kind: NativeTitle, Title: title})
}

// opencodeMessage is a message row's data blob, narrowed to what this adapter
// reads.
//
// Summary is deliberately raw. It means two different things on the two roles: a
// boolean on an assistant message, marking a compaction summary rather than an
// answer, and a diff object on a user message, which is an ordinary submission
// carrying a record of what the turn changed. Decoding it as either would make
// every message of the other role unreadable — and unreadable ends a binding.
type opencodeMessage struct {
	Role     string          `json:"role"`
	ParentID string          `json:"parentID"`
	Error    json.RawMessage `json:"error"`
	Summary  json.RawMessage `json:"summary"`
}

// summarised reports whether this message is a summary of a conversation rather
// than a part of one. Only the literal true says so.
func (m opencodeMessage) summarised() bool { return string(m.Summary) == "true" }

// opencodePart is a part row's data blob, narrowed the same way.
type opencodePart struct {
	Type      string `json:"type"`
	Text      string `json:"text"`
	Synthetic bool   `json:"synthetic"`
	Ignored   bool   `json:"ignored"`
	Reason    string `json:"reason"`
}

// messagesAfter folds in the message rows written since the last poll, and
// reports whether it drained them. A batch that filled up leaves messages this
// poll has not seen, and the parts of those messages must not be read past
// before the messages they belong to are known.
func (s *opencodeSession) messagesAfter() (drained, ok bool) {
	ok = true
	seen := 0
	err := s.db.Rows(s.messages, s.message, opencodeBatch, func(rowid int64, row sqlite.Row) bool {
		s.message = rowid
		seen++
		session, _ := row.Text(s.col.messageSession)
		if session != s.id {
			return true // another tab's conversation, in the same store
		}
		id, found := row.Text(s.col.messageID)
		data, blob := row.Text(s.col.messageData)
		if !found || !blob {
			ok = false
			return false
		}
		var m opencodeMessage
		if json.Unmarshal([]byte(data), &m) != nil {
			ok = false // the shape a turn is read out of has drifted
			return false
		}
		switch m.Role {
		case "user":
			// A new submission is a new act by whoever is at the keyboard, so
			// whatever was pending is not going to be answered. A summary
			// message is a compaction, not a person.
			s.forget()
			if !m.summarised() {
				s.turn = opencodeTurn{
					user:    id,
					answers: map[string][]int64{},
					rows:    map[string]int64{},
				}
			}
		case "assistant":
			if s.turn.user != "" && m.ParentID == s.turn.user && !m.summarised() {
				s.turn.rows[id] = rowid
				if _, seen := s.turn.answers[id]; !seen {
					s.turn.answers[id] = nil
				}
			}
		default:
			ok = false // a role outside the union
			return false
		}
		return true
	})
	return seen < opencodeBatch, err == nil && ok
}

// partsAfter folds in the part rows written since the last poll. It records
// *where* the text of this turn lives rather than the text itself, because a part
// is rewritten in place while it streams and only its value at closing time is
// the answer.
func (s *opencodeSession) partsAfter() bool {
	ok := true
	err := s.db.Rows(s.parts, s.part, opencodeBatch, func(rowid int64, row sqlite.Row) bool {
		s.part = rowid
		if session, _ := row.Text(s.col.partSession); session != s.id {
			return true
		}
		message, found := row.Text(s.col.partMessage)
		data, blob := row.Text(s.col.partData)
		if !found || !blob {
			ok = false
			return false
		}
		var p opencodePart
		if json.Unmarshal([]byte(data), &p) != nil {
			ok = false
			return false
		}
		if p.Type == "" {
			ok = false
			return false
		}
		if s.turn.user == "" {
			return true
		}
		switch {
		case message == s.turn.user:
			// OpenCode's own first-real-user-message predicate: anything that
			// is not a plain, non-synthetic, non-ignored text part is not the
			// operator speaking.
			if p.Type == "text" && !p.Synthetic && !p.Ignored {
				s.turn.prompt = append(s.turn.prompt, rowid)
			}
		case s.turn.rows[message] != 0:
			switch {
			case p.Type == "text" && !p.Synthetic && !p.Ignored:
				s.turn.answers[message] = append(s.turn.answers[message], rowid)
			case p.Type == "step-finish" && p.Reason == "stop":
				// The step that ended the turn rather than stopping to call a
				// tool: this assistant message carries the visible answer.
				s.turn.closer = message
			}
		}
		return true
	})
	return err == nil && ok
}

// close finishes a turn whose closing step has landed, reading the rows it is
// made of again — now that they are final — and emitting the one event.
func (s *opencodeSession) close() bool {
	closer := s.turn.closer
	if closer == "" {
		return true
	}
	prompt, promptOK := s.text(s.turn.prompt)
	answer, answerOK := s.text(s.turn.answers[closer])
	failed, errorOK := s.errored(s.turn.rows[closer])
	s.forget()
	if !promptOK || !answerOK || !errorOK {
		return false
	}
	if failed {
		// An aborted run, a provider error, an overflowed context: the turn
		// finished, but not with an answer.
		return true
	}
	prompt, answer = head(prompt, textCap), head(answer, textCap)
	if prompt == "" || answer == "" {
		return true
	}
	s.out = append(s.out, Event{Kind: HumanTurn, Prompt: prompt, Response: answer})
	return true
}

// text re-reads a run of part rows and joins their visible text.
func (s *opencodeSession) text(rows []int64) (string, bool) {
	var out []string
	for _, rowid := range rows {
		row, ok, err := s.db.Row(s.parts, rowid)
		if err != nil {
			return "", false
		}
		if !ok {
			continue // a part deleted between being seen and being read
		}
		data, blob := row.Text(s.col.partData)
		if !blob {
			return "", false
		}
		var p opencodePart
		if json.Unmarshal([]byte(data), &p) != nil {
			return "", false
		}
		if p.Type == "text" && !p.Synthetic && !p.Ignored && strings.TrimSpace(p.Text) != "" {
			out = append(out, p.Text)
		}
	}
	return strings.Join(out, "\n\n"), true
}

// errored re-reads an assistant message row and reports whether it ended in one.
// The error is written into the row after it was first seen, which is exactly why
// it is asked for here rather than when the row appeared.
func (s *opencodeSession) errored(rowid int64) (bool, bool) {
	if rowid == 0 {
		return true, true
	}
	row, ok, err := s.db.Row(s.messages, rowid)
	if err != nil || !ok {
		return true, true
	}
	data, blob := row.Text(s.col.messageData)
	if !blob {
		return false, false
	}
	var m opencodeMessage
	if json.Unmarshal([]byte(data), &m) != nil {
		return false, false
	}
	return len(m.Error) > 0 && string(m.Error) != "null", true
}
