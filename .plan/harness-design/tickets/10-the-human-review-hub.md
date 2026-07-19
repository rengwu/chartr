---
type: prototype
blocked_by: [06, 08]
claimed_by: claude-fable-5
claimed_at: 2026-07-18T19:04:38Z
assets: [.plan/harness-design/assets/10-human-review-hub.html]
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

Prototype: [10-human-review-hub.html](../assets/10-human-review-hub.html) — open in a browser; four hub variants inside the E "Helm" shell, cycled with ←/→ or `?variant=`; `?` explains how each answers the ticket; ↺ resets the fixture. Round two, from review (too intimidating; distilled per impeccable's distill/clarify references): **D Brief is canonical and default** — one screen with *what was done* (the `## Proposed Answer` verbatim), *what the reviewer found* (one line + the blocking finding), and *recommended* (derived mechanically from the verdict — no agent editorializes at the gate); Done-when check, findings and diff collapse behind expanders; approve is one click on a pass, plus exactly one acknowledgement tick ("I've read the blocking finding") on a reject; the brief is assembled as plain markdown on disk (`{ } raw` shows the file), so a TUI user reads the same text. Send-back is the human's feedback channel: its dialog shows the fix-up session's briefing (standard bundle + blocking finding always; advisories opt-in) plus an optional human note that rides in the injected payload and its archive (ticket 04) — never the ticket file, which only abandonment writes to. Round one, kept for the record: A **Dossier** (contract-first reading order, approve *below* the diff, unread-section nudges surfaced not enforced), B **Counsel** (findings-first; blocking findings gate approve on an explicit disposition — send back, or waive with a recorded reason), C **Evidence** (diff-first PR posture, findings pinned in the margin — the familiar, weakest gate, kept for comparison). Shared everywhere: the forced-arrival banner (#06's rejecting verdict — rejection halts, never loops), the per-Done-when-clause verdict table (a finding may only block by citing a clause — taste is advisory by rule), the abandon dialog (reason addressed to the next attempt; revert as an unticked lever), take-it-further (follow-ups accumulate; the `## Proposed Answer` is rewritten in place; diff scopes: all / since verdict / since last read), and the post-approve strip (next best frontier ticket as a suggestion whose button can never inherit the approve click). Fixture: expensif / export-csv-impl with #03 proposed-pass and #06 proposed-reject.
