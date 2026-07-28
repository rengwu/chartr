package terminal

import "github.com/aymanbagabas/go-pty"

// releaseSlave is a no-op on Windows. ConPTY has no slave descriptor for the
// parent to be holding open: the read side is a pipe that closes when the
// pseudoconsole is freed, so the read loop already ends when the child exits
// (ADR 0006 as amended). See slave_unix.go for what this exists to fix.
func releaseSlave(pty.Pty) {}
