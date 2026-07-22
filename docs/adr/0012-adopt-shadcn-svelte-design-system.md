# Adopt shadcn-svelte + Tailwind v4 on an olive/neutral theme; de-amber the chrome

The cockpit chrome was hand-rolled CSS — a bespoke `app.css` of one-off `.btn`,
`.badge`, `.card`, and layout classes carrying an amber accent (`#d9a441`). Every
new surface re-derived the same buttons and tags by hand, drift-prone and with no
written standard to hold. This ADR adopts a **real design system**: the chrome is
rebuilt on **shadcn-svelte** primitives over **Tailwind v4**, driven by an
**olive / warm-neutral** token theme, with the bespoke chrome retired. The spec is
`docs/design-system.md`; this records the decision.

Three things are decided:

- **The library is shadcn-svelte + Tailwind v4**, the official Svelte port of
  shadcn (Bits UI primitives, tailwind-variants), *not* a rewrite to React. The
  Svelte 5 chrome from ADR 0010 stays exactly as decided there; only how it is
  *styled* changes. Primitives are **vendored** into `web/src/lib/components/ui/`
  (real source we own and edit), configured via `web/components.json` (style
  *Mira*, base colour *Olive*), merged with `cn()`. Tailwind v4 runs CSS-first
  via `@tailwindcss/vite` — no `tailwind.config.js`; tokens and the `@theme`
  mapping live in `web/src/app.css`.

- **The theme is the olive/neutral preset, and the chrome de-ambers.** Every
  token sits at hue ~107 (a warm neutral); **`--destructive` (red) is the only
  chromatic token.** The old amber accent has no home here: active / pinned /
  selected / "on" states move to `--primary` and `--ring`, destructive stays
  `--destructive`, and **no amber appears anywhere in the chrome.** Both light and
  dark token sets ship; the app boots dark. Type is **IBM Plex Sans/Mono** and
  icons are **Phosphor** (`phosphor-svelte`), both self-hosted and bundled into
  the `go:embed` dist — no CDN or runtime fetch, per ADR 0010's offline-binary
  constraint.

- **Island palettes move to the shared tokens at the seam.** The xterm terminal
  and the canvas star-map remain the imperative islands of ADR 0010: the chrome
  never reaches inside their renderers to re-theme them. Instead a token bridge
  (`web/src/lib/tokens.ts`) resolves the CSS custom properties to concrete colour
  strings, and each island's Svelte *wrapper* feeds those in — xterm's `ITheme`,
  the star-map's `setBackground()`. The star-map's **six status hues are exempt
  data-viz colour** (categorical meaning, not brand), kept and re-tuned to sit on
  the theme's warm near-black card; amber survives only as the `claimed` star.

## Consequences

- The bespoke `app.css` chrome is gone. `app.css` now holds the token
  declarations, the `@theme inline` Tailwind mapping, the font faces, a base
  seam, and exactly two hand-written primitives (`.cockpit-bar`, `.prose-sm`) —
  both token-driven. New chrome composes primitives + utilities; it does not
  hand-roll CSS.
- There is now an **enforceable standard**: `docs/design-system.md` (the spec),
  the root `CLAUDE.md` guardrail, and the space's committed `implement` skill
  (`.wayfinder-harness/skills/implement/`, a fork of the shipped one carrying the
  rules — it was `prompts/implement.append.md` before the skill repackaging
  retired the `.append.md` convention) so harness-spawned implementation sessions
  inherit them. New UI styles on tokens + primitives + Phosphor by default.
- A **raw colour in the chrome is now a defect**, not a choice. If a surface needs
  a colour no token covers, the palette is missing a role — that gets flagged, not
  inlined.
- Adding a component is a bounded ritual: `shadcn-svelte add`, then swap lucide
  for Phosphor, prune unused deps, and re-check for raw colour before commit.
- ADR 0010 is unchanged and reinforced: the Svelte-chrome / imperative-island
  split and its seam are the exact boundary the island-palette rule rides on.

## Considered options

- **Stay bespoke, just de-amber the existing CSS** — cheapest edit, but keeps the
  hand-rolled `.btn`/`.badge` drift farm and leaves no primitive to reuse or
  standard to enforce; the next session re-derives buttons by hand again.
- **Rewrite the chrome to React + shadcn (the canonical React port)** — the most
  "standard" shadcn, but throws away the Svelte 5 chrome ADR 0010 deliberately
  chose, for no gain the Svelte port doesn't also give.
- **A different Svelte component library (Skeleton, Flowbite-Svelte, etc.)** — a
  dependency we don't own the source of, styled its own way; shadcn-svelte
  vendors editable source and rides Tailwind tokens directly, which is what makes
  the token theme and the seam bridge clean.
- **Tailwind with a `tailwind.config.js`** — the v3 shape; v4's CSS-first
  `@theme` keeps the single source of truth in `app.css` next to the tokens, with
  no second config file to drift.
- **Fold the star-map hues into the neutral palette too** — visually consistent
  but destroys the map's whole purpose: the six status hues are the at-a-glance
  read. Kept as an explicit data-viz exemption instead.
- **Re-theme the islands inside their renderers** — simplest to write, but breaks
  ADR 0010's island boundary and couples canvas/xterm code to CSS; rejected for
  the seam bridge.
