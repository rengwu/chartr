chartr is the cockpit that drives this repository: it derives maps of tickets from
the files under `.plan/maps/` in this working tree and spawns one agent session
per ticket.

This shell is one chartr opened with no ticket and no role.

A file under `.plan/maps/` is read by chartr only where it follows the format stated at `.chartr/TRACKER-CONVENTION.md`.

---

# Context

## Skill sources

The skills chartr can resolve, in the order it resolves them.

- `house` at `.chartr/skills/house` — grill

Where two of them carry a skill of the same name, the earlier one is what a bare name reaches, and the later one is reached as `source/skill`.
