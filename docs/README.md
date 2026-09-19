# Documentation

These guides describe the current Rust working tree. Start with
[Installation](installation.md) and [Getting started](getting-started.md).

## Current guides

| Topic | Guide |
| --- | --- |
| Source builds, supported platforms, and package installation | [Installation](installation.md) |
| First space, agent setup, and the Wayfinder workflow | [Getting started](getting-started.md) |
| Spaces, panes, terminals, settings, persistence, and data locations | [Workspace](workspace.md) |
| Chats view, Inbox/Archive, agent discovery, and session logs | [Inbox](conversations.md) |
| Package installation, prerequisites, permissions, and SDK contracts | [Plugins](plugins.md) |
| Persistent plugin activity | [Status bar](status-bar.md) |
| Ordered local and Git skill sources | [Skill sources](../plugins/skills/README.md) |
| Shared prompt library and editor modals | [Saved Prompts](../plugins/prompts/README.md) |
| Template composition and explicit project-file saves | [Markdown Prompt](../plugins/markdown-prompt/README.md) |
| Maps, ticket launch, and claim recovery | [Wayfinder](../plugins/wayfinder/README.md) |
| Markdown map/ticket format | [Tracker convention](../plugins/wayfinder/TRACKER-CONVENTION.md) |

## Development and design

- [Code map](code-map.md): implementation owners and entry points.
- [Release builds](releasing.md): Linux packages, macOS development DMGs, build
  caches, and timings.
- [Release acceptance](acceptance.md): automated commands and hands-on checks.
  A checklist is not a record that those checks have passed.
- [Workspace specification](../.plan/maps/chartr-workspace/spec.md): the living
  product contract; its opening problem statement records the rewrite's origin.
- [Architecture decisions](adr/README.md): rationale and subsequent changes to
  architectural boundaries.
- [Plugin examples](../examples/plugins/README.md),
  [theme playground](../misc/theme-playground/README.md),
  [font assets](../crates/chartr/assets/fonts/README.md), and
  [icon provenance](../crates/chartr/assets/icons/README.md).
- [Platform patches](../vendor/zed-platform/README.md) and
  [terminal-view patch](../vendor/zed-terminal-view/chartr-PATCH.md): maintained
  changes to the pinned Zed source.

## Historical and inactive material

[Research](research/README.md) contains dated proposals, experiments, screenshots,
and verification receipts. Its older feature descriptions and test counts are
historical, not current availability or release guarantees. Rich chat has been
replaced by Inbox's original-terminal view.

[Mobile Companion](../plugins/companion/README.md) and its
[development protocol](companion-protocol.md) are retained for future work;
Companion is excluded from the current desktop build.

The [17 September documentation audit](research/2026-09-17-documentation-audit.md)
records the review scope, corrections, validation, and remaining limits. When
changing behavior, update its current guide and acceptance checks together;
preserve dated evidence and annotate it when later decisions supersede it.
