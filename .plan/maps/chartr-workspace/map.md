# chartr workspace rewrite

## Destination

chartr is a coherent, themeable multi-space terminal and plugin workspace whose
ownership, panes, actions, settings, persistence, and interaction conventions
closely follow Zed while retaining chartr's product behavior and visual identity.
The settled product contract is recorded in [the specification](./spec.md).

## Notes

- [Specification](./spec.md)
- The application is **chartr** throughout the UI and codebase. Configuration,
  state, and runtime data use the `chartr` namespace.
- Zed is the architectural, component, accessibility, and interaction reference.
  Go chartr is the current visual-design reference; chartr-rs is the settings and
  Herdr-lifecycle reference.

## Decisions so far

<!-- The settled decisions are captured in spec.md; this planning map has no tickets. -->

## Not yet specified

<!-- The grilling session exhausted the current design frontier. -->

## Out of scope

- Windows support, pending a non-Unix Herdr transport.
- Cross-space item movement.
- Terminal mirroring, preview tabs, and pinned tabs.
- Automatic migration from existing chartr installations.
- Phosphor or user-selectable application-control icon sets.
