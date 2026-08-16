//go:build !darwin && !linux

package proc

// Reading another process's start time, working directory and environment has no
// portable interface: macOS answers through sysctl and lsof, Linux through
// /proc, and the two share nothing. Rather than acquire an implicit Unix
// dependency for the sake of compiling, every other platform reports
// unavailable — a transcript that is never bound, a title that never arrives,
// and a cockpit that is otherwise exactly as it was.
//
// This is the same shape the foreground-process seam already has, and the reason
// the cross-platform build keeps compiling while initial platform support is
// macOS and Linux.
func Lookup(pid int, allow []string) (Info, error) { return Info{}, ErrUnsupported }
