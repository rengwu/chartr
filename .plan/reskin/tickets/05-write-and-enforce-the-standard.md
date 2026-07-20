---
type: task
blocked_by: [02, 03, 04]
---

# Write and enforce the standard — doc, ADR, CLAUDE.md

## Question

Capture the design system now that it is real, so every future session — human-driven or harness-spawned — holds it without being re-taught. Four deliverables:

- **`docs/design-system.md`** — the spec: the token palette and what each semantic token is *for* (with the "chrome is neutral, `--destructive` is the only chroma" rule stated), the type scale (IBM Plex Sans/Mono) and radius, the primitive inventory (which shadcn-svelte components are vendored and how to add more), the Phosphor icon convention, and a short **do / don't** — do reach for a primitive and a token; don't hand-roll a `.btn`, don't add a raw hex, don't re-introduce amber into the chrome, don't reach inside an island's renderer (point at ADR 0010). Include the star-map's data-viz-colour exemption and where its palette lives.
- **`docs/adr/0012-adopt-shadcn-svelte-design-system.md`** — a new ADR in the repo's existing format (title as the decision, prose, `## Consequences`, `## Considered options`): why shadcn-svelte + Tailwind v4 over staying bespoke or rewriting to React, why the olive/neutral theme and the de-ambered chrome, and the island-palette-at-the-seam rule. Cross-reference ADR 0010.
- **Root `CLAUDE.md`** — a concise rules block (the repo has none yet) that points at `docs/design-system.md` and states the hard guardrails as imperatives, so any Claude Code session in this repo styles new UI on tokens + primitives + Phosphor by default and never hand-rolls chrome CSS or amber.
- **`prompts/implement.append.md`** — a short layered append (per the prompt library's override mechanism) so harness-spawned *implementation* sessions that touch `web/` are told to follow `docs/design-system.md`. Keep it to the frontend rule; don't restate the whole doc.

Done when: all four files exist, are internally consistent with what tickets 01–04 actually built (token names, primitive list, and file paths verified against the code, not assumed), and the ADR follows the established `docs/adr/` format; `docs/design-system.md` is accurate enough that a fresh session could reskin a new component correctly from it alone.