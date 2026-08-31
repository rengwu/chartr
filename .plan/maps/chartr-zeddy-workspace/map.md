# Chartr workspace rewrite

## Destination

Chartr is a coherent, themeable multi-space terminal and plugin workspace whose
ownership, panes, actions, settings, persistence, and interaction conventions
closely follow Zed while retaining Chartr's product behavior and visual identity.
The settled product contract is recorded in [the specification](./spec.md).

## Notes

- [Specification](./spec.md)
- The application is user-facing **Chartr**, but this rewrite keeps configuration,
  state, and runtime data isolated under the `chartr-zeddy` namespace.
- Zed is the architectural, component, accessibility, and interaction reference.
  Go Chartr is the current visual-design reference; Chartr-rs is the settings and
  Herdr-lifecycle reference.

## Decisions so far

<!-- The settled decisions are captured in spec.md; this planning map has no tickets. -->

## Not yet specified

<!-- The grilling session exhausted the current design frontier. -->

## Out of scope

- Windows support, pending a non-Unix Herdr transport.
- Terminal scrollback, pending a real Herdr history source.
- Cross-space item movement.
- Terminal mirroring, preview tabs, and pinned tabs.
- Automatic migration from existing Chartr installations.
- Phosphor or user-selectable application-control icon sets.
