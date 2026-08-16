//go:build windows

package proc

// ConPTY exposes no foreground-process-group notion the way a Unix TTY does, so
// there is no group to enumerate — the same Unix-only affordance the activity
// sampler's own foreground lookup is. With nothing to enumerate and no process
// reader either (lookup_other.go), every resolution on Windows is unavailable
// and the build still compiles, which is all this file is here to keep true.
func Group(pgid int) []Member { return nil }
