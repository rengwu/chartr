# Plugin packages

chartr installs packages; it does not build projects. Installation has no
scripts, hooks, package manager, compiler, or Rust-toolchain dependency.
Packages are user-selected code, with no marketplace or audit requirement.
Install sources you trust; manifest permissions describe the host APIs a plugin
can use, not a guarantee that its overall behavior is confined to those APIs.

Every package is a directory with `chartr-plugin.toml` at its root. A web
package includes the declared HTML entry and its assets. Entries may live in
subdirectories; asset URLs resolve within the package root, including filenames
that require URL encoding.

Every manifest also declares one free Hugeicons Stroke Rounded export by its
canonical name, for example `icon = "Clock01Icon"`. The matching SVG must be
packaged at `icons/Clock01Icon.svg`. chartr validates that file before install
or load and uses it in sidebar, outer, and pane-local tabs.
Installation and discovery share the same package validator, including for
disabled plugins. Declared assets must be regular files inside the package;
absolute paths, parent traversal, and symlinks escaping the package are rejected.
Installation rejects source symlinks and omits `.git` and `target` directories.

A hosted package contains only declarative files and names a surface already
implemented by chartr. Hosted surfaces are intended for first-party plugins
that need deep operating-system integration without crossing the GPUI dynamic
library boundary. Browser is the first one: `kind = "hosted"` and
`surface = "browser"`.

## Installation sources

- **Local folder:** chartr copies the selected package into private staging
  before showing the confirmation.
- **Git repository:** chartr shallow-clones the default branch. No release API,
  package manager, or build system is involved. Cloning has a two-minute deadline
  and disables interactive terminal credential prompts. Cancel in Settings, or
  close Settings, to stop preparation and its Git process group.

After the user confirms the package details and declared permissions, chartr
validates the staged contents again and queues the package under
`$XDG_DATA_HOME/chartr/plugin-pending/<plugin id>/`. **Later** leaves the
current catalog and package assets unchanged, including after disable/re-enable.
At the next startup, before loading any plugins, chartr atomically activates the
queued package at `plugins/<plugin id>/`. Activation failure preserves the
installed copy and retains the pending package for retry; interruption after
activation can safely reapply the same package. Persistent plugin data lives
outside both directories and survives replacement. **Restart** performs that
startup immediately.

Each new installation records its source and, for Git, the checked-out commit in
`.chartr-install.json` inside the managed package. Settings shows this alongside
the manifest version. Activation preserves the original record; package-supplied
records are overwritten on installation. Older or manually copied packages may
have no record. This record is for display, not verification of
the author or package contents. Updating means installing the source again;
there is no automatic update or version-history service.

Each plugin has exactly one entry in Settings: its name and info, gear, and
enable controls on the first line, then a short `description` that wraps across
the full width. Prerequisite errors appear below the description. The host owns
this layout; plugins cannot add extra rows. The gear opens the plugin's single
configuration page, combining native settings controls with prerequisite setup
links and host permissions. The info icon opens **Plugin Information**, which
contains the package details, access summary, and **Uninstall** button. Build-time native plugins may
contribute a GPUI view. Web and hosted plugins declare `[settings]` fields;
chartr renders these with the same native controls as its own forms. No plugin
HTML or JavaScript runs inside the Settings window. Legacy `settings_entry`
manifests are rejected with a migration message; replace that declaration with
the schema below. Existing plugin data is retained.

**Uninstall** confirms removal, closes the plugin's panes and configuration view,
and removes its installed and bundled packages and any pending update. Source
repositories, plugin data, and preferences are kept. Bundled removals are saved
as `uninstalled = true` in that plugin's settings so they stay removed on restart.
To restore a compiled-in bundle, clear that marker and restart; it remains disabled
until enabled. Reinstalling a hosted or web package also leaves it disabled.
If removal fails, Settings reports the error and leaves the plugin disabled for
retry. A newly queued plugin appears in the catalog after restart.

Disabling or uninstalling a prerequisite first prompts with the complete list of
dependent plugins, including indirect dependents. Confirmation disables that
entire cascade, closes its panes, and persists the disabled state; cancellation
changes nothing. Dependents remain installed and are never enabled implicitly
when their provider returns. Missing, disabled, or cyclic prerequisites block
enabling, including during startup. These rules apply to all plugin tiers.

Hosted and web packages are architecture-independent and need no release
binary. Installing chartr Browser is therefore only a shallow clone or folder
copy, manifest validation, confirmation, and atomic rename. Browser uses
ephemeral web-engine storage and persists only each pane's last URL.

chartr may link native plugin modules at application build time, but it rejects
separately installed GPUI dynamic libraries. Precompiling does not make Rust
GUI objects or crate-global state ABI-safe across two independently linked
copies of GPUI. Plugins that need native operating-system integration should
use a chartr-implemented hosted surface; portable third-party plugins should use the web
tier.

The bundled Agent plugin is one such native module. Its GPUI pane receives the
owning space's display context and a host capability that opens a normal
chartr-owned terminal. Agent commands, arguments, environment, and prompt
delivery remain structured until the native plugin quotes each shell word; the
resulting session belongs to the pane's space and follows the same persistence
and close lifecycle as a terminal opened by the user.

## Manifest

A complete web example (optional fields shown with their defaults):

```toml
manifest_version = 2
id = "com.example.notes"
name = "Notes"
description = "Keep notes alongside your work."
version = "0.1.0"
kind = "web"
icon = "NoteIcon"             # package includes icons/NoteIcon.svg
entry = "index.html"

[capabilities]
multiplicity = "per_space"    # or "multiple"
cloneable = false
restorable = false
session_binding = false

[permissions]
project_files = "none"        # or "read", "read_write"
network = []                  # e.g. ["api.example.com", "*.example.org"]
process = false
session = false
wayfinder = false            # map workflow and registered-agent launching
```

Portable plugins can add native settings without a settings document:

```toml
[settings]
file = "settings.json"         # relative to this plugin's private data directory

[[settings.fields]]
key = "format"                 # top-level JSON property
label = "Clock format"
type = "select"
default = "24"
options = [
    { value = "24", label = "24-hour" },
    { value = "12", label = "12-hour" },
]

[[settings.fields]]
key = "show_seconds"
label = "Show seconds"
description = "Include seconds in the display."  # optional
type = "toggle"
default = true
```

`select` stores a string from its declared options; `toggle` stores a boolean.
Field keys and option values must be unique, and a select's default must be one
of its options. The host accepts up to 64 fields and 64 options per select.
Unknown control types are rejected. Settings paths must remain inside private
plugin data; parent directories must already exist.

The file is a JSON object, limited to 1 MiB. Missing fields use their declared
defaults. Opening Settings does not write a file. Each change re-reads the file,
updates the selected key, preserves other keys, and replaces the file atomically.
Unreadable, malformed, or unsupported saved values produce a native error and
are not overwritten; correct the file and use **Reload**. Plugins read the same
file through `data.read` (the optional `examples/plugins/clock` example checks
it on each tick).

The ID is the package identity used for replacement, data, preferences, and
saved panes. Use a stable reverse-DNS name. IDs accept ASCII letters, digits,
`.`, `-`, and `_`, up to 128 characters, and cannot start with `.`. Installed
directory names must match the ID. `version` is a display string; chartr does
not compare versions or prevent downgrades. This build accepts only manifest
version 2; unknown TOML fields are ignored.

Web and hosted packages contribute one `main` pane. `per_space` focuses an
existing matching pane when opened again; `multiple` permits additional panes.
`cloneable` enables opening another instance, and `restorable` enables reopening
saved panes. Web cloning and restoration create a fresh document; there is no
host API for serializing per-pane web state. `session_binding` lets a pane bind
to the terminal from which its launcher was opened (required for this capability);
`permissions.session` separately grants
access to that binding. Native settings schemas grant no project, network, process,
or terminal access.

The `data.*` API always accesses
`$XDG_DATA_HOME/chartr/plugin-data/<plugin id>/`. This storage is shared
across the plugin's panes, spaces, and native settings form, not allocated per
instance. File APIs do not create parent directories.

The project-file and network grants constrain their respective APIs.
**`process = true` grants execution with the user's account authority**, including
filesystem and network access outside those brokers. **`session = true` allows
terminal input**, which can execute commands with the bound terminal's authority.
Neither is a sandboxed form of execution. The explicit unsafe-filesystem setting
overrides project-file API restrictions; it does not change the private data root.
**`wayfinder = true` grants map and source reads, ticket-claim updates, and launches
through registered agents with the user's account authority.** It does not grant
arbitrary project writes, raw terminal input, process execution, or unrestricted
access to native services. It is still an execution capability: a registered
agent can carry out the workflow's prompt with its normal permissions.

## Plugin prerequisites and native services

A plugin declares required providers with a short explanation of each prerequisite:

```toml
[[dependencies]]
plugin = "com.chartr.agent"
feature = "Agent launching"

[[dependencies]]
plugin = "com.chartr.skills"
feature = "Ticket methods"
```

Settings displays the provider and feature; missing providers are never
automatically installed or enabled. Consumers check configuration as well as
availability and explain how to finish setup. The host enforces these prerequisites
and activates providers first at startup; it does not resolve package versions.

Trusted native plugins can return `ServiceExport::new(service)` from their
optional `Plugin::services` method. `InstanceContext.services.get::<T>(plugin_id)`
resolves a live export in the owning catalog. Disablement removes its entries;
reenablement publishes fresh exports. Resolve services again before acting.
Native code is already trusted; these are shared typed contracts, not a sandbox.

The initial contracts are `services::Agents` (registered names and input
preparation) and `services::Skills` (asynchronous enabled-source scanning,
method text, directories and commit provenance). Agent and Skills own their
registries. Each plugin has one configuration surface under **Settings → Plugins →
Configure**. Pane setup shortcuts use `InstanceContext.plugin_settings` to open
that surface instead of rendering another configuration page in the workspace.

`InstanceContext.terminal.prepare(cx)` returns a task resolving to an attached
`PreparedTerminal` with a real session `id`; `send` delivers the validated input
and reports errors. This lets a consumer claim a ticket before the agent starts.
`terminal.focus(id, window, cx)` selects that session in its owning space.
These Rust capabilities stay host-side. The narrow Wayfinder web API below
uses them without exposing arbitrary service calls or shell input to JavaScript.

## Web host operations

`window.chartr.invoke(action, options)` runs host operations through a bounded,
ordered background queue for each pane. A full queue rejects new requests instead
of blocking the UI. Closing the pane suppresses replies and queued work; an
already-running operation may finish in the background. Replies are scoped to
the document that issued each request, so navigation cannot deliver an old
reply to a new page. Invalid calls reject their own promise. Navigation clears
pending promises; a ten-minute reply deadline also covers time spent waiting in
the ordered queue.

`invoke` returns a Promise for the result below and rejects with an `Error` on
failure. All path, data, command, argument, and URL values are strings; optional
fields are marked `?`.

| Action | Options | Result | Required grant |
| --- | --- | --- | --- |
| `project.read` | `{ path }` | UTF-8 string | `project_files = "read"` or `"read_write"` |
| `project.write` | `{ path, data }` | `true` | `project_files = "read_write"` |
| `data.read` | `{ path }` | UTF-8 string | Always available |
| `data.write` | `{ path, data }` | `true` | Always available |
| `network.fetch` | `{ url }` | `{ status: number, body: string }` | Host in `network` |
| `process.run` | `{ command, args?: string[], cwd? }` | `{ status: number \| null, stdout: string, stderr: string }` | `process = true` |
| `session.metadata` | Omit options | `{ id: string, workspace: string, title: string, agent: string \| null, cwd: string \| null }` | `session = true` and a live binding |
| `session.send` | `{ data }` | `true` | `session = true` and a live binding |

```js
const text = await window.chartr.invoke("project.read", { path: "README.md" });
await window.chartr.invoke("data.write", { path: "notes.txt", data: text });
```

`network.fetch` performs an HTTP(S) GET without custom headers or a request body;
HTTP 4xx/5xx responses reject. Allowlist entries match hostnames, not paths or
ports. A URL entry contributes its hostname; `*.example.org` includes the base
domain and its subdomains. Packaged web documents cannot make direct network
connections under chartr's content security policy; use the host API.

`process.run` launches a command directly, without a shell unless the plugin
explicitly invokes one. Arguments default to an empty array. A nonzero exit code
is returned as `status`, not a rejection; termination by signal returns `null`.
Output is decoded as UTF-8 with replacement for invalid bytes. `session.send`
writes the supplied terminal input as-is; it does not add a newline.

Requests are limited to 1 MiB of encoded JSON. File reads and HTTP response bodies
are limited to 8 MiB; process stdout and stderr share an 8 MiB limit. HTTP fetches
(including all redirects) and processes have a 30-second execution deadline.
Timeouts, permission failures, and exceeded limits reject the returned promise.
Processes that time out or exceed the output limit are terminated along with
their process group.

Each HTTP redirect must pass the manifest's network-host allowlist, just like the
initial URL. At most ten redirects are followed. Safe filesystem operations open
regular files beneath the project root or the plugin's shared data root; dangling links and
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

### Wayfinder workflow

These actions require `permissions.wayfinder = true` and an owning space pane;
they are unavailable to native settings schemas. No identity-based exemption is used:
an installed web package must declare the same grant as bundled Wayfinder.
All actions use the existing ordered queue and document-scoped replies.

| Action | Options | Result |
| --- | --- | --- |
| `wayfinder.snapshot` | Omit | Space/folder, discovered maps/tickets with safe rendered Markdown, agent names, qualified skill names and setup diagnostics |
| `wayfinder.preview` | `{ slug: string, ticket: number, method?: string, note?: string }` | `{ preview: number, text: string, sources: string[] }` |
| `wayfinder.launch` | `{ preview: number, agent: string }` | `{ session: string }` |
| `wayfinder.release` | `{ slug: string, ticket: number, session: string }` | `true` |
| `wayfinder.focus` | `{ slug: string, ticket: number }` | `true` |
| `wayfinder.open` | `{ slug: string, ticket?: number, target?: string }` | `true` |
| `wayfinder.settings` | `{ provider: "agent" \| "skills" }` | `true` |

Preview requires an existing map slug and a ready ticket.
The optional method pins an exact `source/skill`; omission selects by ticket type.
Notes are bounded to 16 KiB and the composed prompt to 192 KiB. A preview ID is
one-shot, document-bound and replaced by the next preview. Launch rechecks the
source content and on-disk map, prepares a real terminal, writes its claim, then
delivers adapter-correct input. Closing or navigating before delivery cancels the
launch and releases its own claim. An already delivered launch is not terminated.

Release must supply the current claim's session ID; it never stops the terminal.
The UI asks for a second confirmation. Open without a target opens the map/ticket
file; relative targets resolve beside that file and must remain inside the space.
HTTP(S) targets open through the OS. Raw HTML is escaped, images render as links,
and no project Markdown executes in the web document.
