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