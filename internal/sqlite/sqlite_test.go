package sqlite

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// These tests are written against databases produced by SQLite itself, through
// the sqlite3 binary, rather than by a writer of this package's own. That is the
// point: a reader tested only against a writer that shares its assumptions proves
// that the two agree, not that either is right. Every file read here was laid out
// by the real thing.
//
// The writer is also kept *alive* for the whole of a test. A sqlite3 process that
// exits checkpoints and removes its log, so a database written by a series of
// one-shot commands would never exercise the write-ahead log at all — which is
// the half of this reader that a live agent actually depends on.

// live is a sqlite3 process holding one database open, fed statements on stdin.
type live struct {
	t    *testing.T
	cmd  *exec.Cmd
	in   io.WriteCloser
	out  *bufio.Reader
	logs *strings.Builder
}

// newLive starts the writer, skipping the test where sqlite3 is not installed.
func newLive(t *testing.T, path string) *live {
	t.Helper()
	bin, err := exec.LookPath("sqlite3")
	if err != nil {
		t.Skip("sqlite3 is not installed; the fixtures these tests read are written by SQLite itself")
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
	l := &live{t: t, cmd: cmd, in: in, out: bufio.NewReader(out), logs: logs}
	t.Cleanup(l.stop)
	l.exec("PRAGMA journal_mode=WAL;")
	return l
}

// exec runs statements and waits for them to have been applied, so a test never
// reads before the writer has written.
func (l *live) exec(statements ...string) {
	l.t.Helper()
	const done = "chartr-applied"
	script := strings.Join(statements, "\n") + "\nSELECT '" + done + "';\n"
	if _, err := io.WriteString(l.in, script); err != nil {
		l.t.Fatalf("write statements: %v (stderr: %s)", err, l.logs)
	}
	for {
		line, err := l.out.ReadString('\n')
		if err != nil {
			l.t.Fatalf("sqlite3 ended: %v (stderr: %s)", err, l.logs)
		}
		if strings.TrimSpace(line) == done {
			break
		}
	}
	if l.logs.Len() > 0 {
		l.t.Fatalf("sqlite3 reported: %s", l.logs)
	}
}

func (l *live) stop() {
	l.in.Close()
	l.cmd.Wait()
}

// quote renders a Go string as a SQL literal.
func quote(s string) string { return "'" + strings.ReplaceAll(s, "'", "''") + "'" }

// openDB opens a store and closes it with the test.
func openDB(t *testing.T, path string) *DB {
	t.Helper()
	db, err := Open(path)
	if err != nil {
		t.Fatalf("open: %v", err)
	}
	t.Cleanup(func() { db.Close() })
	return db
}

// collect reads a table's rows after a cursor into a slice.
func collect(t *testing.T, db *DB, name string, after int64, limit int) ([]int64, []Row) {
	t.Helper()
	table, ok := db.Table(name)
	if !ok {
		t.Fatalf("no table %q", name)
	}
	var (
		ids  []int64
		rows []Row
	)
	if err := db.Rows(table, after, limit, func(id int64, row Row) bool {
		ids, rows = append(ids, id), append(rows, row)
		return true
	}); err != nil {
		t.Fatalf("rows of %s: %v", name, err)
	}
	return ids, rows
}

// The whole of the reader against one store: the schema it describes itself
// with, every storage class, a value far too large for one page, and the columns
// named rather than numbered.
func TestReadsWhatSQLiteWrote(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.db")
	w := newLive(t, path)
	long := strings.Repeat("a payload larger than any page holds. ", 3000)
	w.exec(
		"CREATE TABLE thing (id INTEGER PRIMARY KEY, name TEXT NOT NULL, weight REAL, blob BLOB, missing TEXT);",
		"INSERT INTO thing VALUES (1, 'first', 1.5, x'0102ff', NULL);",
		"INSERT INTO thing VALUES (2, "+quote(long)+", -2.5, NULL, 'here');",
		"INSERT INTO thing VALUES (9000000000, 'a big rowid', 0.0, NULL, NULL);",
	)

	db := openDB(t, path)
	table, ok := db.Table("thing")
	if !ok {
		t.Fatal("the schema does not describe the table SQLite created")
	}
	for i, want := range []string{"id", "name", "weight", "blob", "missing"} {
		got, ok := table.Column(want)
		if !ok || got != i {
			t.Fatalf("column %q resolved to %d (%v), want %d", want, got, ok, i)
		}
	}

	ids, rows := collect(t, db, "thing", 0, 0)
	if len(ids) != 3 {
		t.Fatalf("read %d rows, want 3", len(ids))
	}
	// An INTEGER PRIMARY KEY is the rowid, and is not stored in the row at all.
	if got, _ := rows[0].Int(0); got != ids[0] || ids[0] != 1 {
		t.Fatalf("rowid alias read as %d with rowid %d", got, ids[0])
	}
	if got, _ := rows[2].Int(0); got != 9000000000 {
		t.Fatalf("large rowid read as %d", got)
	}
	if got, _ := rows[0].Text(1); got != "first" {
		t.Fatalf("text read as %q", got)
	}
	if got, ok := rows[0].Int(2); ok {
		t.Fatalf("a real read as the integer %d", got)
	}
	if f, ok := rows[0][2].(float64); !ok || f != 1.5 {
		t.Fatalf("real read as %v", rows[0][2])
	}
	if got, ok := rows[0][3].([]byte); !ok || len(got) != 3 || got[2] != 0xff {
		t.Fatalf("blob read as %v", rows[0][3])
	}
	if !rows[0].IsNull(4) {
		t.Fatalf("NULL read as %v", rows[0][4])
	}
	// The overflow chain: a value many pages long has to come back whole.
	if got, _ := rows[1].Text(1); got != long {
		t.Fatalf("an overflowing value came back %d bytes, want %d", len(got), len(long))
	}
}

// The property the incremental tail is built on: rows after a cursor, and only
// those. A tab that has been open all day must not re-read the day.
func TestReadsForwardFromACursor(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.db")
	w := newLive(t, path)
	w.exec("CREATE TABLE note (id TEXT PRIMARY KEY, body TEXT NOT NULL);")
	// Enough rows to make the tree more than one page deep, so the descent is
	// actually skipping subtrees rather than filtering a single leaf.
	var inserts []string
	for i := 1; i <= 500; i++ {
		inserts = append(inserts, fmt.Sprintf("INSERT INTO note VALUES ('n%d', %s);",
			i, quote(strings.Repeat("body ", 40)+fmt.Sprint(i))))
	}
	w.exec(inserts...)

	db := openDB(t, path)
	table, _ := db.Table("note")
	max, err := db.MaxRowID(table)
	if err != nil || max != 500 {
		t.Fatalf("MaxRowID = %d (%v), want 500", max, err)
	}
	ids, _ := collect(t, db, "note", 480, 0)
	if len(ids) != 20 || ids[0] != 481 || ids[19] != 500 {
		t.Fatalf("after 480 read %d rows starting at %d", len(ids), ids[0])
	}
	// The limit is a bound on one batch, and the batch is the head of the range.
	ids, _ = collect(t, db, "note", 480, 5)
	if len(ids) != 5 || ids[0] != 481 || ids[4] != 485 {
		t.Fatalf("limited batch = %v", ids)
	}
	if ids, _ := collect(t, db, "note", 500, 0); len(ids) != 0 {
		t.Fatalf("a cursor at the end read %d rows", len(ids))
	}

	// One row by rowid, which is how a row that is updated in place is re-read.
	row, ok, err := db.Row(table, 250)
	if err != nil || !ok {
		t.Fatalf("Row(250) = %v (%v)", ok, err)
	}
	if id, _ := row.Text(0); id != "n250" {
		t.Fatalf("Row(250) is %q", id)
	}
	if _, ok, _ := db.Row(table, 9999); ok {
		t.Fatal("a rowid that is not there came back anyway")
	}
}

// The live half: the writer is still holding the database open, so everything
// committed since the reader opened it is in the write-ahead log and nowhere
// else. A reader that only read the main file would see an empty table forever.
func TestSeesWhatALiveWriterCommits(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.db")
	w := newLive(t, path)
	w.exec("CREATE TABLE note (id TEXT PRIMARY KEY, body TEXT NOT NULL);",
		"INSERT INTO note VALUES ('a', 'the first note');")

	db := openDB(t, path)
	if ids, _ := collect(t, db, "note", 0, 0); len(ids) != 1 {
		t.Fatalf("opened onto %d rows, want the one already committed", len(ids))
	}
	if _, err := os.Stat(path + "-wal"); err != nil {
		t.Fatalf("the writer is not in WAL mode, so this proves nothing: %v", err)
	}

	// Several commits in a row, each of which has to become visible without the
	// database being reopened.
	for i, body := range []string{"the second note", "the third note", "the fourth note"} {
		w.exec("INSERT INTO note VALUES (" + quote(fmt.Sprint("k", i)) + ", " + quote(body) + ");")
		if err := db.Refresh(); err != nil {
			t.Fatalf("refresh: %v", err)
		}
		ids, rows := collect(t, db, "note", int64(i+1), 0)
		if len(ids) != 1 {
			t.Fatalf("after commit %d, the cursor read %d rows", i, len(ids))
		}
		if got, _ := rows[0].Text(1); got != body {
			t.Fatalf("read %q, want %q", got, body)
		}
	}

	// A row rewritten in place keeps its rowid, and re-reading it by rowid is
	// what a reader does when it needs the current value rather than the one it
	// first saw.
	w.exec("UPDATE note SET body = 'the first note, revised' WHERE id = 'a';")
	if err := db.Refresh(); err != nil {
		t.Fatalf("refresh: %v", err)
	}
	table, _ := db.Table("note")
	row, ok, err := db.Row(table, 1)
	if err != nil || !ok {
		t.Fatalf("re-reading the updated row: %v (%v)", ok, err)
	}
	if got, _ := row.Text(1); got != "the first note, revised" {
		t.Fatalf("the updated row still reads %q", got)
	}
}

// A transaction that has not committed has no commit frame in the log, and must
// not be readable — not even partly. This is what makes an unlocked read of a
// live database defensible.
func TestUncommittedWorkIsNotVisible(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.db")
	w := newLive(t, path)
	w.exec("CREATE TABLE note (id TEXT PRIMARY KEY, body TEXT NOT NULL);",
		"INSERT INTO note VALUES ('a', 'committed');")

	db := openDB(t, path)
	w.exec("BEGIN;", "INSERT INTO note VALUES ('b', 'still open');",
		"INSERT INTO note VALUES ('c', 'also still open');")
	if err := db.Refresh(); err != nil {
		t.Fatalf("refresh: %v", err)
	}
	if ids, _ := collect(t, db, "note", 0, 0); len(ids) != 1 {
		t.Fatalf("read %d rows while a transaction was open, want only the committed one", len(ids))
	}

	w.exec("COMMIT;")
	if err := db.Refresh(); err != nil {
		t.Fatalf("refresh: %v", err)
	}
	if ids, _ := collect(t, db, "note", 0, 0); len(ids) != 3 {
		t.Fatalf("read %d rows after the commit, want all three", len(ids))
	}
}

// A checkpoint moves pages out of the log and into the main file, and may reset
// the log entirely. The rows must survive that, and the reader must not be left
// pointing at frames that now hold something else.
func TestSurvivesACheckpoint(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.db")
	w := newLive(t, path)
	w.exec("CREATE TABLE note (id TEXT PRIMARY KEY, body TEXT NOT NULL);",
		"INSERT INTO note VALUES ('a', 'before the checkpoint');")

	db := openDB(t, path)
	collect(t, db, "note", 0, 0)

	w.exec("PRAGMA wal_checkpoint(TRUNCATE);",
		"INSERT INTO note VALUES ('b', 'after the checkpoint');")
	if err := db.Refresh(); err != nil {
		t.Fatalf("refresh after a checkpoint: %v", err)
	}
	ids, rows := collect(t, db, "note", 0, 0)
	if len(ids) != 2 {
		t.Fatalf("read %d rows across a checkpoint, want 2", len(ids))
	}
	if got, _ := rows[0].Text(1); got != "before the checkpoint" {
		t.Fatalf("the pre-checkpoint row reads %q", got)
	}
	if got, _ := rows[1].Text(1); got != "after the checkpoint" {
		t.Fatalf("the post-checkpoint row reads %q", got)
	}
}

// The guarantee the whole package exists for: chartr reads a store it does not
// own and leaves no trace on it. Not a byte written, not a file created — not
// even the shared-memory file an ordinary read-only connection would make.
func TestReadingWritesNothing(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "store.db")
	w := newLive(t, path)
	w.exec("CREATE TABLE note (id TEXT PRIMARY KEY, body TEXT NOT NULL);",
		"INSERT INTO note VALUES ('a', 'a note');")

	db := openDB(t, path)
	before := snapshot(t, dir)
	for range 5 {
		if err := db.Refresh(); err != nil {
			t.Fatalf("refresh: %v", err)
		}
		collect(t, db, "note", 0, 0)
		table, _ := db.Table("note")
		db.MaxRowID(table)
		db.Row(table, 1)
		db.Stable()
	}
	if after := snapshot(t, dir); after != before {
		t.Fatalf("reading changed the store:\n before %s\n after  %s", before, after)
	}
}

// snapshot renders every file in a directory with its size and modification
// time, which is enough to catch a write, a truncation or a new file.
func snapshot(t *testing.T, dir string) string {
	t.Helper()
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("read %s: %v", dir, err)
	}
	var b strings.Builder
	for _, entry := range entries {
		info, err := entry.Info()
		if err != nil {
			t.Fatalf("stat %s: %v", entry.Name(), err)
		}
		fmt.Fprintf(&b, "%s:%d:%d ", entry.Name(), info.Size(), info.ModTime().UnixNano())
	}
	return b.String()
}

// Everything this reader does not handle resolves the same way: it declines.
func TestDeclinesWhatItDoesNotUnderstand(t *testing.T) {
	dir := t.TempDir()

	notADatabase := filepath.Join(dir, "not-a-database")
	if err := os.WriteFile(notADatabase, []byte(strings.Repeat("not sqlite at all", 100)), 0o600); err != nil {
		t.Fatalf("write: %v", err)
	}
	if _, err := Open(notADatabase); !errors.Is(err, ErrUnsupported) {
		t.Fatalf("opening a file that is not a database = %v, want ErrUnsupported", err)
	}

	if _, err := Open(filepath.Join(dir, "there-is-no-such-file")); err == nil {
		t.Fatal("opening a missing store succeeded")
	}

	empty := filepath.Join(dir, "empty")
	if err := os.WriteFile(empty, nil, 0o600); err != nil {
		t.Fatalf("write: %v", err)
	}
	if _, err := Open(empty); err == nil {
		t.Fatal("opening an empty file succeeded")
	}

	path := filepath.Join(dir, "store.db")
	w := newLive(t, path)
	w.exec("CREATE TABLE note (id TEXT PRIMARY KEY);")
	db := openDB(t, path)
	if _, ok := db.Table("no-such-table"); ok {
		t.Fatal("a table that does not exist resolved")
	}
	table, _ := db.Table("note")
	if got, ok := table.Column("no-such-column"); ok {
		t.Fatalf("a column that does not exist resolved to %d", got)
	}
	if err := db.Rows(nil, 0, 0, func(int64, Row) bool { return true }); !errors.Is(err, ErrUnsupported) {
		t.Fatalf("reading no table = %v, want ErrUnsupported", err)
	}
}

// A database with no log at all — the ordinary state of a store nobody has open —
// reads exactly the same way.
func TestReadsAStoreWithNoLog(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.db")
	func() {
		w := newLive(t, path)
		w.exec("PRAGMA journal_mode=DELETE;",
			"CREATE TABLE note (id TEXT PRIMARY KEY, body TEXT NOT NULL);",
			"INSERT INTO note VALUES ('a', 'a note with no log behind it');")
		w.stop()
	}()
	if _, err := os.Stat(path + "-wal"); !os.IsNotExist(err) {
		t.Skipf("the writer left a log behind, so this is not the case under test: %v", err)
	}

	db := openDB(t, path)
	_, rows := collect(t, db, "note", 0, 0)
	if len(rows) != 1 {
		t.Fatalf("read %d rows, want 1", len(rows))
	}
	if got, _ := rows[0].Text(1); got != "a note with no log behind it" {
		t.Fatalf("read %q", got)
	}
}
