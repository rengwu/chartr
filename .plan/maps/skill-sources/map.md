# skill-sources — chartr stops shipping skills

## Destination

A decision set for replacing chartr's embedded skill library with **registered skill sources**: an ordered list of local folders and pinned git repos the operator owns, resolved by name, with chartr itself shipping no skills at all — only two core payloads and a conventions ruleset that tells whatever skill is running how this project writes maps, tickets and specs.

The map is done when every decision an implementation map needs is settled — nothing left to decide before `to-spec` and `to-tickets`. **Plan, don't do:** this map produces decisions, not code.

## Notes

**Read before choosing a ticket:** [`CONTEXT.md`](../../../CONTEXT.md) for the vocabulary (several terms die here — layer, built-in skill, on-ramp, skill launcher) and [`docs/adr/`](../../../docs/adr/) for what is currently settled. This map **amends ADR 0009**: its content half — the three-layer skill model, whole-skill shadowing, "content the project ships wins" — is what this effort deletes. It **reaffirms ADR 0002** (chartr composes its own payload and leans on no agent's skill mechanism) and **ADR 0005** (context assembled fresh, zoomed on demand). [`docs/skill-sync.md`](../../../docs/skill-sync.md) is largely superseded and a ticket must say what replaces it.

**Skills every session should consult:** `grill-me` for the grilling tickets; `research` where a ticket needs primary sources. At the end of this map: `to-spec`, then `to-tickets`. Skills live at `/Users/rengwu/Desktop/Projects/skills/pocock`.

**The standing preferences that bind this effort.** chartr is a **cockpit, not an autopilot**. The client is **hackable** — what chartr injects is visible on disk as plain files. And the rule this map exists to honour: **the operator's repo carries no chartr operating artifacts**; `.plan/maps/` is committed work product, everything else lives under chartr's own config root.

**YAGNI binds every ticket on this map.** Design for what this effort needs today, not for what a source list, a payload or a ruleset could conceivably want later. A field, a kind, a flag, a hook or an extra file earns its place by a *named case that exists now* — a failure mode someone has to handle, a decision already settled above, or a need another ticket on this map states in writing. "Someone might want to" is not one, and neither is symmetry with a mechanism sitting next door in the codebase: precedent is a reason to copy a *stance*, not to copy machinery whose original pressure is absent here. Where a session is torn, the smaller model plus a recorded revisit trigger beats the larger one — this map produces decisions, and a decision that can be widened later is cheaper than a knob that has to be carried forever. The last entry under `Out of scope` is the precedent: the source list *is* the extension point, and a second mechanism would be speculative bloat.

**What is being kept, and is not up for re-litigation:** the spaces/sessions model, the PTY terminal layer, the star map, the wayfinder map format itself, the agent library and its per-spawn selection, and the one-session-per-space invariant (ADR 0003).

## Decisions so far

<!-- Settled with the operator on 2026-08-05 in the grilling session that cut this map, before any ticket existed. Recorded here because the tickets below are scoped *around* them. Do not re-litigate. -->

- **The sources registry replaces the three-layer model.** One ordered list; `resolve(name)` is the first source containing `<name>/SKILL.md`. `prompt.Names()`, `prompt.Roots{}` and the built-in/user/workspace layer tags all go. Rejected: sources as a fourth layer beside the three (four concepts where there were three, and the closed name set survives); replacing only the middle layer.
- **Every source row carries `enabled`.** The default source is pinned — not removable, not reorderable — but can be toggled off like any other.
- **Payload prompts are not skills.** What ships embedded is only the two core payloads and the conventions ruleset. No skill ships inside the binary, so the toggle is pure discovery and there is nothing structural for it to break.
- **The skill launcher, its dropdown and the context modal are deleted.** In their place the space card grows a `new shell` split button: `empty shell` (spawns no agent) above a divider, then the registered agents. `on-ramp:` and `needs-context:` frontmatter die with it, which also empties the old dual-use overlap between role prompts and cold launches.
- **Two core payloads.** A **free session** (agent launched from the split button) is told chartr's capabilities and the registered sources by name and location only — no behavioural instruction. A **ticket session** gets today's `core`.
- **Ticket types and roles remain** — `research`, `prototype`, `grilling`, `task`.
- **Role resolution is an explicit `[roles]` table, seeded on first run.** Four bindings written once against the default source and never rewritten. The table is the whole answer, not a patch list, so reordering sources is a discovery change and never a behavioural one. Rejected: convention-with-override (a well-named skill in a high-priority source would silently capture a ticket type); unseeded (a required onboarding step for no gain).
- **`chartr-skills` is a separate repo** — a minimal subset of Pocock's skills carrying the four role skills — registered as the default source, **vendored into the binary as a seed** so a first run works offline, and thereafter updated by `git fetch` like any other source.
- **The conventions ruleset is embedded, materialized to a stable path, and pointed at — never inlined.** A path is a capability, not a behavioural instruction, so it fits inside the free-session payload's budget. It is unshadowable because it is chartr's contract rather than anyone's preference. Rejected: keeping it a skill in the default source (an operator could disable chartr's own file-format contract).
- **Refreshing a git source is always an explicit act,** against a recorded pin. Nothing fetches unattended: a registered source is executable instruction reaching agents that run with permissions skipped, and a repo trusted on Tuesday must not silently become something else on Friday.
- **chartr writes nothing into the operator's repo but `.plan/maps/`.** `docs/agents/issue-tracker.md` and `.chartr/skills/` are both deleted.
- **Agents in terminals chartr did not launch are out of scope** (see Out of scope).

<!-- Resolved tickets, indexed as they land. The link is the machine-readable part. -->

- **[The skill sources registry](tickets/01-the-skill-sources-registry.md)** — a chartr-owned `sources.toml` whose row order *is* resolution order, with the `chartr-skills` row synthetic and always last. Bare names search enabled sources top-down; a qualified `Source/skill` addresses one source and never falls through. Git sources are shallow clones under the config root, refreshed only on an explicit act.
- **[The two core payloads](tickets/02-the-two-core-payloads.md)** — two payloads from five interchangeable parts, four shared. The free-session payload carries no live facts about the space, and chartr's own voice in it is held to the *ignore test*: every sentence chartr writes must still be true if the agent does nothing about it. The ticket payload concatenates the bound role body rather than pointing at it.
- **[The conventions ruleset](tickets/04-the-conventions-ruleset.md)** — one canonical, generated `<config-root>/conventions.md`, pointed at by every payload, with method teaching kept out of it. It replaces both `docs/agents/issue-tracker.md` and the `tracker-convention` skill. A user-owned `preferences.md` is appended verbatim after it, which **amends the decision that a free session receives no behavioural instruction**.
- **[chartr-skills, and how it ships](tickets/05-chartr-skills-and-how-it-ships.md)** — seven skills ship (the four roles plus `wayfinder`, `to-spec`, `to-tickets`), vendored as a checked-in copy of a pinned commit. `.git` presence is the ownership marker: absent means chartr's bytes, reconciled at startup; present means the operator's pin, never overwritten. Nearly all provenance machinery dies; `SourceRepo`/`SourceCommit` survive as `SeedRepo`/`SeedCommit` and the `Skill:` trailer is re-keyed to name the source.

## Not yet specified

- **How the settings surface renders sources and role bindings.** Sources have order, a pin, an enabled flag, a kind, and — for git — a ref and a refresh action; role bindings are four rows pointing into them. What that screen is, and how much of it is editable versus open-the-file, cannot be drawn until the registry's shape and the binding's failure modes are settled. *Anchored to [The skill sources registry](tickets/01-the-skill-sources-registry.md) and [Binding a ticket type to a skill](tickets/03-binding-a-ticket-type-to-a-skill.md).*
- **Supporting files inside a source's skills.** A skill directory may carry more than `SKILL.md`; today the glossary is one such file and the context bundle inlines it. Whether sourced skills' supporting files are addressable, and how a payload refers to one, is downstream of what the conventions ruleset absorbs. *Anchored to [The conventions ruleset](tickets/04-the-conventions-ruleset.md).*
- **Whether a space can pin a source.** Global sources cannot express "this project's skills", which is the one thing the deleted committed layer did. Whether that need is real, and whether a per-space subset or ordering answers it without putting artifacts back in the repo.
- **Naming collisions in free-session lookup.** A free session resolves "Matt Pocock's prototype skill" from prose against a list of sources. Ticket sessions are safe by construction — bindings are explicit — but free sessions address skills by name and source-name, and two sources may ship the same skill name. *Anchored to [The skill sources registry](tickets/01-the-skill-sources-registry.md).*

## Out of scope

- **Reaching agents in terminals chartr did not launch.** Decided: chartr guarantees what it launches, which after this effort is every session it starts. No managed block in `~/.claude/CLAUDE.md`, no mirroring into a harness's native skills directory, no `chartr` shim on PATH. All three were weighed and refused — each buys partial coverage in exchange for per-harness knowledge and a file outside chartr's own config root.
- **`domain-modeling`, ADR tooling, and `ideate`.** Culled as non-essential to the star map and ticket system. Nothing replaces them; the skills simply stop shipping, and anyone who wants them registers a source that has them.
- **Redesigning the wayfinder method.** Same boundary every map here has kept: chartr drives wayfinder maps and restates the format; it does not change it.
- **A plugin or extension API for skills.** The source list *is* the extension point, and it is a directory of markdown. A second mechanism would be the speculative bloat the `simplify` map already refused.
