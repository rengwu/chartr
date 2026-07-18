---
type: prototype
blocked_by: [06, 08]
claimed_by: claude-fable-5
claimed_at: 2026-07-18T19:04:38Z
---

# The human review hub

## Question

The moment the whole design turns on. A ticket is `proposed` and the human must decide. This is not a modal yes/no — it is a hub with four exits: **approve**, **take it further** (more grilling, ad-hoc improvements, asking the model for advice), **abandon/discard/re-grill**, and arriving here *forced*, because an agent-review rejection always halts to a human and never loops.

Make it concrete, and settle:

- **What the human is shown, and in what order:** the diff, the `## Proposed Answer`, the ticket's Done-when, the agent-review verdict. What does a useful verdict summary look like — and how does the human tell "the reviewer is nitpicking" from "the reviewer found a real bug"? A gate people rubber-stamp is not a gate, and this is the surface that decides which one it becomes.
- **How taking it further works.** It spawns more sessions against the same ticket, each adding commits, until the human is satisfied. How does the human see what has accumulated since the original proposal, and does the `## Proposed Answer` get rewritten as it goes?
- **How approve feels.** Approving is what promotes `## Proposed Answer` to `## Answer` and unblocks the ticket's dependents — the single act that lets work compound, and the only containment against a wrong ticket seeding its successors. It should not be a button anyone clicks by accident, and it should not be so heavy that people stop reading.
- **What abandon asks for and what it destroys.** Ticket 06 owns the git mechanics; this owns the human's side of it.
- Where the **"start the next best frontier ticket"** offer appears once approval lands, and whether it is a suggestion or a shove.

Link the prototype as an asset.
