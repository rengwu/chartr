# chartr

<img src="./docs/assets/v4/icon-mac-1024.png" width="34%" align="right">

**Agent multiplexer with a map of the work.**

[Download macOS app](https://github.com/rengwu/chartr/releases/latest/download/chartr_darwin_arm64.dmg)
(Apple silicon, unsigned)

[Download Linux AppImage](https://github.com/rengwu/chartr/releases/latest/download/chartr_linux_amd64.AppImage)
(`amd64` or `arm64`)

[More platforms](https://github.com/rengwu/chartr/releases)

Chart a wayfinder map inside chartr, then drive it to completion. The plan
renders as a star-map; take a ticket off the frontier, pick an agent, and the
session opens with the map, the ticket, and its blockers' answers already in the
buffer.

Without a map it is a plain multiplexer: projects in a sidebar, shells and agent
CLIs in tabs.

<br clear="right">

<img alt="The chartr cockpit" src="https://github.com/user-attachments/assets/4c1e4e13-e1fb-4bdd-a834-6e2d07415912" />

## Key features

- **Live map view.** Files in `.plan/maps/` appear as a star-map as soon as they
  are written.
- **Start from a ticket.** Select an unblocked ticket, role and agent to open a
  session with the required context already submitted.
- **Your own CLI agents.** Register any compatible CLI on `PATH`. chartr detects claude,
  codex, opencode, kimi, grok and pi by default.
- **Agent status.** The sidebar shows whether each agent is working or waiting
  for input.
- **Sidebar ordering.** Drag spaces to reorder them. The order is saved across
  restarts.
- **Scratch terminals.** Open a shell in your home directory without registering
  or initializing a repository.
- **Session notifications.** Receive a system notification when a long-running
  session finishes, blocks or exits. The tab stays marked until you return.
- **File-based state.** Add an `## Answer` section to resolve a ticket. chartr
  only writes claim and release commits to ticket files.
- **File-based skills.** Add `SKILL.md` directories per repository or machine.
  Launchable skills appear in the sidebar.
- **Terminal settings.** Configure presets, colours, font, cursor, padding and
  keybindings in TOML, including Shift+Enter for a newline.

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
- [Design system](docs/design-system.md) — tokens, primitives, the chrome/island split
- [ADRs](docs/adr/) — why it is shaped the way it is

## Project status

The current release is **`v0.2.1`**. Download it from the
[releases page](https://github.com/rengwu/chartr/releases).

chartr is still alpha. Features and file formats may change before 1.0.

Available in `v0.2.1`:

- [x] **Spaces and tabs.** Manage projects, shells and agent sessions from the
      sidebar.
- [x] **Maps.** View star-maps, start sessions, and claim or release tickets.
- [x] **Release builds.** Checksummed binaries, a macOS `.dmg`, and a Linux
      `.AppImage`.
- [x] **Setup guide.** Instructions for installing chartr and creating a first
      map.
- [x] **Linux desktop app.** The `.AppImage` includes WebKitGTK and is tested on
      a system without WebKit installed.

Available on `main` after `v0.2.1`:

- [x] **Shift+Enter.** Insert a newline without submitting the current input.
- [x] **Codex and Kimi.** Registration, session startup, prompt delivery and
      status reporting were tested with the real CLIs.
- [x] **Agent status fixes.** Recordings from real CLIs now test all six built-in
      agents. Status rules for codex, grok, opencode and pi were corrected, along
      with a UTF-8 parsing bug.
- [x] **Sidebar ordering.** Drag spaces to reorder them, or move the selected
      space with `⌥↑` / `⌥↓`. The order is saved across restarts.
      [Spec](.plan/maps/sidebar-order/spec.md).
- [x] **Scratch space.** Open a shell in your home directory without registering
      a repository. Scratch is shown only while a shell is open and keeps its
      sidebar position.
- [x] **Session notifications.** Receive a system notification when a
      long-running session finishes, blocks or exits. Notifications work when the
      cockpit is closed, and the tab stays marked until you return. Configure the
      timing or turn the feature off in `notify.toml`.
      [Spec](.plan/maps/session-notifications/spec.md).

Not yet:

- [ ] **Windows desktop app.** Add a packaged and tested WebView2 app to the
      release pipeline.
- [ ] **More agent verification.** Run the full registration, startup, prompt and
      status checks for grok, opencode and pi.
- [ ] **Agent configuration examples.** Add a working `[agents.*]` example for
      each provider to the setup guide.
- [ ] **Scratch terminal location.** Allow users to choose the starting directory
      instead of always using the home directory.
- [ ] **Agent status gaps.** Some prompts can still appear idle while waiting for
      input, including opencode's rejection-feedback prompt.
      [Map](.plan/maps/agent-state-detection/map.md).
- [ ] **Payload preview scrolling.** Expanded previews do not scroll correctly.
- [ ] **Ticket details.** Fix markdown rendering and make ticket references
      clickable.
- [ ] **Keyboard controls.** Add shortcuts for more actions and allow users to
      change the bindings.
- [ ] **Settings page.** Reorganize the page to make settings easier to find and
      manage.
- [ ] **Bundled skills.** Improve how included skills are copied to agents.
      [Today](docs/skill-sync.md).

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
