# Zed platform patches

These crates are vendored from Zed revision
`1ea16c1ab9dd6d36649e002dc60995634da04daf`.

chartr patches the narrow dependency seam needed for native child-surface
menus:

- `gpui_macos` implements `WindowKind::AnchoredPopup` as a borderless AppKit
  child panel, including parent-relative placement and focus-loss dismissal.
  Active-window lookup uses AppKit's key window (including panels), so focus
  checks reflect the window receiving input rather than the main workspace.
  Unfocused anchored popups without a grab are passive panels: they cannot
  become key or main and ignore mouse events, including in transparent margins.
- `gpui_linux` implements the same contract for the X11 backend. Zed's
  Wayland backend already supports anchored popups.
- `ui` lets a content-sized popup override `ContextMenu`'s normal 75%-of-window
  maximum height and hosts button tooltips in passive native child windows so
  they can appear above webviews. Tooltip creation is cancelled on dismissal
  and checked against the active parent window; existing tooltips close when
  their parent loses focus, preventing them from raising a background workspace.
  Scrollbars also support container-hover visibility that retains input handling
  during thumb drags, plus surface-specific thumb colors for sidebar contrast.

The root Cargo patch table selects these crates, so builds do not depend on a
modified Cargo checkout.

On a macOS desktop, `cargo run --manifest-path
vendor/zed-platform/gpui_macos/Cargo.toml --example passive_popup` checks native
mouse and focus eligibility for passive tooltips, interactive popups, and normal
windows. This covers AppKit behavior that GPUI's simulated window tests cannot.

The original Apache 2.0 and GPLv3 license texts are retained beside the
vendored crates.
