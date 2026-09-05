# Zed platform patches

These crates are vendored from Zed revision
`1ea16c1ab9dd6d36649e002dc60995634da04daf`.

Chartr patches the narrow dependency seam needed for native child-surface
menus:

- `gpui_macos` implements `WindowKind::AnchoredPopup` as a borderless AppKit
  child panel, including parent-relative placement and focus-loss dismissal.
  Active-window lookup uses AppKit's key window (including panels), so focus
  checks reflect the window receiving input rather than the main workspace.
- `gpui_linux` implements the same contract for the X11 backend. Zed's
  Wayland backend already supports anchored popups.
- `ui` lets a content-sized popup override `ContextMenu`'s normal 75%-of-window
  maximum height and hosts button tooltips in passive native child windows so
  they can appear above webviews. Tooltip creation is cancelled on dismissal
  and checked against the active parent window; existing tooltips close when
  their parent loses focus, preventing them from raising a background workspace.

The root Cargo patch table selects these crates, so builds do not depend on a
modified Cargo checkout.

The original Apache 2.0 and GPLv3 license texts are retained beside the
vendored crates.
