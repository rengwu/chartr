---
type: grilling
blocked_by: [12]
---

# Shipping the harness cross-platform

## Question

The harness targets macOS, Linux and Windows and is meant to be distributed, not merely run on the author's machine. That was decided (ADR 0006) partly *because* it rules out per-platform native work — so this ticket makes sure the bill was read correctly.

- **What is the artifact?** One Go binary with the frontend embedded, serving a browser? That plus a native webview shell — wayfinder-maps builds both, and its webview path needs cgo? Both, as separate downloads, with different support promises?
- **cgo and the webview are the sharp edge.** A pure-Go, browser-only build cross-compiles trivially. The native shell does not, and that asymmetry should drive the answer rather than be discovered during a release.
- **How do people get it** — `go install`, released binaries, Homebrew, a Claude Code plugin marketplace entry (wayfinder-maps ships one, and there is an irony worth noting in distributing an agent-agnostic tool through one agent's marketplace)?
- **What must already be present.** The agent CLIs are not ours to ship, and a committed workspace config can name one the operator lacks (ticket 05). Is there a doctor command, and what does a cold start look like for someone who has none of them?
- **Windows deserves an explicit answer, not an assumption.** PTYs mean ConPTY; path handling differs; and whether the agent CLIs we target even run there is a question that could quietly demote Windows to unsupported. Better to decide that than to imply it.
