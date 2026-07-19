# wayfinder-harness — implementation

## Destination

The [spec](../harness-design/spec.md) implemented end to end: one distributed, self-contained binary an operator runs to register spaces, watch maps appear and derive live on the star-map, spawn prompt-injected agent sessions in real TUIs against frontier tickets, and gate implementation work through agent review and the human review hub — shipped as a checksummed release for macOS, Linux, and Windows. Done looks like the release ticket resolved with every slice before it demoable in the real cockpit.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working code, not a decision — all decisions were made on the [planning map](../harness-design/map.md) and synthesized into the [spec](../harness-design/spec.md), which is the single source of truth here. Do not re-litigate a decision; if implementation exposes one as wrong, mark the *planning* ticket undermined and raise it, rather than quietly deviating.

**Per-session reading order:** the spec, then this map, then your ticket — and any planning asset the ticket names (the UI tickets lean on the prototypes under `../harness-design/assets/`). Vocabulary comes from `CONTEXT.md` at the repo root; the ADRs in `docs/adr/` are binding.

**Testing:** tests live at the process boundary — start the real binary against a temp fixture space, act over HTTP, assert on control-socket snapshots, the filesystem, and git history, with stub agent CLIs on PATH (spec, Testing Decisions). Extend the rig the walking skeleton establishes; do not add internal seams. The star-map island's mount/model/selection seam is the one frontend test point.

**Before commit:** run the static checks and tests as wired by the walking skeleton (`go vet ./...` and `go test ./...`; the frontend's check and build scripts) — run what exists at your ticket's point in the map. Review the diff (the `review-code` skill if available). No map linter is wired in this repo.

## Decisions so far

<!-- one line per resolved ticket: gist + link. -->

- **01 — walking skeleton**: one Go binary serves the embedded Svelte cockpit shell and pushes the whole derived model over a JSON control socket (resent on reconnect); operator actions are plain HTTP; the process-boundary test rig every later ticket extends is established. [ticket](tickets/01-walking-skeleton.md)

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the planning map — it does not open fog here. -->

## Out of scope

<!-- Inherited from the spec's Out of Scope; these never graduate into tickets on this map. -->

- **Cost and token visibility** — declined on the planning map; reopens only on autopilot or explicit demand.
- **Concurrent-session resource limits** — no governor; the operator owns the machine, the rate limits, and the wallet.
- **The network and environment sandbox** — the operator owns sandboxing; a later effort, not a ticket here.
- **Redesigning the wayfinder method** — the harness extends the markdown adapter by one non-resolving heading and nothing more.
- **Charting as a harness capability** — the user's own `/wayfinder` flow in a shell; the harness only notices the result.
- **Autopilot** — strictly local, non-default, and not built here; the named reopen trigger for caps, clean-tree-at-spawn, machine stuck-detection, and cost.
