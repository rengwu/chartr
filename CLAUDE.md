# CLAUDE.md

Guidance for Claude Code sessions working in this repo.

## Wayfinder maps in this repo

This repo plans with wayfinder maps under `.plan/`, and it is a space chartr
watches. The maps are plain local-markdown — the vendored `tracker-convention`
skill's adapter, with nothing added on top. Chart with `/wayfinder` and graduate
with `/to-tickets` exactly as you would anywhere else; chartr notices the new
directory and offers it live. There is no chartr-side registration step.

## Frontend design system (`web/`)

The cockpit chrome runs on a real design system — **shadcn-svelte + Tailwind v4**
on an **olive / warm-neutral** token theme. **Read `docs/design-system.md` before
touching any UI** (ADR 0012; the chrome/island split is ADR 0010). Style new UI on
**tokens + primitives + Phosphor** by default, and hold these hard rules:

- **Use a token for every colour.** Never write a raw hex / rgb / named colour in
  the chrome. If no token fits, the palette is missing a role — flag it, don't
  inline one.
- **Use a primitive for every component.** Reach for a vendored shadcn-svelte
  primitive (`web/src/lib/components/ui/`) for buttons, badges, inputs, dialogs,
  sheets, tabs, cards. **Never hand-roll a `.btn`/`.badge`/`.card` or bespoke
  chrome CSS.** A genuinely new shared pattern earns *one* token-driven
  `@layer components` class, not a pile of one-offs.
- **No amber, no chroma in the chrome.** The chrome is monochrome (hue ~107);
  `--destructive` (red) is the only chromatic token. Emphasis goes on `--primary`
  / `--ring`. The only amber in the whole product is the star-map's `claimed`
  star.
- **Never reach inside an island's renderer to re-theme it.** The xterm terminal
  and the canvas star-map are imperative islands (ADR 0010). Re-theme *at the
  seam* — `web/src/lib/tokens.ts` → the island wrapper — feeding resolved colours
  in, never editing the renderer. The star-map's six status hues are exempt
  data-viz colour, kept in `web/src/lib/starmap/theme.ts`.
- **Icons are Phosphor** (`phosphor-svelte`); text is **IBM Plex Sans/Mono**.
  Fonts and icons are self-hosted and bundled — **no CDN, no runtime fetch** (the
  frontend is `go:embed`ed into one offline binary).

To add a primitive: `cd web && npx shadcn-svelte@latest add <component>`, then
swap lucide icons for Phosphor, prune unused deps, and re-check for raw colour
before committing (see `docs/design-system.md` → *Adding a primitive*).

## Before committing frontend changes

Run the frontend `check` and `build` scripts plus `vitest`, and
`go vet ./...` / `go test ./...` (the embed test compiles against `dist/`). No
amber in the built CSS.
