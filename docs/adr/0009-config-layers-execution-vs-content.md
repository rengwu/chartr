# Config layers: workspace commits shared content and defaults; the local user layer wins for execution

Two layers of chartr config live in different places for different reasons. **Committed workspace config** sits in the space's repo (the file ADR 0007 already established for map-kind) and is shared, versioned, portable. **Local user config** lives under the operator's home, is never committed, and is per-machine. When both speak to the same thing, which wins is **not uniform** — and that asymmetry is the decision:

- **Role→agent bindings resolve user-over-workspace.** A committed binding names a concrete CLI and model, which is an *execution/environment* fact — it may name an agent the operator never installed, or a model they don't want to pay for. It must yield to local reality, so the user layer wins. This is also what makes the absent-agent case solvable by configuration rather than by editing someone else's committed file.
- **Prompts resolve space-over-user (ticket 04, unchanged).** A committed prompt is shared project *content* — deliberate customization that should apply to everyone working the repo — so the space layer wins.

The rule that reconciles them: **content the project ships wins; execution choices the operator makes win.**

Two things follow at the boundary of "what may be committed":

- Bindings are `{adapter, model, args?}` — structured, so chartr can reason about them (compare models for heterogeneity, probe binaries for presence). Machine-specific absolute paths do not belong in the committed layer; that is what the user layer is for. The `args` escape hatch exists for flags the adapter does not model, and using it knowingly forfeits chartr's introspection on that binding.
- **Autopilot has no committed representation.** Disabling both reviews is a per-machine, disclaimed choice; a committed autopilot flag is ignored with a warning. Committing "no human reviews this code" for everyone who clones is the exact drift "cockpit, not autopilot" exists to prevent.

## Consequences

- Heterogeneity (`implement ≠ review` model) is **not enforced in config** — it cannot be an invariant (model strings can alias one backend, and the `args` hatch can swap the effective model), so it is surfaced as an *observed*-model line in the human-review brief (ticket 10) rather than guarded at resolution time. chartr always allows; judgment lives at the one human gate.
- Merge is field-level: a user override may set one field and inherit the rest, so chartr renders the **effective** resolved binding to keep silent inheritance visible.
- The committed config file gains a second tenant beside map-kind. The local user layer is the first chartr state that is neither a space's committed config nor per-map — its home is `~/.config/chartr/`, keyed by space, and the space registry (ticket 11) owns its lifecycle.

## Considered options

- **Uniform direction — one layer always wins** — rejected: space-always-wins makes the absent-agent case unsolvable in config (you cannot override the committed CLI you lack), while user-always-wins lets a personal preference silently override the project's deliberate prompt content.
- **Committed autopilot, honored on clone** — rejected: it resolves a teammate's code unreviewed without their consent from first run.
- **Committed autopilot, confirmed on clone** (mirroring ADR 0007's declared-but-confirmed map-kind) — rejected: even gated behind a local confirmation, it gives the flag a committed meaning the standing preference says it should never have. Autopilot stays purely local.

## Amendment: the autopilot bullets lapse (simplify, ticket 03)

The mechanism this ADR decides is **untouched**: two layers, field-level merge, bindings resolving user-over-workspace and content resolving space-over-user, under the same reconciling rule — content the project ships wins, execution choices the operator makes win.

What lapses is autopilot. With no review to disable, "autopilot" names nothing, so the **"Autopilot has no committed representation"** bullet and the two rejected committed-autopilot options are historical rather than operative, and the resolver's autopilot fields, its ignored-flag warning, and `Resolution.Autopilot` are deleted without replacement — they resolved a value nothing consumed. An `autopilot` key in either layer is now simply an unknown key: ignored, unwarned.

The heterogeneity consequence lapses with it — there is no `review` binding to differ from `implement`, and no review brief to surface an observed model in.

## Amendment: the layers gain a surface, and an edit boundary (simplify, ticket 05)

The mechanism is again **untouched** — the same layers, the same field-level merge, the same reconciling rule. Two things are added on top of it.

**These layers now have a surface.** The effective config surface (ADR 0014) is where this ADR's asymmetry becomes visible: a global `#/settings` route renders every resolved value with the layer it came from and the file that layer lives in, so "silent inheritance stays visible" is a screen rather than a promise. The per-field provenance this ADR made the resolver record is what that screen is built out of.

**The edit boundary follows from the asymmetry.** A UI may write **only the user layer, and only role bindings** (`[spaces."<path>".roles.<role>]`). This is not a policy bolted on top — it is what user-over-workspace *means*: the user layer is where an operator's execution choice belongs, so writing there is always correct and always overridable by nothing. Writing the committed layer from a local UI is refused outright: it is shared content the operator's teammates receive on clone, and a screen that labels a value "workspace" must not then edit it on their behalf. Content (skills) is not editable from the UI at all, in either layer — it resolves the other direction, and the shipped-wins rule makes an in-app edit the wrong instrument. Everything the surface does not edit gets an open-the-file hatch instead.

One practical consequence to record, because the surface has to show it: the user layer is **two files**, not one. Bindings resolve from `<dataDir>/user.toml` while the user *skill* layer lives at `<configDir>/skills/` (adopted with the skill repackaging, ticket 04). One layer in this ADR's sense; two paths on disk, both named on the surface rather than papered over.

## Amendment: bindings gain a name — the agent library

The layering is again **untouched**: the same two layers, the same field-level merge, the same reconciling rule. What changes is the *unit* an operator works in.

Repeating `{adapter, args, prompt}` per role was fine while a binding was two short fields, and stopped being fine the moment real flags arrived. The flags that matter — `--dangerously-skip-permissions`, `--yolo`, `--sandbox danger-full-access`, an `--add-dir` list — are properties of *how you are willing to run a harness*, not of a role, and they are identical across the roles you run that way. So they get a name:

- **An agent is a complete, self-describing launch spec** — `[agents.<name>]` = `{adapter, args?, prompt?}`. Adapter is the only required field; everything a harness wants beyond its own name is args.
- **A role assigns to one by name** — `[spaces."<path>".roles.<role>] agent = "<name>"` — and the agent then supplies the *whole* binding. Not three of its fields plus a leftover: a role runs one registered way of driving a harness, and taking part of an agent would launch something nobody registered. A role table that sets both is resolved in the agent's favour, with a warning naming the lines that stopped mattering.
- **Roles that name no agent resolve exactly as they always did**, field by field across the three layers. The library is an addition, never a migration, and there is no shipped agent — an empty library is the starting state.

**The library is global; assignment stays per space.** This is the one place this ADR's "keyed by space" user layer grows an unkeyed section, and it is deliberate: which agents exist is a property of the *machine* (its PATH, its logins, how much rope its operator wants), while which role runs which agent is a property of the *work*. Registering once and assigning everywhere follows from that split, and so does the safety property — the library is never committed, so no `git pull` can hand a teammate a permission-skipping agent. The edit boundary from the previous amendment is unchanged and now covers a second table: a UI writes the user layer only, `[agents.*]` and `roles.<role>.agent` included.

**Flags are an opaque list, deliberately.** The surface offers no curated per-CLI toggles. chartr cannot know what any given flag means to the harness that defines it, and a menu of them would make the library exactly as agent-specific as ADR 0002 refused to be — it would also silently exclude every harness not on the menu. The honest substitute is the command preview under each agent, built by the same seam that builds the real argv, so what the operator reads is what will run.

Deleting an agent leaves assignments pointing at it. That is not an oversight: the delete reports which roles it stranded, and each stranded role resolves to a visible explanation while falling back to its own fields. A library edit that quietly rewrote a space's bindings would be precisely the kind of action this surface exists not to take.

## Amendment: `model` is not a binding field

`model` is retired from bindings and from agents alike. It is a **flag**, and it lives in `args` with every other flag:

```toml
[agents.claude-yolo]
adapter = "claude"
args    = ["--model", "sonnet", "--dangerously-skip-permissions"]
```

The field only ever looked structural. In practice it was one CLI's spelling promoted to a schema: `--model` is Claude's and Codex's, and a harness that spells it `-m`, configures it in its own file, or has no model concept at all had to route around a first-class field that did not fit. Line 12's claim that structure lets chartr "reason about" bindings has already lapsed twice — the heterogeneity comparison it justified went with the review gate, and PATH presence is probed from `adapter`, not `model`. What remained was a field chartr stored, rendered, and passed through without ever reading. That is not structure; it is a guess with a text box.

Three consequences, all simplifications:

- **The adapter seam models exactly one thing about a CLI: prompt delivery.** `ModelFlag` is gone. Delivery stays modelled only because chartr itself must *behave* differently depending on the answer (type keystrokes, or don't); no flag has ever needed that.
- **The claim trailer records `Args:` instead of `Model:`.** Strictly more of the audit trail, not less: the argv is what actually ran, so the model appears where one was asked for *and* the permission and sandbox flags appear beside it — which is what the trail is read for. The subject line drops its `· model` suffix.
- **The shipped defaults express their model as args** (`args = ["--model", "opus"]` for grill, sonnet for the rest), so behaviour is unchanged for anyone who never touched their config.

A config that still sets `model` is **surfaced, never honoured** — one warning per binding and per agent, naming the args line to write instead. Nothing is migrated automatically, precisely because migration would mean guessing the flag name a given harness wants, which is the guess this amendment exists to stop making. Editing an agent through the surface clears the dead key on the way through.

## Amendment: map kind lapses from the committed layer (kind-cut, ticket 04)

The two-layer mechanism this ADR decides is **untouched**. What lapses is the
other tenant it kept naming: **map kind is removed from chartr entirely**
(ADR 0015, superseding 0007). So line 3's parenthetical is provenance only —
committed workspace config was first established for map-kind, and no longer
holds it — and the consequence that "the committed config file gains a second
tenant beside map-kind" now reads the other way round: role bindings and the
committed skills layer are its tenants, and a space with no
`.chartr/config.toml` at all is fully supported. The rejected
"committed autopilot, confirmed on clone" option stands as written; the
declared-but-confirmed map-kind it mirrored is simply gone, which strengthens
rather than weakens the rejection.
