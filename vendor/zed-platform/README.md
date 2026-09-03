# Zed platform patches

These crates are vendored from Zed revision
`1ea16c1ab9dd6d36649e002dc60995634da04daf`.

Chartr patches the narrow dependency seam needed for native child-surface
menus:

- `gpui_macos` implements `WindowKind::AnchoredPopup` as a borderless AppKit
  child panel, including parent-relative placement and focus-loss dismissal.
- `gpui_linux` implements the same contract for the X11 backend. Zed's
  Wayland backend already supports anchored popups.
- `ui` lets a content-sized popup override `ContextMenu`'s normal 75%-of-window
  maximum height. The default behavior is unchanged.

The root Cargo patch table selects these crates, so builds do not depend on a
modified Cargo checkout.

The original Apache 2.0 and GPLv3 license texts are retained beside the
vendored crates.
