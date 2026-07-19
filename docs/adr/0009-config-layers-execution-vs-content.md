# Config layers: workspace commits shared content and defaults; the local user layer wins for execution

Two layers of harness config live in different places for different reasons. **Committed workspace config** sits in the space's repo (the file ADR 0007 already established for map-kind) and is shared, versioned, portable. **Local user config** lives under the operator's home, is never committed, and is per-machine. When both speak to the same thing, which wins is **not uniform** — and that asymmetry is the decision:

- **Role→agent bindings resolve user-over-workspace.** A committed binding names a concrete CLI and model, which is an *execution/environment* fact — it may name an agent the operator never installed, or a model they don't want to pay for. It must yield to local reality, so the user layer wins. This is also what makes the absent-agent case solvable by configuration rather than by editing someone else's committed file.
- **Prompts resolve space-over-user (ticket 04, unchanged).** A committed prompt is shared project *content* — deliberate customization that should apply to everyone working the repo — so the space layer wins.

The rule that reconciles them: **content the project ships wins; execution choices the operator makes win.**

Two things follow at the boundary of "what may be committed":

- Bindings are `{adapter, model, args?}` — structured, so the harness can reason about them (compare models for heterogeneity, probe binaries for presence). Machine-specific absolute paths do not belong in the committed layer; that is what the user layer is for. The `args` escape hatch exists for flags the adapter does not model, and using it knowingly forfeits the harness's introspection on that binding.
- **Autopilot has no committed representation.** Disabling both reviews is a per-machine, disclaimed choice; a committed autopilot flag is ignored with a warning. Committing "no human reviews this code" for everyone who clones is the exact drift "cockpit, not autopilot" exists to prevent.

## Consequences

- Heterogeneity (`implement ≠ review` model) is **not enforced in config** — it cannot be an invariant (model strings can alias one backend, and the `args` hatch can swap the effective model), so it is surfaced as an *observed*-model line in the human-review brief (ticket 10) rather than guarded at resolution time. The harness always allows; judgment lives at the one human gate.
- Merge is field-level: a user override may set one field and inherit the rest, so the harness renders the **effective** resolved binding to keep silent inheritance visible.
- The committed config file gains a second tenant beside map-kind. The local user layer is the first harness state that is neither a space's committed config nor per-map — its home is `~/.config/wayfinder-harness/`, keyed by space, and the space registry (ticket 11) owns its lifecycle.

## Considered options

- **Uniform direction — one layer always wins** — rejected: space-always-wins makes the absent-agent case unsolvable in config (you cannot override the committed CLI you lack), while user-always-wins lets a personal preference silently override the project's deliberate prompt content.
- **Committed autopilot, honored on clone** — rejected: it resolves a teammate's code unreviewed without their consent from first run.
- **Committed autopilot, confirmed on clone** (mirroring ADR 0007's declared-but-confirmed map-kind) — rejected: even gated behind a local confirmation, it gives the flag a committed meaning the standing preference says it should never have. Autopilot stays purely local.
