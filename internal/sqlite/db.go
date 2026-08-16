// Package sqlite reads a SQLite database file that something else owns and is
// writing to, without opening it as a database.
//
// chartr needs this for exactly one thing: OpenCode keeps its sessions in
// SQLite, and a tab bound to one has to read the rows that appeared since the
// last poll while OpenCode itself is streaming a response into the same file.
// The requirement that shapes everything here is that chartr must not disturb
// that. It may not write, may not migrate, may not take a lock, and may not
// contend with the writer — not even briefly, and not even through the shared
// -shm file an ordinary read-only connection would create.
//
// So this is not a driver. It opens the file O_RDONLY and reads pages: the
// database header, the WAL frames layered over it, the schema table, and rowid
// b-trees. Writing is not implemented, which is a stronger guarantee than
// promising not to. There is no SQL, no query planner and no transaction: a
// caller asks for the rows of one table after a rowid, which is the only access
// pattern an incremental tail needs, and the b-tree descent that answers it is
// what makes several open tabs cheap instead of a repeated full scan.
//
// # Reading a file someone else is writing
//
// In WAL mode the main file is stable between checkpoints and every commit is an
// append to the -wal file, each frame carrying salts and a running checksum. That
// is what makes an unlocked read defensible: a frame is only believed if its
// salts match the log's and its checksum continues the chain, and only frames up
// to the last commit frame are visible, so a half-written transaction is not
// readable rather than partially readable.
//
// A checkpoint can still move the ground — it copies WAL pages into the main file
// and may reset the log. Stable reports whether that happened during a read, and
// a caller that gets false throws the batch away and asks again later. Everything
// unexpected resolves the same way: an unreadable file, a page type this reader
// does not know, a record whose header does not add up. There is no partial
// answer, because the caller's cheapest failure is no answer at all.
//
// # What is not here
//
// Indexes other than the rowid b-tree, WITHOUT ROWID tables, journal-mode
// databases, encryption, and every kind of write. A database this reader does not
// fully understand is one it declines to read.
package sqlite

import (
	"encoding/binary"
	"errors"
	"fmt"
	"os"
)

// ErrUnsupported is what this reader returns for a database it will not read:
// one that is not SQLite, one whose page size or text encoding it does not
// handle, or one carrying a structure outside the narrow subset above.
var ErrUnsupported = errors.New("sqlite: not a database this reader handles")

const (
	headerSize  = 100
	magic       = "SQLite format 3\x00"
	minPageSize = 512
	maxPageSize = 65536

	// maxDepth bounds a b-tree descent. Real trees are a handful of levels
	// deep; anything deeper is a cycle in a file being written under us, and
	// following it would be an infinite read rather than a wrong answer.
	maxDepth = 32
)

// DB is a read-only view of one database file plus its write-ahead log.
//
// It is not safe for concurrent use: it belongs to whatever polls the tab that
// opened it.
type DB struct {
	f    *os.File
	path string

	pageSize int
	// usable is the page size minus the reserved region at the end of every
	// page. Payload arithmetic is all in terms of it.
	usable int
	// pages is how many pages the committed snapshot has. A page beyond it is
	// not part of the database, whatever the file's length says.
	pages uint32
	// change is the file change counter as the committed snapshot has it, which
	// is what says whether the header's own page count can be trusted.
	change uint32
	// file is the *main file's* counter, which moves only when the main file is
	// written — a checkpoint, under normal WAL operation. It is what Stable
	// watches, and it is deliberately not `change`: while a database's first
	// transactions are still in the log, the two are different numbers.
	file uint32

	wal    wal
	schema map[string]*Table
}

// Open reads a database's header and lays its write-ahead log over it. The file
// is opened read-only, and nothing in this package ever opens it any other way.
func Open(path string) (*DB, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	db := &DB{f: f, path: path}
	db.wal.path = path + "-wal"
	if err := db.readPageSize(); err != nil {
		f.Close()
		return nil, err
	}
	if err := db.reload(); err != nil {
		f.Close()
		return nil, err
	}
	return db, nil
}

func (db *DB) Close() error { return db.f.Close() }

// Refresh re-reads what a writer can have changed since the last look: the
// header, and whatever the log has grown by. The log is read forward from where
// the last scan stopped unless it was reset, so a poll costs the frames that were
// actually written rather than the whole log.
func (db *DB) Refresh() error { return db.reload() }

// reload brings the log, the header and the schema up to date, in that order —
// the order matters, because page 1 carries the header and page 1 can itself be
// in the log. A database whose first transaction has not been checkpointed yet
// has a header in the main file that is *older* than the database it describes,
// so reading the header before the log would read a database that no longer
// exists.
func (db *DB) reload() error {
	if err := db.wal.refresh(db.pageSize); err != nil {
		return err
	}
	if err := db.readHeader(); err != nil {
		return err
	}
	file, err := db.changeCounter()
	if err != nil {
		return err
	}
	db.file = file
	return db.readSchema()
}

// Stable reports whether the snapshot a batch of reads was taken from still
// stands: the main file has not been written to and the log has not been reset or
// rewound underneath it. A caller that gets false discards what it read and asks
// again on its next beat, rather than believing a mixture of two snapshots.
func (db *DB) Stable() bool {
	file, err := db.changeCounter()
	if err != nil || file != db.file {
		return false
	}
	return db.wal.stable()
}

// readPageSize reads the one field that cannot change under a live database and
// that the log cannot be read without. It comes from the main file, since
// resolving it from the log would need it already.
func (db *DB) readPageSize() error {
	var buf [headerSize]byte
	if _, err := db.f.ReadAt(buf[:], 0); err != nil {
		return err
	}
	size, ok := pageSizeOf(buf[:])
	if !ok {
		return ErrUnsupported
	}
	db.pageSize = size
	return nil
}

// pageSizeOf validates the magic and reads the page size out of a header.
func pageSizeOf(buf []byte) (int, bool) {
	if len(buf) < headerSize || string(buf[:16]) != magic {
		return 0, false
	}
	size := int(binary.BigEndian.Uint16(buf[16:18]))
	if size == 1 {
		size = maxPageSize // the header's way of saying 65536
	}
	if size < minPageSize || size > maxPageSize || size&(size-1) != 0 {
		return 0, false
	}
	return size, true
}

// readHeader parses the header out of the newest copy of page 1 and validates
// everything this reader relies on.
func (db *DB) readHeader() error {
	buf, err := db.headerPage()
	if err != nil {
		return err
	}
	size, ok := pageSizeOf(buf)
	if !ok || size != db.pageSize {
		// A page size that changed under a live database means a VACUUM has
		// rewritten it, and every offset this reader holds is now about a
		// different file.
		return ErrUnsupported
	}
	reserved := int(buf[20])
	if reserved < 0 || reserved >= size-minPageSize/2 {
		return ErrUnsupported
	}
	// Text encoding: 1 is UTF-8. The other two are UTF-16, which this reader
	// does not decode rather than decoding wrongly.
	if enc := binary.BigEndian.Uint32(buf[56:60]); enc != 1 {
		return ErrUnsupported
	}
	db.usable = size - reserved
	db.change = binary.BigEndian.Uint32(buf[24:28])

	// The log's last commit is the authoritative size whenever there is one.
	// Otherwise the header's page count stands, and only when the
	// version-valid-for number says it was written by the same transaction as
	// the change counter — the same fallback SQLite itself makes.
	if db.wal.pages > 0 {
		db.pages = db.wal.pages
		return nil
	}
	pages := binary.BigEndian.Uint32(buf[28:32])
	valid := binary.BigEndian.Uint32(buf[92:96])
	if pages == 0 || valid != db.change {
		info, err := db.f.Stat()
		if err != nil {
			return err
		}
		pages = uint32(info.Size() / int64(size))
	}
	db.pages = pages
	return nil
}

// headerPage returns the newest copy of page 1, which is where the header lives.
// It deliberately does not go through page: the bounds page enforces come from
// the header this is about to read.
func (db *DB) headerPage() ([]byte, error) {
	if data, ok := db.wal.page(1); ok {
		return data, nil
	}
	buf := make([]byte, db.pageSize)
	if _, err := db.f.ReadAt(buf, 0); err != nil {
		return nil, err
	}
	return buf, nil
}

// changeCounter re-reads just the counter, for Stable.
func (db *DB) changeCounter() (uint32, error) {
	var buf [4]byte
	if _, err := db.f.ReadAt(buf[:], 24); err != nil {
		return 0, err
	}
	return binary.BigEndian.Uint32(buf[:]), nil
}

// page returns one page of the committed snapshot: the log's version of it where
// the log has one, and the main file's otherwise.
func (db *DB) page(n uint32) ([]byte, error) {
	if n == 0 || n > db.pages {
		return nil, fmt.Errorf("sqlite: page %d outside a database of %d pages", n, db.pages)
	}
	if data, ok := db.wal.page(n); ok {
		return data, nil
	}
	buf := make([]byte, db.pageSize)
	if _, err := db.f.ReadAt(buf, int64(n-1)*int64(db.pageSize)); err != nil {
		return nil, err
	}
	return buf, nil
}
