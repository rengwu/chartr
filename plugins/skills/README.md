# Skills

A bundled native plugin, linked into Chartr like Agent. Its pane is intentionally
empty except for a chevron menu leading to **Skill source settings**. The same
page is available through **Settings → Plugins → Skills → Configure**.

The plugin owns its ordered source registry and managed Git checkouts under
`plugin-data/com.chartr.skills/`. Its typed service exposes source content while
the registry remains plugin-owned. This build-time native package is not independently
installable through the Git plugin installer.

- Register local folders with an absolute path or `~/`, including through the
  native folder picker. Local files are only read. Missing and empty folders
  remain registered and show their current status.
- Register remote repositories with an optional branch or tag. The default
  branch is recorded on initial checkout. Refresh is explicit and records the
  current commit. Each registration has its own checkout, so the same repository
  can be registered at different refs without affecting another source.
- Edit reuses the registration modal, preserving position and enabled state.
  Changing a remote URL or ref prepares a replacement checkout. Renaming a
  remote source preserves its checkout.
- Enable/disable and drag to reorder sources; arrow buttons also change order.
  Dragging uses the sidebar's live FLIP sorter: the held row follows the pointer,
  neighbouring rows slide into their new positions, and list edges autoscroll.
  Dropping saves the previewed order; Escape or a failed save restores the
  previous order. Reduced-motion settings skip the animated transitions.
  Skill names are case-insensitive: the first enabled source wins, and later
  duplicates are marked as shadowed. Duplicate names inside one source produce
  a warning; the first discovered path wins.
- Rescan updates local counts. Discovery matches the original Chartr walk:
  directories one to three levels below the source containing `SKILL.md`, sorted
  by path, skipping dot directories and `node_modules`, and not descending into
  a discovered skill's supporting files. Source symlinks are followed within
  that depth bound, as in the original implementation.
- Deletion requires confirmation. Local folders remain untouched; only managed
  remote checkouts are removed. A failed cleanup is logged and leaves an unused
  checkout, never deletes the source repository.

Registry changes use an atomic file replacement. Git work and source scans run
in the background; source mutations are serialized across the plugin's views.
Git has a two-minute limit per command and can be cancelled. Failed preparation
or persistence preserves the registered source and its previous checkout.
Registration and refresh do not execute skill scripts.

The native Skills service now exposes ordered, enabled source content to
dependent plugins. Wayfinder consumes it to compose its agent prompts, respecting
source precedence and exact `source/skill` pins. This plugin does not generate
`CHARTR.md` or mirror skills into projects; supporting resources are read from
the resolved source directory. Standalone Agent sessions are unchanged.
