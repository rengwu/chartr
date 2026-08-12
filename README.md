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

Download the macOS or Linux app using the links above. Other builds are
available on the [releases page](https://github.com/rengwu/chartr/releases).

### macOS

The app is currently unsigned. If macOS blocks the first launch:

1. Open chartr and click **Done**.
2. Go to **System Settings → Privacy & Security → Security**.
3. Click **Open Anyway**.

### Linux

Make the AppImage executable, then run it:

```sh
chmod +x chartr_linux_amd64.AppImage
./chartr_linux_amd64.AppImage
```

WebKitGTK comes bundled. An Arch release is also planned.

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

The current release is **`v0.2.2`**. Download it from the
[releases page](https://github.com/rengwu/chartr/releases).

Development toward `v0.2.3` is underway. Implemented since `v0.2.2`:

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
