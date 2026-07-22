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

## Amended: how the opener *reaches* the agent is per-adapter

Every session still opens with the same one line — read this payload file — but delivering it by typing into a live TUI turned out to be the wrong universal default. Two things break it, both in the agent rather than in the chartr: a TUI that distinguishes return (`\r`) from linefeed (`\n`) reads a typed `\n` as *insert a newline*, so the opener sits in the composer unsent until a human presses enter; and a TUI that buffers pastes swallows a submit key that arrives in the same chunk as its text. An operator watching a spawn land in a chatbox and wait for them is the whole failure this product exists to remove.

So **delivery is now modelled, not assumed** — three modes, chosen per adapter (`internal/adapter`):

- `argv` — the opener is the CLI's trailing positional argument (`claude [prompt]`, `codex [PROMPT]`, both of which start the *interactive* TUI with it already submitted, not a headless run). Nothing is typed, so nothing can race startup or be eaten by paste handling. **Preferred wherever an agent offers it.**
- flag — the same, for CLIs whose positional means something else: `prompt = "--prompt"`.
- `type` — keystrokes into the live TUI, submitted with a carriage return in its own write. The universal fallback, needing nothing of the CLI but a PTY.

This does not reopen line 3. What is agent-specific here is a command-line convention, which is exactly what an adapter is for; the payload, the skills, and the assembly are untouched and still uniform. Delivery is in fact the *only* CLI convention the seam models — `model` was retired from bindings for exactly this reason (ADR 0009, amended): a model is a flag, and flags are the operator's to write, because the chartr has no business knowing what any given harness calls one. Delivery earns its place only because the chartr itself must act differently depending on the answer. The chartr ships `argv` only for command lines it has checked first-hand, because a wrong guess refuses to start while typing merely takes a beat longer — everything else types, and any operator upgrades their harness with one line of binding config (`prompt = "argv"`), without this repo learning about their CLI first. That hatch, not the table, is what keeps the set of drivable agents open.
