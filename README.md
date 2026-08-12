# chartr

<img src="./docs/assets/v4/icon-mac-1024.png" width="34%" align="right">

**AI workspace with a map of your work.**

- [macOS app](https://github.com/rengwu/chartr/releases/latest/download/chartr_darwin_arm64.dmg)
  (Apple silicon, unsigned)
- [Linux AppImage](https://github.com/rengwu/chartr/releases/latest/download/chartr_linux_amd64.AppImage)
  (`amd64` or `arm64`)
- [More platforms](https://github.com/rengwu/chartr/releases)

An approachable agent multiplexer. Open a space, run agents and commands in tabbed terminal sessions.

Plan with an agent, then chart a map of your work. Drive the map to completion, one ticket at a time. Each ticket spawns a session with the exact context it needs to complete its task.

> chartr is still alpha. Features and file formats may change before 1.0.

<br clear="right">

<img alt="The chartr cockpit" src="https://github.com/user-attachments/assets/d69cd749-5c6e-41ef-bd78-e971b89c823b" />

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

Download the macOS or Linux app using the links above. Other builds are
available on the [releases page](https://github.com/rengwu/chartr/releases).

### macOS

The app is currently unsigned. If macOS blocks the first launch:

1. Open chartr and click **Done**.
2. Go to **System Settings → Privacy & Security → Security**.
3. Click **Open Anyway**.

### Linux AppImage

Make the AppImage executable, then run it:

```sh
chmod +x chartr_linux_amd64.AppImage
./chartr_linux_amd64.AppImage
```

WebKitGTK comes bundled. An Arch release is also planned.

### Debian and Ubuntu

The native package uses your distribution's WebKitGTK and receives its security
updates through apt. Ubuntu 24.04+ or Debian 13+ is required.

```sh
sudo apt install ./chartr_linux_amd64.deb
```

### Fedora

The Fedora package likewise uses the distribution's WebKitGTK runtime.

```sh
sudo dnf install ./chartr_linux_amd64.rpm
```

### Windows

The fully-supported Windows desktop app will be coming soon.

### CLI usage and building from source

See
[CLI and source builds](docs/cli-and-source-builds.md).

## Documentation

- [Getting started](docs/getting-started.md) — fresh machine to first star-map
- [CLI and source builds](docs/cli-and-source-builds.md) — run the server or
  build chartr locally
- [ADRs](docs/adr/) — why it is shaped the way it is
- [Security](SECURITY.md) — found a vulnerability? here's how to report it.

## Project status

The current release is **`v0.2.3`**. Download it from the
[releases page](https://github.com/rengwu/chartr/releases).

Highlights in `v0.2.3`:

- **Bring your own skills** - Register local folders or Git repositories as
  skill sources, then reorder, refresh or remove them from Settings.
- **Skills in every space** - Enabled skills are mirrored into each space,
  where sandboxed agents can read them. Fresh installs pre-register the
  `chartr-skills` repository.
- **Configurable Role bindings** - Let grill, prototype, research and implement resolve by
  source order, or pin any role to a specific skill.
- **Simpler agent launches** - Start a bare agent session from the new-shell
  menu, while ticket sessions still receive their complete assembled context.
- **VCS-neutral spaces** - Claims and releases are plain file edits recorded in `.plan/audit.jsonl`; chartr no longer runs git commands, including `git init` on new spaces.
- **CHARTR.md** - Each space gets a `CHARTR.md` file that helps agents quickly understand
  how to work with chartr.
- **Native Linux packages** - Install the desktop app through apt on Ubuntu and
  Debian, or dnf on Fedora; WebKitGTK stays under the system package manager.
- **Refreshed cockpit** - Rename and delete spaces moved into a context menu. Reorderable spaces
  with smoother drag-and-drop. Cleaner, sleeker look.
- **Terminal continuity** - Switching sessions now preserves each terminal's
  scroll position. Used to be super annoying here.

Still to come:

- **Windows desktop app** - Package and test the existing WebView2 shell.
- **AUR release** - Distribute chartr through the Arch User Repository.
- **GitHub Issues integration** - Bring GitHub issues into the chartr workflow.
- **Inbox mode** - Add an alternate view for tasks that need your attention.
- **Built-in updater** - Detect new releases and provide a way to install them.
- **Scratch location** - Make its starting directory configurable.
- **Alternate keybindings** - Add Neovim and Emacs keybinding presets.
- **More panes** - Add browser, source control, code review, and
  token usage panes alongside maps.
- **Agent onboarding** - Detect installed agent CLIs on first launch and guide
  users through registration.
- **Agent status coverage** - Expand live detection for third-party agents and
  uncommon prompts.

Known bugs:

- **Folder picker** - The folder picker does not currently work in the browser.
- **Ticket details** - Markdown does not render cleanly, and clicking a ticket
  reference does not open that ticket.
- **Notifications** - System notifications are sometimes unreliable. Most likely state-detection related.
- **Claude status** - Claude reports as idle while waiting for input in a
  multiple-choice selector.

See the
[GitHub releases](https://github.com/rengwu/chartr/releases) for published release
notes and [open issues](https://github.com/rengwu/chartr/issues) for additional
reports.

No hosted service or user accounts will ever be planned. chartr does not send any usage data or telemetry.

## Related projects

- [herdr](https://github.com/ogulcancelik/herdr) — the agent multiplexer that inspired this, in your terminal instead of a window
- [wayfinder-maps](https://github.com/rengwu/wayfinder-maps) — my read-only map CLI and viewer; where the star-map started
- [mattpocock/skills](https://github.com/mattpocock/skills) — the original `/wayfinder` skill and the method the maps side drives

## Acknowledgements

- @brownoxford for privately reporting localhost trust-boundary vulnerabilities
  that allowed cross-origin WebSocket control of live terminals, DNS-rebinding
  access, and CORS-simple API writes. His report led to strict Origin, Host, and
  content-type validation, owner-only state, verified build tooling, and safer bind
  warnings.
- @bradymwilliams for [reporting an issue](https://github.com/rengwu/chartr/pull/5)
  that led to improvements when opening chartr from monorepo subdirectories.
