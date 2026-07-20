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
- **02 — register a space, and role bindings**: the space registry is a rebuildable index in user config (register with an announced `git init`, forget-not-destroy removal, pinned-then-recency ordering, a path-derived stable id); role bindings resolve `{adapter, model, args?}` across built-in ‹ committed workspace (`.wayfinder-harness.toml`) ‹ local user, field-level and user-over-workspace, with absent adapters badged and a committed autopilot flag ignored with a warning; register/deregister/pin are plain HTTP actions that rebuild-and-push the model. [ticket](tickets/02-register-a-space-and-role-bindings.md)
- **03 — maps appear and derive**: the wayfinder-maps model layer is ported whole (ADR 0001) and extended by exactly the non-resolving `proposed` status (ADR 0004), so the ported `Frontier` is already the stricter one — a merely-proposed blocker never unblocks its dependents. Discovery is layout-agnostic (a map is found by its `map.md`, handling `.plan/<slug>/` and `.plan/maps/<slug>/`, hard-coding neither) and tolerant (a malformed map renders as-is with its malformation surfaced, never refused); an fsnotify watch over each space's `.plan/` makes maps appear by notice, not refresh; the sidebar nests spaces → maps with finished maps sorting last. [ticket](tickets/03-maps-appear-and-derive.md)
- **04 — classify a map's kind**: kind is declared, never inferred (ADR 0007) — a discovered map is inert (offers no session actions) until a human classifies it, with the `-impl`-suffix / all-`task` conventions surviving only as a pre-filled one-keystroke guess. Classification is one HTTP action writing a `[maps."<slug>"]` kind into committed workspace config (the shared layer, so teammates agree); the write *appends* rather than re-encodes, sparing the operator's own bindings and comments, and refuses an already-declared slug. An unrecognised committed kind is surfaced-and-inert; a renamed map directory dangles its entry back to unclassified-and-inert, never an error. The classify write lands in the working tree uncommitted — the harness's own commits stay the enumerated lifecycle writes (ADR 0008). [ticket](tickets/04-classify-a-maps-kind.md)

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
