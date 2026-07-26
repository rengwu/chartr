# chartr icon set, v2

Sources are `../references/v2/` — four Figma exports, one per size band. v2 is
**redrawn per band**, not one drawing scaled: at 16 and 20 the cursor is bigger
and bleeds off the plate's corner against a square art mask; at 32 and 48 it is
contained inside a rounded one, and the star sits smaller in the frame. That is
why v1 could not be re-interpolated to 20px and this set can.

## Masters

The `logo*.svg` files are those exports with the export cruft cut — the
`data-figma-*` metadata blobs, two paths that carried only that metadata and
rendered nothing (the root `fill="none"` is inherited), a duplicate star
gradient, and a mask gradient whose stops were both opaque so it never masked
anything. Each renders pixel-identical to its export at 512px.

| Master | Band it was drawn for |
| --- | --- |
| `logo-16.svg` | 16 |
| `logo-20.svg` | 20 |
| `logo-32.svg` | 32 |
| `logo.svg` | 48 and up — the full-detail drawing |

## Cut set

Every PNG is rasterized natively from the master drawn for its band. Nothing is
downsampled from one big render.

- `icon-16` / `icon-20` / `icon-32` — from their own masters
- `icon-48` … `icon-1024` — from `logo.svg`
- `favicon.ico` — native 16/32/48 frames, each from its own master, stored
  PNG-compressed

## Where they ship

`web/public/` carries the copies Vite puts at the dist root the Go binary embeds
(ADR 0010), so they go out offline with everything else: `favicon.svg`
(`logo-32`), `favicon.ico`, `apple-touch-icon.png` (`icon-180`), `icon-192`,
`icon-512` for the manifest and the macOS Dock tile, and `brandmark.svg`
(`logo-20`) for the header mark beside the wordmark.

`icon-180` still has transparent corners; making it opaque and full-bleed the way
iOS wants would mean redrawing the plate.

v1 is kept in `../v1/` for comparison.
