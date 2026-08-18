---
type: task
blocked_by: [01, 02, 03]
undermined_by: []
claimed_by: s2ac4980d0616
claimed_at: 2026-08-18T03:54:15Z
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


## Answer

Map and Prompts are now one auxiliary pane with a tab switcher in it, at the
operator's direction: the stage's `Map` button became a single **Show Pane**
toggle (still `M`, still absent over Scratch), and the pane's own bar carries
`Map | Prompts` beside the dock and close chrome. That is a stronger reading of
"mutually exclusive uses of the existing auxiliary-pane presentation" than two
cards competing for the same corner, and it cost one move rather than a new
system.

**The frame moved up one level.** `SpacePane` now owns the `<section>`, the
resize grip, the dock/close chrome, and the tabs; `MapCard` lost `dock`,
`floatWidth`, `onclose`, and `onresizestart` and is simply what the Map tab
shows. Its floating chrome keeps Back and the map name (the dock/close chips are
the frame's now), and the picker's own header went with them — the tab already
says Map. Nothing about the map's behaviour changed: the same visibility,
docking, camera, deep-link, and Esc-peeling state, with the peel now skipping the
map's two inner layers while Prompts is the tab on show, and a star link
switching to the Map tab because that is what the link is about.

**The pane holds no catalog.** `PromptsCard` renders `model.prompts` and
`space.prompts` straight off the snapshot; every control posts one of the five
actions in `actions.ts` and waits for the push. `At launch` is written as the
whole list in catalog order (`setSpacePrompts`), never a per-row toggle, so a
refused write leaves the checkbox exactly as the server has it rather than
half-toggled, and two quick clicks cannot interleave. Create, edit, and delete
are one inline draft form at a time with a two-step delete — a short list of
short texts does not need a modal.

**The action vocabulary is derived once.** `prompttarget.ts` is a pure function
over the active tab: `send` when a Chartr-launched live agent reads idle,
`queue` when it is working, running, or blocked, and `ineligible` with one plain
sentence otherwise — no tab, an exited process, or a tab Chartr did not launch
(a hand-started `claude` reads the agent grammar and can show idle; it is still
refused, which is the case the rule exists for). The pending row reads "Queued
for next idle" with Cancel, and every other row's action stands down while one
is pending rather than offering a button the server would refuse. The target is
always the active tab, named in one line at the top of the pane; selecting
another sidebar tab is the whole target picker.

**Omitted deliberately.** No target dropdown, no per-space prompt store, no
settings section, no reorder/group/tag affordances, and no per-row repetition of
the ineligible explanation. The pane is hidden over Scratch for the same reason
the map is: it has no launch selection and every write there is refused, so a
control that only ever fails is worse than none. `CONTEXT.md` gained the one
vocabulary entry the map introduced (**Prompt preset**), which its Free-session
entry was already referencing in bold.

Tests: `web/src/lib/prompttarget.test.ts` covers the whole state vocabulary —
Send, Queue on each busy state, and each ineligible reason including the
hand-started agent. The 18 web unit test files (198 tests) pass, as do `make
test`, `make vet`, and `make check`; `npm run build` compiles the changed
components. The pane was not driven in a browser.
