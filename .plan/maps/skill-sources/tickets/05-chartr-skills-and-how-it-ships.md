---
type: grilling
blocked_by: [04]
claimed_by: s73174bffe493
claimed_at: 2026-08-06T05:26:24Z
---

# chartr-skills, and how it ships

## Question

`chartr-skills` is a new repo — a minimal subset of Pocock's skills carrying the four role skills — registered as chartr's default source, vendored into the binary as a seed so a first run works offline, and updated by `git fetch` thereafter. This ticket settles what goes in it and how the seed and the pin work together.

The interesting tension: the seed is bytes in the binary, and the pin is a commit in a repo. A binary built in August and refreshed in December has two versions of the same source and no obvious rule for which is right.

Settle:

- **The contents.** Four role skills at minimum. Does `wayfinder`, `to-spec`, `to-tickets` or `research` ship there too — they were on the essential list, and a session that can chart a map is more useful than one that cannot, but each is also a method skill an operator might prefer to source elsewhere. Say what ships and, for each, why it is not better left to the operator's own sources.
- **How chartr-specific each skill may be.** These are supposed to be generic skills; [The conventions ruleset](04-the-conventions-ruleset.md) extracts what is chartr's. What is the acceptance test — can a skill in this repo mention `## Answer` at all, given the ruleset already states it? A skill that restates the contract puts it in two places, which is exactly what `docs/skill-sync.md` warns about today.
- **Seed versus pin.** What the seed records about itself (a ref? a build stamp?), what a first run writes to the cache, and what happens when a refresh moves the pin ahead of the seed and then the operator upgrades chartr to a build carrying a newer seed. Does the seed ever overwrite a fetched checkout, and does an operator who has never refreshed see a source that silently changed under them at upgrade?
- **The vendoring mechanic.** Today `internal/prompt/assets/skills` is `go:embed`ed and `docs/skill-sync.md` describes a manual per-skill diff-and-triage procedure. With an upstream repo of chartr's own, most of that procedure's reason for existing is gone. What replaces it — a build step that vendors a pinned ref, or a checked-in copy synced by hand — and what is left of `SourceCommit`.
- **What happens to provenance.** `hashFiles`, `ShippedHash`, `forked_from` and the stale-fork warning were built for a vendored library operators forked in place. Forking is now "register your own source above the default", which has no drift to detect. Say which of that machinery dies and which the claim trailer still needs.
- **The repo's own contract.** `docs/skill-sync.md` states what every shipped skill must satisfy. Does an equivalent live in `chartr-skills` itself, and does chartr validate anything about a source's skills at registration — or is a malformed `SKILL.md` simply a skill that does not resolve.

## Done when

The repo's skill list is fixed with a reason per entry, the seed/pin/upgrade interaction is specified including the overwrite rule, the vendoring mechanic is chosen, and every piece of the current provenance machinery is marked kept-and-repointed or deleted.
