# chartr macOS app icon, v4

v4 is **not** a new web icon set. It is the macOS app-icon masters, which exist
because the app bundle needs a shape and a per-band drawing the web icons must
not have. The web set is still v2 (`../v2`), unchanged — favicon, PWA icons,
`apple-touch-icon` and the header brandmark all still come from there.

## Why this exists

**macOS does not mask app icons.** Whatever silhouette the `.icns` carries is
exactly what the Dock, Finder and the ⌘-Tab switcher draw — unlike iOS, or a web
manifest icon, where the platform clips the artwork for you. The bundle used to
downscale the square `icon-512.png` the PWA uses, which drew a tile that was
visibly larger than every neighbour, cornered too tightly, and ringed by the
plate's own light stroke — the "white border" on the built app.

So the mac masters bake Apple's grid into the pixels, and the web icons keep the
full bleed that a masked context wants. See ADR 0016.

## Masters

Sources are `../references/v4/` — four Figma exports, one per size band, drawn
without the plate stroke. v4 is **redrawn per band**, not one drawing scaled: at
16 and 20 the cursor is bigger and bleeds off the plate, at 32 and 48 it is
contained and the star sits smaller in the frame.

| Master | Band it was drawn for | Feeds |
| --- | --- | --- |
| `logo-app-16.svg` | 16 | `icon_16x16` |
| `logo-app-32.svg` | 32 | `icon_16x16@2x`, `icon_32x32` |
| `logo-app.svg` | 48 and up — the full-detail drawing | every entry from 64 up |

`../references/v4/20x20-unbordered.svg` has no `.icns` entry at its size and is
not cleaned here; the 20px band belongs to the web brandmark, which is still v2.

The cuts to get from each export to its master are the ones v2's README lists:
the `data-figma-*` metadata blobs, the two paths that carried only that metadata
and rendered nothing (the root `fill="none"` is inherited, and the sibling
`data-figma-skip-parse` groups do the actual drawing), the duplicate star
gradient, and the art mask's gradient fill and stroke — every stop opaque, so it
was only ever a square crop.

One v4-specific cut: **the plate's `rx` is dropped.** The artwork must not carry a
second, competing corner shape, because the generator supplies Apple's. Each
master is verified pixel-identical to its export-with-`rx`-stripped at three
sizes.

Each master renders **square and full-bleed**. Do not use one directly as an app
icon — that is the bug this directory fixes.

## The grid

`scripts/mac-app-icon.py` renders each master natively at the art size (never
upscaled), insets it, clips it to a continuous-corner ("squircle") rounded square,
and composites the template's shadow beneath.

| Parameter | Value (in 1024 units) |
| --- | --- |
| Art box | 824, centred (100 inset) |
| Corner radius | 185.4 (`0.225 × 824`) |
| Corner smoothing | 0.7 |
| Shadow | black, 26%, blur 16, `dy` 6 |

**These are measured, not chosen.** The corner construction was fitted to the
alpha channels of six shipping system icons — Calculator, Mail, Music, Notes,
Preview, Reminders, all of which share one template silhouette. The fit lands on
Apple's published template values (824/1024 art, `r = 0.225 × S`) at 0.19px RMS
at 256px, and corner smoothing minimises at 0.7 unanimously across all six. The
generated 1024 master reproduces their opaque bounding box **exactly** (204×204 at
`[26,229]` when downscaled to 256) and their shadow ramp to within 0.02 alpha on
all four edges.

The same grid holds at every band. Apple's own 16px entry measures 12.25/16 =
0.766 against the grid's 0.805 — a sub-pixel optical tightening, not a different
grid, and not worth chasing.

The shadow fit is shallow and degenerate — opacity and blur trade off — so those
three numbers are a near-minimum rather than a unique answer. Don't chase them.

Because 824/1024 of a small canvas is fractional (12.875px at 16), each band is
composited on the smallest whole-number supersample where both the art box and the
inset land on integers, then boxed down by that whole factor. Rasterising art at a
fractional size, or pasting it at a fractional offset, is its own kind of mush.

## Regenerating

```sh
brew install librsvg          # rsvg-convert; Pillow and numpy via pip
python3 scripts/mac-app-icon.py
```

That writes all three PNGs to both `web/public/` and here. The output is
**committed**, and the build does not run this script: `make bundle` only
downscales with `sips`, so the release runner needs neither librsvg nor Python.
Re-run it by hand whenever a master changes.

## Where they ship

`web/public/icon-mac-{16,32,1024}.png` — Vite copies them to the dist root the Go
binary embeds, so the same bytes serve both mac surfaces and they cannot drift:

- **`make bundle`** maps each `.icns` entry to the master drawn for its band
  (`MACAPP_MARK*` in the Makefile). Every canvas is the largest entry its band
  feeds, so the iconset loop only ever downscales.
- **The loose shell's Dock tile** reads the 1024 at runtime via
  `setApplicationIconImage:` (`cmd/webview/appicon.go`), which does not mask
  either.

## Known gaps

**The loose shell gets one drawing.** It hands AppKit a single 1024 PNG, so its
Dock tile at small sizes is AppKit's downscale of the full-detail drawing rather
than the 16 and 32 masters. Only the bundle gets true per-band art. Fixing it
means building a multi-representation `NSImage` from all three PNGs; the bundle is
the artifact operators actually install, so this has not been worth the cgo.

**Apple's corner is rounder than the artwork's was.** The masters were drawn
against a plate at `rx/side = 0.1875`; Apple's grid is `0.225`. Where art bleeds
into a corner — the cursor at the 16 band — the mask therefore trims about half a
pixel more than the drawing intended. Apple's shape wins, because it is the shape
the platform draws and the alternative is a tile that reads as the wrong
silhouette. If that half-pixel ever matters, the fix is to pull the cursor's
bleed in at the 16 band, not to loosen the mask.
