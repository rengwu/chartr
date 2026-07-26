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

## Answer

`make dmg` is the second new target, and it stands to `make bundle` exactly as
`bundle` stands to `webview`: it invokes the target below it with the same three
stamp variables and packages what that produced. Output is
`build/macapp/chartr_<version>_darwin_<arch>.dmg` beside the staged bundle
directory of the same name, plus `<same>.dmg.sha256`. **The architecture is in
the image's name, not only in the directory around it** — an operator reading a
releases page sees `darwin_arm64` on the file itself, which is the whole point of
ticket 02 putting it in the staged name.

- **Staging** is `ditto` (not `cp -R`) of the assembled `chartr.app` into a
  scratch root, a `/Applications` symlink beside it, and `READ ME FIRST.txt`.
  `ditto` is deliberate: it preserves the extended attributes the ad-hoc
  signature lives in, and `codesign --verify --strict` against the copy *inside
  the mounted image* still passes with `Signature=adhoc`. The scratch root is
  deleted after the image is made, so `build/macapp/` holds only the bundle
  directory, the image and the sidecar — a glob ticket 04 can widen onto without
  catching a staging directory.
- **The image** is `hdiutil create -fs HFS+ -format UDZO`, read-only and
  compressed (4.2 MB from a 9 MB executable). Volume name is `chartr <short>`,
  read back off the assembled `Info.plist` with `plutil -extract` rather than
  re-deriving the tag pattern a second time — the artifact is the source of
  truth for its own version.
- **No styled window**, per the cut. The mounted volume is a plain Finder window
  with three items in it.
- **The sidecar** is `shasum -a 256` in `build/macapp`, per-asset.
  `checksums.txt` is untouched and unmentioned; nothing in this target writes
  outside `build/`.
- **Off macOS** it prints `the macOS disk image is only built on macOS (this is
  linux); nothing to package` and exits 0 before doing any work — verified with
  `make dmg GOOS=linux`.

**The Gatekeeper instructions were re-derived from observation, and the spec's
wording did not survive intact.** Verified on **macOS 27.0 (build 26A5368g),
Apple Silicon**, by simulating a download: `ditto` the app out of the mounted
image into `/Applications`, then
`xattr -w -r com.apple.quarantine "0081;<hex time>;Safari;<uuid>"` — the flag
shape copied off a real browser download in `~/Downloads` rather than invented.
What actually happens:

- **The block is real.** `open /Applications/chartr.app` returns `-128`
  (`userCanceledErr`) and macOS shows **"chartr" Not Opened — Apple could not
  verify "chartr" is free of malware that may harm your Mac or compromise your
  privacy.** The buttons are **Move to Trash** and **Done** — and **Move to
  Trash is the highlighted default**, so an operator who hits Return deletes the
  download. The shipped text says so in as many words; the spec did not, and
  that is the sharpest thing this verification turned up.
- **The unblock path is System Settings → Privacy & Security → Security**, which
  carries the row **"chartr" was blocked to protect your Mac.** with the same
  "Apple could not verify…" line under it (read out of the accessibility tree,
  not off a screenshot). Clicking **Open Anyway** there, authenticating with
  Touch ID or a password, and confirming **Open Anyway** once more in the dialog
  that follows launches the app. Confirmed by state, not by eye: the quarantine
  flag flips `0081` → `00c1` (user-approved), and the app then comes up serving
  `/api/health` 200 on loopback with a window named `chartr` and its lock in
  `~/.config/chartr/.chartr/`, nothing at `/.chartr`. **The operator drove the
  clicks and the authentication on their own machine; I staged each round and
  verified the transitions.** The instruction file quotes this path and names
  the macOS it was checked against, because it will go stale.
- **The terminal alternative is verified as written.**
  `xattr -d com.apple.quarantine /Applications/chartr.app` — non-recursive,
  exactly the line in the box — is sufficient: it clears only the bundle's own
  attribute (the nested executable keeps its copy), and the next `open` exits 0
  and runs the app from `/Applications/chartr.app/Contents/MacOS/chartr`
  untranslocated with no prompt. Shipping `-r` would have been cargo-culted.
- **The right-click bypass is dead**, as the map warned, so the file says so
  rather than leaving old advice to be found elsewhere.

**One observation worth recording, deliberately not put in the operator's box.**
While quarantined-and-unapproved, macOS launches the app under **App
Translocation** — a read-only copy at a random
`/private/var/folders/…/AppTranslocation/<uuid>/d/chartr.app` — before the block
resolves. This is harmless here precisely because of ticket 01: the runtime root
is anchored to the config root rather than derived from the executable's
location, so a translocated launch still found the operator's real spaces and
wrote its lock to the same place. It is noise to an operator and would only
frighten them, so it stays out of the text.

**Done-when, clause by clause:** image and sidecar produced on a real Mac ✓;
`shasum -a 256 -c` verifies ✓; `checksums.txt` untouched ✓; mounting shows the
app beside the `/Applications` link and `READ ME FIRST.txt` ✓; dragging installs
— done through Finder's own copy engine
(`tell application "Finder" to duplicate …`), and the installed copy verifies
`--strict` ✓; a quarantined copy is blocked on first launch exactly as the file
warns ✓; the unblock path it names was followed successfully on macOS 27.0 ✓;
`make dmg GOOS=linux` prints its line and exits 0 ✓.

`go vet ./...` clean, `go test ./...` green, `npm run check` 0 errors /
0 warnings, `npm run build` succeeded, `vitest` 185 passed, and the built
stylesheet carries no amber.
