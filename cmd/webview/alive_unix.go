//go:build !windows

package main

import (
	"errors"
	"os"
	"syscall"
)

// processAlive probes a pid with signal 0 — the portable Unix liveness test.
// EPERM means the process exists and belongs to someone else, which still
// counts as alive: a lock we cannot signal is a lock we must not steal.
func processAlive(pid int) bool {
	if pid <= 0 {
		return false
	}
	p, err := os.FindProcess(pid)
	if err != nil {
		return false
	}
	err = p.Signal(syscall.Signal(0))
	return err == nil || errors.Is(err, os.ErrPermission)
}
