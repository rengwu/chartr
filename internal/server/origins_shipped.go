//go:build !chartrdev

package server

// devOriginPatterns adds nothing in a build that is not explicitly a development
// one — which is every build that is ever shipped.
//
// This is the closed half of the `chartrdev` pair (its counterpart is
// origins_dev.go). The gate is a build tag rather than a flag on purpose: a flag
// that defaults to off is still present in the released binary, and anything that
// can pass an argument — a launcher, a desktop entry, a talked-through support
// step — can turn it back on. A tag means the extra origin is not compiled in at
// all, so there is no argument and no environment variable that widens a shipped
// chartr. `make dev-backend` is the only thing that builds with it.
func devOriginPatterns() []string { return nil }
