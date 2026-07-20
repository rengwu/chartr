# cockpit reskin — a shadcn-svelte design system

## Destination

The cockpit chrome rebuilt on a real design system: **shadcn-svelte + Tailwind v4**, driven by the pasted **olive / warm-neutral** theme (`assets/theme.css`), with the hand-rolled `web/src/app.css` retired in favour of tokens and reusable primitives. The chrome de-ambers to the theme's neutral; the star-map keeps its six categorical status hues as exempt data-viz colour. The effort ends with a written, enforceable standard — a design-system doc, an ADR, and a `CLAUDE.md` guardrail — so every future session holds the line without being told.

Done looks like: the cockpit runs on Tailwind + the theme tokens with the bespoke `app.css` chrome gone; every surface uses shadcn-svelte primitives and Phosphor icons in IBM Plex Sans; the xterm and star-map islands read their palette from the shared tokens through a seam bridge (ADR 0010 intact); `svelte-check`, the Vite build, `vitest`, `go vet`, and `go test` are all green; and the governance ticket has landed the doc, the ADR, and the CLAUDE.md rules.

## Notes

**This map delivers code, not decisions.** The design decisions were settled with the operator up front and are recorded below — do not re-litigate them. If the reskin exposes one as wrong, say so plainly in your answer for a human to weigh, rather than quietly deviating.

**Per-session reading order:** this map, then your ticket, then `assets/theme.css` (the exact token values) and the current `web/src/app.css` (the surface you are replacing). Vocabulary is `CONTEXT.md`; the ADRs in `docs/adr/` are binding — especially **ADR 0010** (the chrome hosts the xterm and star-map islands but never reaches inside them; any island re-theming happens *at the seam*, feeding the renderer, never inside it).

**Apply distill as you go.** The reskin is also a simplification pass (the `distill` skill): fold the duplicated hand-rolled buttons/badges/inputs into single primitives, keep one primary action per surface, flatten nesting, and cut decorative variance. Reskin is the moment to remove, not just re-paint — without dropping any function or accessibility.

**Offline binary constraint.** The frontend is `go:embed`ed into a single distributed binary (ADR 0010), so fonts and icons must be **self-hosted / bundled** — no CDN, no Google Fonts, no runtime network fetch. Ship IBM Plex woff2 subsets and the Phosphor Svelte components in the build.

**Before commit:** run the frontend `check` and `build` scripts plus `vitest`, and `go vet ./...` / `go test ./...` (the embed test compiles against `dist/`). Review the diff.

## Decisions so far

<!-- Settled with the operator on 2026-07-20, before the map was cut. -->

- **The library is shadcn-svelte + Tailwind v4** — the official Svelte port of shadcn (Bits UI primitives, tailwind-variants), not a rewrite to React. The Svelte 5 components stay; their markup and classes are rewritten. (ADR 0010's Svelte-chrome decision is unchanged; this only re-tools how the chrome is styled.)
- **The theme is the olive / warm-neutral preset** in `assets/theme.css` (shadcn `create?preset=b6t6ENuIS`; style *Mira*, base colour *Olive*, radius *Small* = `0.45rem`, menu accent *Subtle*). Every token sits at hue ~107; the only chromatic token is `--destructive` (red). Both light and dark ship; the app defaults to dark.
- **The chrome de-ambers to neutral.** The old amber accent (`--accent #d9a441`) has no home in this palette. Active / pinned / selected / "on" states move to `--primary` and `--ring`; destructive stays red (`--destructive`). No amber anywhere in the chrome.
- **The star-map palette is exempt data-viz colour.** Its six status hues (resolved/frontier/claimed/proposed/blocked/out_of_scope) are categorical *meaning*, not brand decoration, so they are kept — re-tuned to sit legibly on the theme's warm near-black card. Amber survives only as the "claimed" star, nowhere in the chrome.
- **Type is IBM Plex Sans (chrome) + IBM Plex Mono (paths/code/eyebrows)**, self-hosted woff2. **Icons are Phosphor** (`phosphor-svelte`), replacing every emoji/unicode glyph in the chrome.
- **The standard is written down and enforced** — `docs/design-system.md`, a new ADR, a root `CLAUDE.md` rules block, and a `prompts/implement.append.md` pointer so harness-spawned UI sessions inherit it too.

<!-- Resolved-ticket outcomes, logged as each lands (below the pre-cut decisions above). -->

- **01 — foundation (Tailwind v4 + shadcn-svelte, tokens, type, icons)**: the design-system toolchain is live — `@tailwindcss/vite` (CSS-first, no config file), `components.json` (Mira/Olive, `$lib` aliases), the `cn()` util, `phosphor-svelte`, and self-hosted IBM Plex Sans/Mono woff2 subsets bundled into the `go:embed` dist. `web/src/app.css` was rewritten onto full Tailwind (Preflight) with the olive `:root`/`.dark` tokens, the `@theme inline` mapping, and a small base seam; `web/src/lib/tokens.ts` scaffolds the island token bridge (resolves the oklch tokens to rgb at the seam for ticket 04) with jsdom tests; a `#probe` surface proves the utilities/font/icon render in dark mode (screenshot-verified). **Operator call mid-ticket:** the bespoke pre-reskin chrome was *ripped out* rather than preserved — so the map's "no visual regression / old `app.css` still applies" clause is **waived**, and tickets 02–04 recast from "delete the legacy block" to pure build-up on the Preflight baseline; the cockpit renders unstyled-by-design until those land. [ticket](tickets/01-foundation-tokens-and-toolchain.md)
- **02 — the leaf surfaces**: the self-contained surfaces are rebuilt on shadcn-svelte primitives — `Modal` → **Dialog** (the Add-a-space call site rides it, external contract unchanged), `RegisterForm` → **Input** + **Button**, `DetailPane` → **Card** / **ScrollArea** with status **Badge**s, `PayloadPreview` → the wide Dialog + ScrollArea with per-segment layer-provenance Badges. Primitives are vendored via the CLI (button / dialog / input / card / badge / scroll-area / label); the CLI's lucide `X` swapped for Phosphor and its unused `@internationalized/date` pruned, `bits-ui` filed under runtime deps and the prop-shape helpers added to `utils.ts`. A shared `.prose-sm` scale in `app.css` rehomes the rendered-markdown styles for both DetailPane and PayloadPreview. Three palette remaps are flagged for the operator (nothing reopened): `resolved` reads as the solid `default` Badge — the monochrome palette has no green to key a literal success tint off; the four payload layers, once told apart by hue, now read by the four non-destructive Badge variants; and the payload `⚠` warnings drop the old amber tint for a neutral bordered box, `--destructive` reserved for real problems. [ticket](tickets/02-reskin-the-leaf-surfaces.md)
- **03 — the shell and panes**: the cockpit shell, sidebar, space header, terminal tab strip, and map-card chrome are rebuilt on token utilities + shadcn-svelte primitives + Phosphor icons. The three chrome bars collapse into one `.cockpit-bar` primitive on a shared `--bar-h`, so the one-tier alignment is held by a single rule; every active / pinned / "on" state de-ambers to `--primary` / `--ring` / the `secondary` Button variant, with `--destructive` kept only for forget / tab-close / the agent-missing badge. The hand-rolled icon/kind/card/field variants fold into the shared `Button` + `Badge` (the binding-layer tags reuse ticket 08's built-in→outline / workspace→secondary / user→default scale). The terminal tab strip becomes a shadcn **Tabs** (the close `X` an adjacent Button since it can't nest in a trigger; the active look driven off the existing `active` derived so the single keyed island renders unchanged) and the bindings drawer a right **Sheet**. Float/dock, the resize grip, the detail-pane holder, and the island-wrapper sizing — all removed with the bespoke `app.css` in ticket 01 — are re-established on tokens (the build-up 01's note recast this ticket to). One additive change: the `Esc` handler yields to a focused dialog, so dismissing the Sheet no longer also dismisses the star-map. `svelte-check` / build / `vitest` / `go vet` / `go test` all green; no amber in the built CSS. [ticket](tickets/03-reskin-the-shell-and-panes.md)

## Not yet specified

<!-- Empty. The decisions above settle the effort; a ticket that surfaces a genuinely new design question flags it for the operator rather than deciding it here. -->

## Out of scope

- **Restructuring the layout or interaction model.** This is a reskin, not a redesign: the sidebar → stage → status-bar shell, the pane/drawer/dock behaviours, and every ticket-11-prototype placement stay. Only the visual system changes.
- **Re-architecting the islands.** The xterm and star-map renderers keep their internals and the ADR-0010 seam; only their *palette source* moves to the shared tokens.
- **A light-mode toggle in the UI.** Both token sets ship, but exposing an in-app theme switcher is a later effort; the cockpit boots dark.
- **New features.** No new panes, actions, or capabilities graduate here; this map only re-skins what exists.
