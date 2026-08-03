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
