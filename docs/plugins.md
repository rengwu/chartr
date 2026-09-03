# Plugin packages

Chartr installs packages; it does not build projects. Installation has no
scripts, hooks, package manager, compiler, or Rust-toolchain dependency.

Every package is a directory with `zeddy-plugin.toml` at its root. A web
package includes the declared HTML entry and its assets.

A hosted package contains only declarative files and names a surface already
implemented by Chartr. Hosted surfaces are intended for first-party plugins
that need deep operating-system integration without crossing the GPUI dynamic
library boundary. Browser is the first one: `kind = "hosted"` and
`surface = "browser"`.

## Installation sources

- **Local folder:** Chartr copies the selected package into private staging
  before showing the confirmation.
- **Git repository:** Chartr shallow-clones the default branch. No release API,
  package manager, or build system is involved.

After the user confirms the package details and declared permissions, Chartr
validates the staged contents again and atomically replaces
`$XDG_DATA_HOME/chartr-zeddy/plugins/<plugin id>/`. Persistent plugin data lives
outside that directory, so reinstalling or upgrading does not remove it. The
new code is not loaded until the user chooses **Restart**; **Later** keeps the
current process running and shows a reminder.

Hosted and web packages are architecture-independent and need no release
binary. Installing Chartr Browser is therefore only a shallow clone or folder
copy, manifest validation, confirmation, and atomic rename. Browser uses
ephemeral web-engine storage and persists only each pane's last URL.

Chartr may link native plugin modules at application build time, but it rejects
separately installed GPUI dynamic libraries. Precompiling does not make Rust
GUI objects or crate-global state ABI-safe across two independently linked
copies of GPUI. Plugins that need native operating-system integration should
use a reviewed hosted surface; portable third-party plugins should use the web
tier.
