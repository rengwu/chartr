//go:build windows

package main

import "golang.org/x/sys/windows"

// processAlive asks Windows directly: signal 0 is not a thing there, and
// os.Process.Signal refuses anything but Kill. A handle that opens and reports
// STILL_ACTIVE is a live process; anything else (gone, access denied on a
// reused pid, exited) reads dead and the lock is takeable.
func processAlive(pid int) bool {
	if pid <= 0 {
		return false
	}
	h, err := windows.OpenProcess(windows.PROCESS_QUERY_LIMITED_INFORMATION, false, uint32(pid))
	if err != nil {
		return false
	}
	defer windows.CloseHandle(h)

	var code uint32
	if err := windows.GetExitCodeProcess(h, &code); err != nil {
		return false
	}
	return code == 259 // STILL_ACTIVE
}
