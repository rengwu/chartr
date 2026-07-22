# Map kind is declared in committed config, never inferred

> **Superseded by [0015](0015-map-kind-removed-role-comes-from-the-ticket.md)
> (the `kind-cut` map).** Map kind is removed from chartr entirely: a
> session's role comes from the ticket's own `type:`, and a discovered map is
> live rather than inert-until-classified. The deciding premise below — that the
> two kinds have different lifecycles — was struck by this ADR's own amendment
> when review was cut, and the ground the amendment kept (kind selects the role
> set) turned out to be redundant with per-ticket `type:`. Nothing below is
> current; it is kept whole, addendum and amendment included, as the record of
> how the decision moved.

A map is either a planning map or an implementation map, and the two have different lifecycles — an implementation ticket passes through review before it resolves, a planning ticket resolves live. chartr must know which kind it is looking at *before it offers any action*, because getting it wrong either gates a conversation that needed no gate or lets code resolve unreviewed. The candidate signals — the `.plan/<slug>-impl/` directory suffix, every ticket typed `task`, the Notes carrying execution — are each individually breakable (a hand-written implementation map follows none of them; wayfinder explicitly permits a `task` ticket and a Notes override on a planning map), so we do **not** infer kind from them at read time. Kind is an **explicit declaration**, recorded in chartr-owned config committed to the space's repo and keyed by map slug; the signals survive only as a one-time *guess* chartr proposes when it first meets a map, for a human to confirm.

## Considered options

- **Purely inferred from conventions** — rejected: a hand-written implementation map reads as planning and its code resolves unreviewed, silently. The exact failure the deciding ticket warns about.
- **Declared in the map body** (`kind:` frontmatter on `map.md`) — rejected: a second extension of the wayfinder markdown adapter beyond the single non-resolving heading ADR 0004 grants, into a map-level frontmatter slot `TRACKER-MARKDOWN.md` never defined. A vanilla wayfinder tool would meet a field the tracker spec does not know.
- **Chartr-local, uncommitted registry** — rejected: the declaration would be per-machine, so every fresh clone or teammate re-classifies and two operators can disagree in silence.
- **Per-ticket kind, derived from `type:`** — rejected: it would gate a planning map's lone `task` ticket against a spec that does not exist, and make the stricter frontier rule (a blocker must be *approved*, not merely answered) apply to some edges and not others on one map.

## Consequences

- Kind is a property of the **map**, uniform across its tickets; mixed lifecycles on one map are unrepresentable. This is what `CONTEXT.md` already asserts, and it is what lets agent review critique a proposed ticket against a spec — only an implementation map has one.
- The committed config layer stays the home for shared, versioned, portable declarations; wayfinder's own format is untouched, so a vanilla tool reads the same map unchanged.
- A map whose kind is not yet declared is **inert until a human classifies it** — it renders and is readable, but chartr offers no session actions until confirmation, with the inferred guess pre-selected. No lifecycle ever runs on a heuristic. This is the first chartr-owned per-map state beyond a space's committed config, and the space registry (the discover-and-classify flow) owns it.
- Graduating a finished planning map into an implementation map is something chartr **notices** — it detects the new `.plan/<slug>-impl/` directory and surfaces it for classification — not an action it offers. chartr stays a cockpit *over* wayfinder rather than a wrapper that wires the method's skills into itself.
- A renamed map directory dangles its config entry; that resolves into unclassified-and-inert, not an error.

## Addendum — kind is recorded at creation; the fallback confirm lives in the panel

The original slice made classification a per-map confirm the operator answers *after* discovery, and rendered that confirm inline in the sidebar — one `kind? plan / impl` row per undeclared map. In practice that hoisted a rare, one-time decision into the top-level nav and cluttered it, and it left chartr guessing for every map because nothing recorded the answer up front.

Two adjustments, neither of which touches the core decision (kind is declared, never inferred; no lifecycle runs on a heuristic; an undeclared map is inert until a human confirms):

- **The declaration is recorded at map creation**, not (only) reactively. Charting a map already knows its kind — `wayfinder` produces a planning map, `to-tickets` an implementation map — so the creating session writes the `[maps."<slug>"]` declaration into `.chartr/config.toml` then, byte-identical to what `config.DeclareMapKind` appends, and commits it with the map. This is the same committed-config declaration this ADR already mandates; it just happens up front. The seam is chartr's wayfinder adapter (`docs/wayfinder-adapter.md`), consulted through the skill's own "consult the adapter for this repo" rule — the cockpit stays an observer of map creation, it does not orchestrate it. Maps created outside that path still fall through to the confirm below, unchanged.
- **The reactive confirm moves out of the sidebar into the star-map panel.** An undeclared map now shows only a quiet dashed marker in the sidebar (no action); the `plan / impl` confirm — guess pre-emphasised, `p` / `i` to declare — surfaces inside the panel when the map is opened. It is the same classify action against the same endpoint; only its location changed, from per-row nav clutter to one confirm at the surface where the operator is already looking at that map.

Net effect: recording-at-creation makes the confirm rare, and relocating it makes the rare case unobtrusive. The invariant is intact — an unclassified map is still inert until a human declares its kind.

## Amendment: kind selects the role set, not a lifecycle (simplify, ticket 03)

One premise of this ADR is **struck**: that the two kinds have different lifecycles. They no longer do. With the review feature gone, a planning ticket and an implementation ticket resolve the same way — the session writes `## Answer` and commits — so "getting the kind wrong lets code resolve unreviewed" is no longer a failure mode, and the stricter-frontier rule the per-ticket-kind option was rejected over does not exist any more.

The decision itself stands, on its remaining ground: **kind is declared in committed config, never inferred**, because kind still selects **which roles a map offers** (planning grills, prototypes and researches; implementation implements), and an undeclared map still offers none — inert until a human classifies it. Recording kind at creation (the addendum above) is unchanged, and so is the reason it is committed rather than local: teammates must agree on it.
