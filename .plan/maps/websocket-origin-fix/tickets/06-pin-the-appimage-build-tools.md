---
type: task
claimed_by: session_01HTp28WKpvHvJ824ycLXjLZ
claimed_at: 2026-08-03T07:36:57Z
---

# `make appimage` runs unverified binaries fetched from a mutable tag

## Question

The AppImage target downloads two tools and executes them with no verification
(`Makefile:400-410`):

```make
curl -fsSL -o "$$tools/linuxdeploy" \
    ".../releases/download/$(LINUXDEPLOY_VERSION)/linuxdeploy-$$tool_arch.AppImage"; \
chmod +x "$$tools/linuxdeploy";
```

and the same shape for `appimagetool`. No checksum, no signature. Worse,
`LINUXDEPLOY_VERSION := 1-alpha-20251107-1` (`Makefile:321`) is a **mutable** tag —
an alpha release that can be re-pointed at different bytes without the version
string changing. `APPIMAGETOOL_VERSION := 1.9.1` (line 322) is a normal release tag
and is better, but is fetched with the same absence of verification.

Whatever those binaries are on the day of the build gets executed on the build
machine and shapes the artefact shipped to users. That is a supply-chain path into
a released binary, gated only on GitHub serving what it served last time.

Scope honestly: this is **build-time**, affects whoever runs `make appimage`
(release builds and any Linux packager), and is unrelated to every other ticket on
this map. It does not touch a running cockpit.

**The fix.** Pin both tools by content, not by tag:

- Record the expected SHA-256 for each tool and architecture in the Makefile, and
  verify after download and **before** `chmod +x`. On mismatch, delete the file and
  fail the target loudly — never proceed with a warning.
- Note that the cache check is `if [ ! -x "$$tools/linuxdeploy" ]`, so an existing
  file is reused unverified. Verify on the cached path too, or the check is
  bypassed on every build after the first.
- Move `LINUXDEPLOY_VERSION` off the mutable alpha tag if a stable release exists.
  If none does — plausible for linuxdeploy — the checksum is what makes the alpha
  tag safe to depend on, so say so in a comment at the version line, and record how
  the checksum was obtained so the next person can re-derive it when bumping.
- `shasum -a 256 -c` and `sha256sum -c` differ across macOS and Linux; the target
  already handles cross-platform detail, so pick one that works on both build hosts
  or branch explicitly.

Document the bump procedure in a comment beside the versions: how to fetch the new
tool, how to derive the checksum, and that both change together. A pin nobody knows
how to update becomes a pin somebody deletes.

Tests lead where they can: this is a Makefile target and the suite does not run it,
so a Go test is the wrong instrument. Verify by hand instead and record it in the
answer — a clean `make appimage` with correct checksums, and a deliberately
corrupted checksum failing the build rather than producing an AppImage. Both
results belong in the answer.

Done when: both tools are checksum-verified before execution, including on the
cached path; a mismatch fails the target; the bump procedure is documented at the
version lines; and the two manual verifications above are recorded in the answer.

## Answer

**Both tools are pinned by content and verified before they are ever executable.**
Commit `49672ea`.

Four checksums are recorded beside the versions — `LINUXDEPLOY_SHA256_x86_64`,
`LINUXDEPLOY_SHA256_aarch64`, `APPIMAGETOOL_SHA256_x86_64`,
`APPIMAGETOOL_SHA256_aarch64` (`Makefile:378-381`). The `case "$goarch"` that
already picked `tool_arch` now also selects the two expected sums, so the
architecture and the checksums it is checked against are chosen in one place and
cannot drift apart.

`verify_tool` is one shell function defined in the recipe. It computes the sum,
and on mismatch deletes the file, prints expected and actual, and `exit 1`s.
There is no warning path and no flag to skip it.

**The cached path was the real bug, and it is what the fix turns on.** The
ticket flagged that `if [ ! -x "$tools/linuxdeploy" ]` reuses an existing file
unverified. Verification is therefore *outside* the fetch branch: the `if` block
now only downloads, and `verify_tool` then `chmod +x` run unconditionally
underneath it (`Makefile:479-492`). Every build verifies, not just the first.
The ordering also gives the cache check its meaning back — a file is only left
executable if it verified, so a rejected or half-downloaded tool is re-fetched
next time rather than skipped.

**`sha256sum` vs `shasum`.** Branched explicitly: `sha256sum` when present,
`shasum -a 256` otherwise. In practice only the first branch is ever reached —
the target exits at `goos != linux` long before this — but the fallback costs
two lines and means the block is not quietly wrong if that gate ever moves.

**How the checksums were derived**, since the ticket asked for it to be
recorded. Each asset was downloaded and summed locally, *and* cross-checked
against the `digest` GitHub reports for the release asset itself, which is
computed server-side and so is not merely one's own download read back. All four
agreed. Both commands are written into the bump comment so the next person
re-derives them the same way. That comment also says the two architectures change
together, and that `build/appimage/tools` must be deleted when bumping — otherwise
the next build verifies the *old* binary against the *new* sum and fails
confusingly.

Worth stating plainly: this is trust-on-first-use. The checksum does not
establish that today's linuxdeploy is honest; it establishes that it cannot
change underneath us without the build stopping. That is the property the mutable
alpha tag was missing, and it is all this ticket claimed to buy.

**The two manual verifications.** Both were run on real Linux — a rootless podman
VM, Debian trixie, **linux/arm64**, Go 1.26.5, WebKitGTK 2.52.5, with
`APPIMAGE_EXTRACT_AND_RUN=1` because a container has no FUSE. Note the arch: this
exercised the **aarch64** checksums, the pair a CI amd64 runner would never touch.

1. *A clean `make appimage` with correct checksums.* Both tools fetched,
   verified, and ran; the target completed and produced
   `chartr_0.2.1_linux_arm64.AppImage` (103,221,768 bytes) with its `.sha256`
   sidecar.
2. *A deliberately corrupted checksum fails the build rather than producing an
   AppImage.* Run in the two shapes that matter:
   - **Cached tool, wrong recorded sum** — the case the old code skipped
     entirely. With both tools already cached from (1),
     `make appimage LINUXDEPLOY_SHA256_aarch64=deadbeef…` failed with
     `build/appimage/tools/linuxdeploy: SHA-256 mismatch — refusing to run it`,
     exit 1, the file deleted, and **no AppImage produced**. Against the old
     code this build would have succeeded, having executed the unverified
     cached binary.
   - **Tampered bytes, correct recorded sum** — the actual attack shape. A
     single `\x90` appended to the cached `appimagetool`, then a run with the
     real recorded sums: linuxdeploy re-downloaded and verified fine,
     appimagetool was caught (`expected f0837e74… actual 5ab1003a…`), deleted,
     the target failed, and again no AppImage.

   A third run afterwards with everything honest re-fetched the deleted tool and
   rebuilt successfully to a byte-identical 103,221,768, which says the gate is
   recoverable rather than sticky.

`go vet ./...` and `go test ./...` pass, and `make appimage` on macOS still
prints-and-succeeds through the non-Linux early exit.

**Deliberately not done.**

- **No test.** Unchanged from the ticket's own reasoning: this is a Makefile
  target the suite does not run. The verification above is the evidence, and it
  is manual by design rather than by omission.
- **`LINUXDEPLOY_VERSION` stays on the alpha tag.** The ticket allowed this if
  no stable release exists, and none does — linuxdeploy publishes only dated
  alphas. That is now stated in a comment at the version line, along with why
  the checksum is what makes depending on it safe.
- **No signature verification.** Neither project publishes signatures or
  checksum sidecars for these assets (checked: the linuxdeploy release carries
  only `.AppImage` and `.zsync` files, appimagetool only `.AppImage`). A
  recorded checksum is the strongest pin available without a key to trust.
- **The x86_64 checksums are recorded but were not exercised by a build.** The
  only Linux available here is arm64. They were derived and cross-checked the
  same way as the aarch64 pair, and CI's amd64 release build is what will
  exercise them first — worth knowing that the next tag build is their first
  real use.

**Nothing here touches a running cockpit**, and unlike the rest of this map it
carries no disclosure sensitivity: it is build-time, and the map's note about
holding the commit is about tickets 01 and 02, not this one. Nothing has been
pushed.
