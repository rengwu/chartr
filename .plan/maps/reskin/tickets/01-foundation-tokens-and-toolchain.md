---
type: task
blocked_by: []
---

# Foundation — Tailwind v4, shadcn-svelte, tokens, type and icons

## Question

Stand up the design-system toolchain the rest of the map builds on, changing the plumbing without yet reskinning any surface. Add **Tailwind v4** (the `@tailwindcss/vite` plugin, no separate config file — v4 is CSS-first) and **shadcn-svelte** wiring to the Svelte 5 + Vite app: a `components.json` (style *Mira*, base colour *Olive*, the `$lib` aliases), the `cn()` utility (`clsx` + `tailwind-merge` via `tailwind-variants`), and the path aliases in `tsconfig.json` / `vite.config.ts` that shadcn-svelte imports assume. No SvelteKit — vendor components manually if the CLI balks at a non-Kit app.

Fold `assets/theme.css` into `web/src/app.css` under Tailwind v4's `@theme`/`@layer` model: the full `:root` (light) and `.dark` token sets, `--radius: 0.45rem`, and the `@theme inline` mapping that exposes `--color-background`, `--color-primary`, … so utilities like `bg-background` / `text-muted-foreground` resolve. The document boots in dark (`.dark` on `<html>`, `color-scheme: dark`). Keep the *old* bespoke rules in `app.css` for now — later tickets delete them as each surface migrates — but the token layer and Tailwind base must coexist with them without visual regression.

**Type:** self-host **IBM Plex Sans** (chrome) and **IBM Plex Mono** (paths/code/eyebrows) as subsetted woff2 under `web/src/assets/fonts/`, wired via `@font-face` in `app.css` and the `--font-sans` / `--font-mono` theme tokens — no CDN or Google Fonts (the build is `go:embed`ed offline). **Icons:** install `phosphor-svelte`; no glyphs are swapped in this ticket, but prove one Phosphor icon renders.

Build the **token bridge scaffold** — `web/src/lib/tokens.ts` — a tiny helper that reads resolved CSS custom properties off `document.documentElement` (via `getComputedStyle`) and returns them as hex/rgb, so tickets 04's islands can source xterm's `ITheme` and the star-map palette from the same tokens. Scaffold + unit test only here; the islands consume it later.

Done when: `npm run dev`, `npm run build`, and `npm run check` all succeed with Tailwind + the theme tokens live; a throwaway probe element proves `bg-background` / `text-foreground` / `rounded-md` / IBM Plex Sans / one Phosphor icon all render in dark mode; `tokens.ts` returns correct values for a known token under a jsdom/vitest test; `go vet ./...` and `go test ./...` (the embed test against `dist/`) stay green; and no existing surface has visually regressed (the old `app.css` rules still apply until their tickets migrate them).

## Answer

The design-system toolchain goes live and, per an **operator call mid-ticket, the bespoke pre-reskin chrome was ripped out rather than preserved** — Tailwind's full base (Preflight) is on, the olive tokens are canonical, and the legacy `app.css` chrome is gone. The cockpit now renders on Preflight defaults until tickets 02–04 rebuild each surface on primitives; that transient unstyled state is intended, not a regression.

> **Scope note for review:** the map's Done-when clause "no existing surface has visually regressed (the old `app.css` rules still apply)" was **waived by the operator** ("no need to preserve anything about the legacy skin; overwrite everything"). This also changes tickets **02–04**: they no longer *delete* legacy `app.css` blocks (there are none) — they are now pure build-up, styling each surface on primitives + tokens from the Preflight baseline. Flagging so the map's per-surface "delete its old block" wording is read in that light. The two `<style>`-block surfaces (`DetailPane`, `PayloadPreview`) still carry scoped legacy CSS — left for ticket 02, which owns those surfaces.

**Toolchain.** Added `tailwindcss@4` + `@tailwindcss/vite` (CSS-first, no config file — the plugin reads `app.css`), `clsx` + `tailwind-merge` + `tailwind-variants`, `phosphor-svelte`, and the `@fontsource/ibm-plex-{sans,mono}` packages as the woff2 source.
- `web/components.json` — shadcn-svelte config (style *Mira*, base *Olive*, `$lib` aliases, `app.css` as the tailwind css entry). Vendored manually; the CLI is not run (non-Kit app).
- `web/src/lib/utils.ts` — the `cn()` helper (`twMerge(clsx(...))`) that vendored primitives import from `$lib/utils`.
- `$lib` alias wired in `vite.config.ts`, `vitest.config.ts` (its own resolver), and `tsconfig.json` `paths`.

**Tokens + Tailwind, in `web/src/app.css` (rewritten).** Full `@import 'tailwindcss'` (theme + **Preflight** + utilities), the olive `:root` (light) / `.dark` (dark) token sets with `--radius: 0.45rem`, and the `@theme inline` mapping that exposes `--color-background`, `--color-primary`, `--color-muted-foreground`, … plus `--font-sans`/`--font-mono` and the `--radius-*` scale, so `bg-background` / `text-muted-foreground` / `rounded-md` / `font-sans` resolve. A small `@layer base` seam sets the token-driven default border colour, `color-scheme: dark`, full-height `html/body/#app`, and the document surface/type from the tokens so bare elements read on the theme. `index.html` boots `<html class="dark">`. With the legacy `:root` gone there is **no token collision** — the tokens are canonical, and `app.css` contains no amber.

**Type.** IBM Plex Sans (400/500/600) and Mono (400/500), latin woff2 subsets, copied to `web/src/assets/fonts/` and wired via `@font-face` in `app.css` — no CDN, bundled into the `go:embed` dist (Vite hashes them into `dist/assets/*.woff2`, confirmed in the build).

**Icons.** `phosphor-svelte` installed; a `Compass` icon renders in the probe (no chrome glyphs swapped this ticket).

**Token bridge.** `web/src/lib/tokens.ts` — `readToken` (raw custom-property read off `documentElement` via `getComputedStyle`), `resolveColor` (any CSS colour → concrete `rgb(...)` through a throwaway probe element, which is how the oklch tokens become the hex/rgb xterm's `ITheme` and the star-map palette need), plus `readColor`/`readTokens` convenience. Scaffold only; ticket 04 wires the islands. `web/src/lib/tokens.test.ts` — 10 jsdom asserts covering token reads, whitespace trim, unset tokens, hex→rgb resolution, and a named-map read.

**Probe.** `web/src/lib/Probe.svelte` (throwaway) exercises `bg-background` / `text-foreground` / `rounded-md` / `font-sans` / a Phosphor icon on a `bg-card` panel; `main.ts` swaps it in only under the `#probe` URL hash so the cockpit is otherwise untouched. Removed with the component in 02–04.

**Against Done-when.** `npm run check` (0 errors / 0 warnings), `npm run build` (fonts bundled, `app.css` → **18.7 kB**, down from 32 kB with the legacy chrome removed, utilities + Preflight emitted), and `npm run dev` (server up on :5173) all succeed; `tokens.ts` passes its jsdom test (33 frontend tests green total); `go vet ./...` clean and `go test ./...` green including the `web` embed compile against `dist/`; the built CSS confirms the probe utilities emit, the `.dark`/`:root` tokens and Preflight reset are present, the `@font-face` src resolved to hashed woff2, and `app.css` carries **no amber**. The *visual* proof is now **closed**: the `#probe` surface was screenshotted in Chrome (dark mode) showing `bg-background`/`bg-card`/`text-foreground`/`text-muted-foreground`, `rounded-md`, IBM Plex Sans, the Phosphor compass, and the near-white `bg-primary` button all rendering correctly; the plain cockpit was also captured, confirming the app is functional (spaces load, control socket open) and intentionally unstyled on the token background pending tickets 02–04.