chartr is the cockpit that drives this repository: it derives maps of tickets from
the files under `.plan/maps/` in this working tree and spawns one agent session
per ticket.

A file under `.plan/maps/` is read by chartr only where it follows the format stated at `.chartr/TRACKER-CONVENTION.md`.

---

# Context

## Skill sources

The skills chartr can resolve, in the order it resolves them.

- `chartr-skills` at `.chartr/skills/chartr-skills` — grill, implement, prototype, research, to-spec, to-tickets, wayfinder
- `matt-pocock` at `.chartr/skills/matt-pocock` — ask-matt, code-review, codebase-design, diagnosing-bugs, domain-modeling, grill-with-docs, implement, improve-codebase-architecture, prototype, research, resolving-merge-conflicts, setup-matt-pocock-skills, tdd, to-spec, to-tickets, triage, wayfinder, wizard, claude-handoff, loop-me, setup-ts-deep-modules, writing-beats, writing-fragments, writing-shape, git-guardrails-claude-code, migrate-to-shoehorn, scaffold-exercises, setup-pre-commit, grill-me, grilling, handoff, teach, to-questionnaire, wait-what, writing-for-agents
- `impeccable` at `.chartr/skills/impeccable` — impeccable
- `emil-kowalski` at `.chartr/skills/emil-kowalski` — animate, animation-vocabulary, apple-design, ask-sonner, emil-design-eng, find-animation-opportunities, improve-animations, pick-ui-library, prototype, review-animations

Where two of them carry a skill of the same name, the earlier one is what a bare name reaches, and the later one is reached as `source/skill`.
