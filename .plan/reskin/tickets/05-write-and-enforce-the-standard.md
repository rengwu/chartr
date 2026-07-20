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

## Answer

All four deliverables landed, each verified against the code rather than assumed:

- **`docs/design-system.md`** — the spec. Token table maps every semantic token
  (from `web/src/app.css`) to its Tailwind utility and its *purpose*, with the
  "chrome is monochrome; `--destructive` is the only chroma" rule stated up front
  and "where the amber went" (→ `--primary`/`--ring`) called out. Covers the type
  scale (IBM Plex Sans/Mono, self-hosted woff2, no CDN), the `0.45rem` radius
  scale, the vendored-primitive inventory (`button` · `badge` · `card` · `dialog`
  · `input` · `label` · `scroll-area` · `sheet` · `tabs`) with actual Button/Badge
  variant lists, the *add-a-primitive* ritual (CLI → swap lucide for Phosphor →
  prune deps), the Phosphor icon convention, the island seam + ADR-0010 rule, the
  star-map data-viz exemption (its six hues live in `starmap/theme.ts`, amber only
  as `claimed`), and a do/don't.
- **`docs/adr/0012-adopt-shadcn-svelte-design-system.md`** — new ADR in the repo's
  format (title-as-decision `#`, prose, `## Consequences`, `## Considered
  options`). Records shadcn-svelte + Tailwind v4 over bespoke/React, the
  olive/neutral de-ambered theme, and island-palette-at-the-seam; cross-references
  ADR 0010.
- **`CLAUDE.md`** (root, repo had none) — a concise frontend design-system rules
  block, guardrails as imperatives, pointing at `docs/design-system.md`; plus the
  before-commit check list.
- **`.wayfinder-harness/prompts/implement.append.md`** — short layered append
  (per the prompt library's `<part>.append.md` mechanism) telling harness-spawned
  implementation sessions that touch `web/` to follow `docs/design-system.md`.
  Frontend rule only, doesn't restate the doc. **Path note:** the ticket wrote
  `prompts/implement.append.md`, but `/prompts/` is the *materialized* runtime
  library and is `.gitignore`d — a file there is per-machine and never committed,
  so a fresh clone / another operator would not inherit it, defeating the whole
  point. The prompt library's committed overlay path (README; ADR 0009: committed
  *content* ships to everyone) is `.wayfinder-harness/prompts/`, which is
  committable. Landed there so the rule actually travels with the repo.

**Verified against code (not assumed):** token names/values from `web/src/app.css`
and `assets/theme.css`; the vendored primitive list from
`web/src/lib/components/ui/`; Button/Badge variants from the component sources;
`cn()` + prop-shape helpers in `utils.ts`; `components.json` (Mira/Olive, `$lib`);
Phosphor imports across the chrome; `index.html`'s `<html class="dark">`; the five
bundled woff2 subsets in `web/src/assets/fonts/`; and the island seam usage
(`readColor` → xterm `ITheme` in `Terminal.svelte`, `setBackground(readColor('--card'))`
in `StarMap.svelte`, palette in `starmap/theme.ts`). ADR format matched against
0009/0010/0011.

**Tests:** this ticket is documentation only — no `web/` source or Go code
changed — so the `check`/`build`/`vitest`/`go vet`/`go test` suites are unaffected
by it (all were green as of ticket 04). Left out deliberately: no code or config
was touched to "prove" the docs; the doc is the deliverable.