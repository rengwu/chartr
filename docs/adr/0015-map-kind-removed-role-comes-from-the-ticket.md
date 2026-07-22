# Map kind is removed; a session's role comes from the ticket's own `type:`

Supersedes [0007](0007-map-kind-declared-not-inferred.md).

Map kind is gone from chartr — the field, the config declaration, the classify action, and the inert-until-classified gate. A session's role is selected by the **ticket's own `type:`** (`grilling`→grill, `prototype`→prototype, `research`→research, `task`→implement), every ticket offers all four roles, and the operator picks at the spawn gate. **A discovered map is live**: it renders and spawns the moment chartr finds it, on no chartr config at all.

The decision rests on two findings, one about the ADR being superseded and one about what it was still doing.

**0007's deciding premise was struck by 0007's own amendment.** The ADR was decided on "the two kinds have different lifecycles — an implementation ticket passes through review before it resolves, a planning ticket resolves live", and the failure mode it named was *getting it wrong lets code resolve unreviewed*. The `simplify` effort deleted review, and 0007's amendment says so outright: planning and implementation tickets now resolve identically. The failure mode that justified gating a map on a human confirmation no longer exists. What is left of an ungated spawn is that it opens a session on a ticket — the cockpit's entire purpose.

**The ground the amendment kept was redundant.** The amendment held the decision up on kind's remaining job: it selects which roles a map offers. But the four roles map one-to-one onto wayfinder's own four ticket types, and every ticket already carries its type. A per-map uniform value was standing in for per-ticket data the tickets state *exactly* — `config.RolesForKind` approximated by map what `type:` said by ticket. Kind was not a second fact about a map; it was a lossy summary of a fact the map already carried, in a place the map format itself defines.

**The lossiness was a live bug, and removing kind fixed it.** Wayfinder explicitly permits a `task` ticket on a planning map. Under kind, that ticket's natural role — `implement` — was clamped away to `grill` because the *map* said planning. It now offers `implement`, which is what the person who typed `type: task` meant.

## Considered options

- **Keep kind, drop only the inert gate** — rejected: it leaves the whole apparatus (the field, the config table, the classify route, the picker confirm, the creation-time obligation on every map-charting session) in place to serve a role clamp that per-ticket `type:` already does better. The cost stays and the benefit was never real.
- **Derive kind from the tickets instead of declaring it** — rejected on 0007's own reasoning, which still holds: the conventions (`-impl` suffix, all-`task` tickets) are individually breakable. But the reasoning now argues for deleting kind rather than declaring it — an inferred summary of per-ticket data is strictly worse than reading the per-ticket data.
- **Move kind to the ticket** (per-ticket kind in committed config) — rejected: that is `type:`, which already exists in the map format, and duplicating it into chartr config would be a second source of truth for one fact.
- **Replace the classify gate with a lighter one** — an "activate this map" confirm, a per-space allowlist — rejected outright, and recorded as out of scope on the deciding map. A gate needs a failure mode to contain. With review gone there is none, and a gate kept for its own sake comes back as kind under another name.

## Consequences

- **Given up: the inert-until-classified gate.** Nothing stands between discovering a map and spawning on it. A map dropped into `.plan/` by a `git pull` is immediately spawnable by anyone with the cockpit open. This was a real containment and it is deliberately not replaced; what it contained (unreviewed code resolving) was deleted before it was.
- **Given up: teammate-level agreement about a map.** Kind was committed precisely so two operators could not silently disagree about it. Nothing about a map is declared outside the map any more, so there is nothing to agree on — but the *capacity* for a shared, versioned, per-map assertion is gone, and a future one would need a new home rather than an existing table.
- **The role is chosen later and by a human.** Every ticket offers all four roles at the spawn gate with its `type:`-derived role emphasised, so a wrong role is now a mis-click rather than a mis-declaration — visible at the moment of the act instead of set once and inherited.
- **Committed workspace config loses a tenant.** It holds role bindings and skills; ADR 0009's "second tenant beside map-kind" framing is now the only tenant. A space with no `.chartr/config.toml` at all is fully supported — this repo is one.
- **The wayfinder adapter is gone.** `docs/wayfinder-adapter.md` existed to add exactly one chartr-side step at map creation; with no step it was deleted, and this repo's maps are plain local-markdown with nothing on top. chartr is one step further from being a wrapper around the method's skills and one step closer to a pure observer of them.
- **Old committed configs cost nothing.** `toml.Decode` is non-strict, so a teammate's checkout carrying `[maps."<slug>"]` tables resolves with no error and no warning; the tables are simply not read.
- **If this is wrong, it comes back as a new decision.** Not as kind under another name.
