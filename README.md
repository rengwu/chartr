# chartr

<img src="./docs/assets/v4/icon-mac-1024.png" width="34%" align="right">

**AI workspace with a map of your work.**

[Download macOS app](https://github.com/rengwu/chartr/releases/latest/download/chartr_darwin_arm64.dmg)
(Apple silicon, unsigned)

[Download Linux AppImage](https://github.com/rengwu/chartr/releases/latest/download/chartr_linux_amd64.AppImage)
(`amd64` or `arm64`)

[More platforms](https://github.com/rengwu/chartr/releases)

An approachable agent multiplexer. Open a space, run agents and commands in tabbed terminals.

Comes with map-charting features. Plan with an agent, then chart a map of your work. Drive the map to completion, one ticket at a time. Each ticket spawns a session with the exact context it needs to complete its task.

<br clear="right">

<img alt="The chartr cockpit" src="https://github.com/user-attachments/assets/4c1e4e13-e1fb-4bdd-a834-6e2d07415912" />

## Key features

- **Bring your CLI agents** - Register the agents you already use, launch them easily afterwards.
- **Use your existing skills** - Register your skills from local folders or remote git repositories.
- **Live star-map** - Visualize your plan and track live progress on an interactive map.
- **Ticket-ready sessions** - Spawn an agent from a ticket with the relevant context already loaded.
- **At-a-glance status** - See which sessions are working, idle or waiting for input.
- **Folders as spaces** - Terminal sessions are grouped into spaces you can filter and reorder.
- **Get notified** - Receive system notifications when a session needs you.
- **Make it yours** - Configure terminal appearance and prompts, or hack the config to suit your workflow.

## Installation

Grab your platform's archive from the
[releases page](https://github.com/rengwu/chartr/releases) and run it:

```
chartr                 # http://127.0.0.1:8787
chartr -addr :9000     # not loopback — see below
chartr -data-dir ~/w   # session root (default: cwd)
```

chartr has **no authentication**. Reaching the port is the whole of the access
check, and the API behind it opens shells, runs commands and spawns agents in
your account. Binding to anything other than loopback — `-addr :9000`,
`-addr 0.0.0.0:9000`, a LAN address — hands that to everyone who can reach the
port. Keep the default `127.0.0.1` unless you mean to expose it; chartr warns at
startup when you don't.

Install your own agent CLIs; chartr ships none.

### Linux desktop app

Download `chartr_linux_amd64.AppImage` (or `arm64`), make it executable, run it:

```
chmod +x chartr_linux_amd64.AppImage
./chartr_linux_amd64.AppImage
```

No install, no dependencies — WebKitGTK is bundled. It borrows only what has to
come from your machine: the GPU driver, your font configuration and your
compositor.

Also available as `make appimage` from source, on Linux.

### macOS first launch

The `.dmg` is <b>unsigned</b>, so macOS blocks it once:

1. Open chartr, click **Done** (_not_ **Move to Trash**).
2. **System Settings → Privacy & Security → Security → Open Anyway**.

Or `xattr -d com.apple.quarantine /Applications/chartr.app`.

### From source

Go 1.26+, Node 22+.

```
make build     # → bin/chartr
make check
make test
make dmg       # the macOS app
```

## Documentation

- [Getting started](docs/getting-started.md) — fresh machine to first star-map
- [ADRs](docs/adr/) — why it is shaped the way it is
- [Security](SECURITY.md) — found a vulnerability? here's how to report it.

## Project status

The current release is **`v0.2.2`**. Download it from the
[releases page](https://github.com/rengwu/chartr/releases).

Development toward `v0.2.3` is underway. Since `v0.2.2`:

- **Bring your own skills** - Register local folders or Git repositories as
  skill sources, then reorder, enable, refresh or remove them from Settings.
- **Skills inside every space** - Enabled skills are mirrored into each space,
  where sandboxed agents can read them. Fresh installs pre-register the
  `chartr-skills` repository.
- **Role bindings** - Let grill, prototype, research and implement resolve by
  source order, or pin any role to a specific skill.
- **Simpler agent launches** - Start a bare agent session from the new-shell
  menu, while ticket sessions still receive their complete assembled context.
- **VCS-neutral spaces** - Register any folder without initializing Git. Claims
  and releases are plain file edits recorded in `.plan/audit.jsonl`; chartr no
  longer runs version-control commands.
- **Local agent context** - Every registered space gets a private `CHARTR.md`, a
  file-format contract and a synchronized skill mirror.
- **Refreshed cockpit** - Rename or forget spaces from a context menu, reorder
  them with smoother drag-and-drop, and distinguish a running process from a
  working agent at a glance.
- **Terminal continuity** - Switching sessions now preserves each terminal's
  scroll position.

Still to come:

- **Windows desktop app** - Package and test the existing WebView2 shell.
- **AUR release** - Distribute chartr through the Arch User Repository.
- **GitHub Issues integration** - Bring GitHub issues into the chartr workflow.
- **Inbox mode** - Add an alternate view for tasks that need your attention.
- **Built-in updater** - Detect new releases and provide a way to install them.
- **Scratch location** - Make its starting directory configurable.
- **Alternate keybindings** - Add Neovim and Emacs keybinding presets.
- **More panes** - Add browser, source control, GitHub issues and reviews, and
  token usage panes alongside maps.
- **Agent onboarding** - Detect installed agent CLIs on first launch and guide
  users through registration.
- **Agent status coverage** - Expand live detection for third-party agents and
  uncommon prompts.

Known bugs:

- **Folder picker** - The folder picker does not work in the browser.
- **Ticket details** - Markdown does not render cleanly, and clicking a ticket
  reference does not open that ticket.
- **Notifications** - System notifications are unreliable.
- **Claude status** - Claude reports as idle while waiting for input in a
  multiple-choice selector.

chartr is still alpha. Features and file formats may change before 1.0. See the
[GitHub releases](https://github.com/rengwu/chartr/releases) for published release
notes and [open issues](https://github.com/rengwu/chartr/issues) for additional
reports.

No hosted service or user accounts are planned. chartr does not send usage data.

## Platform support

One **supported** artifact: the cgo-free binary that serves the cockpit in your
browser, green on all three OSes before a tag ships. The Linux `.AppImage` is
built and smoke-tested as a release gate, so it ships whenever the supported
binary does. The macOS `.dmg` is a best-effort extra: it needs cgo and system
webview libs, and may simply be absent
([ADR 0011](docs/adr/0011-one-supported-artifact-tiered-extras.md)).

| Platform                  | Binary | Desktop app                             |
| ------------------------- | ------ | --------------------------------------- |
| macOS `arm64`             | ✅     | `.dmg`, [unsigned](#macos-first-launch) |
| macOS `amd64`             | ✅     | none (cgo won't cross-compile)          |
| Linux `amd64` / `arm64`   | ✅     | `.AppImage`, WebKitGTK bundled          |
| Windows `amd64` / `arm64` | ✅     | none                                    |

The Linux `.AppImage` carries its own WebKitGTK, so it does not care whether your
distro ships `webkit2gtk-4.1`, the older `4.0`, or neither. Every tag builds it
and runs it against a container with **no WebKit and no GTK installed**,
screenshotting the window to prove the cockpit rendered rather than trusting an
exit code — and that check gates the release
([ADR 0011](docs/adr/0011-one-supported-artifact-tiered-extras.md)).

Windows has no packaged app and none is scheduled; it can be built locally with
`make webview` ([ADR 0013](docs/adr/0013-webview-shell-architecture.md)).

Windows is built and its ConPTY layer is smoke-tested every change, but it isn't
driven daily, so **WSL2 is the sure path**.

## Related projects

- [herdr](https://github.com/ogulcancelik/herdr) — the agent multiplexer that inspired this, in your terminal instead of a window
- [wayfinder-maps](https://github.com/rengwu/wayfinder-maps) — my read-only map CLI and viewer; where the star-map started
- [mattpocock/skills](https://github.com/mattpocock/skills) — the original `/wayfinder` skill and the method the maps side drives

## Demonstration

https://github.com/user-attachments/assets/60c335bb-5d9d-44c6-9798-654b1c70c626
