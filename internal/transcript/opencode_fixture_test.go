package transcript

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/proc"
)

// Synthetic OpenCode fixtures: a data root holding one SQLite database, written
// by SQLite itself through a sqlite3 process that stays open for the whole test.
//
// Every row here was written for this test. Nothing is copied from a real
// session — no personal conversation, no credentials, no reasoning, no real tool
// output.
//
// Two things about this fixture are deliberate. It uses the real thing as the
// writer, because a reader tested against a writer of its own making proves only
// that the two agree. And it keeps that writer *alive*, because a sqlite3 process
// that exits checkpoints and removes its write-ahead log — so a database built
// from one-shot commands would never put the adapter in front of the live-writer
// case it exists for. Every mutation below is a commit into a log the reader has
// to read past the main file to see.

const (
	// opencodeFixtureFormat and opencodeFixtureVersion record what these
	// fixtures represent: OpenCode's session/message/part tables in
	// opencode.db under WAL, as written by OpenCode 1.2.27, which is the
	// version stamped on every session row in the store measured.
	opencodeFixtureFormat  = "opencode SQLite session/message/part under WAL"
	opencodeFixtureVersion = "1.2.27"
)

type opencodeStore struct {
	t       *testing.T
	root    string
	dir     string
	pid     int
	started time.Time
	id      string

	path    string
	tag     string
	w       *opencodeWriter
	seq     int
	pending []string // statements held back to make a turn incomplete

	// The ids of a turn opened by beginTurn, whose answer is still streaming.
	streamAgent string
	streamPart  string
}

func newOpencodeStore(t *testing.T) contractStore { return newOpencodeFixture(t) }

func newOpencodeFixture(t *testing.T) *opencodeStore {
	t.Helper()
	base := t.TempDir()
	s := &opencodeStore{
		t:       t,
		root:    filepath.Join(base, "opencode"),
		dir:     filepath.Join(base, "work"),
		pid:     9191,
		started: time.Now().Add(-time.Minute),
		id:      "ses_03c6735d8ffevluhm6O8zpiM9P",
		tag:     "a",
	}
	for _, d := range []string{s.root, s.dir} {
		if err := os.MkdirAll(d, 0o755); err != nil {
			t.Fatalf("directory %s: %v", d, err)
		}
	}
	s.path = filepath.Join(s.root, opencodeDB)
	s.w = newOpencodeWriter(t, s.path)
	s.schema()
	return s
}

// schema creates the tables exactly as OpenCode's own migrations do, constraint
// clauses included — those share the column list, and a reader that mistook one
// for a column would read every row one position out.
func (s *opencodeStore) schema() {
	s.w.exec(
		"CREATE TABLE `project` (`id` text PRIMARY KEY, `worktree` text NOT NULL);",
		"CREATE TABLE `session` (\n"+
			"\t`id` text PRIMARY KEY,\n"+
			"\t`project_id` text NOT NULL,\n"+
			"\t`parent_id` text,\n"+
			"\t`slug` text NOT NULL,\n"+
			"\t`directory` text NOT NULL,\n"+
			"\t`title` text NOT NULL,\n"+
			"\t`version` text NOT NULL,\n"+
			"\t`share_url` text,\n"+
			"\t`time_created` integer NOT NULL,\n"+
			"\t`time_updated` integer NOT NULL,\n"+
			"\t`time_archived` integer, `workspace_id` text,\n"+
			"\tCONSTRAINT `fk_session_project_id_project_id_fk` FOREIGN KEY (`project_id`) REFERENCES `project`(`id`) ON DELETE CASCADE\n"+
			");",
		"CREATE TABLE `message` (\n"+
			"\t`id` text PRIMARY KEY,\n"+
			"\t`session_id` text NOT NULL,\n"+
			"\t`time_created` integer NOT NULL,\n"+
			"\t`time_updated` integer NOT NULL,\n"+
			"\t`data` text NOT NULL,\n"+
			"\tCONSTRAINT `fk_message_session_id_session_id_fk` FOREIGN KEY (`session_id`) REFERENCES `session`(`id`) ON DELETE CASCADE\n"+
			");",
		"CREATE INDEX `message_session_time_created_id_idx` ON `message` (`session_id`,`time_created`,`id`);",
		"CREATE TABLE `part` (\n"+
			"\t`id` text PRIMARY KEY,\n"+
			"\t`message_id` text NOT NULL,\n"+
			"\t`session_id` text NOT NULL,\n"+
			"\t`time_created` integer NOT NULL,\n"+
			"\t`time_updated` integer NOT NULL,\n"+
			"\t`data` text NOT NULL,\n"+
			"\tCONSTRAINT `fk_part_message_id_message_id_fk` FOREIGN KEY (`message_id`) REFERENCES `message`(`id`) ON DELETE CASCADE\n"+
			");",
		"CREATE INDEX `part_session_idx` ON `part` (`session_id`);",
		"INSERT INTO project VALUES ('prj_fixture', "+sq(s.dir)+");",
	)
}

func (s *opencodeStore) Format() string {
	return opencodeFixtureFormat + " " + opencodeFixtureVersion
}

func (s *opencodeStore) Agent() proc.Agent {
	return proc.Agent{
		Adapter:   "opencode",
		PID:       s.pid,
		PGID:      s.pid,
		Started:   s.started,
		Dir:       s.dir,
		StateRoot: s.root,
	}
}

// Peer is a second OpenCode tab in the same space, sharing the one database.
// OpenCode has no process-to-session registry — it puts its own pid in its
// children's environment and nothing on disk maps one to a session — so two
// sessions in one working directory are indistinguishable and both tabs stay
// untitled.
func (s *opencodeStore) Peer() (contractStore, bool) {
	peer := &opencodeStore{
		t: s.t, root: s.root, dir: s.dir, path: s.path, w: s.w, tag: "b",
		pid: s.pid + 1, started: s.started, id: "ses_03c69adc7ffeUcQlB7SWH4lThZ",
	}
	return peer, false
}

// Start inserts the session row, with the placeholder title OpenCode names a new
// session with — the one an adapter must never publish, since it would pin the
// tab to a timestamp and block paid generation for the life of the session.
func (s *opencodeStore) Start() {
	s.w.exec(fmt.Sprintf(
		"INSERT INTO session VALUES (%s, 'prj_fixture', NULL, 'a-fixture-session', %s, %s, %s, NULL, %d, %d, NULL, NULL);",
		sq(s.id), sq(s.dir), sq(s.placeholder()), sq(opencodeFixtureVersion),
		s.now(), s.now()))
}

// placeholder renders OpenCode's own default title for a new session.
func (s *opencodeStore) placeholder() string {
	return "New session - " + time.Now().UTC().Format("2006-01-02T15:04:05.000Z")
}

func (s *opencodeStore) Turn(prompt, response string) {
	s.w.exec(s.records(prompt, response, true)...)
}

// PartialTurn writes everything but the step that ends the turn — which is what a
// reader sees for as long as the assistant is still streaming, and the case a
// database store has instead of a half-written line.
func (s *opencodeStore) PartialTurn(prompt, response string) {
	records := s.records(prompt, response, true)
	s.w.exec(records[:len(records)-1]...)
	s.pending = records[len(records)-1:]
}

func (s *opencodeStore) Complete() {
	s.w.exec(s.pending...)
	s.pending = nil
}

// Title sets the session's own title, which OpenCode generates once during the
// first turn and then never refreshes.
func (s *opencodeStore) Title(title string) bool {
	s.w.exec(fmt.Sprintf("UPDATE session SET title = %s, time_updated = %d WHERE id = %s;",
		sq(title), s.now(), sq(s.id)))
	return true
}

// Ignored writes the machinery: every part kind that is not visible text,
// a summarised message, and a whole child session — a subagent's, which has its
// own real title and must never reach a tab.
func (s *opencodeStore) Ignored() {
	orphan := s.next("msg")
	s.w.exec(
		s.message(orphan, s.id, map[string]any{
			"role": "assistant", "parentID": s.next("msg"),
			"time": map[string]any{"created": s.now(), "completed": s.now()},
		}),
		s.part(orphan, s.id, map[string]any{"type": "reasoning", "text": "invented reasoning the operator never sees"}),
		s.part(orphan, s.id, map[string]any{"type": "tool", "callID": s.next("call"), "tool": "read",
			"state": map[string]any{"status": "completed", "output": "invented tool output"}}),
		s.part(orphan, s.id, map[string]any{"type": "file", "filename": "/invented/path"}),
		s.part(orphan, s.id, map[string]any{"type": "patch", "hash": "invented"}),
		s.part(orphan, s.id, map[string]any{"type": "snapshot", "snapshot": "invented"}),
		s.part(orphan, s.id, map[string]any{"type": "agent", "name": "invented"}),
		s.part(orphan, s.id, map[string]any{"type": "subtask", "id": s.next("ses")}),
		s.part(orphan, s.id, map[string]any{"type": "compaction", "summary": "a summary of earlier work"}),
		s.part(orphan, s.id, map[string]any{"type": "retry", "attempt": 1}),
		s.part(orphan, s.id, map[string]any{"type": "step-start"}),
		s.part(orphan, s.id, map[string]any{"type": "step-finish", "reason": "tool-calls"}),
	)
	// A subagent's whole session, complete with a finished turn of its own.
	child := s.next("ses")
	childUser, childAgent := s.next("msg"), s.next("msg")
	s.w.exec(
		fmt.Sprintf("INSERT INTO session VALUES (%s, 'prj_fixture', %s, 'a-child', %s, 'A child session title', %s, NULL, %d, %d, NULL, NULL);",
			sq(child), sq(s.id), sq(s.dir), sq(opencodeFixtureVersion), s.now(), s.now()),
		s.message(childUser, child, map[string]any{"role": "user", "time": map[string]any{"created": s.now()}}),
		s.part(childUser, child, map[string]any{"type": "text", "text": "a subagent's own instructions"}),
		s.message(childAgent, child, map[string]any{"role": "assistant", "parentID": childUser,
			"time": map[string]any{"created": s.now(), "completed": s.now()}}),
		s.part(childAgent, child, map[string]any{"type": "text", "text": "a subagent's own answer"}),
		s.part(childAgent, child, map[string]any{"type": "step-finish", "reason": "stop"}),
	)
}

// SyntheticUser writes the user-shaped messages OpenCode itself authors: one
// whose parts are all synthetic — the envelopes it injects around a command — and
// one marked as a summary, which is a compaction rather than a person. A complete
// answer follows both, so if either could open a turn, this is where the adapter
// would invent one.
func (s *opencodeStore) SyntheticUser(text string) {
	for _, injected := range []map[string]any{
		{"role": "user", "time": map[string]any{"created": s.now()}},
		{"role": "user", "summary": true, "time": map[string]any{"created": s.now()}},
	} {
		user, agent := s.next("msg"), s.next("msg")
		s.w.exec(
			s.message(user, s.id, injected),
			s.part(user, s.id, map[string]any{"type": "text", "text": text, "synthetic": true}),
			s.part(user, s.id, map[string]any{"type": "text", "text": text, "ignored": true}),
			s.message(agent, s.id, map[string]any{"role": "assistant", "parentID": user,
				"time": map[string]any{"created": s.now(), "completed": s.now()}}),
			s.part(agent, s.id, map[string]any{"type": "text", "text": "an answer to nobody"}),
			s.part(agent, s.id, map[string]any{"type": "step-finish", "reason": "stop"}),
		)
	}
}

// Malformed inserts a row whose data blob is not readable at all.
func (s *opencodeStore) Malformed() {
	s.w.exec(fmt.Sprintf("INSERT INTO message VALUES (%s, %s, %d, %d, %s);",
		sq(s.next("msg")), sq(s.id), s.now(), s.now(), sq(`{"role":"user","time":`)))
}

// Drift inserts a row of a kind the adapter interprets, in a shape it has never
// seen: a message whose role is outside the union OpenCode's own schema defines.
func (s *opencodeStore) Drift() {
	s.w.exec(s.message(s.next("msg"), s.id, map[string]any{
		"role": "operator", "time": map[string]any{"created": s.now()},
	}))
}

// Replace empties the tables and refills them with a shorter history, which is
// what clearing a history or restoring a backup looks like from outside: the
// rowids a cursor named are gone, and the rows now carrying them are different
// rows.
func (s *opencodeStore) Replace() {
	user, agent := s.next("msg"), s.next("msg")
	s.w.exec(
		"DELETE FROM part;",
		"DELETE FROM message;",
		s.message(user, s.id, map[string]any{"role": "user", "time": map[string]any{"created": s.now()}}),
		s.part(user, s.id, map[string]any{"type": "text", "text": "a question from before the rewrite"}),
		s.message(agent, s.id, map[string]any{"role": "assistant", "parentID": user,
			"time": map[string]any{"created": s.now(), "completed": s.now()}}),
		s.part(agent, s.id, map[string]any{"type": "text", "text": "an answer from before the rewrite"}),
		s.part(agent, s.id, map[string]any{"type": "step-finish", "reason": "stop"}),
	)
}

// Remove deletes the store, as an operator clearing their data does. The writer
// still holds the file open, which is exactly the situation on a live machine.
func (s *opencodeStore) Remove() {
	for _, suffix := range []string{"", "-wal", "-shm"} {
		if err := os.Remove(s.path + suffix); err != nil && !os.IsNotExist(err) {
			s.t.Fatalf("remove store: %v", err)
		}
	}
}

// beginTurn opens a turn and gets as far as an assistant message with one text
// part in it, holding only what has streamed so far. This is the state a database
// store is in for as long as a response is being written: every row is there, and
// the one that matters is not final.
func (s *opencodeStore) beginTurn(prompt, sofar string) {
	user := s.next("msg")
	s.streamAgent = s.next("msg")
	s.streamPart = s.next("prt")
	s.w.exec(
		s.message(user, s.id, map[string]any{"role": "user", "time": map[string]any{"created": s.now()}}),
		s.part(user, s.id, map[string]any{"type": "text", "text": prompt}),
		s.message(s.streamAgent, s.id, map[string]any{
			"role": "assistant", "parentID": user,
			"time": map[string]any{"created": s.now()},
		}),
		fmt.Sprintf("INSERT INTO part VALUES (%s, %s, %s, %d, %d, %s);",
			sq(s.streamPart), sq(s.streamAgent), sq(s.id), s.now(), s.now(),
			sq(s.blob(map[string]any{"type": "text", "text": sofar}))),
	)
}

// growPart rewrites that text part in place, as OpenCode does on every chunk: the
// row's contents change and its rowid does not.
func (s *opencodeStore) growPart(text string) {
	s.w.exec(fmt.Sprintf("UPDATE part SET data = %s, time_updated = %d WHERE id = %s;",
		sq(s.blob(map[string]any{"type": "text", "text": text})), s.now(), sq(s.streamPart)))
}

// finishTurn closes the streamed turn: the assistant message is marked complete
// and the step that ends the turn lands.
func (s *opencodeStore) finishTurn() {
	s.w.exec(
		fmt.Sprintf("UPDATE message SET data = %s, time_updated = %d WHERE id = %s;",
			sq(s.blob(map[string]any{
				"role": "assistant", "time": map[string]any{"created": s.now(), "completed": s.now()},
				"parentID": s.parentOf(),
			})), s.now(), sq(s.streamAgent)),
		s.part(s.streamAgent, s.id, map[string]any{"type": "step-finish", "reason": "stop"}),
	)
}

// parentOf recovers the user message the streamed assistant message answers, so
// rewriting that row does not lose the link.
func (s *opencodeStore) parentOf() string {
	// The assistant message was allocated one id after its user message.
	n := 0
	fmt.Sscanf(s.streamAgent, "msg_fixture"+s.tag+"%d", &n)
	return fmt.Sprintf("msg_fixture%s%08d", s.tag, n-1)
}

// records renders one complete turn: the operator's message and its text, an
// assistant message that calls a tool and finishes on tool-calls, and a second
// assistant message that produces the visible answer and finishes on stop. The
// last statement is the step that ends the turn, held back by PartialTurn.
func (s *opencodeStore) records(prompt, response string, machinery bool) []string {
	user := s.next("msg")
	out := []string{
		s.message(user, s.id, map[string]any{
			"role": "user", "time": map[string]any{"created": s.now()},
			"agent": "build",
			"model": map[string]any{"providerID": "invented", "modelID": "opencode-fixture-1"},
		}),
		s.part(user, s.id, map[string]any{"type": "text", "text": prompt}),
	}
	if machinery {
		tool := s.next("msg")
		out = append(out,
			s.message(tool, s.id, map[string]any{
				"role": "assistant", "parentID": user,
				"time":  map[string]any{"created": s.now(), "completed": s.now()},
				"agent": "build", "modelID": "opencode-fixture-1", "providerID": "invented",
			}),
			s.part(tool, s.id, map[string]any{"type": "step-start"}),
			s.part(tool, s.id, map[string]any{"type": "reasoning", "text": "invented reasoning the operator never sees"}),
			s.part(tool, s.id, map[string]any{"type": "tool", "callID": s.next("call"), "tool": "read",
				"state": map[string]any{"status": "completed", "output": "invented tool output"}}),
			s.part(tool, s.id, map[string]any{"type": "step-finish", "reason": "tool-calls"}),
		)
	}
	answer := s.next("msg")
	return append(out,
		s.message(answer, s.id, map[string]any{
			"role": "assistant", "parentID": user,
			"time":  map[string]any{"created": s.now(), "completed": s.now()},
			"agent": "build", "modelID": "opencode-fixture-1", "providerID": "invented",
		}),
		s.part(answer, s.id, map[string]any{"type": "step-start"}),
		s.part(answer, s.id, map[string]any{"type": "text", "text": response}),
		s.part(answer, s.id, map[string]any{"type": "step-finish", "reason": "stop"}),
	)
}

func (s *opencodeStore) message(id, session string, data map[string]any) string {
	return fmt.Sprintf("INSERT INTO message VALUES (%s, %s, %d, %d, %s);",
		sq(id), sq(session), s.now(), s.now(), sq(s.blob(data)))
}

func (s *opencodeStore) part(message, session string, data map[string]any) string {
	return fmt.Sprintf("INSERT INTO part VALUES (%s, %s, %s, %d, %d, %s);",
		sq(s.next("prt")), sq(message), sq(session), s.now(), s.now(), sq(s.blob(data)))
}

func (s *opencodeStore) blob(data map[string]any) string {
	s.t.Helper()
	out, err := json.Marshal(data)
	if err != nil {
		s.t.Fatalf("marshal fixture blob: %v", err)
	}
	return string(out)
}

func (s *opencodeStore) now() int64 { return time.Now().UnixMilli() }

func (s *opencodeStore) next(prefix string) string {
	s.seq++
	return fmt.Sprintf("%s_fixture%s%08d", prefix, s.tag, s.seq)
}

// sq renders a Go string as a SQL literal.
func sq(s string) string { return "'" + strings.ReplaceAll(s, "'", "''") + "'" }

// opencodeWriter is a sqlite3 process holding the store open, fed statements on
// stdin — the live writer the adapter has to read alongside.
type opencodeWriter struct {
	t    *testing.T
	cmd  *exec.Cmd
	in   io.WriteCloser
	out  *bufio.Reader
	logs *strings.Builder
}

func newOpencodeWriter(t *testing.T, path string) *opencodeWriter {
	t.Helper()
	bin, err := exec.LookPath("sqlite3")
	if err != nil {
		t.Skip("sqlite3 is not installed; OpenCode's fixtures are written by SQLite itself")
	}
	cmd := exec.Command(bin, path)
	in, err := cmd.StdinPipe()
	if err != nil {
		t.Fatalf("stdin: %v", err)
	}
	out, err := cmd.StdoutPipe()
	if err != nil {
		t.Fatalf("stdout: %v", err)
	}
	logs := &strings.Builder{}
	cmd.Stderr = logs
	if err := cmd.Start(); err != nil {
		t.Fatalf("start sqlite3: %v", err)
	}
	w := &opencodeWriter{t: t, cmd: cmd, in: in, out: bufio.NewReader(out), logs: logs}
	t.Cleanup(func() {
		w.in.Close()
		w.cmd.Wait()
	})
	w.exec("PRAGMA journal_mode=WAL;")
	return w
}

// exec runs statements and waits for them to have been applied, so the adapter
// never reads before the writer has written.
func (w *opencodeWriter) exec(statements ...string) {
	w.t.Helper()
	if len(statements) == 0 {
		return
	}
	const done = "chartr-applied"
	script := strings.Join(statements, "\n") + "\nSELECT '" + done + "';\n"
	if _, err := io.WriteString(w.in, script); err != nil {
		w.t.Fatalf("write statements: %v (stderr: %s)", err, w.logs)
	}
	for {
		line, err := w.out.ReadString('\n')
		if err != nil {
			w.t.Fatalf("sqlite3 ended: %v (stderr: %s)", err, w.logs)
		}
		if strings.TrimSpace(line) == done {
			break
		}
	}
	if w.logs.Len() > 0 {
		w.t.Fatalf("sqlite3 reported: %s", w.logs)
	}
}
