---
title: Version 0.3.3 release publication
section: Verification
updated: 2026-11-07
result: Passed
---
# Version 0.3.3 release publication

The user authorized publication after [preparation](../../development/release-0.3.3.md)
completed under the accepted simple-verification scope. [Chartr v0.3.3](https://github.com/rengwu/chartr/releases/tag/v0.3.3)
was published as the latest stable release at **15:36:27 UTC on November 7**.
The exact source revision passed CI before publication. All final tag-package,
asset, public-download and AUR-deferral checks below pass.

## Scope and identity

Affected feature: [distribution](../../features/distribution.md) (`FEAT-distribution`).
This run covers the release tag, final packages, reviewed notes, stable publication
and public downloads. It retains the [accepted limits](../../decisions/index.md#dec-release-033-001-prepare-0-3-3-with-simple-verification):
no desktop takeover, no v0.3.2 upgrade test, and no claim that deferred native
interaction is verified. The [publication authorization](../../decisions/index.md#dec-release-033-002-publish-the-prepared-0-3-3-release)
continues the AUR deferral.

Main was fast-forwarded to prepared revision
`bdbad6da45b21abca72c74caca09553478ac3f7f`. Annotated tag `v0.3.3` points to that
commit; its tag object is `8b350b0a45811fa32be343506c6a36f171bb395d`. Workspace
version and tag agree. Compared with verified fixture commit `56ce59cf`, only
wiki documentation and evidence changed. The [manifest](../../evidence/2026-11-07-release-publication/manifest.json)
and [remote tag identity](../../evidence/2026-11-07-release-publication/tag-identity.json)
record this boundary. Publication records and public-guide updates were added
after tagging and do not move the release tag.

The [duplicate main-branch package and CI runs](../../evidence/2026-11-07-release-publication/cancelled-duplicates.json)
were cancelled; the tag's shipping package workflow passed independently. Source
CI on the exact tagged revision had already passed all three jobs before
publication. The additional automatic tag CI completed successfully after
publication, including all macOS, Ubuntu and Arch jobs.

## Results

| Scenario | Expected and observed | Result | Evidence |
| --- | --- | --- | --- |
| Source CI | Exact tagged revision passes macOS, Ubuntu and Arch CI before publication | Passed | [CI run](https://github.com/rengwu/chartr/actions/runs/42383260488), [receipt](../../evidence/2026-11-07-release-publication/source-ci.json) |
| Additional tag CI | Automatic tag-triggered macOS, Ubuntu and Arch checks finish successfully after publication | Passed | [CI run](https://github.com/rengwu/chartr/actions/runs/42383977988), [receipt](../../evidence/2026-11-07-release-publication/tag-ci.json) |
| Tag packaging | All three platform jobs and draft assembly succeed; tag/version matching, release scripts and real Arch/AUR package checks pass | Passed | [Workflow](https://github.com/rengwu/chartr/actions/runs/42383978011), [receipt](../../evidence/2026-11-07-release-publication/tag-workflow.json) |
| Final asset integrity | Exactly ten assets match GitHub SHA-256 digests; all seven payloads match their checksum files | Passed | [Asset digests](../../evidence/2026-11-07-release-publication/asset-digests.json), [payload checksums](../../evidence/2026-11-07-release-publication/payload-checksums.json) |
| Linux package identity | Both architectures identify 0.3.3, the tagged commit and exact Herdr pin; tar/DEB binaries match, Arch x86_64 matches its tarball, launch symlinks and AUR recipe checksum/version are correct | Passed | [Inspection](../../evidence/2026-11-07-release-publication/linux-inspection.json) |
| macOS package identity | DMG integrity and strict deep signature checks pass; production bundle ID, 0.3.3 version, tagged revision, ARM64 executables and exact Herdr version match | Passed | [Inspection](../../evidence/2026-11-07-release-publication/macos-inspection.json) |
| Stable publication | Reviewed notes and verified assets become public, non-draft, non-prerelease and latest; all asset IDs, sizes and digests stay unchanged | Passed | [Reviewed draft](../../evidence/2026-11-07-release-publication/draft-reviewed.json), [published receipt](../../evidence/2026-11-07-release-publication/published.json), [latest receipt](../../evidence/2026-11-07-release-publication/latest.json), [notes](../../evidence/2026-11-07-release-publication/release-notes.txt) |
| Public downloads | All ten versioned URLs return HTTP 200 without authentication; all three checksum files download and match the inspected copies | Passed | [Public checks](../../evidence/2026-11-07-release-publication/public-downloads.json) |
| AUR deferral | GitHub publication leaves AUR untouched; release-triggered workflow and its job are skipped | Passed | [AUR run](https://github.com/rengwu/chartr/actions/runs/42387204177), [receipt](../../evidence/2026-11-07-release-publication/aur-skipped.json) |

## Inspection and cleanup

The final files came from the tag workflow's draft, not the earlier preparation
artifacts. Downloads are retained under `target/release-publish-0.3.3/assets/`.
The downloader used bounded parallel HTTP ranges, verified TLS and exact lengths,
and checked each complete file against GitHub's digest. Credentials and signed
download URLs were kept in memory only.

The macOS image was mounted read-only without Finder and detached after inspection.
Only its Herdr `--version` command was executed. Chartr was not launched, no package
was installed, and no personal profile, backend or desktop app was controlled.

Executed drivers: [downloads](../../evidence/2026-11-07-release-publication/download-assets.py.txt),
[Linux inspection](../../evidence/2026-11-07-release-publication/inspect-linux.py.txt),
[macOS inspection](../../evidence/2026-11-07-release-publication/inspect-macos.py.txt),
and [publication/public downloads](../../evidence/2026-11-07-release-publication/verify-publication.py.txt).

## Limits

The macOS app remains ad-hoc signed and unnotarized, as disclosed in the release
notes. Publication checks do not certify native UI/banner/physical-input behavior,
live-provider title freshness or the explicitly skipped old-version upgrade.
The earlier Failed and Untested preparation reports remain intact.
