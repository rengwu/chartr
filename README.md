# chartr

<img src="./docs/assets/v4/icon-mac-1024.png" width="34%" align="right">

**AI workspace with a map of your work.**

- [macOS app](https://github.com/rengwu/chartr/releases/latest/download/chartr_darwin_arm64.dmg)
  (Apple silicon, unsigned)
- [Linux AppImage](https://github.com/rengwu/chartr/releases/latest/download/chartr_linux_amd64.AppImage)
  (`amd64` or `arm64`)
- [More releases](https://github.com/rengwu/chartr/releases)
- [Getting started with chartr](docs/getting-started.md)

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
- **Self-titling tabs** - Agent tabs name themselves from the agent's own session, so a row of tabs reads as your work.
- **Folders as spaces** - Terminal sessions are grouped into spaces you can filter and reorder.
- **Get notified** - Receive system notifications when a session needs you.
- **Make it yours** - Configure terminal appearance, titles and notifications, or hack the config to suit your workflow.

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

The current release is **`v0.2.4`**. Download it from the
[releases page](https://github.com/rengwu/chartr/releases).

Highlights in `v0.2.4`:

- **Self-titling tabs** - An agent tab titles itself from the agent's own
  session record: chartr shows the title the harness already wrote, and
  otherwise asks that tab's own agent for one short title from the session's
  first completed turn. Auto titles are searchable, and a long one scrolls in
  place while you hover its row. Turn it off, or keep only the free half with
  `native_only`, under Settings.
- **Notifications from the session, not the screen** - A finished run is now
  read from the agent's own transcript rather than its terminal screen, so
  completion reports when the turn actually ended. Supported for claude, codex,
  grok, kimi and pi; other harnesses keep the older screen-derived timing.
- **A faster terminal** - `terminal.toml` picks the renderer and Linux defaults
  to canvas. A renderer addon that fails now falls back to the DOM instead of
  leaving a blank pane, and the glyph atlas rebuilds once fonts load.
  Input-to-paint latency is sampled, so a regression shows up as a number.
- **Find in the terminal** - `Ctrl+Shift+F` opens search in the active session.
- **Sortable sessions** - Drag session rows to reorder them inside a space. The
  status glyph moved into its own leading column and is present on every row.
- **Free shells that outlive the agent** - A free session preloads the agent
  inside the space's own shell, so `Ctrl+C` or `/exit` leaves you in a shell
  with its scrollback intact instead of closing the tab.
- **Linux desktop fixes** - The AppImage anchors its runtime root, restores the
  host environment for the processes it spawns, stops leaking a pixbuf cache,
  and ships the correct icon. A folder chooser that fails to start now says why
  instead of reporting a dialog you never saw as cancelled.
- **Better agent status** - Claude 2.1.234's half-circle spinner reads as
  working again, and a kimi launch trusts its workspace up front.
- **Sharper chrome** - Every icon sits on one token-driven scale, landed on the
  16px pixel grid, and the sidebar wordmark steps aside under a native title
  bar.

Still to come:

- **Windows desktop app** - Package and test the existing WebView2 shell.
- **AUR release** - Distribute chartr through the Arch User Repository.
- **Prompt presets** - A catalog of reusable prompts a space launches with.
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

- **Ticket details** - Markdown does not render cleanly, and clicking a ticket
  reference does not open that ticket.
- **Folder picker** - The chooser is raised by the server, so it needs `zenity`
  or `kdialog` on Linux and is unavailable on Windows. Where none is found,
  register a space by typing its absolute path instead.
- **Notification coverage** - Completion is only as good as the transcript
  behind it. An agent chartr cannot read a transcript for still falls back to
  screen-derived timing, which can fire late or not at all.
- **Claude status** - Claude reads as blocked on its permission prompt, but
  still reports idle while it waits on its other selectors.

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

- [@brownoxford](https://github.com/brownoxford) for privately reporting localhost trust-boundary vulnerabilities
  that allowed cross-origin WebSocket control of live terminals, DNS-rebinding
  access, and CORS-simple API writes. His report led to strict Origin, Host, and
  content-type validation, owner-only state, verified build tooling, and safer bind
  warnings.
- [@bradymwilliams](https://github.com/bradymwilliams) for [reporting an issue](https://github.com/rengwu/chartr/pull/5)
  that led to improvements when opening chartr from monorepo subdirectories.
