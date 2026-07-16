---
type: grilling
blocked_by: []
---

# Who commits, and how work gets abandoned

## Question

History is linear and there are no branches (ADR 0003), so every commit lands in the space's one working tree and nothing is quarantined anywhere. Decide precisely who writes which commit, and how work is undone.

Known: an implementing session commits its code plus a `## Proposed Answer`, and the harness promotes that to `## Answer` at approval (ADR 0004). Open:

- Is the promotion **its own commit**, or amended into the session's? One resolved ticket could be one clean commit, or an honest two that show proposed-then-blessed. The second is more truthful; the first is more pleasant to read later.
- Agents commit **on their own initiative**, and each does it differently. Can the harness rely on that, constrain it (message format, granularity, "never push"), or must it take commits over entirely — and can it even do that across agnostic agents?
- **Abandon / discard / re-grill** from the human-review hub: `git revert`, `reset` (the ticket's commit is the tip, since sessions serialise), or something gentler? And what happens to the `## Proposed Answer` — deleted, or kept as the record of a rejected attempt that a later session should not repeat?
- A session that **dirties the tree without committing**, or dies leaving work in progress (ticket 02 decides how that is detected; this decides what is done with the mess).
- Does the harness ever **push**, or is the remote strictly the human's business? "Linear history" is only a promise the harness can keep locally.
