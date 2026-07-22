# Ticket state is derived from `.plan`; `## Proposed Answer` is promoted at the gate

State splits in two, with no overlap and no dual-writing. **Ticket state** lives in the `.plan/` markdown and is *derived* by the reused model layer — the agent writes, the chartr watches. **Session state** (which agent, which PTY, running or dead) is the chartr's own and lives nowhere near the map.

This also answers how an agent-agnostic chartr knows a session finished: it does not ask the agent. Resolution already *is*, by wayfinder's own design, an `## Answer` appearing in the ticket file — so the chartr watches `.plan/` and re-derives.

The wrinkle is that the model layer reports a ticket `resolved` the instant `## Answer` exists, which would unblock its dependents before any gate had passed, and leave the map on disk lying about what is settled. So an implementing session writes **`## Proposed Answer`** — a heading the frontier scan (`^## (Answer|Ruled out)`) does not match, leaving the ticket correctly unresolved — and the chartr promotes it to `## Answer` only at final approval.

## Consequences

- On disk, `resolved` always means human-blessed.
- `proposed` is itself *derived* (`## Proposed Answer` present, `## Answer` absent), so it survives a chartr crash rather than living in chartr memory.
- The chartr extends the markdown adapter's derived-status table; it does not replace it, and a vanilla wayfinder tool reading the same map still reads it correctly.
- The chartr's notion of *takeable* is stricter than wayfinder's frontier: a blocker must be approved, not merely answered. **That hold is the containment** — it is what stops a wrong-but-committed ticket from seeding the dependents that would inherit its error.

## Amendment: the `## Proposed Answer` extension is withdrawn (simplify, ticket 03)

The split this ADR opens with **survives untouched**: ticket state is derived from `.plan/` markdown, session state is the chartr's own, and the chartr knows a session finished by watching an `## Answer` appear — never by asking the agent. What is withdrawn is everything the *gate* hung off that split.

- **`## Proposed Answer` is no longer a status.** `StatusProposed`, its derivation, and the promotion-at-approval write are gone with the review feature. The derived-status table is exactly wayfinder's four values again — open, claimed, resolved, out_of_scope — so the chartr now extends the markdown adapter by *nothing*, rather than by one value.
- **The frontier is no longer stricter than wayfinder's.** A blocker unblocks its dependents the instant it is resolved; there is no approval to wait on. `resolved` means "the session said so", not "human-blessed".
- **The containment is forfeited, knowingly.** The hold that stopped a wrong-but-committed ticket from seeding its dependents was the review pipeline's, and it went with it. What replaces it is social and visible, not mechanical: the operator is present, the claim and the resolution are on the star-map, and every write is a commit in the log. This is the trade the simplify spec makes deliberately — the cut exits the judgment business.
- **In-flight wreckage is ignored, not migrated.** A ticket still carrying `## Proposed Answer` is reading an unknown heading: it derives `open`, or `claimed` if its claim marker survived, and never `resolved`. The context bundle likewise stops treating it as an answer — a dependent is never handed an unblessed proposal as though a human had blessed it.
