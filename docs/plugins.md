# Plugin packages

Chartr installs packages; it does not build projects. Installation has no
scripts, hooks, package manager, compiler, or Rust-toolchain dependency.

Every package is a directory with `zeddy-plugin.toml` at its root. A web
package includes the declared HTML entry and its assets. Entries may live in
subdirectories; asset URLs resolve within the package root, including filenames
that require URL encoding.

Every manifest also declares one free Hugeicons Stroke Rounded export by its
canonical name, for example `icon = "Clock01Icon"`. The matching SVG must be
packaged at `icons/Clock01Icon.svg`. Chartr validates that file before install
or load and uses it in sidebar, outer, and pane-local tabs.

A hosted package contains only declarative files and names a surface already
implemented by Chartr. Hosted surfaces are intended for first-party plugins
that need deep operating-system integration without crossing the GPUI dynamic
library boundary. Browser is the first one: `kind = "hosted"` and
`surface = "browser"`.

## Installation sources

- **Local folder:** Chartr copies the selected package into private staging
  before showing the confirmation.
- **Git repository:** Chartr shallow-clones the default branch. No release API,
  package manager, or build system is involved. Cloning has a two-minute deadline
  and disables interactive terminal credential prompts. Cancel in Settings, or
  close Settings, to stop preparation and its Git process group.

After the user confirms the package details and declared permissions, Chartr
validates the staged contents again and queues the package under
`$XDG_DATA_HOME/chartr-zeddy/plugin-pending/<plugin id>/`. **Later** leaves the
current catalog and package assets unchanged, including after disable/re-enable.
At the next startup, before loading any plugins, Chartr atomically activates the
queued package at `plugins/<plugin id>/`. Activation failure preserves the
installed copy and retains the pending package for retry; interruption after
activation can safely reapply the same package. Persistent plugin data lives
outside both directories and survives replacement. **Restart** performs that
startup immediately.

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

The bundled Agent plugin is one such native module. Its GPUI pane receives the
owning space's display context and a host capability that opens a normal
Chartr-owned terminal. Agent commands, arguments, environment, and prompt
delivery remain structured until the native plugin quotes each shell word; the
resulting session belongs to the pane's space and follows the same persistence
and close lifecycle as a terminal opened by the user.

## Web host operations

`window.chartr.invoke(action, options)` runs host operations through a bounded,
ordered background queue for each pane. A full queue rejects new requests instead
of blocking the UI. Closing the pane suppresses replies and queued work; an
already-running operation may finish in the background. Replies are scoped to
the document that issued each request, so navigation cannot deliver an old
reply to a new page. Invalid calls reject their own promise. Navigation clears
pending promises; a ten-minute reply deadline also covers time spent waiting in
the ordered queue.

Requests are limited to 1 MiB of encoded JSON. File reads and HTTP response bodies
are limited to 8 MiB; process stdout and stderr share an 8 MiB limit. HTTP fetches
(including all redirects) and processes have a 30-second execution deadline.
Timeouts, permission failures, and exceeded limits reject the returned promise.
Processes that time out or exceed the output limit are terminated along with
their process group.

Each HTTP redirect must pass the manifest's network-host allowlist, just like the
initial URL. At most ten redirects are followed. Safe filesystem operations open
regular files beneath the project's or instance's data root; dangling links and
symlinks swapped into a validated path are rejected. Existing links to files
inside the allowed root continue to work. Explicit unsafe filesystem permission
still bypasses the project-root restriction, but not the private data-root check.

`project.write` and `data.write` require both `path` and a string `data` payload.
An explicitly empty string empties the file; an omitted or mistyped payload is
rejected before opening it. Whole-file writes atomically replace the destination,
preserving existing permission bits. Readers see a complete old or new file;
concurrent writes use last-completed-write semantics. New files are private to
the user. These operations write regular files, rather than devices or pipes.

`process.run` accepts `{ command, args?, cwd? }`. By default it runs in the
owning project, or the plugin data directory for folderless/settings views.
A relative `cwd` resolves from that same base; an absolute `cwd` is also accepted.
Process execution remains subject to the manifest's process permission.

Bound-session metadata reflects current session information. Terminal
reattachment retains the capability and redirects input to the new attachment;
closing the session makes its capabilities unavailable.
