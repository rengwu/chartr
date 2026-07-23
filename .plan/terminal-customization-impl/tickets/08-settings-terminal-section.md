---
type: task
blocked_by: [01]
---

# Settings "Terminal" section

## Question

An operator sees their current terminal settings in the Settings surface and opens
`terminal.toml` in their own editor from there — read-value-plus-open-file, never a
second config store.

Add a "Terminal" section on the **Global** scope of the Settings surface (beside
the user config — these are per-machine cosmetic settings, not per space). It
renders the current effective settings read from the snapshot and an
open-`terminal.toml` row using the existing files-on-disk / open-in-editor pattern
(the same `ConfigLayer` open path the agent-library files use). Built on
design-system tokens and vendored primitives with Phosphor icons; no raw colour,
no amber.

Any parse/validation warnings already flow to the config-warnings surface (ticket
01); this section does not re-implement that — it may point at it.

Done when: the Global scope shows a Terminal section with the current effective
settings and an open-`terminal.toml` row that launches the operator's editor;
built on tokens + primitives; frontend + Go checks pass.
