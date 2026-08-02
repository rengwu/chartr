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

- **Chart the map without leaving.** Launch a planning agent from the sidebar and
  whatever it writes to `.plan/maps/` draws as a star-map the moment it hits disk.
- **Spawn off the frontier.** Take an unblocked ticket, pick a role and an agent,
  and the session opens in that agent's own TUI with full context already
  submitted.
- **Whatever CLI you run.** The adapter models one thing, how a binary takes its
  opening line, so anything on `PATH` registers and claude, codex, opencode,
  kimi, grok and pi are detected for you.
- **Tells blocked from thinking.** Tab status comes from what an agent broadcasts
  and draws, so the sidebar flags whichever space is waiting on you.
- **The map on disk is the state.** A ticket resolves when its `## Answer`
  appears, and chartr's only writes are the claim and release commits on the
  ticket file.
- **Skills are files.** Plain `SKILL.md` directories you can shadow per repo or
  per machine, and your own launchable skill shows up in the sidebar.
- **Terminal in one TOML.** Presets, 21 colour slots, font, cursor, padding,
  keybindings.

## Installation

Grab your platform's archive from the
[releases page](https://github.com/rengwu/chartr/releases) and run it:

```
chartr                 # http://127.0.0.1:8787
chartr -addr :9000
chartr -data-dir ~/w   # session root (default: cwd)
```

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

**`v0.2.1` is out** — grab a binary from the
[releases page](https://github.com/rengwu/chartr/releases). chartr is used to
build chartr, but it is still alpha: the shape moves and breaking changes are
expected before 1.0.

Shipped in `v0.2.1`:

- [x] Spaces, tabs, activity detection
- [x] Maps: star-map, spawn, claim/release
- [x] Release pipeline — checksummed binaries, macOS `.dmg`, Linux `.AppImage`
- [x] Getting started, written against a fresh machine
- [x] **Linux desktop app** — an `.AppImage` with WebKitGTK bundled, gated on a
      render check against a host with no WebKit installed.

Fixed on `main` since `v0.2.1`:

- [x] **Shift+Enter as a literal newline** — the keystroke now lands: the seam
      swallows the stray keypress xterm turned into a `\r` right after the
      newline, which was submitting the line it had just broken.

Not yet:

- [ ] **Windows desktop app** — a native-windowed chartr for Windows as a
      shipped, tested artifact, the way the macOS `.dmg` and Linux `.AppImage`
      already are: WebView2, packaging, and a slot in the release pipeline.
- [ ] **Registered agents verified end to end** — `codex` and `kimi` are
      believed working (register, spawn, opener delivered, states read) but the
      checklist has never been formally run; then extend it to the harnesses
      whose flags are still unchecked: `grok`, `opencode`, `pi`.
- [ ] **Agent registration examples in the docs** — one working `[agents.*]`
      block per provider in the getting-started guide, so registering an agent
      is a paste, not a read of the adapter source.
- [ ] **Scratch shells open where you say** — the starting directory of a
      scratch terminal is configurable, instead of always the cockpit's own
      working tree.
- [ ] **Session-status inference misfires** — known cases where a tab's
      working/idle/done reads wrong, the OSC C1/UTF-8 misfire among them.
      [Map](.plan/maps/agent-state-detection/map.md).
- [ ] **Payload preview loses its scroll** — expanding the composed document in
      the payload preview modal breaks scrolling.
- [ ] **Ticket details** — markdown that renders incorrectly, and ticket
      references that should be clickable links to the ticket they name.
- [ ] **Keyboard shortcuts: coverage and config** — more of the cockpit is
      reachable from the keyboard, and the bindings are the operator's to remap,
      not fixed in the chrome.
- [ ] **Pre-packaged skill delivery** — a better way for the bundled skills to
      reach an agent than today's sync.
      [Today](docs/skill-sync.md).
- [ ] **Drag to reorder the sidebar** — spaces stay where you put them, and
      `pinned` goes away with the recency sort it fought over.
      [Spec](.plan/maps/sidebar-order/spec.md).
- [ ] **"It's done" notifications** — a session that worked longer than a
      configurable _n_ seconds tells you when it stops, whether it landed, blocked
      or died, with the cockpit closed.
      [Spec](.plan/maps/session-notifications/spec.md).

Not planned: a hosted service, an account, anything that phones home.

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
