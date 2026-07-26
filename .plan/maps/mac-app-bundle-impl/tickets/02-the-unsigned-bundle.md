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
