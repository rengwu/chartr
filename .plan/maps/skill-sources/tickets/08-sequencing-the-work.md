---
type: grilling
blocked_by: [01, 02, 03, 04, 05, 06, 07]
claimed_by: s30d77be9aa83
claimed_at: 2026-08-06T09:39:23Z
---

# Sequencing the work

## Question

The last ticket on the map. With every decision settled, this one orders them into something `to-spec` and then `to-tickets` can turn into an implementation map — and decides what, if anything, ships before the whole effort does.

The effort has an awkward shape: it deletes a working system and the replacement is not useful until several pieces land together. A registry with no bindings resolves nothing; bindings with no seeded source resolve to nothing; payloads that point at a ruleset that does not exist point at nothing. There may be no honest half-way state, and saying so is a legitimate answer.

Settle:

- **The tracer bullet.** What is the thinnest end-to-end slice that actually runs — plausibly: seed materializes, one source resolves, one binding spawns a ticket session. Say what it is and what it deliberately leaves broken.
- **Whether anything ships independently.** The new-shell control is close to separable — it is a UI change over a payload that could start as today's `core`. The conventions ruleset is separable in the other direction: it can be written and materialized while the layer model is still live. Identify what can genuinely land alone and what only looks like it can.
- **The deletion order.** Deleting the layer model early makes everything after it simpler and leaves the tree broken meanwhile; deleting it last means every intermediate ticket carries both models. This repo has taken both approaches before and the `simplify` map's precedent is worth reading before choosing.
- **What the effort does to the documentation.** `CONTEXT.md` loses several terms, ADR 0009's content half is amended, `docs/skill-sync.md` is largely superseded, `docs/getting-started.md` describes the skill library, and `CLAUDE.md` describes the maps convention. Which are ticket-sized work in the implementation map and which are a single documentation pass at the end.
- **Whether a new ADR is owed.** The layer model is ADR 0009's content half and this effort deletes it. An amendment to 0009 in the style the file already uses, or a new ADR that supersedes that half outright — the file has four amendments already and a fifth may be one too many.
- **What is verified before it is called done.** The repo's standing bar is `go vet`, `go test`, the frontend `check`/`build`/`vitest`, and reading a composed payload in the cockpit's preview. Say what this effort adds — plausibly a first-run test on a clean config root, and an offline first-run test, since the seed exists for exactly that case.

## Done when

An ordered sequence exists that a `to-spec` session can consume without re-deciding anything, naming the tracer bullet, what ships independently, the deletion order, the documentation and ADR work, and the verification bar for the effort.
