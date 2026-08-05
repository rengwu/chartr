---
type: grilling
claimed_by: s575cb57b227d
claimed_at: 2026-08-05T14:28:00Z
---

# The conventions ruleset

## Question

This is the ticket that makes "chartr ships no skills" actually work. Every shipped skill today leaks chartr-specific material — all eleven hit `.plan/`, `.chartr`, `## Answer` or the word chartr. A generic skill from someone else's repo knows none of it and will write maps wherever its own defaults point. The ruleset is what closes that gap: chartr's file-format contract, embedded, materialized to a stable path, and pointed at from both payloads.

Two prior attempts at this exist and both are being deleted — `docs/agents/issue-tracker.md` (a committed artifact in the operator's repo) and the `tracker-convention` skill (shadowable and toggleable, which a contract must not be). Understanding why each failed is the best guide to what this one must be.

Settle:

- **What is in it.** The candidates are the map directory layout, the ticket filename and numbering rule, the frontmatter fields, the section names, the derived-status table, the frontier definition, the claim fields, and the method vocabulary in `glossary.md`. Some of that is *format chartr parses* and some is *method a skill teaches*. Only the first belongs here — a ruleset that also teaches wayfinding has re-created the skill it replaced. Draw that line explicitly, field by field.
- **Whether one document is enough.** The ruleset is read by a session about to write a ticket, and also by one about to chart a map. One file the agent reads whole, or a small directory it zooms into? The payload cost is identical — it is a pointer either way — so this is about what an agent does with it once opened.
- **Materialization.** Where it lands, whether it is rewritten on upgrade, and what happens if the operator edits it. The existing `Materialize` never overwrites an operator's edits because hackability is the point; a *contract* the operator has edited into disagreement with the parser is a different proposition, and the answer may differ from the skill library's.
- **Enforcement versus statement.** The destination says the ruleset "enforces" conventions. Nothing in a markdown file enforces anything — `internal/wayfinder/lint.go` does. What does the ruleset promise, what does the lint actually check, and is a session told about the lint so a malformed ticket is caught by the agent rather than by the operator later?
- **The glossary's fate.** It is inlined into every ticket payload's context bundle today. If its vocabulary moves here, the bundle either sources it from here or stops carrying it because the payload already points at it. This decision is shared with [The two core payloads](02-the-two-core-payloads.md) and one of the two must own it.
- **What `.plan/maps/` being a convention implies.** It is stated in the ruleset, parsed by `internal/mapscan`, and restated in CLAUDE.md. Is the path itself configurable, or fixed forever — and if fixed, does the ruleset say so in a way that stops an agent from being creative about it.

## Done when

The ruleset's contents are enumerated with the format/method line drawn through them, its file shape and materialization behaviour are settled, its relationship to the lint is stated, and the glossary has exactly one home.

## Answer

**Ship one canonical, generated `<config-root>/conventions.md`; point every
chartr-built payload at it, and keep method teaching out of it.** It replaces both
`docs/agents/issue-tracker.md` and the `tracker-convention` skill. It is chartr's
write contract, not a skill, not a source entry, and not a configurable tracker
adapter.

There is one deliberate addition to the map's starting decision: a user-owned
`<config-root>/preferences.md` is appended verbatim after the conventions pointer
in both payloads. This **amends the settled statement that a free session receives
no behavioural instruction**. The operator chose maximum control: preferences
carry no wrapper saying they are subordinate, and chartr does not prevent them
from contradicting the conventions. A contradictory preference can therefore
make an agent write a map chartr cannot read. That is an accepted consequence,
not a case for silently restoring precedence in implementation.

### The two files and the built prompt

- **`conventions.md` is one complete document**, not a directory an agent must
  zoom through. Map-writing and ticket-writing share most of the contract, and
  splitting a small document buys no payload reduction while making partial
  reading possible.
- Chartr embeds its canonical bytes and atomically writes them to
  `<config-root>/conventions.md` at startup. It reconciles the file again before
  every payload composition, including a preview: missing or differing bytes are
  replaced. An upgrade therefore updates it automatically, and an operator edit
  lasts only until the next composition. This is the narrow exception to
  hackability a parser contract requires: the file is plain and inspectable, but
  generated rather than an override surface.
- Chartr creates an empty, owner-writable `<config-root>/preferences.md` on first
  run and never rewrites or merges it. If it is later missing, it is recreated
  and behaves as empty. If it exists but cannot be read, composition fails visibly
  rather than silently dropping the operator's instructions.
- The instruction order is: **core payload; bound role skill for a ticket
  session; conventions path; raw preferences; then the fresh ticket context
  bundle**. A free session omits the role and ticket context but gets the same
  conventions-then-preferences tail. Preferences have no heading, warning,
  precedence label or conflict check.
- The conventions remain a **path**, never inlined. Preferences are the opposite:
  their bytes are appended to the built prompt. Ticket 02 owns the prose around
  the path and the rest of each core payload; this ticket fixes their order and
  lifecycle.

### When the contract applies

The document opens with an applicability rule: when specifications, ideas or
requirements are requested to be **charted into a map**, **created as a
wayfinder map**, **created as tickets**, or equivalent wording, the output is a
chartr-compatible wayfinder artifact using this in-house tracker convention.
That routes a generic sourced skill to the right storage contract. It does not
teach the skill how to interview, decompose, sequence or size the work.

### Fixed layout

`.plan/maps/` is fixed and non-configurable:

```text
.plan/maps/<slug>/
  map.md
  spec.md                 optional synthesized specification
  tickets/
    NN-kebab-case-slug.md
  assets/                 optional map-owned supporting files

.plan/maps/<slug>-impl/   sibling implementation map, when one is created
```

Maps anywhere else are invisible. In particular, implementation must narrow
`internal/mapscan`: its current recursive walk accepts `.plan/<slug>/` and any
other nested `map.md`, which contradicts this decision. Compatibility with that
old flat layout is knowingly dropped. There is no setting, fallback search or
per-space root.

A specification's only chartr convention is its placement at
`.plan/maps/<slug>/spec.md`, with a resulting implementation map at the sibling
`<slug>-impl/`. Chartr does not parse the specification body, so this ruleset
does not prescribe its headings or recreate a `to-spec` template.

A ticket file is `NN-kebab-case-slug.md`: numbers below 100 are zero-padded
(`01` through `99`), then use natural width (`100`, `101`, and so on). The number
is the ticket's permanent identity and is never reused or renumbered; frontmatter
edges and prose links address that number. The reader may remain tolerant of
legacy widths, but every new writer uses the canonical form.

### `map.md` format

Every map carries, in this order:

```markdown
# <title>

## Destination
## Notes
## Decisions so far
## Not yet specified
## Out of scope
```

- `Destination` contains non-empty prose. `Notes` is freeform standing
  orientation.
- `Decisions so far` indexes every resolved ticket using a relative
  `./tickets/NN-slug.md` link. The link is the machine-readable part; the
  surrounding summary is prose.
- `Not yet specified` is a list of top-level bullets with a bold lead title.
  Where a bullet has a machine-readable clearing edge, its syntax is
  `<clears-with: NN>`.
- `Out of scope` may also carry freeform boundaries, but every ticket closed by
  `## Ruled out` is indexed there with its relative ticket link.

Those are storage rules only. How fog is discovered, how a destination is
chosen, when something becomes a ticket, and how planning differs from
implementation remain the sourced method skill's job.

### Ticket format

The recognized frontmatter is:

```yaml
---
type: grilling
blocked_by: [01, 02]
undermined_by: [03]
assets: [sketch.png]
claimed_by: s1a2b3c4d5e6f
claimed_at: 2026-08-06T01:02:03Z
---
```

- `type` is required and is exactly one of `grilling`, `research`, `prototype`
  or `task`.
- `blocked_by` is an optional list of ticket numbers whose resolved answers are
  premises; absent means none.
- `undermined_by` is an optional list of ticket numbers that call this ticket's
  answer into question. It flags for human judgment and never reopens anything
  automatically.
- `assets` is an optional list of paths relative to the map's `assets/`
  directory.
- `claimed_by` and `claimed_at` are chartr-owned. Agents do not create or edit
  them. `claimed_at`, when present, is RFC 3339.
- `status` is forbidden: it is a stale second copy of a derived fact.
- Unknown keys are tolerated and ignored, but the ruleset does not present them
  as chartr features.

Every ticket carries:

```markdown
# <title>

## Question

## Done when
```

An open ticket has no closing heading. A closed ticket has exactly one of
`## Answer` or `## Ruled out`, with non-whitespace prose beneath it. The heading
names are exact; a bare heading closes nothing, and any differently named
heading is ordinary unknown content.

Status is derived in this order:

| File content | Derived status |
| --- | --- |
| prose-bearing `## Answer` | `resolved` |
| otherwise, prose-bearing `## Ruled out` | `out_of_scope` |
| otherwise, non-empty `claimed_by` | `claimed` |
| otherwise | `open` |

Closure wins over a leftover claim. The **frontier** is the `open` tickets whose
every `blocked_by` ticket is `resolved`. A claimed ticket is not open, and an
out-of-scope blocker does not clear an edge.

### The format/method line and the glossary

`glossary.md` is deleted and the context bundle stops carrying a glossary part.
There is no second glossary sourced through a skill. `conventions.md` defines
chartr-specific format terms where it uses them — map, ticket, blocker, frontier,
answer and ruled-out — so the meanings needed to write parseable files have one
home. Session and role belong in the core payloads; wayfinding method vocabulary
belongs in sourced method skills.

The ruleset therefore contains the fixed layout, filename and identity rule,
recognized frontmatter, exact structural headings, derived-status table,
frontier calculation, claim ownership, and the map-index syntax chartr parses.
It deliberately excludes interviewing, research and prototyping technique,
ticket decomposition and sizing, the wayfinder decision process, role behaviour,
commit discipline, and spec templates. Putting those in this file would recreate
the method skill chartr has just stopped shipping.

Supporting files inside a sourced skill need no replacement for the old
glossary-specific `Support()` path. They are ordinary files addressed relative
to that skill's `SKILL.md`; the source path already given to the agent makes them
readable on demand. Chartr neither indexes nor inlines them and gains no generic
attachment API.

### Statement, discovery and lint

The markdown **states** the writable contract; it does not enforce it. Two pieces
of deterministic code remain deliberately narrower:

1. Discovery enforces only the fixed root by scanning beneath `.plan/maps/` and
   nowhere else.
2. The existing `internal/wayfinder/lint.go` remains unchanged and non-blocking.
   Its diagnostics continue surfacing as cockpit malformations: invalid or
   stored types/statuses, bad or stale claims, duplicate/missing/self/cyclic
   edges, invalid closing sections, missing destinations, drift between closed
   tickets and the map indexes, and malformed or stale fog links. It remains a
   tolerant reader's diagnostic set, not a complete validator for every
   canonical writing rule above.

No lint command, payload instruction, label, launch gate or automatic repair is
added. A session is not told to run lint. This knowingly leaves canonical details
such as heading completeness and zero-padding as statement rather than mechanical
enforcement, and it leaves the operator able to override the statement through
last-position preferences. Malformed maps still render where the tolerant reader
can parse them and surface diagnostics where it cannot; adoption is never gated.

### Rejected alternatives and accepted costs

- **A small ruleset directory** was rejected: both writer types need the same
  small contract, so splitting adds partial-read failures without saving prompt
  bytes.
- **Preserving edits to the generated ruleset** was rejected: an edited contract
  that disagrees with the parser lies. Versioned copies or an operator override
  inside the contract have the same failure with more machinery.
- **A configurable map root or legacy fallback** was rejected: every skill would
  need discovery/config plumbing, and the standing rule already names the only
  committed work-product root.
- **Keeping `tracker-convention` or `glossary.md` in a source** was rejected:
  either could be disabled or shadowed, while the parser contract cannot be.
- **Inlining conventions** was rejected by the settled map decision: a stable
  path keeps the payload small and the exact contract inspectable.
- **A lint command, a complete new validator, or a launch gate** was rejected:
  the operator wants lint absent from agent instruction and the current cockpit
  diagnostics preserved as advisory only.
- **A generic supporting-file index/API** was rejected: relative files already
  solve the one real case after `glossary.md` disappears.
- **Making preferences subordinate, labelled, conflict-checked, or ticket-only**
  was rejected by the operator. The accepted price of maximum hackability is
  that the final user text can defeat chartr compatibility. This is also a
  deliberate second behavioural extension beside registered sources, rather
  than the source-list-only posture the map originally leaned toward.

### Revisit triggers

- Reopen fixed-root discovery only when a real space must be driven while its
  maps cannot live under `.plan/maps/`; convenience for an old layout is not
  enough.
- Split `conventions.md` only when it grows enough that agents measurably skip or
  misapply sections; directory symmetry alone is not enough.
- Add lint invocation or wider enforcement only after malformed artifacts escape
  often enough that cockpit-only surfacing is demonstrably too late.
- Reconsider unlabelled last-word preferences after a real contradiction causes
  unexpected unreadable work and the operator wants protection more than raw
  control.
- Add a spec-body contract only when chartr itself begins parsing one.
