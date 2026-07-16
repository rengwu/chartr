# Ticket state is derived from `.plan`; `## Proposed Answer` is promoted at the gate

State splits in two, with no overlap and no dual-writing. **Ticket state** lives in the `.plan/` markdown and is *derived* by the reused model layer — the agent writes, the harness watches. **Session state** (which agent, which PTY, running or dead) is the harness's own and lives nowhere near the map.

This also answers how an agent-agnostic harness knows a session finished: it does not ask the agent. Resolution already *is*, by wayfinder's own design, an `## Answer` appearing in the ticket file — so the harness watches `.plan/` and re-derives.

The wrinkle is that the model layer reports a ticket `resolved` the instant `## Answer` exists, which would unblock its dependents before any gate had passed, and leave the map on disk lying about what is settled. So an implementing session writes **`## Proposed Answer`** — a heading the frontier scan (`^## (Answer|Ruled out)`) does not match, leaving the ticket correctly unresolved — and the harness promotes it to `## Answer` only at final approval.

## Consequences

- On disk, `resolved` always means human-blessed.
- `proposed` is itself *derived* (`## Proposed Answer` present, `## Answer` absent), so it survives a harness crash rather than living in harness memory.
- The harness extends the markdown adapter's derived-status table; it does not replace it, and a vanilla wayfinder tool reading the same map still reads it correctly.
- The harness's notion of *takeable* is stricter than wayfinder's frontier: a blocker must be approved, not merely answered. **That hold is the containment** — it is what stops a wrong-but-committed ticket from seeding the dependents that would inherit its error.
