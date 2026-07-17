---
type: grilling
blocked_by: [01]
---

# Cost and token visibility

## Question

Graduated from the **Cost and token visibility** fog patch once [the agent adapter
contract](./01-the-agent-adapter-contract.md) established what the CLIs actually report:
**token counts are universal** in every agent's JSON event stream; **native dollar cost** exists only
on claude (in-stream `total_cost_usd`) and opencode (`stats`, out of band), while codex and pi report
tokens only; **native budget caps** exist only on claude (`--max-budget-usd`, `--max-turns`).

So the raw material is settled — the open decision is what the cockpit *does* with it. Runaway cost
is a named risk of automating this at all (map Notes), and long-running sessions across several
spaces burn real money.

Decide:

1. **What the cockpit surfaces, and at what grain** — tokens always; dollars derived from tokens × a
   per-model price table the harness maintains (uniform across agents, rather than trusting each
   agent's native figure). Per session? Per map? Per space? A running total?
2. **Whether it caps or merely reports.** A harness-enforced cap (watch stream tokens, call `stop()`
   at a threshold) works for every agent and doesn't depend on claude's native `--max-budget-usd`.
   Is a cap in scope, and is it a hard stop or a warn-and-continue? Who sets the threshold — per
   space, per role, global?
3. **Where the price table lives and how it stays current** — config the operator edits, since model
   pricing drifts and the harness is agent-agnostic.

Keep it a *cockpit, not an autopilot* (map Notes): the human should see spend accumulating and be
able to intervene; anything that must always hold (a hard ceiling, if there is one) belongs in
deterministic code, not an agent.
