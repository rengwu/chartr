package transcript

import (
	"bytes"
	"io"
	"os"
)

// This file is the reader four of the six providers share, and the reason the
// storage family a provider chose is invisible above the Adapter contract.
//
// Claude, Codex, Pi, Kimi and Grok all keep their conversation as append-only
// JSONL: one complete JSON object per line, written while chartr reads. The
// cursor over that is a byte offset, and the two rules that make it safe are the
// same for all of them:
//
//   - A record with no newline behind it is a record the provider is still
//     writing. It is left unconsumed, so the cursor stays behind it and the whole
//     record is read once it lands. Nothing is ever parsed from half a line.
//   - The cursor only advances over records that were consumed whole, so an
//     append is read once and a poll that finds a very large append reads what it
//     can and leaves the rest for the next one. Neither an append after the
//     cursor nor a store rewritten shorter than it causes a re-read of history:
//     the first is read forward from where the last poll stopped, and the second
//     is re-seated in history mode, which publishes a title and no turn.
//
// The provider-specific half is the fold: what a record *means*. That is the
// only thing an adapter has to write.

// jsonlSuffix is what every one of these stores names its files, and what an
// adapter checks before treating a directory entry as one.
const jsonlSuffix = ".jsonl"

const (
	// tailReadCap bounds one poll's read of newly appended bytes. Whatever is
	// left over is read on the next poll, since the cursor only ever advances
	// over records that were completely consumed.
	tailReadCap = 4 << 20

	// headCap and headRecords bound the metadata peek that accepts or rejects a
	// candidate store. Every one of these providers writes what discovery needs
	// — the working directory, the session identity, the format version — in the
	// first records of the file, so the peek reads a little way in and never the
	// conversation.
	headCap     = 64 << 10
	headRecords = 64
)

// lineFn folds one complete JSONL record into an adapter's state, returning
// false to end the binding: this is not a store the adapter can read, and
// reading past it would be a guess.
//
// history is true for the records read while the cursor is being seated. An
// adapter uses it to suppress turn events without suppressing the provider's own
// title, which is the difference between establishing a position in a
// conversation and being charged for it.
type lineFn func(line []byte, history bool) bool

// tail is a byte-offset cursor over one append-only JSONL store.
type tail struct {
	path string
	off  int64
}

// seat reads the store whole in history mode and leaves the cursor at the end of
// its last complete record. It runs when a binding is made, and again whenever
// the file turns out to be shorter than the cursor.
func (t *tail) seat(fold lineFn) bool {
	data, err := os.ReadFile(t.path)
	if err != nil {
		return false
	}
	n, ok := foldLines(data, true, fold)
	if !ok {
		return false
	}
	t.off = int64(n)
	return true
}

// advance reads whatever the provider has appended since the last call.
//
// It reports whether the store had to be re-seated — truncated or rewritten in
// place, after which the old offset means nothing and the only safe position is
// the new end — so an adapter can drop the turn it was holding open across a
// rewrite that swallowed it. ok=false ends the binding.
func (t *tail) advance(fold lineFn) (reseated, ok bool) {
	info, err := os.Stat(t.path)
	if err != nil {
		return false, false // the store went away
	}
	switch {
	case info.Size() < t.off:
		return true, t.seat(fold)
	case info.Size() == t.off:
		return false, true
	}
	data, ok := readAt(t.path, t.off, info.Size()-t.off)
	if !ok {
		return false, false
	}
	n, ok := foldLines(data, false, fold)
	if !ok {
		return false, false
	}
	t.off += int64(n)
	return false, true
}

// foldLines folds every *complete* record in data and reports how many bytes
// those records occupied. A trailing fragment with no newline is left for the
// caller's cursor to stay behind.
func foldLines(data []byte, history bool, fold lineFn) (int, bool) {
	consumed := 0
	for {
		i := bytes.IndexByte(data[consumed:], '\n')
		if i < 0 {
			return consumed, true
		}
		line := bytes.TrimSpace(data[consumed : consumed+i])
		consumed += i + 1
		if len(line) == 0 {
			continue
		}
		if !fold(line, history) {
			return 0, false
		}
	}
}

// readAt reads the bytes past the cursor, bounded.
func readAt(path string, off, size int64) ([]byte, bool) {
	if size > tailReadCap {
		size = tailReadCap
	}
	f, err := os.Open(path)
	if err != nil {
		return nil, false
	}
	defer f.Close()

	buf := make([]byte, size)
	n, err := f.ReadAt(buf, off)
	if err != nil && err != io.EOF {
		return nil, false
	}
	return buf[:n], true
}

// peek folds the opening records of a candidate store, so discovery can accept
// or reject it from its metadata without reading the conversation. read returns
// false to stop early — as soon as it has what it came for — and the whole peek
// reports false when a record it had to understand was not readable.
func peek(path string, read func(line []byte) bool) bool {
	f, err := os.Open(path)
	if err != nil {
		return false
	}
	defer f.Close()

	buf := make([]byte, headCap)
	n, err := io.ReadFull(f, buf)
	if err != nil && err != io.EOF && err != io.ErrUnexpectedEOF {
		return false
	}
	data := buf[:n]
	for records := 0; records < headRecords; records++ {
		i := bytes.IndexByte(data, '\n')
		if i < 0 {
			return true
		}
		line := bytes.TrimSpace(data[:i])
		data = data[i+1:]
		if len(line) == 0 {
			continue
		}
		if !read(line) {
			return true
		}
	}
	return true
}
