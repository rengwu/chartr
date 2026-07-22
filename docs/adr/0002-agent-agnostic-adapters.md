# Agent-agnostic adapters, not Claude Code natively

The wayfinder method, its skills, and the whole surrounding ecosystem are Claude Code, so building natively on Claude Code would have near-zero impedance. We are deliberately not doing that. Sessions are spawned through per-agent **adapters** covering the popular harnesses (claude, codex, opencode, pi), and are wired by **prompts the chartr injects itself** rather than by any one agent's skill mechanism.

The cost is real: the chartr must ship its own prompt library and may not lean on Claude Code's skills, hooks, or memory. The purchase is that role→agent mapping becomes ordinary configuration — which is also what makes model heterogeneity at the review gate natural rather than bolted on.

## Consequences

- The chartr owns prompt and context assembly. Nothing may assume a Claude-specific affordance.
- "Is this session finished?" cannot rely on an agent-specific signal — see ADR 0004.

## Reaffirmed: the format opens, the injection path does not (simplify, ticket 04)

The injected library is now seven standard `SKILL.md` directories rather than bespoke `<part>.md` files, and this ADR is **unchanged by that**. What was chosen here was never a file format — it was that the *chartr* wires a session to its role and context, rather than leaning on any one agent's skill mechanism. That still holds: the chartr reads the resolved `core` and role bodies itself, composes them with a freshly-built context bundle into one payload, and hands it over with the same one-line read-this-file opener every adapter uses. Nothing is materialised into an agent's native skills path, and no agent's loader is trusted to find, rank, or inject anything.

- What the standard buys is the *other* direction: the same skills are readable and reusable in any agent CLI that reads the format, and hackable on disk without learning a chartr-specific layering convention. Openness of the source, determinism of the path.
- Line 3's "prompts the chartr injects itself" reads **skills** the chartr injects itself; the substance is identical.
- The model-heterogeneity clause in the paragraph above lapsed with the review gate (ADR 0004, amended) — heterogeneity remains ordinary configuration, it just no longer has a gate to be natural at.
