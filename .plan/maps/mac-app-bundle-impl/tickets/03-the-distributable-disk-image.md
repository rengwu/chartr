---
type: task
blocked_by: [02]
---

# The distributable disk image

## Question

Turn the bundle into the one file an operator downloads. A build target stages a
directory containing the app, a symbolic link to `/Applications` as the drag
target, and a short plain-text file carrying the Gatekeeper instructions;
compresses it into a read-only image named for project, version and architecture;
and writes a **per-asset `.sha256` sidecar** beside it. The sidecar is not a
detail — the supported release owns `checksums.txt`, and a best-effort artifact
never mutates it.

**The architecture in the name is load-bearing**, not decoration. Ticket 02 ships
one slice deliberately, so the name is what tells an operator whether the image
is theirs — and it is what lets a second architecture appear beside it later
without renaming the first.

Off macOS the target prints a line and **succeeds**, like the bundle target it
builds on.

**No styled disk-image window.** Background art with positioned icons needs
scripting the Finder and committing a window-state file — cosmetic work on a tier
that ships with a "your Mac will block this on first launch" note in the box. It
is declined until the tier stops carrying that note.

**The Gatekeeper instructions are this ticket's real deliverable, and they are
perishable.** The app is ad-hoc signed and not notarized, so a disk image
downloaded through a browser carries the quarantine attribute and the first
launch is blocked. Apple has changed the unblock path within recent memory — the
long-standing right-click-to-Open bypass no longer clears it — so **do not copy
the spec's wording forward on faith.** Verify against the macOS you are on by
simulating a quarantined download: apply the quarantine attribute by hand to the
installed app, launch it, and follow the instructions you intend to ship. What
goes in the box is what you watched work. Publishing unverified instructions is
worse than publishing none.

Nothing here is defeating quarantine. The cost is stated plainly, in one
sentence, in advance.

## Done when

The disk-image target produces an image and its sidecar on a real Mac; the
checksum verifies; `checksums.txt` is untouched. Mounting the image shows the app
beside an `/Applications` link and the instruction file, and dragging installs
the app. A quarantined copy of the installed app is blocked on first launch
exactly as the instruction file warns, and the unblock path it names is followed
successfully on the current macOS — with the answer stating which macOS version
that was checked against. Run off macOS, the target prints its line and exits 0.
`go vet ./...`, `go test ./...`, the frontend `check` / `build` / `vitest`, and
the no-amber check are green.
