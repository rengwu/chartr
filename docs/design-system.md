# Cockpit design system

The cockpit chrome is built on **shadcn-svelte + Tailwind v4**, driven by an
**olive / warm-neutral** theme. Everything visual comes from two places: semantic
**tokens** (CSS custom properties) and vendored **shadcn-svelte primitives**. New
UI reaches for a token and a primitive; it never hand-rolls a `.btn`, drops a raw
hex, or re-introduces amber into the chrome.

This doc is the spec. `docs/adr/0012-adopt-shadcn-svelte-design-system.md` records
*why*; the root `CLAUDE.md` states the hard rules as imperatives. Token *values*
live in `web/src/app.css`.

## Tokens

The palette is the shadcn *Mira* style on base colour *Olive* (radius *Small* =
`0.45rem`). Both light (`:root`) and dark (`.dark`) sets ship in
`web/src/app.css`; **the app boots dark** (`index.html` sets `<html class="dark">`).
Every colour sits at hue ~107 — a warm neutral — **except `--destructive`, which
is the only chromatic token.** That is the core rule: *the chrome is monochrome;
`--destructive` (red) is the only chroma.*

Tokens are declared as oklch custom properties, then mapped onto Tailwind's
`--color-*` / `--font-*` / `--radius-*` namespaces via `@theme inline` so
utilities resolve (`bg-background`, `text-muted-foreground`, `rounded-md`,
`font-sans`, …). The `inline` keeps the mapping pointing at the runtime
properties, so `.dark` overrides flow through.

| Token | Utility | What it is *for* |
| --- | --- | --- |
| `--background` / `--foreground` | `bg-background` / `text-foreground` | The page surface and its default text. |
| `--card` / `--card-foreground` | `bg-card` / `text-card-foreground` | A raised surface sitting on the background (panes, the map card, the chrome bars). |
| `--popover` / `--popover-foreground` | `bg-popover` / … | Floating surfaces — dialogs, sheets, menus. |
| `--primary` / `--primary-foreground` | `bg-primary` / … | The one emphasis token. Active / pinned / selected / "on" states, the primary action. **This is where the old amber accent went.** |
| `--secondary` / `--secondary-foreground` | `bg-secondary` / … | Lower-emphasis fills — secondary buttons, quiet tags. |
| `--muted` / `--muted-foreground` | `bg-muted` / `text-muted-foreground` | Recessed fills and de-emphasised text (captions, dim labels). |
| `--accent` / `--accent-foreground` | `bg-accent` / … | Hover/active surface tint for interactive rows. (Neutral here — *not* an amber accent.) |
| `--destructive` | `text-destructive`, `bg-destructive/10` | **The only chroma.** Forget / delete / close, error states, the agent-missing badge. Nothing "just to add colour." |
| `--border` / `--input` | `border-border`, `border-input` | Hairlines and field borders. |
| `--ring` | `ring-ring` | Focus rings and the "on" outline. Neutral emphasis alongside `--primary`. |
| `--chart-1`…`--chart-5` | `bg-chart-1` … | The neutral chart ramp (unused by the chrome today; reserved). |
| `--sidebar*` | `bg-sidebar`, … | The sidebar's own surface/border/accent set. |
| `--radius` | `rounded-sm/md/lg/xl` | `0.45rem` base; the `--radius-*` scale derives from it. |

**Never write a raw colour.** If a surface needs a colour that is not a token,
that is a signal the palette is missing a role — flag it, don't inline a hex.

### The one bespoke chrome primitive

`app.css` keeps a deliberately tiny hand-written layer, all token-driven:

- **`.cockpit-bar`** — the single chrome-bar primitive. The space header, the
  terminal tab strip, and the map-card header all share one tier height
  (`--bar-h`, `2.5rem`) and this class, so they align on one line across the
  stage. Height is fixed and never grown by contents.
- **`.prose-sm`** — a small token-driven prose scale for rendered markdown
  (ticket bodies, blocker answers, payload segments). A lightweight stand-in for
  a typography plugin; shared by `DetailPane` and `PayloadPreview`, styled
  entirely off tokens so it tracks the theme.
- A short **base seam** (`@layer base`): default border colour follows `--border`
  (Tailwind v4 Preflight otherwise defaults borders to `currentColor`), the app
  fills the viewport, the document surface/type come from tokens, and the two
  island wrappers (`.terminal-island`, xterm scrollbar chrome) are sized/tuned
  at the seam — see the island rule below.

Do not grow this layer with per-surface CSS. New surfaces compose primitives +
utilities; a genuinely new shared pattern earns *one* new `@layer components`
class, token-driven, not a pile of one-offs.

## Type & radius

- **IBM Plex Sans** — chrome UI (`font-sans`).
- **IBM Plex Mono** — paths, code, eyebrows/labels (`font-mono`).
- Both are **self-hosted woff2 subsets** in `web/src/assets/fonts/`, declared via
  `@font-face` in `app.css` and bundled into the `go:embed` dist. **No CDN, no
  Google Fonts, no runtime fetch** — the frontend ships inside a single offline
  binary (ADR 0010). Any new weight ships as another bundled subset.
- Radius base is `0.45rem`; use the `rounded-sm/md/lg/xl` scale, never a literal
  pixel radius.

## Primitives

shadcn-svelte components are vendored into `web/src/lib/components/ui/` — real
source in the repo, ours to edit, not an opaque dependency. Currently vendored:

`accordion` · `button` · `badge` · `card` · `checkbox` · `dialog` · `input` ·
`label` · `scroll-area` · `sheet` · `tabs`

Config lives in `web/components.json` (style *Mira*, base colour *Olive*,
`$lib` aliases). The class-merge helper is `cn()` in `web/src/lib/utils.ts`
(clsx + tailwind-merge), plus the `WithElementRef` / `WithoutChild*` prop-shape
helpers the vendored primitives are typed against.

**Button** variants: `default` (primary action) · `secondary` · `outline` ·
`ghost` · `destructive` · `link`. Sizes: `default`, `xs`, `sm`, `lg`, and the
`icon` / `icon-xs` / `icon-sm` / `icon-lg` square sizes.

**Badge** variants: `default` · `secondary` · `outline` · `ghost` ·
`destructive` · `link`. The binding-layer tags use the built-in→`outline` /
workspace→`secondary` / user→`default` scale.

### Overriding a vendored class

Passing `class` to a primitive merges through `cn()` (tailwind-merge), which
resolves same-group conflicts by *last one wins* — `px-3` beats the primitive's
`px-(--card-spacing)`. Two kinds of base class slip past that, and both have
already bitten `DetailPane`:

- **A different utility group.** `cn()` only de-duplicates within a group, so a
  base class the override doesn't conflict with simply survives. `Card.Header`
  ships `items-start`; overriding its `grid` with `flex flex-col` left that
  alignment in place, and in a *column* it shrank every row to its content —
  no edge for a title to ellipsis against, and a `flex-1` spacer collapsed to
  nothing. Set the axis property you actually want (`items-stretch`).
- **A two-class selector.** Variants like `[.border-b]:pb-(--card-spacing)`
  compile to `.\[\.border-b\]\:pb-\(--card-spacing\).border-b` — specificity
  (0,2,0), which outranks any plain `.py-*` (0,1,0). Adding `border-b` to a
  `Card.Header` therefore silently reinstates the card's full bottom padding.

**Retune the variable, don't escalate the selector.** These rules read
`--card-spacing`, so `[--card-spacing:--spacing(2)]` on the element fixes the
padding at its source; reach for `!` or a bespoke CSS override and the next
person inherits a fight. When an override looks ignored, read the primitive's
own class string first — the answer is usually in it.

### Adding a primitive

Use the CLI against the vendored config:

```
cd web && npx shadcn-svelte@latest add <component>
```

Then bring it in line with the house rules before committing:

- **Swap lucide → Phosphor.** The CLI vendors lucide icons (e.g. the dialog/sheet
  close `X`); replace them with the Phosphor equivalent.
- **Prune unused deps** the CLI pulls in (e.g. `@internationalized/date` was
  removed when unused).
- Keep `bits-ui` under runtime deps; add any new prop-shape helpers to
  `utils.ts`.
- Re-check for raw colours or amber the generated component might carry, and that
  it resolves tokens (not hard-coded palette values).

## Icons

Icons are **Phosphor** via `phosphor-svelte`, replacing every emoji/unicode glyph
in the chrome. Import the named icon and render it as a component:

```svelte
import { Plus, PushPin, X, Warning, Check } from 'phosphor-svelte'
```

`phosphor-svelte` is tree-shaken, so import only what a surface uses. Size via the
primitive's icon slots (Button/Badge scale their `svg`) or a `size-*` utility;
colour follows `currentColor` from the token'd text colour — don't paint an icon
a raw hex.

## The islands (data-viz colour + the seam)

Two surfaces are **imperative islands** the chrome hosts but never reaches inside:
the **xterm.js terminal** and the **canvas star-map** (ADR 0010). Their renderers
own their own pixels. The chrome must **never reach into an island's renderer to
re-theme it**; any island re-theming happens *at the seam*, feeding the renderer
resolved colours — never inside it.

The seam is `web/src/lib/tokens.ts`: `readToken` / `resolveColor` / `readColor` /
`readTokens` read the live CSS custom properties off the document and resolve
oklch tokens to concrete `rgb(...)` strings the canvas/xterm code needs. The
island wrappers call it and pass colours in:

- **Terminal** (`web/src/lib/Terminal.svelte`) builds xterm's `ITheme` in its
  wrapper from `--background`/`--foreground` (it sits directly on the page, no
  Card), cursor from `--ring`, selection from `--muted`, dim from
  `--muted-foreground`, red from `--destructive`. The six remaining ANSI colours
  (green/yellow/blue/magenta/cyan/…) have **no chrome token to draw from** — the
  theme is monochrome plus `--destructive` — so those slots are literal muted hex
  tuned to sit quietly on the surface, contrast-checked. This is the one place
  literal colour is legitimate, and only because the tokens genuinely don't cover
  ANSI.
- **Star-map** (`web/src/lib/StarMap.svelte` → `starmap.ts`'s `setBackground()`
  seam method) feeds the token-resolved `--card` colour in as the canvas
  background.

### Star-map data-viz exemption

The star-map's palette lives in `web/src/lib/starmap/theme.ts` (`STAR` and
`LABEL` maps). Its **five status hues** — `resolved` / `frontier` / `claimed` /
`blocked` / `out_of_scope` — are **categorical data-viz colour (meaning), not
brand decoration**, and are therefore *exempt* from the monochrome rule. They are kept (re-tuned to sit legibly on the warm near-black `--card`), not
folded into the neutral chrome. **Amber survives only as the `claimed` star and
nowhere else in the product.** When the card colour changes, re-check each status
hue for contrast against it (`out_of_scope`, the dimmest, is the first to fail)
and re-tune in `theme.ts` — a palette re-tune, never a renderer change.

The same exemption covers `SESSION_HUE` in that file: the session overlay's
amber moon and the grey of a dead session, plus the gold of the island's own
ticker line (spec, stories 25–27). The set is **closed** — the moons grammar
carries every state on motion or shape as well as colour
(`starmap/session.ts`, `GRAMMAR`), so a new state earns a new *motion*, not a
new hue. None of this leaks past the island
seam: the chrome around the canvas stays monochrome.

## Do / Don't

**Do**

- Reach for a **token** for every colour, and a **primitive** for every button,
  badge, input, dialog, sheet, tab, card.
- Put emphasis on `--primary` / `--ring`; reserve `--destructive` for real
  destructive/error states.
- Use Phosphor for every icon; IBM Plex Sans/Mono for every glyph of text.
- Compose utilities + primitives; if you truly need a new shared pattern, add one
  token-driven `@layer components` class.
- Re-theme an island **at the seam** (`tokens.ts` → the wrapper), feeding the
  renderer resolved colours.

**Don't**

- Hand-roll a `.btn` / `.badge` / `.card` or any bespoke chrome CSS when a
  primitive exists.
- Write a **raw hex / rgb / named colour** in the chrome — if no token fits, flag
  the missing role instead of inlining one.
- Re-introduce **amber** (or any chroma) into the chrome. The only chroma is
  `--destructive`; the only amber is the star-map's `claimed` star.
- Reach **inside an island's renderer** to re-theme it (ADR 0010) — go through the
  seam.
- Add fonts or icons over the network — everything is self-hosted and bundled.
