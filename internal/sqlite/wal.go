package sqlite

import (
	"encoding/binary"
	"os"
)

// The write-ahead log is where a live SQLite database keeps everything committed
// since the last checkpoint, so a reader that ignored it would be reading the
// database as it stood minutes ago — which for a session being written right now
// means reading nothing at all.
//
// A log is a 32-byte header followed by frames of (24-byte header + one page).
// Two things make it safe to read without a lock, and this file is those two
// things:
//
//   - Every frame carries the log's salts and a checksum that continues a chain
//     from the log header. A frame that does not match both is not a frame this
//     log wrote: it is either the tail of an older, longer log being overwritten,
//     or a frame the writer is in the middle of appending. Either way the scan
//     stops there.
//   - Only frames up to the last *commit* frame — one whose page-count field is
//     non-zero — are visible. A transaction still being written has no commit
//     frame yet, so it cannot be read half-applied.
//
// The scan is incremental: a log that has only grown is read forward from where
// the last scan stopped, carrying the running checksum with it. A log whose salts
// or checkpoint sequence changed has been reset by a checkpoint, and is read
// again from the beginning.

const (
	walHeaderSize = 32
	frameHeadSize = 24

	// walMagic is the header's magic with the low bit clearing to say which byte
	// order the checksums are computed in.
	walMagicLE = 0x377f0682
	walMagicBE = 0x377f0683

	walFormat = 3007000
)

// wal is the index over one write-ahead log: which committed frame holds each
// page, and how far the log has been read.
type wal struct {
	path string

	// frames maps a page number to the offset of the frame holding its newest
	// committed copy.
	frames map[uint32]int64
	// pages is the database size the last commit frame declared.
	pages uint32

	// The scan's position and the state needed to continue it.
	off        int64
	salt1      uint32
	salt2      uint32
	checkpoint uint32
	sum0       uint32
	sum1       uint32
	native     bool // checksums are computed in big-endian word order
	pageSize   int

	// pending holds the frames of a transaction whose commit frame has not been
	// seen. They become visible together or not at all.
	pending map[uint32]int64

	data []byte
}

// refresh brings the index up to date with the log as it now stands.
func (w *wal) refresh(pageSize int) error {
	data, err := os.ReadFile(w.path)
	if err != nil {
		if os.IsNotExist(err) {
			// No log: either the database is not in WAL mode, or a checkpoint
			// removed it. Both mean the main file is the whole truth.
			w.reset()
			return nil
		}
		return err
	}
	w.data = data
	if len(data) < walHeaderSize {
		// A log that has been created but not written to yet.
		w.reset()
		return nil
	}

	magic := binary.BigEndian.Uint32(data[0:4])
	if magic != walMagicLE && magic != walMagicBE {
		return ErrUnsupported
	}
	if binary.BigEndian.Uint32(data[4:8]) != walFormat {
		return ErrUnsupported
	}
	if int(binary.BigEndian.Uint32(data[8:12])) != pageSize {
		// A log written for a different page size than the header declares is a
		// database mid-change, not one to read.
		return ErrUnsupported
	}
	checkpoint := binary.BigEndian.Uint32(data[12:16])
	salt1 := binary.BigEndian.Uint32(data[16:20])
	salt2 := binary.BigEndian.Uint32(data[20:24])

	restarted := w.frames == nil || salt1 != w.salt1 || salt2 != w.salt2 ||
		checkpoint != w.checkpoint || int64(len(data)) < w.off
	if restarted {
		// A checkpoint reset the log, or this is the first look. Everything the
		// old index said about frame offsets is now about different frames.
		w.frames = make(map[uint32]int64)
		w.pending = make(map[uint32]int64)
		w.pages = 0
		w.off = walHeaderSize
		w.salt1, w.salt2, w.checkpoint = salt1, salt2, checkpoint
		w.native = magic == walMagicBE
		w.pageSize = pageSize
		// The chain starts from the header's own checksum, which was computed
		// over the header's first 24 bytes.
		w.sum0 = binary.BigEndian.Uint32(data[24:28])
		w.sum1 = binary.BigEndian.Uint32(data[28:32])
		if !w.headerValid(data) {
			// A header whose checksum does not check out is one being written.
			w.reset()
			return nil
		}
	}
	w.scan()
	return nil
}

// headerValid recomputes the log header's own checksum, which is what the frame
// chain continues from. A header that does not check out cannot anchor a chain.
func (w *wal) headerValid(data []byte) bool {
	var s0, s1 uint32
	s0, s1 = checksum(0, 0, data[:24], w.native)
	return s0 == w.sum0 && s1 == w.sum1
}

// scan reads frames forward from the last position, committing each transaction
// as its commit frame is reached.
func (w *wal) scan() {
	frameSize := int64(frameHeadSize + w.pageSize)
	for {
		if w.off+frameSize > int64(len(w.data)) {
			return // a frame that is not all there yet
		}
		head := w.data[w.off : w.off+frameHeadSize]
		page := binary.BigEndian.Uint32(head[0:4])
		commit := binary.BigEndian.Uint32(head[4:8])
		salt1 := binary.BigEndian.Uint32(head[8:12])
		salt2 := binary.BigEndian.Uint32(head[12:16])
		want0 := binary.BigEndian.Uint32(head[16:20])
		want1 := binary.BigEndian.Uint32(head[20:24])

		if salt1 != w.salt1 || salt2 != w.salt2 || page == 0 {
			return // not a frame this log wrote
		}
		body := w.data[w.off+frameHeadSize : w.off+frameSize]
		s0, s1 := checksum(w.sum0, w.sum1, head[:8], w.native)
		s0, s1 = checksum(s0, s1, body, w.native)
		if s0 != want0 || s1 != want1 {
			return // half-written, or the tail of an older log
		}

		w.pending[page] = w.off + frameHeadSize
		w.sum0, w.sum1 = s0, s1
		w.off += frameSize

		if commit != 0 {
			// A commit frame: everything since the last one becomes visible at
			// once, and the database is now this many pages.
			for p, off := range w.pending {
				w.frames[p] = off
			}
			clear(w.pending)
			w.pages = commit
		}
	}
}

// stable reports whether the log is still the one the index was built from and
// has only grown. A reset or a rewind means a checkpoint moved the ground.
func (w *wal) stable() bool {
	info, err := os.Stat(w.path)
	if err != nil {
		return w.frames == nil || len(w.frames) == 0
	}
	if info.Size() < w.off {
		return false
	}
	var head [walHeaderSize]byte
	f, err := os.Open(w.path)
	if err != nil {
		return false
	}
	defer f.Close()
	if _, err := f.ReadAt(head[:], 0); err != nil {
		return false
	}
	return binary.BigEndian.Uint32(head[16:20]) == w.salt1 &&
		binary.BigEndian.Uint32(head[20:24]) == w.salt2 &&
		binary.BigEndian.Uint32(head[12:16]) == w.checkpoint
}

// page returns the log's copy of a page, when it has a committed one.
func (w *wal) page(n uint32) ([]byte, bool) {
	off, ok := w.frames[n]
	if !ok || off+int64(w.pageSize) > int64(len(w.data)) {
		return nil, false
	}
	return w.data[off : off+int64(w.pageSize)], true
}

func (w *wal) reset() {
	w.frames, w.pending, w.data = nil, nil, nil
	w.pages, w.off = 0, 0
	w.salt1, w.salt2, w.checkpoint = 0, 0, 0
}

// checksum is SQLite's own log checksum: two accumulators run over the data as
// 32-bit words, in the byte order the log's magic declares. It is not a
// cryptographic hash and does not have to be — it is there to tell a complete
// frame from one still being written.
func checksum(s0, s1 uint32, data []byte, native bool) (uint32, uint32) {
	read := binary.LittleEndian.Uint32
	if native {
		read = binary.BigEndian.Uint32
	}
	for i := 0; i+8 <= len(data); i += 8 {
		s0 += read(data[i:i+4]) + s1
		s1 += read(data[i+4:i+8]) + s0
	}
	return s0, s1
}
