# The macOS app bundle — implementation

## Destination

The [spec](../mac-app-bundle/spec.md) implemented end to end: an operator on a Mac
downloads one `.dmg` from the releases page, drags chartr into `/Applications`,
clicks through the one Gatekeeper prompt the disk image warned them about, and
launches the cockpit from Launchpad — with the mark on the icon, the app's own
name in the menu bar, their existing spaces already registered, and their state
written where the command-line binary writes it. Done looks like a tag attaching
an unsigned, ad-hoc-signed, single-architecture bundle and its checksum sidecar
as a best-effort asset, without the supported cgo-free release or its manifest
having been touched at any point.

**Four tickets, and that is the point.** This map is deliberately the smallest
route to a `.dmg`: the spec cut a native failure dialog, a universal binary and a
round of ADR amendments, each recorded in *Out of scope* below with the condition
that brings it back. A ticket that finds itself wanting one of them is finding a
cut, not a gap — see the note on re-opened decisions.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions are settled in the
[spec](../mac-app-bundle/spec.md), which is the single source of truth here. This
effort has no planning map: the spec was synthesized from a working conversation
rather than grilled out on a map. So there is nowhere to send a re-opened
decision — if implementation exposes one as wrong, **stop and raise it with the
operator** rather than quietly deviating.

**Per-session reading order:** the spec, then this map, then your ticket. The
spec carries the settled seams and symbols; prefer them to brittle line-level
paths. Vocabulary comes from `CONTEXT.md` at the repo root. The ADRs under
`docs/adr/` are binding, and this effort **writes exactly one new ADR** — in
ticket 02, the ticket that falsifies 0013's premise. That ADR **names what it
supersedes** in 0011 and 0013; neither of those files is edited. Amending them in
place was considered and cut. A ticket that touches an ADR says so in its answer.

**Sequencing is a straight line:** `01 → 02 → 03 → 04`. Ticket 01 comes first
because it is load-bearing in a way the ordering hides: until the runtime root is
anchored, a bundled launch dies claiming the single-instance lock, before a
window exists, writing to a stream nobody reads — so every packaging ticket after
it would be verified against an app that cannot start. 01 is verifiable from a
terminal with no bundle in existence. After it, each ticket needs the artifact
the one before it produced, so there is no parallelism to find here.

**One seam, and it is the existing one.** All testable behaviour goes in the
shell package's **tag-free half** — where the single-instance lock already lives,
compiling and testing at `CGO_ENABLED=0` in the ordinary suite. Bundle detection
and root resolution **take their inputs as arguments** so a test drives them with
constructed paths and never needs a real bundle, display or home directory; the
lock's own test file is the prior art for the shape. Nothing about the property
list, the signature or the disk image is unit-tested — those are properties of an
artifact produced by platform tooling, verified by reading the built artifact
back. ADR 0013 already establishes that stance for the window and the menu.

**The tiering guarantee is structural and must stay that way.** The packaging
runs on the macOS leg of the existing `continue-on-error` shells job, which
`needs` the already-published release. No ticket may weaken that to accommodate
the new artifact, and no ticket may write into `checksums.txt` — a best-effort
asset carries its own per-asset `.sha256` sidecar.

**Ship unsigned, honestly.** There is no Apple Developer account, so the app is
ad-hoc signed only (which Apple Silicon requires to execute at all) and not
notarized. Gatekeeper *will* block the first launch. Nothing in this effort tries
to defeat quarantine; the cost is stated in the disk image, the release notes and
the ADR. **The unblock instructions are perishable** — Apple has changed that path
within recent memory — so ticket 03 verifies them against the current macOS by
simulating a quarantined download rather than copying them forward from the spec
on faith.

**A bundled failure is silent, on purpose.** There is no failure dialog: fatal
startup errors stay on standard error, which Finder discards. Ticket 01 removes
the cause that made this likely, and the diagnostic is running the bundled
executable straight from a terminal — which ticket 02 confirms still works and
ticket 04 documents. If a ticket is tempted to add an alert to make its own
verification easier, that is the cut trying to grow back.

**Before commit:** run the CLAUDE.md gates — `go vet ./...` and `go test ./...`
(the embed test compiles against `web/dist/`), the frontend `check` / `build`
scripts and `vitest`, and confirm **no amber in the built CSS**. Review the diff
and drive the real behaviour where a "Done when" is only real at runtime — for
this map that means a real Mac, and from ticket 03 onward a real double-click. No
map linter is wired in this repo.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

- **01 — survive a launch with no terminal**: bundled-launch awareness is one new tag-free file, `cmd/webview/bundle.go`, beside the lock — `isBundled(exePath)` as a pure path predicate (`MacOS` / `Contents` / `.app`, no stat and no platform call), `runtimeRoot(explicit, exePath, configRoot)` returning the explicit root, else the working directory when loose, else the config root, and `parseFlags(args, bundled)` on a `ContinueOnError` `FlagSet` that keeps what parsed and swallows the rest only when bundled. The home-anchored root is **not** re-derived in the shell: `server.userConfigRoot` was exported as **`server.ConfigRoot`**, which the shell needs before it can construct a `Server`, so one function decides where both artifacts write and the bundled app cannot fork the command-line binary's state. `main` reads `os.Executable` once; an unreadable path is "not a bundle". Both confirmations held — the registry resolves through the config root (the simulated bundled launch listed the operator's real spaces) and `env.HydratePATH()` still runs first in `run()`. Verified on a real Mac against a hand-staged `chartr.app/Contents/MacOS/chartr` launched from `/` with an injected `-psn_0_…`: lock at `~/.config/chartr`, nothing at `/.chartr`, health 200, window drawn, clean `SIGTERM` teardown. **Deviation, raised not quiet:** `main` was already red — `7ea529a` left `terminal.scaffold.toml` at font size 14 against a default of 13 — so the scaffold was corrected to meet the gate. [ticket](tickets/01-survive-a-launch-with-no-terminal.md)

- **02 — the unsigned bundle**: `make bundle` is the shell target plus packaging — it invokes `make webview` with the same three stamp variables and packages that executable into `build/macapp/chartr_<version>_darwin_<arch>/chartr.app`, so the app keeps the plain name an operator drags while the architecture rides the staged directory around it. The property list carries the declared keys and nothing else (identifier `io.github.rengwu.chartr`, no `LSUIElement`, **no ATS exception** — a verified absence: the bundled window loads the cockpit over loopback with nothing declared); `CFBundleShortVersionString` takes the tag's numeric part via a pattern that only accepts a tag shape and falls back to `0.0.0` rather than emit nonsense from an untagged `git describe`, while `CFBundleVersion` takes the whole stamp. **One open decision taken and named: `LSMinimumSystemVersion` is read off the executable's own `LC_BUILD_VERSION`** rather than hard-coded — whatever the runner's SDK targeted is the real floor, and a lower claim only promises a launch the loader refuses. The icon is generated (`sips` → nine-entry iconset, no upscales → `iconutil`) from `web/public/icon-512.png`, the same bytes the PWA and the runtime Dock icon use; `codesign --sign -` runs last and `--verify --strict` gates it; off macOS the target prints its line and exits 0 before doing any work. ADR **0016** is written and names what it supersedes in 0011 (its best-effort enumeration) and 0013 (its bare-binary premise) — neither edited. The falsified comments in `icon_darwin.go` and `menu_darwin.go` are corrected in the same slice and the runtime app-name seeding and icon are kept, because the loose shell still has nothing else. Verified on macOS 27.0 / arm64: `plutil -lint` OK, `Signature=adhoc`, thin `arm64`, a Finder launch drew the cockpit with the operator's real spaces and `chartr` in the menu bar, the mark shows in Finder, the Dock and the ⌘-Tab switcher, ⌘Q / window close / `SIGTERM` all tore down cleanly (⌘Q's stale lock taken over on the next launch), and the terminal diagnostic path still prints to standard error and exits 1. [ticket](tickets/02-the-unsigned-bundle.md)

- **03 — the distributable disk image**: `make dmg` stands to `make bundle` as `bundle` stands to `webview` — same three stamp variables, packaging what the target below it produced — and writes `build/macapp/chartr_<version>_darwin_<arch>.dmg` plus a per-asset `.sha256`, with the **architecture on the image's own name**, not only on the directory around it. Staging is `ditto` (it preserves the extended attributes the ad-hoc signature lives in: `codesign --verify --strict` passes against the copy inside the mounted image), a `/Applications` symlink, and `READ ME FIRST.txt`; the scratch root is deleted so `build/macapp/` holds only the bundle directory, the image and the sidecar. `hdiutil -fs HFS+ -format UDZO`, volume name read back off the assembled `Info.plist` with `plutil -extract` rather than re-deriving the tag pattern. No styled window; `checksums.txt` untouched; off macOS the target prints its line and exits 0. **The Gatekeeper wording was re-derived from observation and the spec's version did not survive: the block dialog's highlighted default is `Move to Trash`**, so an operator who hits Return deletes the download — the box now says so. Verified on **macOS 27.0 (26A5368g), arm64** by hand-quarantining an installed copy with a flag shape copied off a real browser download: `open` returns `-128` with **"chartr" Not Opened**; the unblock is System Settings → Privacy & Security → Security → **Open Anyway** on the row **"chartr" was blocked to protect your Mac.**, then Touch ID, then a second **Open Anyway** — confirmed by the flag flipping `0081` → `00c1` and the app then serving health 200 with its lock in `~/.config/chartr/.chartr/`. The documented terminal alternative is verified **as written**: non-recursive `xattr -d com.apple.quarantine /Applications/chartr.app` is enough; `-r` would have been cargo-culted. Noted but kept out of the operator's box: a quarantined launch runs under **App Translocation**, which is harmless only because ticket 01 anchored the runtime root away from the executable's location. [ticket](tickets/03-the-distributable-disk-image.md)

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question goes back to the operator — it does not open fog here. -->

## Out of scope

<!-- Inherited from the spec's Out of Scope; these never graduate into tickets on this map. The first three are deliberate cuts, each with the condition that brings it back. -->

- **A native failure dialog and log file** — *cut.* Fatal startup errors stay on standard error. Back when an operator reports a launch that bounced with nothing to send; until then the terminal path is the diagnostic.
- **A universal two-architecture binary** — *cut.* One slice, the host's, named in the filename. Back when an Intel operator asks; the naming makes that a `lipo` call, not a rename.
- **Amending ADR 0011 and 0013 in place** — *cut.* The new ADR names what it supersedes instead. The falsified *code comments* are still corrected in ticket 02 — that part is not deferred.
- **Developer ID signing and notarization** — both need the paid account; they slot into the assembly step when there is one.
- **Auto-update in any form** — no update framework, no feed.
- **A styled disk-image window** — background art and positioned icons are cosmetics on a tier that ships with a "your Mac will block this" note in the box.
- **An installer package** — the drag-to-Applications gesture is the whole install.
- **A Homebrew cask or any channel but the releases page** — ADR 0011's distribution decision is unchanged.
- **A custom URL scheme** — ADR 0013 declined it for want of a producer, and nothing here supplies one.
- **Windows and Linux packaging** — both keep the loose best-effort binary.
- **Promoting the shell out of the best-effort tier** — the supported artifact stays the cgo-free browser binary.
- **Any change to the supported binary or its release lane.**
- **Deleting the runtime app-name seeding or runtime icon** — the loose shell still needs both.
