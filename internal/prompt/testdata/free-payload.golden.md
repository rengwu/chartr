chartr is the cockpit that opened this terminal: it derives maps of tickets from
the files under `.plan/maps/` in this working tree and spawns one agent session
per ticket, and this shell was opened with no ticket and no role.

A file under `.plan/maps/` is read by chartr only where it follows the format stated at `<config>/conventions.md`.

---

# Context

## Skill sources

The skills chartr can resolve, in the order it resolves them.

- `chartr-skills` at `<config>/sources/chartr-skills` — grill, implement, prototype, research, to-spec, to-tickets, wayfinder

Where two of them carry a skill of the same name, the earlier one is what a bare name reaches, and the later one is reached as `source/skill`.
