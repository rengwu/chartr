---
type: task
blocked_by: [01]
---

# The unsigned universal bundle

## Question

Add a build target that assembles a real `chartr.app` from the webview shell, and
record the decisions it forces. The end-to-end behaviour: a maintainer runs one
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
goes in the build version. Both derive from the stamp variables the build file
already defines for the loose shell, so one tag yields one identity across all
three artifacts.

**The icon is generated, not curated** — downscaled from the largest mark the
cockpit already embeds, into the icon set the platform tooling expects. The
shell's runtime icon reads that same mark for the same reason: generating rather
than committing a second copy is what makes drift impossible.

**Architecture and signature, in this order.** One macOS runner builds both Mac
architecture slices and joins them into one universal executable; the ad-hoc
signature comes **last**, because joining strips signatures. The ad-hoc signature
is not decoration — Apple Silicon refuses to execute a binary carrying no
signature at all. **Leave the loose shell target single-architecture**; the bundle
target does its own build. If the second slice fails, ship
**single-architecture and say so in the filename** — degrading loudly is what the
best-effort tier is for, and a half-universal binary under a universal name is
not acceptable.

Off macOS the target prints a line and **succeeds**, mirroring the existing shell
target's contract — that is what lets the shells job stay green on every runner.

**This ticket owns the ADR work, because it is the ticket that falsifies the
premise.** ADR 0013 reasons *from* the shell being unbundled: the runtime
app-name seeding and the runtime icon exist only because a bare binary has no
property list, and a comment in the platform icon code states outright that
handing the shell a real bundle is a packaging change out of its reach. Amend
0013, amend ADR 0011's enumeration of the best-effort tier, and write a new ADR
recording the unsigned-bundle decision with its Gatekeeper cost. Correct the
falsified comments in the same slice. **Keep** the runtime app-name seeding and
runtime icon — the property list supersedes them inside the bundle, but the loose
shell is still a shipped artifact and still needs them.

Nothing here is unit-tested: these are properties of an artifact produced by
platform tooling, and a Go test asserting a property list contains keys the same
code just wrote would test nothing.

## Done when

The bundle target produces a `chartr.app` on a real Mac, and reading the artifact
back confirms it: the property list carries the declared keys with the tag's
stamp in both version fields; the signature is present and ad-hoc; the executable
carries both architecture slices (or one, with the filename saying so). The app
double-clicks from Finder into a native window with the cockpit loaded over
loopback, showing the mark in Finder, the Dock and the app switcher, and `chartr`
in the menu bar; the operator's existing spaces are present; ⌘Q, closing the
window and a signal all shut it down cleanly. Run off macOS, the target prints
its line and exits 0. ADR 0011 and 0013 are amended, the new ADR is written, the
falsified comments are corrected, and the runtime app-name seeding and icon are
still in place. `go vet ./...`, `go test ./...`, the frontend `check` / `build` /
`vitest`, and the no-amber check are green.
