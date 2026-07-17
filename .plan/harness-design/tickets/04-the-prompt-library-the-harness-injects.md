---
type: grilling
blocked_by: [01]
---

# The prompt library the harness injects

## Question

Because the harness cannot lean on any one agent's skill mechanism (ADR 0002), it ships **its own prompts** and injects them to wire a session to a role. What are they, and where do they live?

The wayfinder method already exists as skills written for Claude Code. Decide the relationship, because every option costs something: **vendoring** a copy as prompt text means drift from upstream wayfinder; **pointing** agents at a checked-out skills directory means depending on files the harness does not own and cannot version; **synthesising** condensed per-role prompts means maintaining a second, quietly diverging expression of the method.

Settle:

- **The role set** — grill, prototype, research, implement, review — and whether each needs one prompt or several. A `review` prompt in particular must carry the ticket's **Done-when and its spec**, not merely "review this diff": hand a reviewer only the diff and the gate silently degrades to a style check, losing spec conformance entirely.
- **Composition** — how a role prompt combines with the context bundle (ADR 0005) into a single injected payload, within the size limits ticket 01 turns up. Note the injection mechanics may differ between a headless spawn and an interactive PTY — ticket 02 owns that split; this ticket inherits whatever it decides.
- **Overridability** — whether a space's committed config may replace or extend a role's prompt. A project with house rules will want to, and that wish collides with the harness's interest in prompts it can reason about.
- **Versioning** — what happens when the harness's prompts change underneath a half-driven map, and whether a map should record which prompts resolved its tickets.
