# Agent-agnostic adapters, not Claude Code natively

The wayfinder method, its skills, and the whole surrounding ecosystem are Claude Code, so building natively on Claude Code would have near-zero impedance. We are deliberately not doing that. Sessions are spawned through per-agent **adapters** covering the popular harnesses (claude, codex, opencode, pi), and are wired by **prompts the harness injects itself** rather than by any one agent's skill mechanism.

The cost is real: the harness must ship its own prompt library and may not lean on Claude Code's skills, hooks, or memory. The purchase is that role→agent mapping becomes ordinary configuration — which is also what makes model heterogeneity at the review gate natural rather than bolted on.

## Consequences

- The harness owns prompt and context assembly. Nothing may assume a Claude-specific affordance.
- "Is this session finished?" cannot rely on an agent-specific signal — see ADR 0004.
