---
type: task
blocked_by: [02]
---

# Delete `pinned`

## Question

Remove the ordering authority the stored order replaced. This is the contract half
of the sequence: tickets 01 and 02 left `pinned` in place but reading it for
nothing, and it now goes end to end so the codebase describes what the app
actually does.

**What goes.** `Entry.Pinned` and `Registry.SetPin` in `internal/registry`; the
`POST /api/spaces/{id}/pin` route and its handler in `internal/server`; the
`pinned` field on the wire model in `internal/model` and its mirror in
`web/src/lib/model.ts`. There is no pin control in the cockpit to remove — the
flag was reachable only through the API — so this is a smaller deletion than it
looks. Confirm that before starting rather than assuming it: a control added since
this ticket was written is part of the deletion.

**The TOML key is ignored, not rejected.** An existing `spaces.toml` still
carrying `pinned` must load without complaint; the key is simply not read and
stops being written on the next save. An operator who upgrades does not have to
touch the file, and a stale key never produces a warning about something they
cannot act on.

**Comments are part of the deletion.** Several comments across `internal/model`,
`internal/registry`, `web/src/App.svelte` and the frontend tests describe the
sidebar as pinned-first-then-recency. That description is now false in every one
of them. A stale comment about ordering is exactly the drift that makes the next
session re-derive the wrong rule, so grep for it and fix each site.

**Existing tests that assert pin ordering are the specification of the old
behaviour** — `spaces_test.go` covers pin ordering directly. They are replaced by
the ordering assertions ticket 02 added, not deleted quietly; the replacement
should be visible in the diff as a substitution.

Tests lead: assert that `POST /api/spaces/{id}/pin` returns `404`, and that a
registry file still carrying `pinned = true` loads with every space present and in
its stored order — the flag having no effect on the sequence. `CONTEXT.md` has no
entry for pin; check rather than assume, and add nothing.

Done when: no `Pinned`, `SetPin`, `pin` route or `pinned` wire field remains in
the Go or TypeScript source; an old registry file loads cleanly and drops the key
on its next save; no comment anywhere still describes the sidebar as ordered by
pin or recency; `go vet ./...`, `go test ./...` and the frontend `check`, `build`
and `vitest` scripts pass, with no amber in the built CSS.
