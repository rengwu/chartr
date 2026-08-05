---
type: grilling
blocked_by: [01]
---

# Binding a ticket type to a skill

## Question

A ticket says `type: grilling`. chartr ships no grilling skill. The `[roles]` table is the whole answer — four explicit bindings, seeded once on first run against the default source, never rewritten. This ticket settles what that table is and, mostly, what happens when it is wrong.

The failure modes are the substance here, because every one of them is now reachable by ordinary operator action: disabling a source, deleting a folder, refreshing a pin onto a commit where a skill was renamed. Under the old model none of these existed — the embedded floor could not go missing.

Settle:

- **The table's shape and home.** Is a binding a qualified string (`"chartr-skills/grill"`), a pair of fields, or a bare skill name resolved through the source order? It belongs in the operator's global, never-committed layer beside the agent library — confirm that, and say which file, since ADR 0009 already notes the user layer is two paths on disk.
- **Seeding.** What exactly triggers the write — first run, or first run *after* the default source materializes? What if the operator has an existing config? Is the seed idempotent, and how does an operator who deleted a row get it back without hand-editing TOML?
- **Unresolvable bindings.** A binding pointing at a disabled source, a removed source, or a skill that no longer exists in the source it names. Does the spawn refuse, fall back to the default source, or spawn with core alone and a visible warning? Refusing is honest and blocks work; falling back silently re-introduces the implicit capture the explicit table exists to prevent. This is the ticket's central call.
- **What the claim commit records.** `Skill: <name>=<layer>:<hash>` (`internal/server/claim.go:205`) has no layer any more. Source name and pinned ref are the obvious substitutes, and the trailer is written at spawn *before* the agent runs (ADR 0008), so whatever it records must be knowable then. Say what the trailer becomes and whether a git source's pin appears in it.
- **`RoleForTicketType` and `config.Roles`.** The type→role mapping (`internal/config/binding.go:55`) survives, but role is now an indirection to a binding rather than a skill name. Is `Role` still a closed type, or does the type map straight to a binding key? A closed role set that exists only to be looked up in a four-row table may be a layer of naming with nothing left in it.
- **Whether four is fixed.** The types are fixed by the map format, so the table has exactly four rows forever. Confirm — or say what a fifth would mean.

## Done when

The binding's on-disk shape, its seeding rule, and its behaviour under every unresolvable case are settled; the claim trailer's new form is written down; and `RoleForTicketType` / `config.Roles` are either kept with a stated reason or collapsed into the table.
