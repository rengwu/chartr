---
type: task
blocked_by: [01]
---

# Reskin the shell and panes — sidebar, stage, tabs, map card

## Question

Rebuild the shell — the surfaces that own the cockpit layout — onto tokens and primitives, retiring the rest of the bespoke `app.css`. In scope: `App.svelte` (the `.cockpit` grid, `.sidebar` with its space/map rows and filter, the `.statusbar`); `SpacePane.svelte` (the `.space-header` identity bar, the `.space-panes` row, the `.map-toggle`, the `.term-*` tab strip and bindings drawer → shadcn **Tabs** + **Sheet**); `MapCard.svelte` (the `.map-card*` chrome, float/dock, the left-border resize grip); `StarMap.svelte` (the island wrapper's card frame only — **not** the canvas). The sidebar adopts the theme's `--sidebar-*` tokens.

This is where de-ambering lands across the shell: `.map-toggle[aria-pressed]`, `.term-config`, `.map-card-btn`, `.space-row.active`, the pinned-pin highlight, the amber hover borders — all move to `--primary` / `--ring` / `--accent` (the subtle neutral), keeping only `--destructive` for forget/close-danger affordances. Emoji glyphs become Phosphor icons: 📌 pin, ＋ add, × close/forget, ▲ agent-missing badge, ⚠ malformation, ✓ finished, the map-toggle glyph. Apply distill: the space-row's pin/forget/classify controls fold into the shared icon-Button with hover-reveal preserved; the duplicated bar styles (`.term-bar`, `.map-card-bar`, `.space-header` all share `--bar-h`) collapse into one bar primitive/util.

Behaviour is fixed (map "Out of scope"): the `--bar-h` one-tier alignment of the three chrome bars, the float/dock split with the terminal's frozen width and no reflow on window resize, the drawer summon, the `M`/`Esc` bindings, and the hover-reveal row affordances all survive unchanged. The xterm and star-map canvases are untouched here — only their surrounding chrome.

Done when: the shell, sidebar, space header, terminal tab strip + bindings drawer, and map-card chrome all run on tokens + primitives + Phosphor icons with the remaining bespoke `app.css` chrome deleted; no amber survives in the chrome; the three bars still align on one tier; float/dock, resize-without-reflow, the drawer, and the keyboard bindings all behave as before; `svelte-check`, the Vite build, `vitest`, `go vet`, and `go test` pass.

## Answer

The whole shell — `App.svelte` (cockpit grid, sidebar, status bar), `SpacePane.svelte` (space header, tab strip, bindings drawer), `MapCard.svelte` (card chrome, float/dock, resize grip), `StarMap.svelte` (island wrapper) — now runs on Tailwind token utilities + shadcn-svelte primitives + Phosphor icons. No new scoped `<style>` blocks; the only added CSS is shared, in `app.css`.

**Two primitives vendored** — `tabs` and `sheet` — via the CLI (`npx shadcn-svelte add tabs sheet`, the same manual-vendor path as tickets 01/02). They pull only `bits-ui` + `tailwind-variants`, both already present, so `package.json`/`package-lock.json` were reverted to committed after the add (the CLI's re-added `@lucide/svelte` + `@internationalized/date` are the two ticket 02 already pruned). `sheet-content.svelte`'s one `@lucide/svelte` `X` was swapped for `phosphor-svelte`'s, keeping "icons are Phosphor" true inside vendored primitives too (ticket 02's precedent).

**The one bar primitive.** `.term-bar` / `.map-card-bar` / `.space-header` collapse into a single `.cockpit-bar` component class in `app.css`, driven by a new `--bar-h` token (2.5rem). All three stage bars — plus the sidebar head, which now aligns on the same tier — use it, so the one-tier alignment is guaranteed by one rule rather than three copies (distill).

**De-ambering.** Every "on"/active/pinned state moved to the neutral tokens: sidebar active row → `bg-sidebar-accent`; pinned pin, frontier emphasis → `--primary`; map-toggle / notes / dock pressed → the `secondary` Button variant; the resize grip hover → `--ring`. `--destructive` is kept only for forget, tab-close hover, and the "agent not found" badge. `grep` for `amber`/`d9a441` in the built CSS is empty.

**Emoji → Phosphor**, everywhere in the chrome: 📌 → `PushPin` (filled when pinned), ＋ → `Plus`, × → `X`, ▲ agent-missing → `Warning` (in an outline `Badge`), ⚠ malformation → `WarningDiamond`, ✓ finished → `Check`, the ✦ map-toggle glyph → `Sparkle` (filled when shown); the bindings ● / ▲ status became `CheckCircle` / a `destructive` `Badge`, and the bindings button took a `SlidersHorizontal`.

**Distill.** The hand-rolled `.icon-btn` / `.kind-btn` / `.map-card-btn` / `.term-*` / `.field-src` variants all fold into the shared `Button` + `Badge` primitives with named variants — the space-row pin/forget are `icon-xs` ghost Buttons with hover-reveal preserved (`group-hover/row`), the classify guess is a `default` vs `outline` Button pair, and the binding-field layer tags reuse the same built-in→`outline` / workspace→`secondary` / user→`default` badge scale ticket 08 set for the payload preview (a `layerVariant` map). A dead write-only `termColEl` binding was dropped.

**Tab strip → Tabs, drawer → Sheet.** The strip is a `Tabs.Root` whose `value` tracks the effective active shell (`onValueChange` writes `activeId`); each shell is a `Tabs.Trigger` with the close `X` as an adjacent Button inside a hover-group wrapper (a close button can't nest inside a trigger), and the active look is driven off the existing `active` derived on the wrapper rather than the primitive's data-state, so the single-keyed-island rendering below is untouched. The bindings drawer is a right `Sheet` summoned from the bar (`Sheet.Trigger` → the bindings Button via the child snippet), scrolled with `ScrollArea`.

**Behaviour preserved.** Only templates/classes changed in the three logic-bearing files — the resize handlers, dock effects, deep-link/`replaceState` effects, and `M`/`Esc` bindings are verbatim. **One additive guard:** the `svelte:window` Escape handler now also treats focus inside a `[role="dialog"]` as "editing", so closing the bindings Sheet with Esc no longer also dismisses the star-map (the old in-pane drawer never consumed Esc).

**Flags for the operator.**
1. *Float/dock, `.dp-holder`, and island-wrapper positioning were rebuilt, not preserved.* Ticket 01's operator call ripped out the bespoke `app.css` that implemented them, so there was no chrome block left to "delete" (Done-when) — this ticket re-establishes the terminal-priority split (docked = flex with the terminal's inline frozen basis; floating = absolute, right-pinned at 10px, `max-w-[calc(100%-40px)]`), the detail-pane holder (full-height right / half-height bottom), the resize grip, and the terminal-island fill, on tokens. This is the build-up the map's ticket-01 note recast 02–04 to; flagging because it's more than a re-paint.
2. *The bindings Sheet dims the whole app and closes on Esc/backdrop* — the old drawer was an in-pane aside with only a manual close. That is the intended "→ Sheet" reskin, but it is a behavioural nuance beyond pure skin; flagging in case a non-modal in-pane drawer was wanted.
3. *Star-map island wrapper sizing* lives in `StarMap.svelte` (its own in-scope surface, `h-full w-full`); the terminal island's wrapper — `Terminal.svelte` being out of scope — is sized by one `.terminal-island` rule in `app.css`. Both are seam-only (size the wrapper, never the canvas — ADR 0010).

**Checks.** `npm run check` — 0 errors / 0 warnings; `npm run build` — clean; `npm test` — 33/33 (unchanged suite; nothing here adds logic beyond markup/props); `go vet ./...` and `go test ./...` — clean; the built binary boots and serves the freshly embedded dist (HTTP 200, control socket up).