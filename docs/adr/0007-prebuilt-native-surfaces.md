# 0007 — Prebuilt native surfaces

A separately installed native plugin uses `kind = "embedded"` and the versioned
C-compatible interface in `chartr-native-plugin`. It owns a native child surface;
Chartr owns its pane, layout, focus and optional row of buttons/text fields.
No GPUI object, Rust trait object, allocation ownership or runtime global crosses
the boundary. This replaces the code-free hosted tier. Build-time GPUI modules
and portable web packages retain their existing contracts.

**All plugin installation is prebuilt-only.** A local folder or repository must
contain ready-to-run assets or declare a downloadable platform package. Missing
native binaries are an installation error. There are no build hooks, dependency
installation, compiler invocations or source-compilation fallback paths.

Libraries load lazily when a pane opens, on the main thread. Event delivery wakes
the existing host executor; there is no extra UI process, renderer, or idle timer.
Native libraries execute with user authority and are explicitly described as
such before installation. They are not a sandbox or a crash-isolation boundary.

The interface exposes create, destroy, resize, visibility, focus, messages, and
shutdown. Plugins synchronize callbacks before destroy returns. Chartr closes
surfaces before their native parent and calls shutdown after its event loop.
Libraries remain mapped until process exit to keep native callbacks safe.

Plain JSON describes the small toolbar and state events. This deliberately does
not expose arbitrary GPUI layout or services. The toolbar reuses existing host
controls, avoiding another UI runtime and retaining native text composition.

External engines, helpers, icons, source, security updates and release packaging
belong in their plugin repositories. Chartr releases contain none of that code
or payload. System packages may install prebuilt plugins in the system plugin
root; a user-installed version takes precedence.
