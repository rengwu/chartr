# The macOS app bundle — spec

## Problem Statement

An operator on a Mac who wants the cockpit as a real application cannot get one.
The supported artifact is a cgo-free binary in a tarball: they extract it, find a
terminal, `cd` to the right directory, and run it — and the cockpit opens in a
browser tab. The best-effort webview shell gives them a native window, but it too
ships as a loose executable with no icon in Finder, no entry in Launchpad, no way
to launch it by double-clicking, and no place in `/Applications`. There is nothing
to drag, nothing to keep in the Dock between reboots, and nothing that looks like
software the operator installed.

The obvious fix — ship a `.app` in a `.dmg` — runs into a second problem: the
project has no Apple Developer account, so the bundle cannot be signed with a
Developer ID or notarized. The naive reaction is to wait until there is an
account. That is the wrong call, because an unsigned bundle is still strictly
better than a loose binary for every operator who is willing to click through one
Gatekeeper prompt — and because the cost of *not* shipping is that the shell tier
stays permanently undemoable to anyone who is not already a Go developer.

There is also a failure mode hiding behind the packaging. The shell has only ever
been launched from a terminal, and it assumes it. Launched from Finder it inherits
`/` as its working directory, and the runtime root it derives from that is a
directory it cannot write to. The single-instance lock is the first thing to touch
that root, so the app would exit before drawing a window — and it would exit
writing to a standard error stream that, under Finder, nobody will ever read. The
operator would double-click an icon and watch nothing happen.

## Solution

chartr's macOS shell gains a third artifact on the **best-effort tier**: an
unsigned, ad-hoc-signed `chartr.app`, delivered in a `.dmg` with the customary
drag-to-`/Applications` layout. It is built by the same release job that already
builds the loose shell, stays `continue-on-error`, and carries a per-asset
checksum sidecar — so it cannot touch the one supported artifact or the manifest
that vouches for it.

The operator downloads one file, opens it, drags the app across, and launches it
from Launchpad or the Dock like anything else. The first launch is blocked by
Gatekeeper — the honest, unavoidable cost of having no Developer ID — and both the
disk image and the release notes tell them, in advance and in one sentence, the
exact click that unblocks it. Nothing pretends the app is signed, and nothing
tries to work around Gatekeeper by trickery.

Underneath, the shell learns that it can be launched with no terminal attached.
It notices it is running from inside a bundle, anchors its runtime root to the
operator's home instead of the working directory it was handed, and tolerates the
stray arguments the window server sometimes injects.

**This is deliberately the smallest thing that ships.** Three things a fuller
version of this spec would carry are cut, and cut on purpose: a native failure
dialog, a universal two-architecture binary, and a round of ADR amendments. Each
is recorded under *Out of Scope* with the condition that would bring it back.
What remains is one build target, one disk image, and one release step.

## User Stories

1. As an operator on a Mac, I want to download a single `.dmg` from the releases page, so that I do not have to know what a tarball is to try the cockpit.
2. As an operator, I want to drag chartr into `/Applications` from the opened disk image, so that installing it is the gesture I already know.
3. As an operator, I want chartr to appear in Launchpad and Spotlight after installing, so that I can launch it the way I launch every other application.
4. As an operator, I want to double-click chartr and get the cockpit in a native window, so that I never have to open a terminal to start work.
5. As an operator, I want chartr's mark to show on the app in Finder, in the Dock, and in the app switcher, so that I can recognise it among a screen of windows.
6. As an operator, I want the menu bar to read "chartr" rather than an executable's name, so that the app does not look like a science project.
7. As an operator, I want to keep chartr in my Dock and launch it after a reboot, so that it behaves like installed software rather than a thing I ran once.
8. As an operator whose Mac blocks the app on first launch, I want the disk image to have already told me why and what to click, so that I am not left guessing whether the download is broken or malicious.
9. As an operator, I want the unblock instructions I am given to be the instructions that actually work on my macOS version, so that I do not burn twenty minutes on advice that stopped being true two releases ago.
10. As a cautious operator, I want the release notes to state plainly that the app is unsigned and not notarized, so that I can decide for myself whether to trust it.
11. As an operator, I want the app to launch at native speed on my Mac, so that I am not paying a translation tax on every session.
12. As an operator, I want to verify the download against a published checksum, so that I can confirm I got the bytes the project built.
13. As an operator who launches the bundled app from Finder, I want my sessions and payload archives written somewhere I own, so that the app does not fail trying to write to the root of my disk.
14. As an operator, I want the spaces I registered from the command line to be there when I open the bundled app, so that I have one registry rather than two.
15. As an operator, I want the bundled app and the command-line binary to agree on where my state lives, so that switching between them does not silently fork my history.
16. As an operator, I want launching the app a second time to raise the window I already have, so that I never end up with two cockpits fighting over one working tree.
17. As an operator, I want the agents I installed with Homebrew or in my home directory to be findable from the bundled app, so that a Finder launch is not missing the tools a terminal launch has.
18. As an operator, I want ⌘Q, closing the window, and a signal to shut the app down the same clean way, so that quitting never leaves a session or a lock behind.
19. As an operator, I want the app's version to match the release I downloaded, so that a bug report names a build the project can reproduce.
20. As a maintainer, I want to build the bundle and the disk image with one command on my own Mac, so that I can inspect the artifact before tagging.
21. As a maintainer, I want those commands to do nothing but say so when run on Linux or Windows, so that they can sit in a shared build file without breaking anyone.
22. As a maintainer, I want the packaging to run in the release job that already builds the shell, so that a release does not grow a second pipeline to keep in sync.
23. As a maintainer, I want a packaging failure to be incapable of failing or altering the supported release, so that the tiering guarantee stays structural rather than aspirational.
24. As a maintainer, I want the disk image's checksum to live in its own sidecar, so that a best-effort artifact never mutates the supported manifest.
25. As a maintainer, I want the app's icon generated from the mark the cockpit already ships, so that the two can never drift apart.
26. As a maintainer, I want the bundle stamped with the same version, commit and date as the supported binary, so that all three artifacts of a tag report one identity.
27. As a maintainer, I want the bundled-launch behaviour covered by tests that run in the ordinary cgo-free suite, so that the part that can break silently is the part that is actually tested.
28. As a maintainer, I want the decision to ship unsigned recorded with its costs, so that the next person does not rediscover the Gatekeeper trade-off from scratch.
29. As a maintainer, I want the existing comments that assert the shell is never bundled corrected, so that the code does not carry a claim the build has falsified.
30. As a maintainer, I want a documented way to simulate a quarantined download, so that the unblock instructions we publish are verified rather than assumed.
31. As a maintainer, I want the command-line binary's behaviour left exactly as it is, so that adding a Mac app costs the supported artifact nothing.
32. As a maintainer, I want the cut scope recorded with the condition that brings it back, so that shipping small now does not read later as having forgotten it.

## Implementation Decisions

### The tier, and what it may not touch

The bundle and the disk image join the **best-effort tier** established by
ADR 0011. They are built natively on the macOS runner of the existing
`continue-on-error` shells job, which already `needs` the published release. The
supported cgo-free binary, its goreleaser configuration, and `checksums.txt` are
not touched by any of this work. The disk image carries a **per-asset `.sha256`
sidecar**, the same rule the loose shell already follows: a best-effort artifact
never mutates the supported manifest.

### Signing: ad-hoc only, and why that is not "unsigned"

Three distinct things get conflated as signing, and only one is free:

- an **ad-hoc signature**, which costs nothing and requires no account;
- a **Developer ID signature**, which requires the paid account; and
- **notarization**, which requires the account and a submission round-trip.

Apple Silicon refuses to execute a binary carrying *no* signature at all, so the
ad-hoc signature is not optional — it is the minimum that makes the app launch.
The Go linker already ad-hoc signs the darwin binaries it produces, but the
signature covers the executable, not the bundle around it, so the assembled
bundle is signed ad-hoc as the **last** step of assembly, after the property list
and the icon are in place.

The consequence is Gatekeeper. A disk image downloaded through a browser carries
the quarantine attribute, and an ad-hoc-signed, un-notarized application is
blocked on first launch. On current macOS the long-standing right-click-to-Open
bypass no longer clears it; the working path is the "Open Anyway" control in
Privacy & Security after the first blocked attempt, with clearing the quarantine
attribute by hand as the documented alternative. This cost is **stated, not
worked around**: it appears in the disk image, in the release notes, and in the
ADR. No attempt is made to defeat quarantine.

### Bundled-launch awareness lives in the shell's tag-free half

The shell's package is already split by build tag (ADR 0013): the cgo window
lives behind `webview`, and the single-instance lock is deliberately tag-free so
it compiles and tests at `CGO_ENABLED=0`. **All bundled-launch logic joins the
tag-free half**, beside the lock, for exactly that reason — it is the half a test
can reach without a display.

Three behaviours are added there:

- **Bundle detection.** The shell asks whether its own executable sits inside a
  `Contents/MacOS` directory whose grandparent carries the `.app` extension. A
  pure path predicate over the executable's path, taking the path as an argument
  so a test can drive it without a real bundle.
- **Runtime-root resolution.** When the shell is bundled and the operator passed
  no explicit runtime root, the root resolves to the same home-anchored location
  the config root already resolves to, rather than to the working directory.
  Passing the flag explicitly always wins.
- **Argument tolerance.** When bundled, an unrecognised argument is ignored
  rather than fatal, because the window server can inject its own. The
  command-line parsing behaviour is unchanged for a terminal launch.

**The runtime root resolves to the config root, not to an Apple-conventional
application-support directory.** ADR 0013's rationale for the config root was one
path an operator can reason about on every platform, and the same reasoning
applies with more force here: if a Finder launch and a terminal launch chose
different roots, an operator would have two registries, two session archives and
two single-instance locks depending on how they started the app, and the split
would be invisible until it confused them. One root keeps the bundled app and the
command-line binary honestly interchangeable.

Note what does **not** need to change. The registry already resolves through the
config root, so a bundled launch finds the operator's spaces with no work. Login
shell `PATH` hydration already runs at startup, added precisely because a Finder
launch inherits a `PATH` that carries neither Homebrew nor the operator's own
bin directory.

### Failure stays on standard error, and that is a known cost

Fatal startup errors go to standard error and keep going there, bundled or not.
A Finder launch discards that stream, so a fatal failure under the bundle is a
silent bounce in the Dock — including ADR 0013's deliberate "the native runtime
is missing, use the supported binary" error, which is exactly the message an
operator most needs.

**This is accepted rather than solved.** The fix is a native Cocoa alert plus a
log file, and it is real work — platform code behind the `webview` and `darwin`
tags, a tag-free sink decision to keep testable, a second surface to keep in sync
— on a failure mode that anchoring the runtime root makes rare by removing its
most likely cause.
It is deferred until the bundle has shipped and a failure is observed in the
wild; the moment an operator reports a bounce with nothing to send, it stops
being deferred. Recorded in *Out of Scope*.

The consolation is that a bundled app is still runnable from a terminal:
`/Applications/chartr.app/Contents/MacOS/chartr` prints exactly what the loose
shell prints. That is the diagnostic path, and it belongs in the README beside
the Gatekeeper note.

### The bundle

A conventional bundle: an information property list, the shell executable, and a
generated icon resource. The property list declares the bundle's name, display
name, identifier, executable, icon, package type, version strings, a minimum
system version, high-resolution capability, and a developer-tools category. It
deliberately does **not** declare the app an agent-style background app — it is a
real windowed application — and does not declare transport-security exceptions,
because loopback is already exempt and the loose shell demonstrably loads the
cockpit over it today; that this still holds inside a bundle is an explicit
acceptance check rather than an assumption.

The two version strings differ by necessity: the human-facing short version takes
the numeric part of the tag, because the format is constrained, while the full
build stamp goes in the build version. Both derive from the same stamp variables
the loose shell and the supported binary already use, so one tag yields one
identity across all three artifacts.

**The icon is generated, not curated.** The bundle's icon is produced by
downscaling the largest mark the cockpit already embeds into the icon set the
platform tooling expects. The shell's runtime icon path reads that same mark out
of the embedded frontend for the same reason — generating rather than committing
a second copy is what makes drift impossible.

### Architecture: whatever the runner is, named in the filename

The bundle is **single-architecture — the runner's own** — exactly like the loose
shell beside it. cgo does not cross-compile, and while the macOS toolchain can in
principle build both Mac slices for joining into a universal executable, that
buys a second slice, a join step, a signature ordering constraint (joining strips
signatures) and a degrade path, all for a best-effort artifact whose audience is
on Apple Silicon.

So: one `GOARCH`, the host's, **named in the disk image's filename** so an
operator can see what they are getting and a second architecture can appear later
beside it without renaming anything. This is the same contract `make webview`
already follows, which is the point — the bundle target is the shell target plus
packaging, not a new kind of build.

Universal is a filename and a `lipo` call away when an Intel operator asks; the
naming is chosen so that day costs nothing. Recorded in *Out of Scope*.

### The disk image

A staged directory containing the app, a symbolic link to `/Applications` as the
drag target, and a short plain-text file carrying the Gatekeeper instructions,
compressed into a read-only image named for the project, version and
architecture, with its checksum sidecar beside it.

**No styled disk-image window.** A background image with positioned icons
requires scripting the Finder and committing a window-state file, which is
cosmetic work on a tier that ships with a "your Mac will block this on first
launch" note in the box. It is declined until the tier stops carrying that note.

### Build surface

Two new targets in the existing build file, sharing the stamp variables already
defined there: one assembles the bundle, one stages and produces the disk image
from it. Off macOS both print a line and **succeed**, mirroring the existing
shell target's established contract — a target that cannot run on this host is
not a failure, which is what lets the shells job stay green on every runner. The
existing clean target already removes the build output directory and needs no
change.

### Release and documentation

The release workflow's macOS leg gains one packaging step, guarded on the shell
having built, and the upload glob widens to include the disk image and its
sidecar. The job keeps `continue-on-error` and its dependency on the published
release; that structure is the guarantee, and it is not weakened to accommodate
the new artifact.

Documentation changes: the support-tiers section of the README gains the disk
image, the Gatekeeper instructions and the run-from-terminal diagnostic line; the
release notes footer gains one line under best-effort.

**One new ADR, and no amendments.** The bundle does disturb two existing records
— ADR 0011 enumerates the best-effort tier as the native shell per platform, and
ADR 0013 reasons *from* the shell being unbundled, since the runtime app-name
seeding and the runtime icon exist only because a bare binary has no property
list. Editing both in place is three documents' worth of churn for one decision.
Instead the new ADR states the unsigned-bundle decision with its Gatekeeper cost
and **names what it supersedes in 0011 and 0013 explicitly**, which is what an
ADR log is for — later records amending earlier ones is the format working, not a
shortcut around it.

What is *not* deferred is the code: a comment in the platform icon code states
outright that handing the shell a real bundle is a packaging change out of its
reach, and the build now falsifies it. **Comments that assert something untrue
get corrected by the ticket that makes them untrue** — a stale comment misleads
the next reader in a way a superseded ADR does not.

The runtime app-name seeding and runtime icon are **kept, not deleted**. The
property list supersedes them inside the bundle, but the loose shell is still a
shipped artifact and still needs them.

## Testing Decisions

**What makes a good test here.** Only externally observable behaviour: what the
shell resolves when handed a path, and what the assembled artifact *is* on disk.
No test asserts how a helper is factored, and none reaches inside the window.

**The tested seam is the shell's tag-free half** — the one the single-instance
lock already occupies, which compiles and runs at `CGO_ENABLED=0` in the ordinary
suite. This is the existing seam and the highest one available; no new seam is
introduced. The prior art is the lock's own test file: pure-Go table tests over
path resolution, with the platform probe held behind a variable so a test can
drive it without touching the real system. The new resolution follows that shape
exactly — bundle detection and root resolution take their inputs as arguments, so
a test drives them with constructed paths and never needs a real bundle, a real
display, or a real home directory.

Covered there: a path inside a bundle detected as bundled; a plain path not; an
explicit runtime root winning over the bundled default; and the bundled default
landing on the home-anchored root rather than the working directory.

**Nothing about the bundle, the signature or the disk image is unit-tested.**
Those are properties of an artifact produced by platform tooling, and the tooling
is the thing under test — a Go test asserting that a property list contains keys
the same code just wrote would test nothing. They are verified by **acceptance
checks against the built artifact**: reading the property list back, confirming
the signature is present and ad-hoc, confirming which architectures the
executable carries, mounting the image, and launching the installed app. ADR 0013
already establishes this stance for the window and the menu; this extends it to
packaging rather than inventing a policy.

**One acceptance check is not optional: the simulated quarantined download.**
Applying the quarantine attribute by hand to the installed app and then launching
it is the only way to verify that the unblock instructions shipped in the box are
the instructions that actually work. Publishing unverified instructions is worse
than publishing none.

The project's existing gates — Go vet and tests, the frontend check, build and
unit tests, and the no-amber assertion over the built stylesheet — apply
unchanged to every ticket.

## Out of Scope

Three of these are cuts from a fuller version of this spec rather than things
never considered, and each carries the condition that brings it back.

- **A native failure dialog and log file.** *Cut.* Fatal startup errors stay on
  standard error, which a Finder launch discards. Brought back when an operator
  reports a launch that bounced with nothing to send — until then the terminal
  path (`chartr.app/Contents/MacOS/chartr`) is the diagnostic.
- **A universal two-architecture binary.** *Cut.* The bundle is the runner's own
  architecture, named in the filename. Brought back when an Intel operator asks;
  the filename convention is chosen so that is a `lipo` call, not a rename.
- **Amending ADR 0011 and 0013 in place.** *Cut.* The new ADR names what it
  supersedes instead. The falsified *code comments* are still corrected — that
  part is not deferred.
- **Developer ID signing and notarization.** Both require the paid account. When
  there is one, they slot into the assembly step; nothing here is designed to
  make that harder.
- **Auto-update, in any form.** No update framework, no update feed.
- **A styled disk-image window** — background art, positioned icons, a committed
  window-state file.
- **An installer package.** The drag-to-Applications gesture is the whole
  install.
- **A Homebrew cask, or any distribution channel other than the releases page.**
  ADR 0011's distribution decision is unchanged.
- **A custom URL scheme.** ADR 0013 declined it for want of a producer, and
  nothing here supplies one.
- **Windows and Linux packaging.** Both keep the loose best-effort binary.
- **Promoting the shell out of the best-effort tier.** The supported artifact
  remains the cgo-free browser binary.
- **Any change to the supported binary or its release lane.**
- **Deleting the runtime app-name seeding or runtime icon.** The loose shell
  still needs both.

## Further Notes

**The first ticket is load-bearing in a way the ordering hides.** Until the
runtime root is anchored, a bundled launch dies claiming the single-instance lock
— before a window exists, writing to a stream nobody reads. Every packaging
ticket after it would otherwise be verified against an app that cannot start, so
it comes first and is verifiable on its own, from a terminal, with no bundle in
existence yet.

**This repo drives its own maps, so the artifact is self-demoing.** The finished
bundle can be installed and used to drive this very map, which is the strongest
acceptance test available and costs nothing to run.

**The Gatekeeper instructions are perishable.** Apple has changed the unblock
path within recent memory — the right-click bypass used to work and no longer
does. Whatever ships must be re-checked against the current macOS at the time it
ships, not copied forward from this spec on faith.
