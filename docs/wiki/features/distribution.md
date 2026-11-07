---
title: Installation, packages and release boundaries
section: Features
updated: 2026-11-07
basis: Source-reviewed migration baseline
---
# Installation, packages and release boundaries

Feature ID: `FEAT-distribution`

## Intent and basis

This record uses the inspected current implementation under
[the accepted migration decision](../decisions/index.md). It describes the
working-tree baseline, not a newly executed product result. See
[baseline identity and limitations](../verification/setup.md#baseline-and-build-identity).

Architectural basis: [ADR 0002](../decisions/adr/0002-the-zed-layer.md), [ADR 0007](../decisions/adr/0007-prebuilt-native-surfaces.md). Historical portions of ADRs remain dated rationale.

Implementation owners: [.github/workflows/ci.yml](https://github.com/rengwu/chartr/blob/main/.github/workflows/ci.yml), [.github/workflows/release.yml](https://github.com/rengwu/chartr/blob/main/.github/workflows/release.yml), [.github/workflows/aur.yml](https://github.com/rengwu/chartr/blob/main/.github/workflows/aur.yml), [scripts/package-linux.sh](https://github.com/rengwu/chartr/blob/main/scripts/package-linux.sh), [scripts/build-dev-dmg.sh](https://github.com/rengwu/chartr/blob/main/scripts/build-dev-dmg.sh), [rust-toolchain.toml](https://github.com/rengwu/chartr/blob/main/rust-toolchain.toml).

## Entry points and prerequisites

Follow the public installation guide for supported packages/source builds; contributors use the release runbook and existing CI.

Use [shared setup](../verification/setup.md) and a disposable profile/project for
stateful recipes. Related areas are linked from the [feature index](index.md).

## Expected outcomes

- Current release targets are Apple silicon macOS and native Linux x86_64/ARM64. Windows is deferred; Linux uses X11/XWayland and the documented system dependencies.
- Linux formats reuse one release compilation per architecture; x86_64 additionally produces Arch/AUR artifacts. Tag/version matching, checksums and draft releases remain part of packaging.
- macOS DMGs are ad-hoc signed and unnotarized. The local development bundle identifier differs from the production identifier.
- AUR publication automation is opt-in for published stable releases through `AUR_PUBLISH_ENABLED=true`. Manual dispatch still validates credentials and the published recipe. The user deferred AUR setup on October 16 because account registration is unavailable; GitHub packages remain independent.
- Chartr pins its Rust/Zed/Herdr inputs; there is no app frontend build or plugin compilation during installation.

## Verification recipe

**Unvalidated recipe — not executed during this migration.** Complete shared
setup first; use actual UI entry points and observe underlying state where relevant.

1. For a future release, follow the release runbook and existing CI on the shipping platforms. Record exact revisions, artifact names and checksums.
2. Install the .deb and Arch package on disposable supported systems; launch from a terminal and app menu and exercise a terminal/web pane. Extract and launch the tarball through its symlink layout.
3. Mount the macOS DMG, inspect bundle version/signing, and launch on a supported Mac. Record platform warnings as documented.
4. Keep AUR publication and release publishing separate from local validation; use the existing explicit release procedure.

Preserve the selected scenario's logs/screenshots/file observations and exact
build identity before [cleanup](../verification/setup.md#cleanup). Record a fresh
run if this recipe is exercised; source inspection alone is not a pass.

## Coverage and limitations

The [November 7 publication run](../verification/runs/2026-11-07-release-publication.md)
records v0.3.3 at `bdbad6da45b2`, with passing source CI, tag builds, downloaded
artifact inspection and public asset checks. It is the latest stable release;
AUR publication was skipped under the accepted deferral. This does not expand
the accepted verification scope or certify the skipped upgrade/native scenarios.

No package was built, installed, published or downloaded during migration. Source configuration does not establish current external release or AUR availability; public status claims were not recertified.

The [release matrix](../verification/acceptance.md) retains broader cross-feature,
platform, theme, pointer and accessibility coverage. Choose a bounded subset for
ordinary changes; release requirements remain separate from this advisory workflow.

## Verification history

The [November 7 publication](../verification/runs/2026-11-07-release-publication.md)
records all ten final asset digests, seven payload checksums, Linux/macOS package
identity, latest stable publication, public download links and the skipped AUR job.

[0.3.3 preparation](../verification/runs/2026-11-07-release-0.3.3-recheck.md) records
bounded automated checks, isolated real-terminal checks and candidate package
verification under the user's request to preserve the desktop. Native coverage
limits remain explicit.

The [October 21 publication](../verification/runs/2026-10-21-release-publication.md)
records passing tag builds, all ten downloaded asset digests, Linux/macOS package
inspection, latest stable publication, all public asset URLs and the skipped AUR
workflow. The earlier [0.3.1 publication](../verification/runs/2026-10-16-release-publication.md)
retains its own evidence and limits.

The [October 19 0.3.2 preparation](../verification/runs/2026-10-19-release-0.3.2.md)
records passing local tests, the macOS/Ubuntu/Arch CI matrix and all shipping
package builds for candidate `4afea26f`. Native notifications are explicitly
deferred; the complete native acceptance matrix remains unverified. The user
[accepted this focused verification scope for 0.3.2](../decisions/index.md#dec-release-032-001-focused-verification-is-sufficient-for-0-3-2)
and deferred the remaining desktop matrix. The later publication run above covers
tagging and release distribution separately.

The [October 16 desktop recheck](../verification/runs/2026-10-16-release-desktop-recheck.md)
tracks the latest fixes and packages. AUR publication is explicitly deferred;
automatic stable-release publication now requires `AUR_PUBLISH_ENABLED=true`.
GitHub package production remains independent.

The [0.3.1 candidate follow-up](../verification/runs/2026-10-16-release-candidate.md)
at `e915365a` passes the macOS, Ubuntu and Arch/AUR CI matrix and all three
package builds. Downloaded-package inspection and selected native attention
checks are recorded separately; full desktop release acceptance remains incomplete.

The [October 16 release-readiness assessment](../verification/runs/2026-10-16-release-readiness.md)
at `c11b2dc5` finds a failing local workspace gate, no matching candidate CI or
package builds, the unchanged `0.3.0` version, and missing AUR publication
configuration. Passing older GitHub runs do not certify the current candidate.

No current product run was performed for this migration. Earlier
[research and audits](../archive/research/index.md) remain historical evidence for
their recorded snapshots; they do not certify this baseline.
