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

## Answer

**`chartr-skills` ships seven skills — the four roles plus `wayfinder`,
`to-spec` and `to-tickets` — vendored into chartr as a checked-in copy of a
pinned commit, materialized on startup into the default source's directory, and
owned by the seed until the operator refreshes it exactly once. `.git` presence
is the ownership marker: no `.git` means chartr's bytes and chartr reconciles
them at every startup; a `.git` means the operator's pin and chartr never writes
there again.** The seed/pin tension the ticket names does not need arbitration
because the two versions never coexist: the directory is in exactly one of two
states, and the transition between them is a deliberate human act that runs one
way.

Nearly all of the provenance machinery dies. `hashFiles`, `ShippedHash`,
`Skill.Hash`, `forked_from`, `Skill.Stale`, `staleWarning`, `LibraryWarnings`
and `Materialize` were built to detect drift between a shipped default and an
operator's in-place fork; forking is now "register your own source above the
default", which has no drift to detect. What survives is one constant pair and
one trailer line, both re-pointed.

### The repo

`github.com/rengwu/chartr-skills`, MIT, carrying the attribution
`github.com/rengwu/skills` already carries — `wayfinder`, `to-spec` and
`to-tickets` derive from Matt Pocock's skills and the notice travels with them.
`grill`, `prototype`, `research` and `implement` are chartr originals
(`docs/skill-sync.md` records that) and need none. This is not optional
housekeeping: the seed puts those bytes inside a distributed binary.

| ships | why it is not better left to the operator's own sources |
| --- | --- |
| `grill` | Bound to the `grilling` ticket type by the seeded `[roles]` table. Without it a first run cannot spawn a ticket session at all. |
| `prototype` | Same, for `prototype`. |
| `research` | Same, for `research`. It is a role skill today, not a method skill — the ticket's framing lists it as an open candidate, but `config.Roles` already fixes it as one of the four. |
| `implement` | Same, for `task`. It is also the only skill that knows an implementation map differs from a planning one. |
| `wayfinder` | **Nothing else creates a map.** chartr's whole surface — star map, frontier, ticket spawn, the four role bindings — is inert until a map exists, and a free session is what charts one. A default source that can drive maps but not make one leaves a first run staring at an empty cockpit with no offline way out. |
| `to-tickets` | The planning map's terminus. Without it a resolved planning map goes nowhere and the `implement` role is unreachable, so shipping `implement` without `to-tickets` ships half a loop. |
| `to-spec` | Weakest of the seven, and it still earns its place: ticket 04's fixed layout reserves `.plan/maps/<slug>/spec.md`, and a slot in chartr's own contract with no skill that fills it is a gap an operator has to fill from somewhere else before the layout means anything. |

**Does not ship, and why it is not an oversight:** `core` and
`tracker-convention` stop being skills — they become the two embedded payloads
(ticket 02) and `conventions.md` (ticket 04). `ideate` and `domain-modeling`
were culled by the map's Out of scope. Everything else in Pocock's library
(`handoff`, `grill-with-docs`, `review-code`, …) is exactly what the source list
exists for.

**The operator who prefers someone else's method skills pays nothing.** The
default row sits last (ticket 01), so registering Pocock's repo shadows
`wayfinder` and `to-spec` by bare name in one action. Shipping them costs that
operator one shadowed name; not shipping them costs every offline first run the
ability to start.

**The seed is the repo at the vendored commit, wholesale** — no subset, no
manifest of which skills to embed. Eight hundred lines of markdown does not
justify a selection mechanism, and a subset rule is a second place the skill
list would be written down.

### The acceptance test

A skill in `chartr-skills` is accepted when both clauses hold:

1. **No chartr runtime.** No sentence whose truth depends on chartr running.
   Today's vendored copies fail this repeatedly and concretely — `wayfinder`
   says *"In a chartr space the cockpit does this driving"* and *"the map's
   physical format is the `tracker-convention` skill's contract"*, `to-tickets`
   says *"what turns the ticket green in the cockpit"*. Every one of those is a
   re-authoring, not a word swap: the first two name a product the reader may
   not have, the third names a skill that no longer exists.
2. **No format rules.** The skill does not state a rule `conventions.md` states:
   directory layout, filename numbering, frontmatter fields, the derived-status
   table, the frontier calculation, claim ownership.

**Naming is allowed; specifying is not.** A skill may write `## Answer`,
`map.md`, "the frontier", "a blocker" — a method cannot discuss its own outputs
without naming them, and a role skill whose entire product is one artifact must
be able to say which. What it may not do is be the place a rule is *stated*.
`## Answer` in `grill/SKILL.md` is a name; "a prose-bearing `## Answer` derives
the ticket as resolved" is a rule and belongs only in `conventions.md`.

**No skill carries a section skeleton.** This reverses the one exception
`docs/skill-sync.md` granted — method skills mirroring the convention's section
names in a template — and it is the operator's call, taken with the alternatives
in hand. The reasoning: a frozen token cannot drift, a six-heading skeleton with
prose guidance can, and the two copies are now two repositories rather than two
directories in one. `conventions.md` as ticket 04 wrote it already states both
skeletons, so nothing is lost and ticket 04 needs no amendment.

The cost is real and lands on the vendoring work, not on chartr: `wayfinder` and
`to-tickets` carry method guidance *inside* their placeholders — *"\<what
reaching the end of this map looks like … every session orients to it before
choosing a ticket\>"* — which is method, not format, and disappears with the
template unless it is re-authored as prose. That re-authoring is a required part
of seeding the repo, not a follow-up.

On any disagreement `conventions.md` wins — stated in the repo's own contract,
and reinforced by ticket 04's instruction order, which puts the conventions
pointer after the role body.

**Frontmatter is `name` and `description` only, and chartr reads neither.** This
is a consequence of ticket 02 rather than a new decision: the sources block
lists names without descriptions, and the composer strips frontmatter before
concatenating the body. `on-ramp:` and `needs-context:` are stripped from all
four role skills — they die with the launcher. The two fields stay in the files
for humans and for any other tool that reads the `SKILL.md` standard.

The test's positive form, and the phrase to hold the repo to: **each skill must
be correct for a reader who has never run chartr, given the same
`conventions.md`.** The pair is complete; the skill alone is method-complete and
format-silent.

### Seed, pin and upgrade

The default source's directory `<configDir>/sources/chartr-skills/` is in one of
two states, and which one it is in is read off the filesystem:

| state | how it is recognized | who owns the bytes |
| --- | --- | --- |
| **seeded** | no `.git` | chartr. Reconciled against the embedded seed at every startup. |
| **pinned** | `.git` present | the operator. chartr never writes into it again. |

- **The seed records nothing about itself on disk.** Its identity is the
  `SeedCommit` constant compiled into the binary. A marker file would be a
  second copy of a fact the binary already holds, and it would be the copy that
  goes stale — the exact failure `forked_from` had.
- **First run writes the seed directory and nothing else.** No README (the repo
  brings its own), no marker, no `[[source]]` row — ticket 01's default row is
  synthetic and only its toggle persists.
- **Reconciliation, not blind rewriting.** At startup, when the directory is
  seeded, chartr compares the file set and bytes against the embedded seed and
  replaces the whole directory when anything differs — temp directory then
  rename, the discipline ticket 01 already uses for a clone. Deliberately
  wholesale, so a skill deleted upstream actually disappears; deliberately
  startup-only, because unlike `conventions.md` this is not a parser contract
  and re-materializing before every composition would buy nothing.
- **The overwrite rule, stated flatly: the seed never overwrites a pinned
  checkout.** A fetch is the operator asserting ownership, and an upgrade must
  not silently revert it.
- **Refresh converts seeded → pinned, once.** `git clone --depth 1
  --single-branch --branch main <SeedRepo>` into a temp directory, rename over
  the seeded one, record the commit. From then on it refreshes like any other
  git source (ticket 01) and the seed is dead to it.
- **Deleting the directory is the reset.** No "restore shipped skills" action:
  removing it drops the row back to seeded, and the next startup re-materializes
  it. One rule covers reset, repair, and a directory an operator wrecked.
- **An operator who has never refreshed does get changed role prompts at
  upgrade, without being asked.** That is correct and it is the same stance
  ticket 04 took for `conventions.md`: upgrading chartr is the act of taking
  chartr's bytes, and the default source is chartr's bytes. The mitigation is
  visibility, not a gate — the row shows which build's seed it is carrying.
- **The sharper risk is the inverse, and it is why the pin is displayed.** An
  operator who refreshed once in December and upgraded chartr three times since
  is running December's role prompts against March's chartr, permanently and
  silently. Without a recorded pin they have no way to see it. So:

**Two scalars persist beside `default_enabled`** — `default_commit` and
`default_fetched`, written only by a refresh, absent while seeded. The row then
reads either *"shipped with this build — `<SeedCommit>`"* or *"fetched
2026-08-06 — `9e8b5ea`"* with no filesystem inspection at load time. This
**amends ticket 01's "only its toggle persisted"**, inside the delegation that
ticket made explicitly ("what lands at that path, and how the seed and the pin
interact, is ticket 05's"). It also makes the default row `git`-kinded rather
than `dir`-kinded: it has a known upstream, and the map's settled decision says
it is refreshed by `git fetch` like any other source. Nothing else about the row
moves — still synthetic, still last, still not removable, still not reorderable.

Rejected here: **a frozen default that can never be refreshed** (the operator
who wants newer role skills registers the repo as an ordinary git source above
it). It is genuinely attractive — it deletes this entire section, and it makes
the seeded `[roles]` bindings unbreakable by construction, since a
source-qualified `chartr-skills/grill` would always resolve into frozen bytes.
It was rejected because it contradicts a settled decision of this map for a
gain that is real but smaller than it looks: the operator who registers the repo
above the default then has *two* rows shipping the same seven names, and if
ticket 03's bindings are source-qualified their ticket sessions keep silently
getting the frozen copy — a papercut with no visible cause. Refusing to fetch
does not remove the problem, it relocates it into shadowing.

### The vendoring mechanic

**A checked-in copy of a pinned ref, refreshed by one `make` target.** The seed
lives with whatever package owns sources after ticket 01 —
`internal/sources/assets/chartr-skills/` — not in `internal/prompt`, which this
effort is gutting.

- **Not a build step that clones at build time.** `go build ./...`, `go test
  ./...` and goreleaser all run against a plain checkout in this repo, and the
  Makefile's own standing rule is that a release's contents must be a function
  of the commit, not of the day it was cut. A fetching build would also make the
  test suite need network.
- **Not a submodule.** Decisive on one case: `go install
  github.com/rengwu/chartr/cmd/chartr@latest` and any source tarball carry no
  submodules, so the binary would ship with an empty default source and a first
  run would have no role skills at all.
- **`make vendor-skills`** clones `chartr-skills` at a ref, replaces the vendored
  directory wholesale, and rewrites `SeedCommit`. Three steps, no triage, no
  per-skill diffing: `chartr-skills` *is* chartr's authored repo, so there is
  nothing to reconcile — the adaptation work moved upstream into that repo,
  where someone porting a Pocock skill does it once against `CONTRACT.md`.
- **`docs/skill-sync.md` is deleted.** Its three surviving pieces have homes: the
  vendor procedure becomes the Make target's comment, the shipped-skill contract
  moves into `chartr-skills/CONTRACT.md`, and its format/duplication warning is
  superseded by ticket 04. Nothing in chartr's `docs/` replaces it.

**Deleting that file retires the decision it carried**, and this ticket retires
it out loud rather than by deletion. `docs/skill-sync.md` holds a *"decided
2026-07-22, do not re-litigate"* block with two rules, and no ADR ever recorded
either of them — checked against all sixteen — so the file is their only home.

- ***Re-author, never wrap*** — **survives, relocated.** It becomes
  `chartr-skills/CONTRACT.md`'s rule and now applies upstream, where a Pocock
  skill is adapted once, rather than at vendor time.
- ***No runtime loading*** — **reversed, knowingly.** Its stated reason was
  reproducibility: skills embedded in the binary mean two machines resolve
  identical bytes for the same ticket, which is what made the payload hash a
  provenance trailer worth trusting. Registered sources give that up outright. A
  `dir` source is a folder the operator can edit between two spawns and it
  records no version at all, so "this prompt ran" is no longer verifiable off a
  teammate's machine. What replaces it is weaker and honest about being weaker:
  the `Skill:` trailer *names the source* and pins it where the source is a git
  checkout, and `Payload-SHA256` still fixes the exact bytes for the machine that
  composed them. The map chose an operator-owned, hackable library over
  cross-machine reproducibility; this is the sentence that says so.
- The stale duplicate at the repo root (`/skills/`, gitignored, already drifted
  from the embedded copy) goes with it.

### Provenance, piece by piece

| piece | verdict |
| --- | --- |
| `SourceRepo` / `SourceCommit` | **kept, renamed and re-pointed** to `SeedRepo` / `SeedCommit` on `github.com/rengwu/chartr-skills`. One constant pair, three uses: the vendored pin, the clone URL a refresh uses, and what the seeded row displays. |
| `hashFiles`, `Skill.Hash` | **deleted.** Ticket 02 concatenates the role body into the payload, so `Payload-SHA256` already covers the exact bytes of the only skill a session composes. The hash was a second, weaker copy of that. |
| `ShippedHash` | **deleted.** There is no shipped default for a fork to be behind. |
| `forked_from` | **deleted** as a recognized field. Nothing parses it; a source that carries it has an unknown frontmatter key, tolerated and ignored. |
| `Skill.Stale`, `staleWarning`, `LibraryWarnings` | **deleted.** Ticket 02 already re-homed `Warnings` to one named case (a source whose status is `unavailable`). |
| `Materialize`, `<configDir>/builtin-skills/`, `readmeText` | **deleted**, replaced by seed materialization into `sources/chartr-skills`. The README explained a layer model that no longer exists. |
| `splitFrontmatter` | **kept, demoted** to stripping the body. Every field it parses is now ignored. |
| the `Skill:` claim trailer | **kept, re-keyed.** `<name>=<layer>:<hash>` becomes `<name>=<source>` for a dir source and `<name>=<source>@<commit>` for a git one — `Skill: grill=chartr-skills@9e8b5ea`. One line, not a repeated key: after this map a ticket session composes exactly one role skill. It answers the one thing `Payload-SHA256` cannot, which is *which source that prompt came from* — the same silent failure ticket 02 kept the origin badge for. A dir source records no version, and that absence is honest rather than a gap to fill. |
| `Payload-SHA256` | untouched (ADR 0008). |

### The repo's own contract, and what chartr validates

`chartr-skills/CONTRACT.md` states: the seven skills and what each is for; the
two-clause acceptance test above, with its naming allowance and its ban on
section skeletons; frontmatter is `name` + `description` only; no Claude-Code
framing (slash commands, hooks, loaders); no relative links between skill
directories — refer to another skill by name, since a source may be registered
as a subset; role skills short, method skills long; and `conventions.md` wins on
any disagreement.

**chartr validates nothing.** Ticket 01 already fixed discovery as "a directory
with `SKILL.md` at depth 1–3", and that is the whole test at registration and at
resolve. A malformed `SKILL.md` is not rejected — it is a skill that injects
whatever it says. A directory without one is not a skill and simply does not
resolve. This is the same posture ticket 04 took with lint: state the contract,
do not gate on it. Validating a source's markdown would mean chartr rejecting
someone else's repo for not being written the way chartr writes, which is the
opposite of what a source list is for.

### Rejected

- **A frozen, never-refreshable default.** Argued and rejected above: cleaner
  mechanically, but contradicts a settled map decision and relocates the
  staleness problem into shadowing rather than removing it.
- **A marker file (`.seed`, a build stamp) in the seeded directory.** Rejected:
  the binary already knows its own `SeedCommit`, and an on-disk copy is a second
  source of truth whose only distinctive behaviour is going stale.
- **Content-hashing the seeded directory to detect operator edits.** Rejected:
  ticket 01 already settled that a source checkout is chartr's, not a workspace
  ("the answer to *I want to edit this* is a `dir` source"). Presence of `.git`
  answers the only question that has consequences, for free.
- **Shipping only the four role skills**, method skills left to the operator.
  Rejected: a first run could then work tickets but never chart the map that
  produces them, and offline is exactly when that matters.
- **Shipping `ideate` and `domain-modeling` anyway** because they exist and the
  repo is right there. Rejected by the map's Out of scope; symmetry with what
  used to ship is not a reason.
- **A build step or submodule for vendoring.** Rejected above — `go install`
  and offline builds are the deciding cases.
- **Keeping the per-skill diff-and-triage procedure**, re-pointed at
  `chartr-skills`. Rejected: it existed because chartr re-authored someone
  else's skills in place. Against a repo chartr owns, a sync is a copy.
- **Keeping section skeletons in the method skills** (`docs/skill-sync.md`'s one
  exception, carried forward). Rejected by the operator: the skeletons duplicate
  what `conventions.md` already states, across two repositories now, and a
  skeleton is the part of a skill most likely to drift and least likely to be
  noticed drifting. Also rejected in the other direction: **forbidding the
  `## Answer` token as well**, which would leave the injected role body — the
  only text that reaches a session by force — silent about where its output
  goes, against a payload that by ticket 02's ignore test may not tell the agent
  to read the conventions.
- **Validating a source's skills at registration** (frontmatter present, body
  non-empty, name matches directory). Rejected: it makes chartr the judge of
  other people's repos and adds a failure mode — a source that registers but
  rejects half its skills — where "it does not resolve" already suffices.
- **A "restore shipped skills" action** on the default row. Rejected: deleting
  the directory already does it, and the reconcile makes that self-healing.

### Surfaced doubts

- **This amends ticket 01's file shape** (`default_commit`, `default_fetched`,
  and a `git`-kinded default row). It is inside the delegation ticket 01 wrote,
  but it is a change to a resolved ticket's stated model and a human should see
  it as one.
- **The `## Answer` naming allowance is the last of the two-places problem
  `docs/skill-sync.md` warned about**, and it is deliberately the smallest
  version of it: one token, frozen by ticket 04, in a repository chartr vendors
  wholesale. It survives only because the injected role body is the one text a
  session cannot skip. If ticket 04 ever renames the closing heading, seven
  skills and a vendored seed move with it.
- **A refresh of the default can break a seeded role binding.** If ticket 03
  binds `grilling = "chartr-skills/grill"` and a fetched commit renames or drops
  `grill`, the binding resolves to nothing and chartr's own refresh button broke
  chartr's own role resolution. Ticket 03 owns what a spawn does with an
  unresolvable binding; this ticket supplies the failure mode, and notes it is
  the sharpest consequence of choosing a refreshable default over a frozen one.
- **The reproducibility this effort gives up is recorded above, not resolved.**
  Ticket 07 or 08 should carry the retirement into ADR 0009's amendment,
  wherever that lands; the reasoning it has to answer is now written down rather
  than deleted with `docs/skill-sync.md`.

### Revisit triggers

- **The seven-skill list** is revisited when a real operator's first hour is
  blocked by something absent from it, or when a shipped skill goes a full
  release cycle unused by any session. Adding an eighth needs the same
  first-run-offline argument `wayfinder` won on; "it was in the old library" is
  not one.
- **The `.git`-as-ownership rule** is revisited the first time an operator wants
  chartr's newer seed *after* having refreshed. The escape is an explicit
  "reset to shipped" action that deletes the checkout, not a comparison rule.
- **The no-skeletons rule** is revisited if prose-only method skills measurably
  produce worse-shaped maps than templated ones did. The escape is enriching
  `conventions.md`'s existing skeleton, never restoring a template to a skill.
- **The `Skill:` trailer** is revisited if a dir source's unversioned line ever
  has to answer "what exactly ran" in an incident. The escape is recording the
  role body's own hash again — which is re-deriving what `Payload-SHA256`
  already covers, so the bar is a case where the payload file is gone and the
  trailer is all that is left.
- **Silent prompt changes at upgrade** are revisited if a seed change ever
  surprises an operator into wrong work. The escape is a release-note link on
  the row, not a gate.
