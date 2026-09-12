# Workspace status bar

The bottom bar shows terminal service health, the number of running terminal sessions across open spaces, and persistent plugin services. Companion reports whether sharing is off, active, connected to clients, or needs attention. A separate count shows sessions currently viewed on mobile.

Click a service to open its controls. Companion opens directly in Settings, so its pane does not need to stay open. Close the bar with its × button; restore it from Settings → General → Show status bar or the “Workspace: Toggle status bar” command. Visibility is saved globally as `[general] show_status_bar` and hiding it does not stop services.

Bundled native plugins can contribute a status with `Plugin::background_status`. The host refreshes these once per second and redraws only when their reported values change. See [plugin documentation](plugins.md#background-status).

## Verification — 2026-09-08

- Normal desktop build and `cargo check -p chartr` succeeded.
- `cargo test -p chartr -p chartr-plugin -p chartr-plugin-host -p chartr-companion`: 267 tests passed.
- Coverage verifies saved visibility across settings reloads, application-global settings updates, background sharing status without constructing a pane, and connection counts after disconnect.
- An isolated native window verified the bottom-bar layout, hiding, command-palette restoration, the General settings switch, and opening Companion controls directly from the bar.
- A real TLS client connected to the isolated host on loopback port 19848, created a test terminal, and viewed it at mobile dimensions. The bar showed one connection, one running session, and one session on mobile with Companion controls closed.

Changes are source edits; restart the rebuilt desktop app to use the new bar.
