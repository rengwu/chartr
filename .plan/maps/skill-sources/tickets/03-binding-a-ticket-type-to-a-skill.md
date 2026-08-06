---
type: grilling
blocked_by: [01]
claimed_by: s5dcc337e877c
claimed_at: 2026-08-06T07:39:26Z
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

## Answer

**The table is a flat scalar table in `user.toml`, beside `[agents.*]`: four `role = "Source/skill"` lines, nothing more.**

```toml
[roles]
grill     = "chartr-skills/grill"
prototype = "chartr-skills/prototype"
research  = "chartr-skills/research"
implement = "chartr-skills/implement"
```

- **File confirmed: `~/.config/chartr/user.toml`** (`internal/server/spaces.go:22`), the same file `SetUserAgent` already writes (`internal/config/useragent.go`). ADR 0009's "two paths on disk" for the user layer was bindings-vs-skills; this is a binding, so it sits with the other binding-shaped thing already there, not with the skill-content paths `sources.toml` owns.
- **Flat, not a subtable.** `[agents.<name>]` needs a subtable because an agent is several fields (`adapter`, `args`, `env`, `prompt`). A role binding is one fact — a qualified skill reference — so `[roles.grill]\nskill = "..."` would be a subtable holding exactly one key, forever, unless a binding grows a second field nobody has named a need for. YAGNI cuts it to the flat form.
- **Always qualified, never bare.** A bare name (`grill = "grill"`, resolved through source order at spawn time) was the real alternative and it is **rejected**: it reintroduces the exact implicit capture the explicit table exists to prevent — reordering sources, or a higher-priority source later shipping its own `grill` skill, would silently change what every grilling ticket runs, with no line in `user.toml` showing it happened. The qualified form is also the one ticket 01 already built the resolver for and explicitly handed forward here (01 §"What this hands the neighbouring tickets").

**Seeding: write all four rows once, the first time chartr starts with no `[roles]` table at all — never partially, never again.**

- **Trigger.** The ticket worried about a race between "first run" and "first run after the default source materializes." There isn't one: ticket 05 reconciles `<configDir>/sources/chartr-skills` unconditionally on every startup, before any spawn is possible. So "seed if `[roles]` is absent" can run immediately after that reconciliation, on the same startup, with the default source already guaranteed present. No ordering flag, no deferred write.
- **Existing config.** The presence test is the table, not the file. An operator upgrading into this ticket has a `user.toml` with `[agents.*]` and no `[roles]` — that reads as absent, seeds once, same as a brand-new install.
- **Idempotent by construction.** The write is guarded by "table missing." A second startup sees the table (even one row is enough) and never touches it again — matching "four bindings written once... and never rewritten" in the map's decisions verbatim.
- **A deleted row is not auto-restored.** Auto-refilling any startup that finds fewer than four rows would silently overwrite an operator's own edit — including a deliberate deletion, which is itself a legitimate way to say "let this role refuse until I rebind it." Recovery is instead an explicit, single-row action: a "restore default" control on the settings surface (the map's own "not yet specified" entry) that writes exactly `role = "chartr-skills/<role>"` back through the same comment-preserving TOML surgery `SetUserAgent` already uses (`internal/config/tomlsurgery.go`) — one field, in place, nothing else in the file disturbed. No CLI verb is needed for this; the settings surface is the only place a role binding is ever edited, same as `[agents.*]`.

**Unresolvable binding: refuse the spawn. No terminal, no claim commit.** *(confirmed with the operator — this was the ticket's central call)*

- Falling back to the default source was already off the table before this session started: ticket 01 settled that a qualified reference "never falls through" (01 §Resolution) — a role binding is always qualified, so honoring that precedent here is following the blocker, not re-deciding it.
- That leaves refuse vs. spawn-on-`core`-alone-with-a-warning. **Refuse wins.** The operator picked "grill" (or any role) for its behavioural contract; degrading silently to bare `core` hands the ticket to an agent with no interrogation stance and no idea it's missing one, and this map's review gate is gone entirely (ADR 0008's simplify amendment: chartr writes only claim + release, the agent writes its own `## Answer` straight to the map, nothing checks it). A bad grill Answer under a name nobody flagged as degraded is a wrong decision baked into the map; a refused spawn costs one click after the operator fixes the binding or reaches for the always-available escape hatch — an "empty shell" free session (ticket 02) needs no binding at all.
- **This needs no new code path.** `prompt.Compose` already returns `(Payload, error)` and `launchSession` already aborts on that error *before* `writeClaimCommit` runs (`internal/server/spawn.go:270-299` — the same short-circuit `agentSpec`'s failures use a few lines up, at `spawn.go:123-127`). Resolving a role's binding through `sources.Registry.Resolve` instead of today's `prompt.Resolve(role, roots)` is a swap inside that call; an unresolvable binding is just one more reason `Compose` returns an error, handled by machinery that already exists. The error names the role, the recorded binding string, and which of the three unresolvable shapes it hit (disabled source / removed source / skill missing from that source) — mirroring the specificity every row in ticket 01's failure-mode table already carries.

**The claim trailer becomes `Skill: <name>=<source>[@<commit>]:<hash>`.**

- `<source>` replaces `<layer>` (`internal/server/claim.go:205`) — the resolved `Source.Name`, always knowable at spawn since resolution has already succeeded by the time the trailer is written (refusal happens first, per above).
- `<hash>` is unchanged: the content hash over the skill directory's files (`hashFiles`, `internal/prompt/prompt.go:224`), still knowable at spawn and still the thing that answers "did this session run against exactly this text."
- `@<commit>` is appended only when the source carries a pin (a `git`-kind row, or a `chartr-skills` row that ticket 05 gives an operator-owned `.git` pin). A `dir` source has no commit and the trailer simply omits the segment — there is nothing to pin. This answers the ticket's question directly: yes, a git source's pin appears in the trailer, opportunistically, because it is strictly more audit trail for the sources that have one and asks nothing of the sources that don't.
- Downstream note for whichever ticket touches `internal/prompt/prompt.go`'s `Skill` struct (07, migrating off the layer model): it needs `Source string` and `Commit string` (empty for `dir`) where `Layer` sits today; this ticket fixes the trailer's grammar, not that struct's fields.

**`RoleForTicketType` and `config.Roles` are kept, unchanged, with the binding riding on top of them rather than replacing them.**

`Role` already carries weight independent of skill selection: it is the stable key the spawn gate offers all four values under, it drives the AFK/HITL quiet hint (`internal/config/binding.go:1-7`'s own package doc), it validates external input at the spawn boundary (`config.IsRole`, `spawn.go:84`), and it is what the claim trailer's separate `Role:` line already records. None of that vanishes if the `[roles]` table vanished tomorrow — the table is one more consumer of an enum that has three other reasons to exist, not a naming layer built solely to be looked up in four rows. The `[roles]` table is therefore `map[config.Role]string`, keyed by the existing closed type; `RoleForTicketType` is untouched.

**Four is fixed, and stays fixed for the life of this map's model.** The table's row count follows wayfinder's ticket types (`grilling`, `prototype`, `research`, `task`), which this map's own "Out of scope" refuses to redesign. A fifth row means wayfinder itself grew a fifth ticket type — an event outside this effort entirely — and when it happens the change is mechanical: one more entry in `config.Roles`, one more seeded line, one more row on the settings surface. Nothing about today's design has to change shape to admit it; it just isn't a case that exists now.

### Rejected

- **Bare-name binding, resolved through source order at spawn.** Reintroduces implicit capture — the exact failure the explicit table exists to prevent. See "always qualified" above.
- **Per-role subtable (`[roles.<role>]\nskill = "..."`).** Structurally mirrors `[agents.<name>]` for no reason: a binding has one field today and no named case wants a second. Four subtables holding one key each is ceremony the flat table doesn't pay.
- **Falling back to the default source on an unresolvable binding.** Contradicts ticket 01's already-settled "a qualified miss never falls through," and reintroduces implicit capture from the other direction — the operator would silently get `chartr-skills`'s `grill`, not the one they bound.
- **Spawning with `core` alone plus a visible warning.** Never blocks work, but hands a role-typed ticket to an agent missing exactly the behavioural contract the operator chose the role for, with no review gate downstream to catch a bad result. Rejected in favour of refuse — confirmed with the operator as this ticket's central call.
- **Auto-refilling missing rows on every startup.** Would silently overwrite an operator's intentional deletion of a row — itself a legitimate way to force a role to refuse until rebound. Recovery is an explicit, single-row settings action instead.

### Revisit trigger

**If a binding ever needs more than one field** — a per-role model override, a per-role fallback list, anything beyond "which skill" — the flat table is revisited first; the qualified-string grammar and the file location do not need to move, only the shape of one row. **If refuse-at-the-gate turns out to block real work often enough to hurt** (a source flapping, a shared machine where sources churn), the warning-plus-`core` path is the one to reopen, and it costs nothing today's design forecloses — `prompt.Compose` already has the error path to soften into a warning later.
