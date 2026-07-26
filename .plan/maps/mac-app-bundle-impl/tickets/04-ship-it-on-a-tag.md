---
type: task
blocked_by: [03]
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
the Gatekeeper instructions — the exact wording ticket 03 verified against a real
macOS, not a paraphrase. It also gains **one line naming the terminal diagnostic
path**, `/Applications/chartr.app/Contents/MacOS/chartr`, because the bundle has
no failure dialog by design and that command is the only way an operator sees why
a launch died. The release-notes footer gains one line under best-effort.

Both must say plainly that the app is unsigned and not notarized, and name the
architecture the image carries, so a cautious operator can decide for themselves
and an Intel operator is not left guessing. Nothing here undersells the cost to
make the download more appealing.

The ADR work is **not** in this ticket — ticket 02 did it, where the premise
broke.

## Done when

A tag builds and attaches the disk image and its sidecar as best-effort assets
alongside the loose shells, with the supported binaries and `checksums.txt`
unchanged and unmentioned by the new step. The shells job still carries
`continue-on-error` and its dependency on the published release, and a simulated
packaging failure attaches nothing while the release itself succeeds. The README
and the release-notes footer carry the verified Gatekeeper wording, state the app
is unsigned and un-notarized, and name its architecture; the README also names
the terminal diagnostic path. `go vet ./...`, `go test ./...`, the frontend
`check` / `build` / `vitest`, and the no-amber check are green.

## Answer

Three files, no new job, no new target.

**Release wiring.** `.github/workflows/release.yml` gains one step in the
`shells` matrix job, between the shell build and the upload:

```yaml
- name: Package the macOS app bundle and disk image
  if: matrix.goos == 'darwin' && steps.build_shell.outcome == 'success'
  run: make dmg WEBVIEW_VERSION=${GITHUB_REF_NAME#v}
  shell: bash
```

and the upload glob widens from `build/shell/*` to
`build/shell/* build/macapp/*.dmg build/macapp/*.dmg.sha256`. That is the whole
change to the pipeline. The step carries **no `id`** — nothing needs to branch on
its outcome, and adding one would only invite a later `if:` that softens the
failure. Ticket 03's decision to delete the scratch root is what makes the glob
safe to widen: `build/macapp/` also holds the staged bundle *directory*, and
`gh release upload` would choke on it, so the glob names the two files rather
than the directory. It survived a stray `.DS_Store` in the same directory for the
same reason. `nullglob` (already set) drops both macapp patterns unmatched on the
Linux and Windows legs, where the packaging step is skipped by its `if` and
`build/macapp/` never exists.

**The tiering structure is untouched and was re-read to confirm it:** `shells`
still has `continue-on-error: true` and `needs: release`, so the supported
release is published and checksummed before packaging is attempted. Packaging
runs *inside* that job rather than beside it, which is what makes a failure
structurally incapable of reaching the release. The step comment says so and says
not to lift it out or make it gating.

**What a packaging failure does, verified rather than reasoned about.** Sabotaged
`hdiutil` to exit 1 and ran the real target: `make dmg` exits 2, and
`build/macapp/` is left holding the bundle directory and the scratch root with
**no `.dmg` and no sidecar in it** — so there is not even a half-built image to
attach. In Actions the failing step then fails the job, and because
`Attach shell to the release` has an `if:` with no status function, GitHub ANDs
an implicit `success()` into it and the upload is **skipped entirely**.
`continue-on-error` swallows the job failure. Net: a packaging failure attaches
nothing, and the published release is untouched. **Note the deliberate
consequence** — "nothing" includes the loose macOS shell, which would otherwise
have uploaded. That is the ticket's own "attaches nothing", and buying the shell
back would mean `if: always()` on the upload, i.e. exactly the softening the
structure exists to prevent. Raising it here rather than taking it.

**The happy path was driven end to end on this Mac**, not just read:
`make dmg WEBVIEW_VERSION=1.4.0` from a clean `build/`, then the upload step's
own bash replayed verbatim against the result. It resolves to exactly four
assets — `chartr-shell_1.4.0_darwin_arm64` and its `.sha256`,
`chartr_1.4.0_darwin_arm64.dmg` and its `.sha256`. No directory, no `.DS_Store`,
nothing from `build/goreleaser`, and `checksums.txt` is never named by either
step.

**Documentation.** The README's support-tiers section gains a fourth subsection,
*Best-effort — the macOS app in a disk image*, after the shells and before the
Windows note. The Gatekeeper steps are ticket 03's verified wording, checked
line by line against the `READ ME FIRST.txt` read back out of a freshly mounted
image — same dialog title, same "Apple could not verify…" sentence, **Done** and
not the highlighted **Move to Trash**, the Privacy & Security → Security row
quoted as it reads, the double **Open Anyway** with authentication between, the
dead right-click bypass called out, and the non-recursive `xattr -d` line as
written. It states the app is unsigned and not notarized with the reason
(ad-hoc is what Apple Silicon requires to execute at all, not a half-measure
toward a Developer ID), names the architecture as Apple Silicon `arm64` with the
filename convention and Intel operators pointed at the supported binary, and
carries the one diagnostic line: `/Applications/chartr.app/Contents/MacOS/chartr`
prints the startup error Finder discards. The release-notes footer in
`.goreleaser.yaml` gains one *Best-effort, macOS* paragraph under the existing
best-effort one, saying the same three things in a sentence.

The workflow's own header comment — which enumerates what a tag cuts — gains the
disk image in its second bullet, because it now reads as false without it.

**No ADR work**, per the ticket: 0016 landed in ticket 02 and already records the
release structure and the Gatekeeper cost. Nothing in this ticket touches the
supported lane, goreleaser's `builds`/`archives`/`checksum` blocks, or the
Makefile.

**Done-when, clause by clause:** the packaging step and widened glob attach the
image and sidecar beside the loose shells, verified by replaying the step ✓; the
supported binaries and `checksums.txt` unchanged, and unmentioned by the new step
✓; `continue-on-error` and `needs: release` still on the job ✓; simulated
packaging failure attaches nothing with the release untouched ✓; README and
footer carry the verified wording, the unsigned/un-notarized statement and the
architecture ✓; README names the terminal diagnostic path ✓. `go vet ./...`
clean, `go test ./...` green, `npm run check` 0 errors / 0 warnings,
`npm run build` succeeded, `vitest` 185 passed, and the built stylesheet carries
no amber.

**Not verified, and cannot be from here:** a real tag. Everything above is the
real target and the real step body run on a real Mac, but the first `v*` push is
what proves the assets land on the releases page.
