# chartr

<img alt="chartr icon" src="./docs/assets/v4/icon-mac-1024.png" width="34%" align="right">

**A fast, organizable terminal-based agentic workspace.**

- [Website](https://chartr.dev/) (launching soon)
- [Build from source](docs/installation.md)
- [Getting started](docs/getting-started.md)
- [Documentation](docs/README.md)

Keep your projects, tools, workflows, and agents together. chartr is an
open-source native desktop app built with Rust and GPUI. Organize projects into
spaces, arrange terminals and tools side by side, and make the workspace your own.

> The Rust rewrite is in active development for macOS and Linux; release packages
> are being prepared. [v0.2.4](https://github.com/rengwu/chartr/releases/tag/v0.2.4)
> is the legacy Go/Svelte version.

<img width="1169" height="857" alt="Screenshot 2026-09-08 at 4 11 24 AM" src="https://github.com/user-attachments/assets/8af894fe-4a0f-4247-b733-e9d521fd44a1" />

*Earlier Rust workspace snapshot, 8 September 2026; current controls have changed.*

<br clear="right">

## Key features

- **Folders as spaces** — Keep each project's terminals and tools together, with
  Free sessions for work outside a project.
- **Flexible views** — Switch between Tabs, Spaces, and Chats with your sessions
  and pane arrangements intact.
- **Tabs and split panes** — Move tabs, split them into columns or rows, and
  group terminals and plugin tools within each space.
- **Durable sessions** — Quit chartr and keep work running. Herdr keeps terminal
  processes alive by default; reopen to restore your sessions and layout.
  Closing a terminal tab ends that session.
- **Bring your CLI agents** — Register the agents you already use and launch
  them with your own arguments, environment, and prompts.
- **Tools beside your terminals** — Browse pages, reuse prompts, and manage
  skills with bundled plugins. Install more or [build your own](docs/plugins.md)
  with HTML, CSS, and JavaScript.
- **Plan with Wayfinder** — Explore a live map of your work, review tickets,
  and launch agents with the context they need.
- **Make it yours** — Choose themes and fonts, rebind shortcuts, and configure
  plugins in native Settings.

chartr runs locally and does not require a chartr account.

## Related projects

- [Zed](https://github.com/zed-industries/zed) — GPUI, UI components, themes, and the terminal stack
- [Herdr](https://github.com/herdrdev/herdr) — persistent terminal sessions
- [wayfinder-maps](https://github.com/rengwu/wayfinder-maps) — the map CLI and viewer where the star-map started
- [mattpocock/skills](https://github.com/mattpocock/skills) — the original `/wayfinder` skill and workflow

## Acknowledgements

- [@brownoxford](https://github.com/brownoxford) for privately reporting
  vulnerabilities that helped harden the original implementation's localhost
  trust boundaries.
- [@bradymwilliams](https://github.com/bradymwilliams) for
  [reporting an issue](https://github.com/rengwu/chartr/pull/5) that led to
  improvements when opening chartr from monorepo subdirectories.

## Licence

[GPL-3.0-or-later](LICENSE-GPL).
