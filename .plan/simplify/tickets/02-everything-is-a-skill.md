---
type: grilling
---

# Everything is a skill

## Question

The operator's directive: everything the harness injects becomes a **`SKILL.md`** — the role prompts, the core "how to use this harness" prompt, and the tracker convention (map format, kinds, statuses, the planning-vs-implementation ticket flow). Maximum hackability and openness, on the open standard every major agent CLI now reads natively. The current system — vendored markdown parts with a bespoke `<part>.{replace,append}.md` layering convention and fork-provenance markers — was designed pre-standard (harness-design ticket 04). This ticket designs its replacement.

The tension to grill: the standard wins openness and lets operators reuse the same skills *outside* the harness, but the harness gave up control it used to have — it could reason about parts, compose them deterministically, and stamp provenance into claim trailers. A `SKILL.md` directory is a heavier unit than a prompt part (frontmatter, supporting files, a name/description contract). And the three-layer merge semantics get stranger: what does "append" mean to a skill directory?

Settle:

- **The skill set.** What ships: one skill per role (grill, prototype, research, implement — and ideate?), a core harness-usage skill, a tracker-convention skill. One skill or many for the convention material? Where does the glossary live? What happens to the review prompt — deleted with ticket 01, or kept as an unshipped example of the extension seam?
- **Location and layering.** Built-in (embedded, materialised to the data dir) ‹ user (`~/.config/wayfinder-harness/skills/`?) ‹ workspace (committed, `<space>/.wayfinder-harness/skills/`?). How do layers combine for *directories* — whole-skill shadowing (most-specific skill wins outright), or some merge? Is the replace/append convention preserved per-file inside skills, or dropped as bespoke in favour of plain shadowing? What happens to fork-provenance and stale-fork warnings?
- **Injection.** Today the harness composes one payload file and types a one-line opener. With skills, the options diverge: (a) keep composition — skills are sources the harness assembles into the payload as today; (b) materialise skill dirs into the agent CLI's *native* skills path for the session (per-adapter knowledge of where each CLI reads skills); (c) point the opener at skill files on disk and let the agent read them. Each trades determinism, agent-agnosticism, and openness differently. Pick one, and say what the context bundle (map body, ticket, blockers' answers) becomes — it is not a skill; does it stay a composed payload section?
- **Management and sync.** The skills are vendored from the upstream skills repo today. As SKILL.md dirs, does the harness keep vendoring (drift risk the old ticket accepted), point at a checkout, or stop owning content and ship only the convention? What does the hackable surface look like on disk after first run?
- **What the harness still guarantees.** Ticket 04's answer leaned on composition to *force* certain content (the old review prompt's Done-when + spec). With injection opened up, which guarantees survive in code, and which degrade to convention?

## Answer

**The prompt library becomes a skill library — standard `SKILL.md` directories on disk — but the harness keeps composing.** The format opens up; the injection path does not. Each role, the core, and the tracker convention ship as vendored skill directories the operator can read, edit, and reuse *outside* the harness; at spawn the harness still assembles core + role + a freshly-built context bundle into one gitignored payload and points the agent at it with a one-line opener, exactly as today (harness-design ticket 04, retained). The openness win is the on-disk format and the standard's name/description contract; the control the harness keeps is deterministic assembly. Layers combine by **whole-skill shadowing** — most-specific skill wins outright, no per-file merge, no bespoke `replace`/`append`. This supersedes harness-design ticket 04's prompt-library answer and **reaffirms ADR 0002** rather than amending it: the harness still injects prompts it composes itself and leans on no agent's skill mechanism.

### The skill set

**Seven skills ship:** five role skills — `grill`, `prototype`, `research`, `implement`, `ideate` — plus a `core` skill and a `tracker-convention` skill. This is today's library minus `review` (deleted by ticket 01, not this ticket's to keep) and re-homing the two non-role parts:

- **`core`** — the "how to use this harness" skill: what a session is, the ground rules, commit conventions, *never push*, how work is recorded. Today's `core.md` body, now a `SKILL.md`. Injected first for every role, unchanged.
- **`tracker-convention`** — one skill, not several, carrying the map format, kinds, statuses, and the planning-vs-implementation ticket flow. **The glossary lives inside it** as a supporting file (`glossary.md`), because the glossary *is* the method vocabulary this skill documents; splitting convention material across two or three skills fragments the name/description contract for no gain. The composer still pulls the glossary text into the context bundle (below) — the skill is its *source*, the bundle is its *delivery*.
- **`ideate`** stays a skill like the rest but keeps its special injection: composed alone, no core, no context bundle, since an ideate session is ticketless and mapless (unchanged from ticket 15). Being a skill directory changes its packaging, not its wiring.

**The review prompt is deleted, not kept as an example.** Ticket 01 already removes `prompts/review.md` and its asset; shipping a `review` skill "as an example of the extension seam" would be a zero-consumer artifact — exactly the speculative bloat ticket 01 refused when it rejected emitting anything for the hypothetical reviewer. The extension seam is documented convention (ticket 01's five seams), not a shipped skill nobody loads. If review returns, it vendors its own skill then.

Rejected: **splitting convention into map-format / lifecycle / glossary skills** (three thin skills fragment one coherent body and multiply the sync surface); **folding the glossary into `core`** (the glossary is method vocabulary, not harness house rules — it belongs with the convention it defines, and `core` already carries the harness's own rules).

### Location and layering

Three layers, unchanged in precedence (ADR 0009, content half): **built-in** (embedded, materialised to `<dataDir>/skills/`) ‹ **user** (`~/.config/wayfinder-harness/skills/`) ‹ **workspace** (committed `<space>/.wayfinder-harness/skills/`). Workspace still wins for content.

**Layers combine by whole-skill shadowing.** The most-specific layer that defines a skill of a given name wins its whole directory — `SKILL.md` and every supporting file — and the layers beneath it are not consulted for that skill. The bespoke `<part>.replace.md` / `<part>.append.md` convention is **dropped**: it was pre-standard machinery (this map's own framing), and "append to a *directory*" never had a clean meaning. Replace was already whole-file shadowing under another name; append is the only real loss.

- **The append affordance is knowingly given up.** Today an operator can add a house rule as a small committed `append` without forking. Under shadowing, a one-line tweak means shadowing the whole skill and owning it. This is the accepted cost of killing bespoke convention — and it is softened, not eliminated: because a shadowing skill is a plain copy, the diff against the built-in default is small and legible, and the stale-fork warning (below) tells the operator when their copy has drifted behind.
- **Fork provenance moves into frontmatter.** The `<!-- forked from <hash> -->` HTML-comment marker becomes a `forked_from:` field in the shadowing skill's `SKILL.md` frontmatter — cleaner, and it rides the standard's own frontmatter rather than a comment the composer has to peel. The **stale-fork warning survives unchanged in spirit**: if a shadowing skill's `forked_from` hash differs from the built-in's current content hash, the cockpit surfaces "your `grill` skill is behind the shipped default; re-fork it." Never auto-merged (story 47, retained).
- **What a content hash covers** grows from one file to a directory: the hash is over the skill's `SKILL.md` plus its supporting files in a stable order, so a change to a supporting file is not invisible to drift detection.

Rejected: **per-file merge inside a shadowed directory** (resurrects exactly the bespoke layering the operator is cutting, now harder because it must reconcile file *sets*, not just one file's text); **keeping a lightweight `append`** (the operator chose to end bespoke convention outright — offered and declined).

### Injection

**Composition is retained; the source format changes, the payload path does not.** At spawn the harness reads the resolved `core` and role skills' `SKILL.md` *bodies* as the prompt text, assembles them with the context bundle into one markdown payload written to `run/<sid>/`, and the opener typed into the PTY stays one line: read that file and proceed. One assembly path for every agent (ADR 0002), the exact injected payload still a visible on-disk artifact (hackability), provenance trailers still recording which layer won each part (below).

- **The context bundle stays a composed payload section — it is not a skill.** The map body, the ticket, its blockers' answers, and the glossary are per-spawn material assembled fresh (ADR 0005), never a durable skill directory. The glossary portion is now *sourced* from the `tracker-convention` skill's supporting file, but it is still delivered as a `# Context` part, exactly as `glossary.md` is today. Nothing about "everything is a skill" turns the ticket you were handed into a skill.
- **Supporting files are on disk, not in the payload.** The composer injects the `SKILL.md` body only. A skill's supporting files (the glossary, any reference the operator adds) sit on disk for the agent to zoom into on demand (ADR 0005's "zooms on demand") and for reuse outside the harness — an openness win that costs the payload nothing.
- **`SKILL.md` frontmatter is metadata, not prompt.** `name` / `description` / `forked_from` are stripped before the body reaches the payload — they drive the cockpit's listing and drift detection and the standard's own tooling, and they never leak into what the agent is told.

Rejected — **materialise into each agent CLI's native skills path** (injection option b): it is per-adapter path knowledge, it is non-deterministic (the agent's skill mechanism decides *whether* to load the role skill by description-match, so a session could run its whole ticket never having read its role), and it directly reverses ADR 0002. The operator's "agents read them natively" goal is real, but it is served by the *format* being standard and on disk — not by handing the wiring to the agent. Rejected — **point-and-read** (option c): the opener points the agent at the skill dir to read itself. It still forces the harness to compose the context bundle (so it saves nothing structural), and it trades away the guarantee that the role prompt was read for a marginal nativeness the format already delivers.

### Management and sync

**Keep vendoring.** As `SKILL.md` directories the vendored surface is still a few hundred lines of reviewable markdown, still records the upstream commit per sync (`SourceCommit`, bumped on each sync), still accepts the drift risk harness-design ticket 04 already weighed and took. After first run the hackable surface is `<dataDir>/skills/<name>/SKILL.md` (+ supporting files), materialised and editable, with a README explaining the shadow model; the committed workspace overlay lives at `<space>/.wayfinder-harness/skills/`.

Rejected: **point at an external skills checkout** (the harness would depend on files it cannot version and cannot guarantee exist — rejected in harness-design ticket 04 and still true); **stop owning content, ship only the convention** (a bare install would spawn sessions with no role wiring at all — the wired role prompts *are* the harness's value, and "open" does not mean "empty").

### What the harness still guarantees

**The content-forcing guarantee is already gone — and its loss is ticket 01's, not this ticket's.** Harness-design ticket 04 leaned on composition to *force* the review payload to carry the Done-when and the spec. Ticket 01 deletes the review role and that composed guarantee (`compose.go`'s review branch) with it. So there is no content guarantee left for this ticket to preserve or weaken; the question resolves to *which structural guarantees survive the format change* — and because injection option (a) keeps composition, they all do:

- **Deterministic assembly survives in code.** The harness always composes core + role + context bundle into the one payload and always points the session at it. That the session is wired to its role and its context is a fact the code enforces, regardless of what the agent then does with it. Had we chosen option (b) or (c), *this* is the guarantee we would have traded to convention; we did not.
- **Provenance survives in code**, re-keyed from parts to skills: the claim commit's trailers still record which layer won each composed part and the content hash, and the stale-fork surfacing still fires — now over a skill directory's hash rather than one file's.
- **Degrades to convention (as it always did):** whether the agent reads the injected payload, and whether it zooms a skill's supporting files. Both are surfaced-not-enforced — visible in the agent's own pane — exactly as under the old library. The format change moves nothing across the surfaced/enforced line.

### Documentation this answer must rewrite

- **`CONTEXT.md`** — the **Prompt library** term becomes a **Skill library** term (its `_Avoid_: skills` line inverts — it now *is* skills); the **Workspace config** and **User config** entries' "prompts" content-half references become "skills."
- **`docs/wayfinder-adapter.md` / the tracker-convention skill** — the map-format/kind/status/flow material this adapter and the glossary describe is what the new `tracker-convention` skill restates; the adapter doc points at it rather than duplicating it.
- **Code** (for the implementation map, not decided here): `internal/prompt/` moves from `<part>.md` files and the `replace`/`append` overlay to skill-directory resolution with whole-skill shadowing and frontmatter-based fork detection; `prompts/` on disk and `internal/prompt/assets/` become skill directories.

### Revisit trigger

Two tripwires, either sufficient:

- **The append loss bites.** If operators in practice keep forking whole skills to add one-line house rules and the stale-fork churn becomes a real burden — not an anticipated one — a *single* additive affordance returns (a per-skill house-rules file the composer stacks after the body), as a new ticket, without reviving the full replace/append matrix.
- **A native-skills consumer earns it.** If an operator genuinely wants the harness's skills loaded by an agent's *own* mechanism (option b) — reusing them in an external Claude Code session, say — that is served today by the on-disk standard format needing no harness involvement. Only if that stops sufficing does per-adapter native materialisation get re-argued against ADR 0002.
