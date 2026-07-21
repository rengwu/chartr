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
