# CLAUDE.md

Guidance for Claude Code sessions working in this repo.

## Wayfinder maps in this repo

This repo plans with wayfinder maps under `.plan/maps/`, and it is a space chartr
watches. `.plan/maps/<slug>/` is the fixed layout — chartr's `conventions.md`
states the whole file-format contract, and discovery reads that root and no
other. Chart with the `wayfinder` skill and graduate with `to-tickets`; chartr
notices the new directory and offers it live. There is no chartr-side
registration step.

Skills come from **registered sources**, not from this repo — chartr ships none
(ADR 0017). The set chartr seeds itself with is vendored at
`internal/sources/assets/chartr-skills/`, refreshed with `make vendor-skills`;
edit those skills in their own repo, never in the vendored copy.

## Frontend design system (`web/`)

shadcn-svelte + Tailwind v4 on an olive/warm-neutral token theme.
**Read `docs/design-system.md` before touching any UI** (ADR 0012; chrome/island
split is ADR 0010). Hard rules:

- **A token for every colour** — never a raw hex/rgb in the chrome. No token
  fits? The palette is missing a role; flag it.
- **A primitive for every component** — use `web/src/lib/components/ui/`, never
  hand-roll a `.btn`/`.badge`/`.card`.
- **Chrome is monochrome** — `--destructive` is the only chromatic token;
  emphasis via `--primary`/`--ring`.
- **Re-theme islands at the seam** — `web/src/lib/tokens.ts` → island wrapper,
  never inside the xterm/star-map renderers.
- **Phosphor icons, IBM Plex text**, self-hosted — no CDN or runtime fetch.

Add a primitive with `cd web && npx shadcn-svelte@latest add <component>`, then
swap lucide→Phosphor and prune unused deps.

## Before committing frontend changes

Run the frontend `check` and `build` scripts plus `vitest`, and
`go vet ./...` / `go test ./...` (the embed test compiles against `dist/`).
