# Workspace status bar

The bottom bar shows persistent plugin services. Mobile Companion is excluded
from the current app build, so its sharing status and mobile session count are
absent.

Click a service to open its settings. Right-click the bar and choose Hide Status Bar; restore it from Settings → General → Show status bar or the “Workspace: Toggle status bar” command. Visibility is saved globally as `[general] show_status_bar` and hiding it does not stop services.

Bundled native plugins can contribute a status with `Plugin::background_status`. The host refreshes these once per second and redraws only when their reported values change. See [plugin documentation](plugins.md#background-status).

## Verification

Check hiding and restoration from Settings and the command palette, saved
visibility after restart, and opening a contributed service's settings. A plugin
without `background_status` contributes no button. Historical Companion counts
and connection tests do not establish current desktop behavior because Companion
is excluded. See the [release checklist](acceptance.md).
