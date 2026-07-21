---
type: task
blocked_by: [09]
claimed_by: s75c291ca53ca
claimed_at: 2026-07-21T09:29:12Z
---

# The ideate on-ramp

## Question

The one opinionated nudge toward charting (planning ticket 07). An "ideate" affordance spawns an open-ended, ticketless session from a harness-provided starter prompt — on-disk hackable markdown filed explicitly as a non-role on-ramp, so the five-role set stays closed — that prods the user on what's on their mind and, if the idea proves big, suggests escalating to `/wayfinder` in prose only: escalation is advice, never a harness transition or tracked state. Like an ad-hoc shell the session is ticketless, live, un-reviewed, and ends only when the human ends it — the harness never derives finished for it, and no quiet badge applies. It shares only the adapter's spawn primitive with real sessions: no claim commit, no lifecycle.

Done when: the button spawns a live TUI session opened with the starter prompt's payload; process-boundary tests assert no claim commit is written, no ticket is bound, no lifecycle state ever derives for it, and editing the on-disk starter prompt changes what the next ideate session is told.
