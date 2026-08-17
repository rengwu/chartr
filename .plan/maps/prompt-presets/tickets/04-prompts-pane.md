---
type: task
blocked_by: [01, 02, 03]
undermined_by: []
---

# Add the minimal Prompts pane

## Question

Expose the completed catalog, launch selection, and live-delivery behavior in one
small Prompts pane beside the existing Map tool. The pane is scoped to the selected
registered space and always targets its currently active terminal; selecting a
different sidebar tab is the existing target picker.

Each catalog row needs its name and text, an `At launch` control, and the one action
its target state permits: Send when idle, Queue when working/running/blocked, or a
plain explanation when the active tab is ineligible. One pending row reads
“Queued for next idle” and can be cancelled. Provide simple create, edit, and
delete affordances in this pane. Map and Prompts are mutually exclusive uses of
the existing auxiliary-pane presentation; do not build a generalized pane system,
target dropdown, prompt organizer, or settings section.

## Done when

- The operator can open and close Prompts without disturbing terminal lifetime or
  Map state, and Map and Prompts never compete for the same auxiliary space.
- Catalog CRUD and `At launch` changes use the server actions and confirm through
  the authoritative snapshot rather than a second client-side store.
- Send, Queue, queued identity, Cancel, and ineligible explanations follow the
  active terminal's model state and use the exact narrow vocabulary in the spec.
- Switching spaces or active tabs immediately retargets the pane through existing
  selection behavior; no target selector or cross-space action is introduced.
- The pane remains compact and keyboard/accessibility-correct, with no decorative
  or organizational features beyond the workflow.
- Focused frontend tests pass, followed by `make test`, `make vet`, and
  `make check` for the complete map.

