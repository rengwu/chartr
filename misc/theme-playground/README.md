# Chartr Theme Playground

A local theme builder for Chartr's native GPUI app. It mirrors the palette model in
`crates/zeddy/src/settings.rs`, including Chartr's four sidebar-only colors.

## Run it

```sh
npm install
npm run dev
```

Use `npm run build` for a production build.

## Theme workflow

1. Pick one of the 15 themes currently registered by Chartr.
2. Edit a `0xrrggbb` field directly, or click its swatch for HSV and RGB controls.
3. Check the Workspace, Settings, and UI states previews.
4. Use the download button to save a portable JSON draft. The import button restores it later.
5. Choose **Export to Rust** and copy or download the generated Rust entries.
6. Add the `ThemePalette::new(...)` entry to `THEME_PALETTES` and the
   `SidebarThemePalette::new(...)` entry to `SIDEBAR_THEME_PALETTES` in
   `crates/zeddy/src/settings.rs`. Increment both fixed array lengths.

`init_themes` already registers both arrays, so the new theme appears in Chartr on the next build.

Draft edits are also saved automatically in browser local storage.

## Token coverage

The playground exposes the 16 fields in `ThemePalette` and the four fields in
`SidebarThemePalette`: card inactive, card active, session hover, and session active.

The preset values are intentionally kept in `src/lib/themes.ts` in the same order as the
Rust catalog, followed by Chartr Dark and Chartr Light.
