# wayfinder-harness — the design

## Destination

A design spec for **wayfinder-harness**: a cross-platform, agent-agnostic cockpit that drives wayfinder maps to completion — switching between project spaces, reading a map as a star-map, spawning prompt-injected agent sessions against its frontier tickets, and gating implementation work behind agent and human review.

The map is done when every decision the spec needs is settled — nothing left to decide before `to-spec` and then `to-tickets` turn it into an implementation map. **Plan, don't do:** this map produces decisions, not the harness.

## Notes

**Read before choosing a ticket:** [`CONTEXT.md`](../../CONTEXT.md) for the vocabulary, and [`docs/adr/`](../../docs/adr/) for what is already settled and why. Those decisions are **not open** — re-litigating one costs a session. If a ticket's answer breaks an ADR's premise, say so out loud rather than quietly deciding around it.

**Skills every session should consult:** `domain-modeling` — keep `CONTEXT.md` and the ADRs current as terms crystallise; `grill-me` / `grill-with-docs` — the interview engine the grilling tickets lean on; `research`; `prototype`. At the end of this map: `to-spec`, then `to-tickets`.

**Reference material, not dependencies** (sibling checkouts, paths relative to this repo's parent):

- `../wayfinder` — the **wayfinder-maps** repo. Its model layer (`internal/wayfinder`) and star-map renderer are what this project lifts; copy freely where it elevates the result (ADR 0001). `docs/starmap-design.md` records why the star-map looks and moves the way it does — read it before touching the map view.
- `../skills/pocock/wayfinder/` — the **wayfinder skill** and its markdown adapter (`TRACKER-MARKDOWN.md`), whose derived-status model the harness extends (ADR 0004). `../skills/pocock/to-tickets/` defines what an implementation map is.
- `../expensif/.plan/export-csv-impl/` — an example implementation map, pre-implementation. Useful as a real fixture to prototype against.
- [iudex](https://github.com/rengwu/iudex) — **inspiration only. Do not build on it, fork it, or import from it.** Its lifecycle is deliberately not ours.

**Standing preference:** the harness is a **cockpit, not an autopilot**. A human drives; the deterministic code exists to make that driving safe. Anything that must always be true belongs in code — an agent belongs only where judgment is the product.

**Standing preference:** the client is **hackable**. What the harness injects and reasons with — prompts first of all — is visible on disk as plain markdown, accessible, and editable by the operator; never sealed inside the binary. (Surfaced in ticket 04.)

**The honest ceiling, worth remembering when a ticket promises too much:** this design can make *orchestration* correct, reliable and reversible. It cannot make the *work* correct. Residual risk lands on leaf-implementation quality and on human diligence at the one gate.

## Decisions so far

<!-- one line per resolved ticket: enough to judge relevance, then zoom the link for the detail -->

- [The agent adapter contract](./tickets/01-the-agent-adapter-contract.md) — claude/codex/opencode/pi all drive headless; the contract is `spawn(cwd, model, promptText)` + `observe → {alive, exited, tokens}` + `stop`, with the role wired in the prompt body (no universal system-prompt flag) and *finished* derived from the ticket, not the agent (ADR 0004). Dollar cost, budget caps, and system-prompt flags are optional-with-degradation; resume is excluded by design (ADR 0005). *Undermined by ticket 02 — the headless floor moved; see the ticket.*
- [Knowing a session finished, hung, or died](./tickets/02-knowing-a-session-finished-hung-or-died.md) — every session runs the agent's interactive TUI in a PTY (no headless mode): `observe` degrades to alive/dead, tokens go optional and out-of-band, and the harness *surfaces* working/quiet/dead but never acts on a heuristic. A death halts to the human — resume, respawn fresh, or abandon — with resume narrowed to same-ticket crash recovery (ADR 0005 amended), which also settles deferred tmux: not needed.
- [Telling a planning map from an implementation map](./tickets/03-telling-a-planning-map-from-an-implementation-map.md) — a map's **kind** is declared, never inferred: an explicit, map-level property (not per-ticket) in committed harness config, with the breakable signals (`-impl` suffix, all-`task` tickets) demoted to a one-time guess a human confirms (ADR 0007). An undeclared map is inert until classified; `to-tickets` graduation is something the harness *notices*, not an action it offers. Kind stays out of the wayfinder map format, so vanilla tools read the map unchanged.
- [The cockpit layout](./tickets/08-the-cockpit-layout.md) — three surfaces: a collapsible spaces→maps sidebar, a full-width terminal column with the space's sessions as tabs (plus ad-hoc shells), and the star-map as a floating card summoned over the terminal (no split — xterm never reflows; the tucked-away map's handle carries the "Next up" badge). Nesting = attention + history, not concurrency (ADR 0003 stands). One click: star → full ticket as a bottom sheet, camera easing the star into the free space. Prototype asset holds all seven variants; E "Helm" is canonical.
- [The prompt library the harness injects](./tickets/04-the-prompt-library-the-harness-injects.md) — vendored from the wayfinder skills and **hackable**: five role prompts plus a common core, plain markdown resolved space → user → built-in with per-file replace/append semantics. The payload (core + role + context bundle) is written to a gitignored file in the space and injected via a one-line "read this" opener — one path for every TUI, closing ticket 02's injection loose end. Claim-commit trailers plus a retained payload archive record exactly what each session was told; review payloads always carry Done-when + spec by assembly.
- [Who commits, and how work gets abandoned](./tickets/06-who-commits-and-how-work-gets-abandoned.md) — ownership splits and the harness only appends (ADR 0008): it commits claim/promotion/demotion as pathspec-limited commits (so approval never waits on a live session), agents commit their own work by prompt convention (never push), and violations are surfaced, not enforced. Abandonment demotes `## Proposed Answer` to `### Rejected` prose; undoing commits is the human's, with revert/reset as optional levers. Dirty tree is a badge, not a spawn gate. Git is the audit trail — the fog patch's nothing-to-build outcome, made true by the commit discipline.

## Not yet specified

- **Concurrent-session resource limits.** How many spaces can cook at once before the machine, the provider's rate limits, or the wallet gives out — and whether the harness should govern that rather than let the operator discover it.
- **First-run onboarding.** What someone sees before any space is registered, and how they get from an empty cockpit to a first driven ticket. <clears-with: 11>

## Out of scope

- **The network and environment sandbox.** A leaf agent still hits live APIs, seeds real databases and spends money; serialising sessions and keeping history linear isolates none of that. Worktree-style isolation would not have helped either — it contains the filesystem and git, never the network, and that false safety is the trap. Containing it needs a separate network/env sandbox layer, which is a deep, self-contained problem orthogonal to orchestration. The harness assumes nothing and documents the boundary: **the operator owns sandboxing.** A later effort may design it; redrawing this destination to include it would be a fresh map, not a resumption.
- **Redesigning the wayfinder method itself.** The harness *drives* wayfinder maps and extends the markdown adapter as narrowly as it can — one non-resolving heading (ADR 0004). Changing the method, its skills, or its storage shapes is a different effort. A ticket here that reaches for it is mis-scoped, and should be closed rather than resolved.
