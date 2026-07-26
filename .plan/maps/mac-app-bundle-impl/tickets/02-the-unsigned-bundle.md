---
type: task
blocked_by: [01]
---

# The unsigned bundle

## Question

Add a build target that assembles a real `chartr.app` from the webview shell, and
record the decision it forces. The end-to-end behaviour: a maintainer runs one
command on their Mac and gets a bundle they can double-click to open the cockpit
in a native window, with the mark on the icon and the app's own name in the menu
bar.

**Assembly.** A conventional bundle — an information property list, the shell
executable, and a generated icon resource. The property list declares name,
display name, identifier, executable, icon, package type, both version strings, a
minimum system version, high-resolution capability and a developer-tools
category. It deliberately does **not** declare an agent-style background app, and
does **not** declare transport-security exceptions: loopback is already exempt and
the loose shell demonstrably loads the cockpit over it today. That this still
holds *inside a bundle* is an explicit check below, not an assumption.

**Two version strings, one stamp.** The human-facing short version takes the
numeric part of the tag because the format is constrained; the full build stamp
goes in the build version. Both derive from the `WEBVIEW_VERSION` / `_COMMIT` /
`_DATE` variables the build file already defines for the loose shell, so one tag
yields one identity across all three artifacts.

**The icon is generated, not curated** — downscaled from the largest mark the
cockpit already embeds, into the icon set the platform tooling expects. The
shell's runtime icon reads that same mark for the same reason: generating rather
than committing a second copy is what makes drift impossible.

**One architecture — the host's — and it goes in the name.** Build one `GOARCH`,
the runner's own, exactly like `make webview` already does. **No second slice and
no `lipo`**: joining two slices buys a degrade path and a signature-ordering
constraint for an audience that is on Apple Silicon. Put the architecture in the
artifact's name so an operator sees what they are getting and a second one can
appear beside it later without a rename.

**Then sign, ad-hoc, last.** The ad-hoc signature is not decoration — Apple
Silicon refuses to execute a binary carrying no signature at all. The Go linker
signs the executable, but not the bundle around it, so sign the assembled bundle
after the property list and icon are in place.

Off macOS the target prints a line and **succeeds**, mirroring the existing shell
target's contract — that is what lets the shells job stay green on every runner.

**One new ADR, no amendments.** Write an ADR recording the unsigned-bundle
decision and its Gatekeeper cost, and have it **name explicitly what it
supersedes** in ADR 0011 (whose best-effort tier enumerates only the loose shell
per platform) and ADR 0013 (which reasons *from* the shell being unbundled).
Do not edit 0011 or 0013 — a later record superseding an earlier one is the ADR
format working as intended.

**The code comments are a different matter and are not deferred.** A comment in
the platform icon code states outright that handing the shell a real bundle is a
packaging change out of its reach, and this ticket makes that false. Correct it
and any sibling comment asserting the shell is never bundled, in this same slice;
a stale comment misleads the next reader in a way a superseded ADR does not.
**Keep** the runtime app-name seeding and runtime icon — the property list
supersedes them inside the bundle, but the loose shell is still a shipped
artifact and still needs them.

Nothing here is unit-tested: these are properties of an artifact produced by
platform tooling, and a Go test asserting a property list contains keys the same
code just wrote would test nothing.

## Done when

The bundle target produces a `chartr.app` on a real Mac, and reading the artifact
back confirms it: the property list carries the declared keys with the tag's
stamp in both version fields; the signature is present and ad-hoc; the executable
is the host architecture and the artifact's name says which. The app
double-clicks from Finder into a native window with the cockpit loaded over
loopback, showing the mark in Finder, the Dock and the app switcher, and `chartr`
in the menu bar; the operator's existing spaces are present; ⌘Q, closing the
window and a signal all shut it down cleanly. Running the bundled executable
directly from a terminal still prints to standard error as the loose shell does —
that path is the failure diagnostic and the answer confirms it works. Run off
macOS, the target prints its line and exits 0. The new ADR is written and names
what it supersedes in 0011 and 0013, both of which are left unedited; the
falsified comments are corrected; the runtime app-name seeding and icon are still
in place. `go vet ./...`, `go test ./...`, the frontend `check` / `build` /
`vitest`, and the no-amber check are green.

## Answer

`make bundle` is one new target in the existing build file, and it is the shell
target plus packaging exactly as the spec framed it: it invokes `make webview`
with the same three stamp variables and packages the executable that target just
built, so the loose shell and the bundle can never report different identities.
Output is `build/macapp/chartr_<version>_darwin_<arch>/chartr.app` — the app keeps
the plain name an operator drags, and **the architecture is in the staged
directory around it**, which is what ticket 03's image name will inherit and what
lets a second slice appear beside it later without renaming this one.

- **The property list** carries all the declared keys and nothing else:
  `CFBundleName` / `DisplayName` / `Executable` / `IconFile` (`chartr`),
  `CFBundleIdentifier` `io.github.rengwu.chartr` (read off the module path, not
  invented), `CFBundlePackageType` `APPL`, `CFBundleInfoDictionaryVersion`,
  `NSHighResolutionCapable`, `LSApplicationCategoryType`
  `public.app-category.developer-tools`, and both version strings. **No
  `LSUIElement`** — it is a real windowed app — and **no ATS exception**, which
  is a verified absence, not an assumption: the bundled window loads the cockpit
  over `http://127.0.0.1:<port>` with nothing declared.
- **Two version strings, one stamp.** `CFBundleShortVersionString` is the numeric
  part of the tag, extracted with a pattern that only accepts a tag shape
  (optional `v`, one to three dotted integers, then end or `-`/`+`) and falls
  back to `0.0.0` rather than emit nonsense — an untagged `git describe` like
  `8ab9e30-dirty` yields `0.0.0`, not `8`. `CFBundleVersion` is the whole
  `WEBVIEW_VERSION` build stamp. Checked end to end with a tag-shaped stamp:
  `WEBVIEW_VERSION=v0.2.1` gives `0.2.1` / `v0.2.1`, and the executable's
  `--version` reports `v0.2.1` with the same commit and date.
- **One decision the ticket left open, taken and named:
  `LSMinimumSystemVersion` is read off the built executable's own
  `LC_BUILD_VERSION` `minos`** (14.2 on this toolchain), with the constant only
  as a fallback if `otool` says nothing. Hard-coding a number was the first cut
  and it was wrong: whatever the runner's SDK targets is the real floor, and a
  lower claim only promises an operator a launch the loader then refuses.
- **The icon is generated.** `sips` downscales `web/public/icon-512.png` — the
  same file Vite copies to the dist root, the same bytes the PWA fetches and the
  runtime Dock icon reads — into a nine-entry iconset (16/32/128/256/512 with
  `@2x` companions, none of them an upscale), and `iconutil` produces
  `Contents/Resources/chartr.icns`. The iconset is scratch and is deleted.
- **Ad-hoc signature, last.** `codesign --force --sign - --identifier <id>` runs
  after the list and the icon are in place, then `codesign --verify --strict`
  fails the target if it did not take.
- **Off macOS** the target prints `the macOS app bundle is only assembled on
  macOS (this is linux); nothing to package` and exits 0, before doing any work —
  verified with `make bundle GOOS=linux`.

**ADR [0016](../../../../docs/adr/0016-unsigned-macos-app-bundle.md) is written**
and names what it supersedes, in part, in **0011** (whose best-effort tier
enumerated only "the native webview shell per platform"; macOS now has a second
artifact on it, while everything 0011 decided about the tier itself holds) and in
**0013** (whose `CFBundleName`-seeding and runtime-Dock-icon rationale reasoned
*from* the shell being a bare binary, and is now true of the loose shell only).
**Neither file was edited.**

**The falsified comments are corrected in the same slice.** `icon_darwin.go`
claimed outright that "handing the shell a real `.app` is a packaging change …
not something this can reach" — it now says this is the *loose* shell's only
surface, that the bundle's `.icns` comes from the same PNG, and that inside the
bundle the call is a no-op over identical artwork. `menu_darwin.go` said "the
shell is a bare binary, not a `.app` bundle" in both its doc comment and the
inner note about seeding `CFBundleName`; both now scope that to the loose launch
and say the bundled one already carries the name. `appicon.go` gained the
forward pointer that all three surfaces come from one file. **The runtime
app-name seeding and runtime icon are kept** — the loose shell is still shipped
and still has nothing else.

Nothing here is unit-tested, per the spec's stance; the acceptance checks are
against the built artifact.

**Verified on a real Mac** (macOS 27.0, Apple Silicon). Reading the artifact
back: `plutil -p` shows the twelve keys above, `plutil -lint` OK;
`codesign -dv` reports `Signature=adhoc`, `Format=app bundle with Mach-O thin
(arm64)`, identifier `io.github.rengwu.chartr`; `lipo -archs` is `arm64` and the
staged directory says `darwin_arm64`. Opened through Finder (`tell application
"Finder" to open`, the LaunchServices path a double-click takes) it came up with
cwd `/`, wrote its lock to `~/.config/chartr/.chartr/shell.lock` and nothing to
`/.chartr`, served `/api/health` 200 on loopback, drew the cockpit in a native
window with **this operator's existing command-line-registered spaces in the
sidebar**, and titled the menu bar `chartr`. The mark shows in Finder's list, on
the Dock tile and as the first tile in the ⌘-Tab switcher (all three
screenshotted). **⌘Q**, **clicking the window's close button** and **`SIGTERM`**
each shut it down with no process left; ⌘Q leaves the lock behind exactly as
ADR 0013 says it must, and the next launch took the stale lock over and got a
fresh port. **The terminal diagnostic path works**:
`chartr.app/Contents/MacOS/chartr --data-dir /var/empty/nope-chartr` printed
`chartr shell: mkdir …: permission denied` on standard error and exited 1, with
standard output empty — the same line the loose shell prints. `--version` from
that path prints the stamp and returns.

`go vet ./...` clean, `go test ./...` green (including the cgo-free
`cmd/webview` suite and the embed test against `web/dist/`), `npm run check`
0 errors / 0 warnings, `npm run build` succeeded, `vitest` 185 passed, and the
built stylesheet carries no amber.
