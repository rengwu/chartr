---
type: task
blocked_by: [04]
---

# Ship it on a tag

## Question

Make a tag attach the disk image, and tell operators what they are getting.

**Release wiring.** The release workflow's macOS leg gains **one** packaging
step, guarded on the shell having built, and the upload glob widens to include
the disk image and its sidecar. That is the whole change: the packaging runs
inside the `continue-on-error` shells job that already `needs` the published
release, so a packaging failure attaches nothing and leaves the supported release
untouched. **That structure is the guarantee and this ticket may not weaken it** —
not to make the new artifact more reliable, not to surface a packaging failure
more loudly, not for anything. If packaging looks like it wants to be gating,
that is a decision to raise with the operator, not to take here.

**Documentation.** The README's support-tiers section gains the disk image and
the Gatekeeper instructions — the exact wording ticket 04 verified against a real
macOS, not a paraphrase. The release-notes footer gains one line under
best-effort. Both must say plainly that the app is unsigned and not notarized, so
a cautious operator can decide for themselves; nothing here undersells the cost
to make the download more appealing.

The ADR work is **not** in this ticket — ticket 03 did it, where the premise
broke.

## Done when

A tag builds and attaches the disk image and its sidecar as best-effort assets
alongside the loose shells, with the supported binaries and `checksums.txt`
unchanged and unmentioned by the new step. The shells job still carries
`continue-on-error` and its dependency on the published release, and a simulated
packaging failure attaches nothing while the release itself succeeds. The README
and the release-notes footer carry the verified Gatekeeper wording and state the
app is unsigned and un-notarized. `go vet ./...`, `go test ./...`, the frontend
`check` / `build` / `vitest`, and the no-amber check are green.
