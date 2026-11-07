# Chartr backend patch

Chartr pins Herdr source and applies `terminal-identity.patch` before building.
The patch preserves terminal IDs across live daemon handoff and clears abandoned
mouse reporting when a known foreground job returns control to its owning shell.
Both normal and handed-off runtimes use the existing foreground observation;
unknown probes, live TUIs, and shell-owned modes are preserved. It does not reset
screen contents, scrollback, or PTY termios settings. Chartr uses Herdr's
existing agent states for best-effort attention; no custom attention protocol,
producer registration, or provider-specific outcome adapter is added.

`crates/chartr-herdr/src/lib.rs` pins the upstream revision, patch SHA-256, complete
build version, and protocol. `fetch.sh` verifies the patch even when reusing a
binary. Its source cache includes the patch hash; failed application/builds leave
the previous executable intact.

When changing the patch:

1. Apply it to the pinned, clean upstream archive and edit that disposable tree.
2. Regenerate a patch with `a/` and `b/` paths. Preserve unrelated upstream files.
3. Update the full SHA-256 and the version suffix `identity.<first 12 hash chars>`.
4. Run the upstream terminal identity/handoff and mouse recovery tests, Chartr's client/policy tests,
   and `scripts/tests/test_herdr_fetch.py`.
5. Build with `sh vendor/herdr/fetch.sh` and exercise the exact sidecar in an isolated
   profile, including a live handoff and the killed-TUI direct-attach regression
   in `crates/chartr/tests/support/mouse.rs`. See `docs/wiki/verification/setup.md`.

Upstream upgrades require reviewing and reapplying this patch. It is a local
Chartr distribution change, not an upstream release or accepted upstream PR.
