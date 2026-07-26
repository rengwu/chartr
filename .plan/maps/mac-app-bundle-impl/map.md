# The macOS app bundle — implementation

## Destination

The [spec](../mac-app-bundle/spec.md) implemented end to end: an operator on a Mac
downloads one `.dmg` from the releases page, drags chartr into `/Applications`,
clicks through the one Gatekeeper prompt the disk image warned them about, and
launches the cockpit from Launchpad — with the mark on the icon, the app's own
name in the menu bar, their existing spaces already registered, and their state
written where the command-line binary writes it. Done looks like a tag attaching
an unsigned, ad-hoc-signed, universal bundle and its checksum sidecar as a
best-effort asset, without the supported cgo-free release or its manifest having
been touched at any point.

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
`docs/adr/` are binding, and this effort **amends 0011, amends 0013, and writes a
new ADR** for the unsigned bundle — all in ticket 03, the ticket that falsifies
0013's premise. A ticket that touches an ADR says so in its answer.

**Sequencing.** The order is `01 → 02`, `01 → 03 → 04 → 05`. Ticket 01 comes
first because it is load-bearing in a way the ordering hides: until the runtime
root is anchored, a bundled launch dies claiming the single-instance lock, before
a window exists, writing to a stream nobody reads — so every packaging ticket
after it would be verified against an app that cannot start. 01 is verifiable
from a terminal with no bundle in existence. 02 (visible failure) and 03
(the bundle) are independent of each other and can be worked in either order once
01 lands.

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
within recent memory — so ticket 04 verifies them against the current macOS by
simulating a quarantined download rather than copying them forward from the spec
on faith.

**Before commit:** run the CLAUDE.md gates — `go vet ./...` and `go test ./...`
(the embed test compiles against `web/dist/`), the frontend `check` / `build`
scripts and `vitest`, and confirm **no amber in the built CSS**. Review the diff
and drive the real behaviour where a "Done when" is only real at runtime — for
this map that means a real Mac, and from ticket 03 onward a real double-click. No
map linter is wired in this repo.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question goes back to the operator — it does not open fog here. -->

## Out of scope

<!-- Inherited from the spec's Out of Scope; these never graduate into tickets on this map. -->

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
