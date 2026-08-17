---
type: task
blocked_by: [01]
undermined_by: []
---

# Compose selected prompts into ticket and free launches

## Question

Make the selection settled in
[Persist the prompt catalog and per-space launch selection](./01-prompt-catalog-and-space-selection.md)
take effect on future Chartr-launched agents using the existing payload and adapter
delivery seams.

Ticket payloads carry each selected preset as its own operator prompt part after
global preferences and before context, so preview, spawn, and payload hashing stay
one path. A registered space's free launch creates and opens a small run payload
only when at least one preset is selected; an empty selection preserves today's
bare launch exactly. Do not change `CHARTR.md`, Scratch, skill resolution, global
preferences, claim trailers, or sessions already running.

## Done when

- Ticket preview and spawn compose the same selected preset bytes in catalog
  order, at the specified position, with useful per-part identity and provenance.
- The existing payload hash covers the selected bytes without a parallel audit
  mechanism.
- A free agent with selected presets receives a gitignored owner-only run payload
  through the existing adapter opener, under both argv/flag and typed delivery.
- A free agent with no selection still launches bare and creates or injects
  nothing new.
- Changing the space selection affects only later compositions, and Scratch and
  `CHARTR.md` are demonstrably untouched.
- Composition, payload process-boundary, and spawn/free-session tests pass.

