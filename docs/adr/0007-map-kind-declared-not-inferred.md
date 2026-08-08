# Map kind is declared in committed config, never inferred

> **Superseded in full by [0015](0015-map-kind-removed-role-comes-from-the-ticket.md).
> Nothing here is operative.**

**What was decided:** that every map is either a planning map or an
implementation map; that chartr must know which *before offering any action*,
because the two had different lifecycles (implementation tickets passed a review
gate, planning tickets resolved live); and that kind is therefore an explicit
declaration in committed, chartr-owned config rather than something inferred
from the `-impl` suffix, the ticket types or the Notes.

**Why it is gone:** review was deleted, which struck the deciding premise — the
two kinds now resolve identically. What the amendment had kept the decision
alive for, kind selecting a map's role set, turned out to restate per-ticket
`type:` less accurately than the tickets already did. Map kind is removed
entirely: role comes from the ticket, and a discovered map is live on no config
at all. 0015 has the full argument.

*Original record in `git log -p -- docs/adr/0007-map-kind-declared-not-inferred.md`.*
