//go:build !windows

package terminal

import "github.com/aymanbagabas/go-pty"

// releaseSlave drops the parent's copy of the PTY slave descriptor once the
// child owns it, which is what lets the read loop ever see the child exit.
//
// A master read returns EIO only when the *last* descriptor on the slave end
// closes. go-pty keeps one open for the pty's whole lifetime (`unixPty.slave`,
// released only in its Close), and `Start` dups the slave into the child on top
// of that — so after the child dies the parent is still holding the slave open,
// the master never reaches its last close, and `pty.Read` blocks forever. The
// pump goroutine then never reaps, never marks the tab dead, and never pushes
// the model that drops it.
//
// macOS hides this completely: there a master read returns EOF when the child
// exits whether or not the parent still holds the slave, so every one of these
// paths worked on the machine they were written on and hung on Linux. That
// asymmetry is why this needs to be explicit rather than left to the library.
//
// Closing it here is safe because the descriptor has already served its only
// purpose — `Start` has dup'd it onto the child's stdio. go-pty's own Close
// closes it a second time later and joins the resulting EBADF into its error,
// which the callers already discard.
//
// Windows is exempt: ConPTY has no slave descriptor and its pipe closes when the
// pseudoconsole is freed, so the read loop already terminates there.
func releaseSlave(p pty.Pty) {
	if u, ok := p.(pty.UnixPty); ok {
		if s := u.Slave(); s != nil {
			_ = s.Close()
		}
	}
}
