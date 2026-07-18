---
type: grilling
blocked_by: [01]
---

# The prompt library the harness injects

## Question

Because the harness cannot lean on any one agent's skill mechanism (ADR 0002), it ships **its own prompts** and injects them to wire a session to a role. What are they, and where do they live?

The wayfinder method already exists as skills written for Claude Code. Decide the relationship, because every option costs something: **vendoring** a copy as prompt text means drift from upstream wayfinder; **pointing** agents at a checked-out skills directory means depending on files the harness does not own and cannot version; **synthesising** condensed per-role prompts means maintaining a second, quietly diverging expression of the method.

Settle:

- **The role set** — grill, prototype, research, implement, review — and whether each needs one prompt or several. A `review` prompt in particular must carry the ticket's **Done-when and its spec**, not merely "review this diff": hand a reviewer only the diff and the gate silently degrades to a style check, losing spec conformance entirely.
- **Composition** — how a role prompt combines with the context bundle (ADR 0005) into a single injected payload, within the size limits ticket 01 turns up. Note the injection mechanics may differ between a headless spawn and an interactive PTY — ticket 02 owns that split; this ticket inherits whatever it decides.
- **Overridability** — whether a space's committed config may replace or extend a role's prompt. A project with house rules will want to, and that wish collides with the harness's interest in prompts it can reason about.
- **Versioning** — what happens when the harness's prompts change underneath a half-driven map, and whether a map should record which prompts resolved its tickets.

## Answer

**Vendored — and hackable.** The harness ships its own prompt library, seeded from the wayfinder skills and adapted away from their Claude-Code-specific conventions (frontmatter, skill-invocation mechanics), owned and versioned by the harness, recording the upstream commit each sync was taken from. The surface is small enough to actually review on sync: the entire relevant method is ~760 lines of markdown. Pointing at a checked-out skills directory was rejected — it makes the harness depend on files it cannot version and cannot guarantee exist on an operator's machine — and synthesis was rejected as the quietly-diverging second expression the vendoring option at least keeps traceable. Surfaced here and now standing in the map's Notes: **the client is hackable.** The prompts are plain markdown, visible on disk, and editable by the operator — never sealed inside the binary.

**The library is five role prompts plus a common core.** One file per role — grill, prototype, research, implement, review — plus a `core` file carrying what every role must hear (ticket 06's commit conventions, *never push*, the glossary pointer), injected before the role prompt so shared rules exist once rather than drifting across five copies. Per-map-kind variants were rejected: the map's kind already selects which roles ever spawn, and near-identical variants double the library for nothing. The review prompt's known failure mode — degrading into a style check — is closed by assembly, not prose: the harness composes the ticket's Done-when and the spec into every review payload, so the reviewer cannot be handed only a diff.

**Three layers, most-specific-wins, with both replace and append.** Resolution per role walks space committed config → user config dir → embedded defaults. The defaults live in the binary so a bare install works, and materialise (`eject`, or first run) into the user config dir as plain markdown. At any layer, `<role>.replace.md` resets the base and `<role>.append.md` stacks on top, applied in layer order — so a project can carry house rules as a small committed append without forking the shipped prompt, or replace it wholesale when it means to. The cost is owned openly: a replace forks that file, and the cockpit surfaces "your prompt is behind the shipped default" rather than ever auto-merging.

**Composition and injection: one path — a file plus a one-line opener.** At spawn the harness composes core + role prompt + context bundle (ADR 0005) into a single markdown payload, writes it to a gitignored path inside the space, and the opening prompt typed into the TUI is one line: read that file and proceed. One assembly path for every agent (ADR 0002), no per-TUI paste limits or `[pasted text]` collapse, and the exact injected payload becomes a visible on-disk artifact — hackability again. The accepted reliance: the agent's first act must be reading the file, and an agent that doesn't is visible in its own pane — surfaced, not enforced, like every other violation. This also closes ticket 02's flagged loose end: the per-TUI injection question shrinks to "can the adapter type one line into it," which is the PTY's whole job.

**Versioning: trailers plus archive.** The claim commit (ticket 06's mechanism) gains trailers recording which layer won each part of the payload and a content hash — the greppable trail. The composed payload file itself is retained per session in harness-owned state outside git, so the exact text a session was told is rereadable word for word; it already exists at spawn, so keeping it is nearly free. Payloads assemble at spawn, so a prompt edit never touches a live session — a half-driven map simply gets newer prompts for later tickets, with the record saying which sessions got what.
