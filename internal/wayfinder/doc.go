// Package wayfinder is the wayfinder-maps model layer — Load, ParseMap,
// ParseTicket, derived Status, Frontier, and Lint — ported into the harness so
// it derives ticket state from `.plan/` markdown exactly as a vanilla wayfinder
// tool does (ADR 0001). The harness needs the opposite runtime from
// wayfinder-maps (live PTYs, pushed state), but the reading of a map is the same
// reading, so this layer is lifted rather than reinvented, and its test suite
// (wayfinder_test.go) travels with it as the guard against drift.
//
// Ported from github.com/rengwu/wayfinder-maps internal/wayfinder at commit
// 94a3be97d937db06574c15515ad8c0cd23854ffd (2026-07-14). Re-sync by re-copying
// these files and re-running the ported tests; the harness adds nothing to the
// derived-status table — the `proposed` status it once carried was withdrawn
// with the review feature (ADR 0004, amended), so this layer reads a map exactly
// as a vanilla tool does.
package wayfinder
