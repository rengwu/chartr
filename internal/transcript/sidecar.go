package transcript

import (
	"os"
	"time"
)

// sidecar is the other kind of cursor in this package: a small JSON file that is
// *rewritten whole* rather than appended to, which is how three providers keep
// the metadata a transcript needs.
//
// Kimi's state.json and Grok's summary.json hold the session's own title beside
// an append-only log, and Claude's sessions/<pid>.json holds the session a live
// process is driving. None of them can be tailed, because none of them grows —
// the file is replaced each time it changes. So the cursor over one is a stat: an
// unchanged sidecar costs one syscall per poll instead of a read and a parse,
// which is what keeps several open tabs off a provider's metadata.
type sidecar struct {
	path string
	mod  time.Time
	size int64
}

// changed reports whether the file has been rewritten since the last look. A
// file that is not there has not changed — an absent sidecar is an ordinary
// state, not an event.
func (s *sidecar) changed() bool {
	info, err := os.Stat(s.path)
	if err != nil {
		return false
	}
	if info.ModTime().Equal(s.mod) && info.Size() == s.size {
		return false
	}
	s.mod, s.size = info.ModTime(), info.Size()
	return true
}

// read returns the file's bytes, refusing anything larger than max. These files
// are a few hundred bytes to a few kilobytes; something of a different order is
// not the file this adapter is looking at.
func (s *sidecar) read(max int64) ([]byte, bool) {
	info, err := os.Stat(s.path)
	if err != nil || info.IsDir() || info.Size() > max {
		return nil, false
	}
	data, err := os.ReadFile(s.path)
	if err != nil {
		return nil, false
	}
	return data, true
}
