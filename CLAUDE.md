# CLAUDE.md

Guidance for Claude Code sessions working in this repo.

## Where the context is

- **`CONTEXT.md`** — the glossary, and the vocabulary this codebase is written
  in. Each term carries an _Avoid_ list of the near-synonyms it exists to
  displace. Use its words in code, comments, commits and tickets; if a term you
  need is missing, that is worth raising rather than inventing a synonym for.
- **`docs/adr/`** — 17 numbered decision records, in three states. Most are
  live. **0004, 0008, 0011, 0013 are partly retired**: the body stands unedited
  and a banner at the top names what is no longer true, so *read the banner
  first* — the body alone will mislead you. **0007, 0009, 0014 are tombstones**:
  wholly superseded, collapsed to what was decided and where the live decision
  went. Numbers are permanent references and are never reused; a collapsed
  record's full body is in `git log -p` on its file.

  The one to know: review was deleted, so there is no approval gate, no
  promotion, and no `proposed` status anywhere. A ticket resolves when its
  session writes `## Answer`.
- **`docs/design-system.md`** — required before any UI change (see below).

Code comments carry the *why* and are load-bearing; several exist to stop a
plausible-looking simplification of a security control. Don't strip them.

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
