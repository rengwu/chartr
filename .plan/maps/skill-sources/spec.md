# Skill sources — chartr stops shipping skills

Synthesized from the resolved [`skill-sources` planning map](map.md) and its eight
tickets. Every decision below is settled there; nothing here is new. Ticket links
are the provenance — read them when a detail is disputed.

## Problem Statement

chartr ships its own skill library and resolves it through three layers — built-in,
user, workspace — with whole-skill shadowing. That model has four problems the
operator lives with today:

- **The skill set is chartr's, not the operator's.** Wanting Matt Pocock's
  `prototype` skill instead of chartr's means forking chartr's copy in place and
  maintaining a drift warning, rather than pointing at the repo it already lives in.
  The name set is closed: a skill chartr does not ship cannot be reached at all.
- **chartr writes into the operator's repository.** `.chartr/skills/` is a
  committed layer, and `docs/agents/issue-tracker.md` is a file chartr offers to
  install into every space. Both put chartr's operating artifacts into someone
  else's git history.
- **Every shipped skill is chartr-specific and cannot be replaced by a generic
  one.** All eleven mention `.plan/`, `.chartr`, `## Answer` or the word chartr.
  A skill from anyone else's repo knows none of chartr's file format and will write
  maps wherever its own defaults point — so the library cannot open up without the
  format contract leaving the skills.
- **A cold launch is a skill launch.** Starting an agent without a ticket goes
  through a skill launcher, a dropdown and a context modal, gated by `on-ramp:`
  frontmatter — an entire mechanism whose only job is deciding which of chartr's
  own skills may start a session.

The result is a cockpit that claims to be hackable and agent-agnostic while owning
the one thing the operator most wants to own: what their agents are told.

## Solution

chartr ships **no skills at all**. In their place:

- **Registered skill sources.** An ordered list of local folders and pinned git
  repos the operator registers, manages from a screen, and reorders at will.
  Position in the list is resolution order. A bare skill name takes the first hit;
  a `Source name/skill` reference addresses one source exactly.
- **Two core payloads.** A **ticket session** gets today's core plus the bound role
  skill's body. A **free session** — an agent launched from the space card with no
  ticket — is told what chartr is and what skills exist, and nothing about how to
  behave.
- **One conventions ruleset.** chartr's file-format contract, embedded in the
  binary, generated to a stable path under chartr's config root, and pointed at by
  both payloads. It is what lets a generic skill from someone else's repo write a
  map chartr can read. A user-owned `preferences.md` is appended after it, verbatim
  and unranked.
- **Explicit role bindings.** Four lines in the operator's own config bind each
  ticket type to one source-qualified skill. Reordering sources is a discovery
  change, never a behavioural one.
- **A default source, seeded from the binary.** `chartr-skills` is a separate repo
  of seven skills, vendored into chartr so a first run works offline, sitting last
  in the list so anything registered shadows it, and pinnable by the operator with
  one explicit fetch.

Afterwards the space card's `new shell` split button replaces the skill launcher,
chartr writes nothing into the operator's repository but `.plan/maps/`, and the
extension point for skills is a directory of markdown the operator controls.

## User Stories

1. As an operator, I want to register a folder or a git repo of skills, so that my
   sessions run prompts I own instead of prompts chartr shipped.
2. As an operator, I want the list's visible order to be the resolution order, and
   to rearrange it, so that which skill wins is something I read rather than infer.
3. As an operator, I want to address a skill through its source's name, so that a
   skill shadowed by a higher source stays reachable without reordering the list.
4. As an operator, I want to turn a source off without removing it, so that I can
   park a library without losing its registration.
5. As an operator, I want each ticket type bound to one explicitly named skill, so
   that reordering my sources never silently changes what a grilling ticket runs.
6. As an operator, I want a spawn to refuse when its role binding cannot be
   resolved, so that a ticket is never answered by an agent missing the behavioural
   stance I chose the role for.
7. As an operator, I want to launch an agent shell from the space card with no
   ticket attached, so that I can chart a new map or work outside the frontier.
8. As an operator, I want that free session told only what chartr is and what
   skills exist, so that it behaves as the agent I chose rather than as chartr
   thinks it should.
9. As an operator, I want an empty shell that gets nothing injected at all, so that
   a plain terminal stays a plain terminal.
10. As an operator, I want one generated file stating chartr's whole file-format
    contract, so that a generic skill from someone else's repo still writes maps
    chartr can read.
11. As an operator, I want my own preferences file appended last and never
    overridden, so that I have the final word on how my sessions behave.
12. As an operator, I want chartr's role skills to arrive with the binary and work
    on a first run with no network, so that a fresh install can chart a map offline.
13. As an operator, I want to pin chartr's own skills with one explicit fetch and
    never be moved off them by a later upgrade, so that my role prompts change only
    when I say so.
14. As an operator, I want a git source refreshed only when I ask, so that a repo I
    trusted on Tuesday cannot silently become something else on Friday.
15. As an operator, I want to see which source each part of a composed payload came
    from, so that I can catch a source that shadowed what I expected.
16. As an operator, I want to upgrade off the layer model without losing a fork I
    edited, so that months of edits are still findable and still usable afterwards.
17. As an operator, I want chartr to stop writing its own files into my repository,
    so that my git history carries only my work.
18. As an operator, I want to register, remove, toggle, reorder and refresh sources
    — and restore a role binding I deleted — from a screen, so that routine work
    never means hand-editing a config file.

## Implementation Decisions

### The sources registry — [ticket 01](tickets/01-the-skill-sources-registry.md)

A new `sources` package owns a chartr-owned registry file under the config root.
It replaces the layer resolver wholesale — the three-layer roots, the layer tags,
the closed name set and the manifest builder are deleted, not ported.

The file's shape is the decision, so it is stated exactly:

```toml
default_enabled = true

[[source]]
name    = "Matt Pocock Skills"
kind    = "git"
url     = "https://github.com/mattpocock/skills"
ref     = "main"
commit  = "9e8b5ea1c4d7"
path    = "<config-root>/sources/3f2a1b9c4d7e"
fetched = 2026-08-05T10:12:00Z

[[source]]
name    = "House skills"
kind    = "dir"
path    = "/Users/x/work/skills"
enabled = false
```

- **Position is precedence.** There is no order field, and therefore no
  densification, no duplicate-order case and no legacy-row decode.
- **Identity is the operator's name** — trimmed, 1–64 characters, letters, digits,
  space, hyphen, underscore, no `/` (reserved by the qualified form), unique
  case-insensitively. No hashed id: a source's order is its position and its git
  state is on its own row, so nothing keys per-source state elsewhere.
- **`kind` is declared, never inferred.** Two kinds, `dir` and `git`, in one table
  — two tables could not interleave, and resolution is one ordered list.
- **`enabled` sits on the row**, written only when false.
- **The default row is synthetic** — never written as a row, fixed name
  `chartr-skills`, always last, not removable, not reorderable. Only its scalars
  persist (see the seed section).
- Written atomically at `0600` under a `0700` root. A missing or unparseable file
  is the default row alone: the first-run state, not an error. Losing it costs
  re-registration and never work, with one honest asymmetry — git checkouts are
  orphaned by the loss and chartr does not collect them.

The registry borrows the space registry's *stance* (atomic temp-then-rename,
missing-is-first-run, losing it costs re-registration) and deliberately none of its
machinery, because the pressures that produced the hashed id and the dense order
are absent here.

**Discovery** is a bounded, uncached walk: a skill is any directory at depth 1–3
below the source root containing `SKILL.md` at its top level. Never descend into a
directory that has one — everything below is that skill's supporting files. Skip
dot-entries and `node_modules`. A skill's name is its directory's basename, so
names stay the short tokens a payload lists and a binding writes. Every caller
walks; the walk stats directories and reads no file, so a skill folder created a
second ago is usable in the very next spawn with no invalidation rule to explain.

**Resolution** takes two forms:

```
resolve(ref):
  ref contains "/"  →  split at it: (source, skill)
                       that source, if it exists and is enabled.
                       miss = not found. never a fall-through.
  otherwise         →  each enabled source in file order, then the
                       default row if enabled. first hit wins.
```

A disabled source is skipped by both forms — *disabled* means one thing — and a
qualified reference into a disabled source reads as not-found **and names the
source as disabled**, because that is the one failure fixed in a click. A qualified
miss never falls through: naming a source and silently receiving a different
source's skill is worse than an error.

**Git sources** are shallow single-branch clones under the config root, keyed by a
hash of the URL so renaming a source is a pure metadata edit. The clone lands in a
temp directory and is renamed into place; the row is written only after that
rename, so nothing half-cloned is ever a source. There is **no confirm gate before
the clone** — pasting the URL is the deliberate act. Refresh is explicit and quiet:
fetch, hard-reset the checkout, record the new commit and timestamp, report the new
short sha. Nothing ever fetches unattended. The checkout is chartr's, not a
workspace — a refresh discards local edits inside it, and the answer to "I want to
edit this" is a `dir` source.

**Named failure modes**, each with a stated reading:

| case | reads as |
| --- | --- |
| dir path vanished | row survives, status `unavailable`, zero skills, flagged. Never auto-removed |
| `git` absent from PATH | registering a git source is refused at the gate, naming why. Existing checkouts keep resolving; only refresh fails |
| clone fails partway | no row, no directory, git's own error reported verbatim |
| source yields zero skills | row kept, status `empty`, reading `0 skills` with a remove action beside it |
| duplicate skill name across sources | not an error — it is what the order is for. The lower one stays reachable by qualification and is marked shadowed |
| duplicate basename inside one source | sorted walk order wins; the loser is named on the source's row |
| duplicate source name at registration | refused before anything is cloned |
| duplicate source name in a hand-edited file | first row wins, later ones dropped with a warning naming them |
| unknown kind, or neither path nor url | row dropped with a warning; the rest of the list stands |
| a row named `chartr-skills` | dropped with a warning — the name belongs to the synthetic default |
| file missing or unparseable | the default row alone; first-run state |

### Role bindings — [ticket 03](tickets/03-binding-a-ticket-type-to-a-skill.md)

A flat scalar table in the operator's existing `user.toml`, beside the agent
library:

```toml
[roles]
grill     = "chartr-skills/grill"
prototype = "chartr-skills/prototype"
research  = "chartr-skills/research"
implement = "chartr-skills/implement"
```

- **Always qualified, never bare.** A bare name resolved through source order would
  reintroduce the implicit capture the table exists to prevent: reordering sources,
  or a higher source later shipping its own `grill`, would silently change what
  every grilling ticket runs with no line in the file showing it.
- **Flat, not a subtable.** A binding is one fact. A subtable holding exactly one
  key forever is ceremony.
- **Seeded once**, on the first startup that finds no `[roles]` table at all, after
  the default source has been reconciled on that same startup. The presence test is
  the table, not the file, so an operator upgrading with an existing `user.toml`
  seeds exactly like a new install. Idempotent by construction; a second startup
  that sees even one row never touches it again.
- **A deleted row is never auto-restored** — deletion is a legitimate way to make a
  role refuse until rebound. Recovery is an explicit single-row settings action,
  written back through the comment-preserving TOML surgery the agent library
  already uses.
- **An unresolvable binding refuses the spawn** — no terminal, no claim commit.
  This needs no new code path: payload composition already returns an error and the
  spawn already aborts on it before the claim is written. The error names the role,
  the recorded binding string, and which of the three unresolvable shapes it hit
  (disabled source / removed source / skill missing from that source).
- **`Role` stays a closed type and the type→role mapping is untouched.** Role
  carries weight independent of skill selection — it is the key the spawn gate
  offers all four values under, it drives the quiet hint, it validates external
  input at the spawn boundary, and the claim trailer records it on its own line.
- **Four rows, fixed.** A fifth would mean wayfinder grew a fifth ticket type,
  which this effort refuses to redesign; admitting one later is mechanical.

The claim trailer becomes `Skill: <name>=<source>[@<commit>]`. The source name
replaces the layer; the commit is appended only where the source carries a pin.

### The two payloads — [ticket 02](tickets/02-the-two-core-payloads.md)

Two payloads from five interchangeable parts, four shared. Order is: core, role,
conventions, preferences, then the context region. One rule holds the document
together: **instructions, then data.**

| part | origin | free | ticket |
| --- | --- | --- | --- |
| core | chartr (embedded) | free variant | ticket variant |
| role | the source's registered name | — | the bound skill's body |
| conventions | chartr (embedded) | ✓ identical bytes | ✓ identical bytes |
| preferences | operator | ✓ | ✓ |
| sources | context | ✓ identical rendering | ✓ identical rendering |
| map, ticket, blockers | context | — | ✓ |

**The free-session payload carries no live facts about the space** — same bytes in
an empty tree and a tree mid-effort. No map list, no frontier, no branch for a
space without `.plan/`. The agent is already in the tree and can list a directory;
a list composed at spawn is wrong the moment another session resolves a ticket, and
a free session outlives that snapshot. What varies is the sources block and the
preferences bytes, and that is all.

**The restraint, stated as a test:**

> Every sentence chartr writes in its own voice must still be true if the agent
> does nothing about it.

A fact about the machine survives being ignored; an instruction's entire content is
what the agent should do. That admits a location (`conventions.md`) and an
inventory (the sources block), and rejects the next addition, which will almost
always arrive phrased as a *should*. It forces consequence-framing, and that is the
technique rather than a loophole: chartr may not say "read the conventions"; it may
say "a file under `.plan/maps/` is read only where it follows the format stated at
`<path>`". Both point at the same file; only the second survives being ignored, and
it carries the entire reason to open it. The operator's `preferences.md` is exempt
— it is the operator's voice, not chartr's.

**The sources block** renders identically in both payloads: enabled sources in file
order, default row last, each with its **local path and its skill names,
comma-joined, no descriptions**. Names fall out of the walk for free; descriptions
would cost one file read per skill at every compose and would grow the block
without bound. **The git URL is never printed** — it is a fetchable address in a
payload handed to an agent running with permissions skipped, on an effort whose
standing decision is that nothing fetches unattended. Disabled sources do not
appear. A closing sentence states how resolution behaves and names the qualified
form, as a fact rather than advice.

**The ticket payload keeps today's shape**, with two edits: the core's sentence
listing what was assembled drops its glossary clause, and **the role body is
concatenated, not pointed at**. Concatenating is what makes the claim trailer's
skill record mean *this text ran* rather than *this text was suggested*. No origin
line sits above the body — the preview badge and the trailer both carry that, and
the payload is what the agent reads, not an audit record.

**The glossary is deleted** and the context bundle stops carrying a glossary part.
There is no second glossary and no replacement supporting-file path: format
vocabulary lives in the conventions where it is used, method vocabulary in the
sourced method skill. The bundle's own shape is unchanged — it never held the
glossary; composition read it inline from a skill's supporting file, and that whole
block goes.

**The preview keeps provenance**, re-pointed: the per-segment layer tag becomes a
free-string origin — `chartr` for the embedded cores and the conventions pointer,
*the source's registered name* for a resolved skill body, `operator` for
preferences, `context` for every assembled part. The badge answers the one silent
failure source order can cause. The frontend's variant lookup keeps its neutral
fallback, so an open string set degrades rather than breaks.

**The free payload gets a preview**, space-independent — same seam, same modal,
four parts instead of eight, no ticket or role selector. It hangs off the sources
settings surface, not the space card. It is also the only place the operator sees
their own preferences land in an assembled document.

Composition warnings survive with one named case: a registered source whose status
is `unavailable`, because a source that vanished is precisely what silently changes
what a payload says.

### The conventions ruleset and preferences — [ticket 04](tickets/04-the-conventions-ruleset.md)

One canonical, generated `conventions.md` under the config root — chartr's write
contract, not a skill, not a source entry, not a configurable tracker adapter. It
replaces both the committed `issue-tracker.md` artifact and the `tracker-convention`
skill, each of which failed for a reason this one must not repeat: one lived in the
operator's repo, the other was shadowable and toggleable, which a contract must not
be.

- **One complete document, not a directory.** Map-writing and ticket-writing share
  most of the contract; splitting a small document buys no payload reduction and
  makes partial reading possible.
- **Generated, not an override surface.** chartr embeds the canonical bytes and
  atomically writes them at startup, then reconciles the file again before every
  composition including a preview: missing or differing bytes are replaced. An
  upgrade updates it automatically; an operator edit lasts until the next
  composition. This is the narrow exception to hackability a parser contract
  requires.
- **`preferences.md` is created empty on first run and never rewritten or merged.**
  If later missing it is recreated and behaves as empty; if present but unreadable,
  composition **fails visibly** rather than silently dropping the operator's
  instructions. Its bytes are appended raw — no heading, no wrapper, no precedence
  label, no conflict check.
- **The conventions are always a path, never inlined; preferences are the
  opposite.** A path is a capability, not a behavioural instruction, which is what
  fits it inside the free payload's restraint.

**This amends the map's starting decision that a free session receives no
behavioural instruction.** The operator chose maximum control: a contradictory
preference can make an agent write a map chartr cannot read. That is an accepted
consequence, not a case for restoring precedence during implementation.

The document opens with an **applicability rule** — when specifications, ideas or
requirements are asked to be charted into a map, created as a wayfinder map,
created as tickets, or equivalent wording, the output is a chartr-compatible
wayfinder artifact using this convention. That routes a generic sourced skill to
the right storage contract without teaching it how to interview, decompose,
sequence or size work.

**Contents — format only:**

- The fixed layout: `.plan/maps/<slug>/` holding `map.md`, an optional `spec.md`,
  `tickets/NN-kebab-case-slug.md`, an optional `assets/`, and the sibling
  `<slug>-impl/` for an implementation map. Maps anywhere else are invisible.
- The filename and identity rule: numbers below 100 zero-padded, natural width
  above; the number is permanent, never reused or renumbered. Readers stay tolerant
  of legacy widths; every new writer uses the canonical form.
- `map.md`'s five sections in order, the relative-link index syntax under
  `Decisions so far`, the bold-lead bullets and clearing-edge syntax under `Not yet
  specified`, and the rule that a ruled-out ticket is indexed under `Out of scope`.
- The recognized ticket frontmatter: `type` (required, one of four), `blocked_by`,
  `undermined_by`, `assets`, and the chartr-owned `claimed_by` / `claimed_at`.
  `status` is forbidden — a stale second copy of a derived fact. Unknown keys are
  tolerated and ignored.
- The exact structural headings, the derived-status table (a prose-bearing
  `## Answer` → resolved; else prose-bearing `## Ruled out` → out of scope; else a
  non-empty claim → claimed; else open), and the frontier calculation.
- The chartr-specific format terms where they are used — map, ticket, blocker,
  frontier, answer, ruled-out.

**Deliberately excluded:** interviewing, research and prototyping technique, ticket
decomposition and sizing, the wayfinder decision process, role behaviour, commit
discipline and spec templates. Putting those here would recreate the method skill
chartr has just stopped shipping.

**The markdown states the contract; it does not enforce it.** Two pieces of
deterministic code stay deliberately narrower: discovery enforces only the fixed
root, and the existing lint stays unchanged, non-blocking and advisory, surfacing
as cockpit malformations. No lint command, payload instruction, launch gate or
automatic repair is added, and no session is told to run lint. **Implementation
must narrow map discovery to `.plan/maps/` only** — the current recursive walk
accepts other nested layouts, and compatibility with the old flat layout is
knowingly dropped.

### chartr-skills and the seed — [ticket 05](tickets/05-chartr-skills-and-how-it-ships.md)

A separate MIT repo of **seven skills** — the four roles plus `wayfinder`,
`to-spec` and `to-tickets` — carrying the attribution that travels with the three
Pocock-derived ones, because the seed puts those bytes inside a distributed binary.

| ships | why not left to the operator |
| --- | --- |
| `grill`, `prototype`, `research`, `implement` | bound by the seeded role table; without them a first run cannot spawn a ticket session at all |
| `wayfinder` | nothing else creates a map, and chartr's whole surface is inert until one exists — a default source that drives maps but cannot make one leaves a first run staring at an empty cockpit with no offline way out |
| `to-tickets` | the planning map's terminus; shipping `implement` without it ships half a loop |
| `to-spec` | the fixed layout reserves a spec's path, and a slot in chartr's own contract with no skill that fills it is a gap |

The operator who prefers someone else's method skills pays one shadowed name,
since the default row sits last. Not shipping them costs every offline first run
the ability to start.

**The acceptance test**, stated in the repo's own `CONTRACT.md`: no sentence whose
truth depends on chartr running, and no rule the conventions file states.
**Naming is allowed; specifying is not** — a skill may write `## Answer`, `map.md`,
"the frontier"; what it may not do is be the place a rule is stated. **No skill
carries a section skeleton**: a frozen token cannot drift, a six-heading skeleton
with prose guidance can, and the conventions already state both skeletons. The
method guidance currently living *inside* those placeholders must be re-authored as
prose — that is part of seeding the repo, not a follow-up. On any disagreement the
conventions win. Frontmatter is `name` and `description` only, and chartr reads
neither; the launcher-era frontmatter fields are stripped.

**The seed and the pin never coexist.** The default source's directory is in one of
two states, read off the filesystem:

| state | recognized by | who owns the bytes |
| --- | --- | --- |
| seeded | no `.git` | chartr — reconciled against the embedded seed at every startup |
| pinned | `.git` present | the operator — chartr never writes there again |

- The seed **records nothing about itself on disk**; its identity is a constant
  compiled into the binary. A marker file would be the copy that goes stale.
- Reconciliation compares the file set and bytes and replaces the whole directory
  when anything differs — temp-then-rename, deliberately wholesale so a skill
  deleted upstream actually disappears, deliberately startup-only.
- **The seed never overwrites a pinned checkout.** A fetch is the operator
  asserting ownership and an upgrade must not silently revert it.
- **Refresh converts seeded → pinned, once**, then it refreshes like any other git
  source and the seed is dead to it.
- **Deleting the directory is the reset** — one rule covering reset, repair, and a
  directory an operator wrecked. There is no "restore shipped skills" action.
- An operator who has never refreshed **does get changed role prompts at upgrade,
  without being asked**. That is correct and matches the conventions file's stance:
  upgrading chartr is the act of taking chartr's bytes.
- **Two scalars persist beside the default toggle** — the fetched commit and its
  timestamp, written only by a refresh, absent while seeded — so the row reads
  either "shipped with this build" or "fetched <date> — <sha>" with no filesystem
  inspection at load. **This amends the registry's "only its toggle persists"** and
  makes the default row git-kinded rather than dir-kinded. Nothing else about the
  row moves.

**Vendoring is a checked-in copy of a pinned ref, refreshed by one make target**
that clones at a ref, replaces the vendored directory wholesale and rewrites the
pin constant. Not a build step that clones (a release's contents must be a function
of the commit, and a fetching build would make the test suite need network); not a
submodule (`go install` and source tarballs carry none, so the binary would ship
with an empty default source). The seed lives with the sources package, not the
package this effort is gutting.

**chartr validates nothing** about a source's skills. Discovery is the whole test
at registration and at resolve. A malformed `SKILL.md` is a skill that injects
whatever it says; a directory without one simply does not resolve. Validating
someone else's markdown would mean chartr rejecting a repo for not being written
the way chartr writes.

**Provenance, piece by piece:** the shipped-hash comparison, the per-skill content
hash, the fork marker, the staleness flag and its warnings, and the materializer
are all **deleted** — they existed to detect drift between a shipped default and an
in-place fork, and forking is now "register your own source above the default".
The seed repo/commit constant pair is **kept, renamed and re-pointed**. Frontmatter
splitting is **kept but demoted** to stripping the body. The claim trailer is
**kept and re-keyed**. The payload hash is untouched.

### The new-shell control — [ticket 06](tickets/06-the-new-shell-control.md)

The space card grows one split button whose body is `empty shell` and whose caret
opens a menu: `empty shell`, a divider, then the registered agents in registration
order. The launcher and the `+` shell button collapse into it, so the actions row
gets shorter, not longer. The empty-library state keeps the divider with one
disabled row routing to registration, reusing the existing message and callback.

**A free session is today's on-ramp tab, renamed.** Its session stays nil, so it
never counts toward the one-session-per-space gate and never freezes dead. But the
non-nil-session test is doing two jobs today — *has a ticket* and *chartr knows
which binary runs here* — and the free session splits them. **The launch spec and
the terminal gain an agent-name field**; the identification branch and the
working-state seed read that field, while the gate, the death-pinning, the dead
state, the title and the pushed session info keep reading the session. The
unknown-session sampler is renamed and reached by both paths.

This incidentally fixes a bug that exists today: an on-ramp tab on an agent with no
shipped manifest reads permanently idle for its whole life, because its root
process *is* the agent. Free sessions would have turned that into the common case.
It also removes a boot flash where the tab reads idle for up to one slow tick.

**Not a third tab kind.** The model gains one field, not a `Kind` enum — three
names for two independent booleans would force every existing session test to be
re-read and re-decided.

**Titling is the agent's registered name**, so the tab is titled by the thing the
operator clicked. Three free sessions on one agent get three identical titles, and
that is fine — every ad-hoc shell in a space is titled `zsh` today. A
disambiguating counter is per-space state to allocate, recycle and keep stable
across a model push, for a cosmetic gain.

**The payload rides the same file-plus-opener mechanism**, unchanged. Inlining it
into the opener line is refuted by that function's own reason for existing — it
strips newlines, sends a carriage return and sleeps, because a text-plus-return
chunk looks like a paste and gets swallowed. The free payload is also not small and
its size is not chartr's to know: preferences are operator-owned and unbounded. The
existing per-session run directory already applies, since a free session already
mints a session id; archiving keeps running, because the archive is the record.

**The empty shell gets zero changes.** Empty means empty: no conventions pointer,
no sources list, no payload of any kind. If the operator types an agent name into
it, that is an agent in a terminal chartr did not launch — already out of scope.
Injecting a pointer on the chance an agent appears later would be chartr writing
instructions into a tree on speculation.

**Deleted:** the skill launcher component and its test; the launch-menu module and
its test (its job was ranking agents × on-ramp skills; the new menu is a static
list plus one fixed row — the spawn picker's own choice module survives); the
ideate route, handler and client action; the cold-launch and ideate composition
entry points; the launcher-era frontmatter fields with their model and TypeScript
mirrors; and the context modal. **Renamed rather than deleted:** the on-ramp
terminal opener becomes the free-session opener, and the launch route's handler
becomes the free-session handler with the skill lookup and the on-ramp allowlist
check removed. The app-level launch wiring and the space card's now-unused props go
with them.

### Migration and the first run — [ticket 07](tickets/07-migrating-off-the-layer-model.md)

**Who owns the bytes decides all four fates.** The two directories under chartr's
config root get an active migration; the two inside the operator's repo get a
stated-and-left-alone fate, because chartr may only stop touching them.

- **The operator's own skills directory** auto-registers as an ordinary `dir`
  source named `Legacy skills`, but only if it exists and the discovery walk finds
  at least one skill. An empty or absent directory contributes no row. The worry
  that an auto-registered fork stops driving its old role is real but bounded to
  free-session bare-name lookups: qualified bindings already closed that door for
  every source. Auto-registering is the only option where the fork keeps existing
  anywhere chartr can find it with zero operator action, and removing a dir source
  touches nothing on disk.
- **The materialized built-in directory** is compared once against the shipped copy
  being retired, using the comparison this effort is otherwise deleting. Untouched
  (or empty or absent): **renamed aside, left unregistered**, for the operator to
  remove or ignore. Diverging anywhere: **left exactly where it is and registered**
  as a `dir` source named `Migrated built-in skills`, so the edit stays findable.
  The rename rather than a delete is [ticket 08](tickets/08-sequencing-the-work.md)'s
  amendment — see Further Notes.
- **Ordering:** the legacy-skills row first, the migrated-built-in row second, both
  before the default row. The old resolution order was workspace › user › built-in;
  with the workspace layer gone, the surviving relative order carries forward.
- **The committed workspace skills directory:** chartr simply stops resolving it.
  The directory is untouched and **nothing is said** — no warning row, no notice.
  It goes inert exactly as silently as it goes unread.
- **The committed tracker file:** chartr stops writing new ones, stops refreshing
  existing ones, and **the whole offer surface is deleted** — the install and
  dismiss handlers, the classify and install call sites, the offer model on both
  sides, the dismissal flag in the space registry, the banner component and its
  wiring, the tracker package and its embedded template. The offer's entire reason
  to exist is reaching an agent chartr did not launch, which this effort refuses
  everywhere else. **Existing files are declared harmless and left alone** — their
  content stays true, because this effort does not move or reshape `.plan/maps/`;
  the file simply stops being refreshed.

**One trigger: the absence of the sources file.** It never existed under the layer
model, so its absence is indistinguishable between a brand-new install and an
upgrade — which is the right property, because migration is not a version check,
it is the same first-run path a from-scratch install already takes.

The first-run sequence, in order:

1. **Scan** both config-root skill directories with the bounded discovery walk, and
   diff the built-in copy against the shipped one.
2. **Rename the built-in copy aside** if it is empty, absent or byte-identical.
3. **Write the sources file** — the default toggle, plus a `Legacy skills` row if
   step 1 found anything, plus a `Migrated built-in skills` row if step 2 did not
   dispose of it, legacy row first.
4. **Reconcile the default source** from the embedded seed.
5. **Seed the role table** if it is absent — after the default source exists to
   point into.
6. **Materialize the conventions file** — order-independent, every startup.
7. **Nothing is reported.** Every first-run write this effort produces is quiet.
   The migrated rows are discoverable the moment the operator opens the settings
   surface; nothing pushes them, or the fact that a migrated fork no longer drives
   its old role, at the operator on the run it happens.

This is one function assembled across four implementation tickets in that order —
each adds its step in place rather than inventing its own startup hook.

**Downgrade is stated, not solved.** The new file and table are things an old
binary has never heard of, and the TOML decoder ignores unknown tables, so a
downgraded config is inert rather than a parse error. A renamed built-in directory
is recreated by the old binary's own never-overwrite materializer on its next
startup. A preserved edited copy and a migrated skills directory are read exactly
as before, because migration never moved either. What actually breaks: any source
registered *after* upgrading is invisible to the old binary, and any rebound role
reverts to whatever three-layer resolution finds — silently, since the old binary
has no concept of a binding. That is acceptable to state; guarding it would mean
the new binary writing old-format files nobody asked for.

### Sequencing, the cut and the ADR — [ticket 08](tickets/08-sequencing-the-work.md)

**One implementation map at the sibling `skill-sources-impl/`, build-then-cut,
opening on a four-ticket frontier and closing on one deletion-only ticket.**
Fourteen tickets; sizing is the ticket-cutting session's call, but **every edge
below must survive any folding.**

| # | ticket | must land after |
| --- | --- | --- |
| 01 | the `chartr-skills` repo (prose, not Go) | — |
| 02 | the sources registry, `dir` only | — |
| 03 | conventions + preferences: embed and materialize | — |
| 04 | the launched-agent split | — |
| 05 | seed and vendoring | 01, 02 |
| 06 | migration + the tracker-adapter surface | 02 |
| 07 | role bindings + the claim trailer | 02, 05 |
| 08 | the two payloads — **tracer bullet completes here** | 02, 03, 07 |
| 09 | the new-shell control | 04, 08 |
| 10 | git sources: clone, refresh, pin | 02 |
| 11 | **the cut** + ADR 0017 + delete the skill-sync doc | 06, 08, 09 |
| 12 | discovery narrows to `.plan/maps/` | 03 |
| 13 | the sources settings section | 07, 10, 11 |
| 14 | documentation pass | all |

**The cut-first precedent does not transfer.** It won on the `simplify` map because
a later ticket would otherwise have ported code an earlier one was about to delete.
That trap is absent here — the sources package is new, not a port, and the layer
resolver is deleted un-ported. The deciding argument is sharper than preference:
`simplify`'s cut removed a *gate*, so the escape hatch was a terminal and git. This
cut removes the *engine*. A tree with the layer model gone and the registry not yet
wired cannot compose a payload, so it cannot spawn the sessions that would do the
remaining work — chartr would be unable to drive its own implementation map. There
is a hard edge at `06 → 11` besides: the migration's byte-comparison consumes the
very embed the cut deletes.

The intermediate cost of carrying both models is smaller than it sounds: exactly
one ticket touches both, and it touches the old one only to stop calling it.

**Three pieces no planning ticket owned are in scope:** the `chartr-skills` repo
itself (the longest lead, and not code — seven skills re-authored, placeholder
method guidance rewritten as prose, the contract document, licence and
attribution); the **sources settings section**, which is load-bearing for three
resolved tickets and is what makes the silent migration survivable — editable means
the actions already designed (register, remove, toggle, reorder, refresh, restore a
role binding), open-the-file means everything else; and the documentation pass.

**A hard release gate:** a release may be cut after any of 01–05 and after 12, but
**no release may be cut between 06 and 11.** Migration fires on the absence of the
sources file, exactly once per machine, silently — an operator who upgrades into
that window burns their one migration on a binary where sources do not yet drive
role resolution. Making the migration re-runnable was rejected as a permanent
control bought for a five-ticket window.

Operationally, this means **every first-run path is developed and tested against a
throwaway config root**, because a real one gives exactly one migration.

**Documentation: vocabulary follows its ticket, narrative gets one pass.** The
glossary is edited by the ticket that makes an entry false — *skill library* and
*committed skills* die with the cut, *context bundle* loses its glossary and
manifest clauses at the payloads ticket, *role* loses "it selects a skill" at the
bindings ticket, *settings surface* is rewritten by 13, and *source*, *free
session* and *conventions* are written by 02, 09 and 03. The skill-sync doc is
deleted **by the cut, in the same commit as the ADR**, because it carries a
do-not-re-litigate block whose no-runtime-loading rule this effort reverses and no
ADR records — deleting it earlier opens a window where a retired decision has no
home. The getting-started doc and the repo's own guidance go to the documentation
ticket, which also does a final coherence read of the glossary. The design-system
doc is untouched: the split button composes existing primitives, so no new
component needs adding and no new token is required.

**A new ADR 0017 — *Skills come from registered sources; chartr ships none* —
written by the cut**, with ADR 0009 gaining a second banner beside its existing
one. A sixth amendment fails on that file's own pattern: all five existing
amendments open by saying the mechanism is untouched, and here the mechanism dies;
0009's execution half is already superseded, so amending the content half would
leave the file deciding nothing while still reading as operative policy. 0017
carries three things — the model (reaffirming that chartr composes its own payload
and assembles context fresh), the trust posture and its cost, and the
reproducibility retirement.

## Testing Decisions

A good test here asserts on what the design makes public — the files chartr writes,
the payload it composes, the git history it produces, the snapshots it pushes — and
never on how a package arrived there. That bar is the repo's existing one; this
effort adds seams rather than standards.

### The seams

**The process boundary is the primary seam, unchanged.** The existing test rig
starts the real chartr against a throwaway config root and a temporary space and
drives it exactly as an operator would, over HTTP and the control socket. Five of
the effort's new behaviours land there, and three of them exist *because* the
behaviour is silent — no UI reports it, so a test is the only place it is visible
at all:

1. **Clean-root first run.** An empty config root produces: the conventions and
   preferences files written, the default source materialized from the seed, the
   sources file written with the default enabled, the role table seeded with four
   qualified rows, and a ticket composition for a grilling ticket returning a
   payload containing the seed's `grill` body. One test, asserted end to end.
2. **Offline first run** — the same, with no `git` on `PATH`. The seed exists for
   exactly this case, and the test doubles as the registry's absent-git row.
3. **Migration, all three cases**, against a pre-populated old root: a legacy
   skills directory with one skill becomes a `Legacy skills` row; a built-in
   directory byte-identical to shipped is renamed aside and **not** registered; a
   built-in directory with one edited byte survives in place and becomes a
   `Migrated built-in skills` row.
4. **Payload goldens, both payloads**, through the preview endpoint. The free
   payload's golden is the **only** mechanical guard on the ignore test — chartr's
   own voice there is four sentences and nothing else will notice a fifth. A diff
   on that file is the review.
5. **Refuse the spawn** — an unresolvable binding aborts before the claim commit,
   with the error naming the role, the recorded binding string, and which of the
   three unresolvable shapes it hit. Asserted on the response and on the absence of
   a commit in the space's git history.

**One focused unit seam, in the new sources package.** This is exactly where the
existing prompt-package test sits today, and it exists for the same reason:
resolution and failure combinatorics are unreachable through HTTP without absurd
fixtures. It covers the discovery walk (depth bound, the descend-stop rule, the
skipped entries, basename naming), both resolution forms and the never-fall-through
rule, and the sixteen-row failure-mode table above. Real directories on disk,
asserted through the package's public surface.

### Prior art

The rig's package doc already states the stance this effort keeps — "no test
reaches into chartr internals; the one seam is the process boundary" — and it was
built to be extended by later tickets rather than joined by new seams. The payload
preview already has process-boundary tests asserting on the returned payload and
the files on disk; those are re-pointed rather than rewritten. The space registry's
own tests are the model for the sources package's unit tests: a TOML file under a
temp root, driven through the public API, with hand-written malformed files for the
degradation cases. The agent-detection tests in the terminal package show the
stub-executable-in-a-real-PTY technique, which this effort does not use.

### What is deliberately not tested, and why

- **The launched-agent split is verified by hand**, not by a test — open a free
  session on an agent with no shipped manifest and watch the tab's status. Decided
  with the operator. Worth recording that it ships alone and early precisely
  because it is a bug that exists at HEAD today, which is the argument the other
  way; the call was made with that in front of it.
- **The new-shell control and the settings section get no new frontend tests.**
  The deleted launch-menu module took its own test with it; the split button is
  existing primitives composed, with no logic in a testable module, and the
  settings section's shape is the operator's call at the time the ticket is worked.
  This is a design still moving, and a test written against it is churn.
- **The `chartr-skills` repo's contract is not machine-checked.** chartr validates
  nothing about a source's skills by design, so there is nothing in chartr to
  assert against; the acceptance test is a human reading against the contract
  document.

### Two manual gates

Neither is automatable, and both are required before the effort is called done:
**read both payloads in the cockpit's preview**, and **spawn a real grilling
session on this repo's own next map** — the self-hosting acceptance and the tracer
bullet's own demonstration. The cut adds one grep: the dead symbol list returns
nothing and the prompt package no longer embeds a skills directory.

### The bar per commit

Unchanged and applied per commit: `go vet ./...`, `go test ./...`, and in the
frontend the `check` and `build` scripts plus `vitest`. Every intermediate commit
must leave a chartr that builds, derives ticket status, and spawns. The escape
hatch mid-effort is weaker than the `simplify` map's and is named as such — an
empty shell plus the last good payload read off disk — so the discipline that
replaces it is: land the migration, the payloads and the cut as small,
independently-green commits, and never leave a red tree overnight.

## Out of Scope

- **Reaching agents in terminals chartr did not launch.** chartr guarantees what it
  launches, which after this effort is every session it starts. No managed block in
  a harness's global instruction file, no mirroring into a native skills directory,
  no shim on `PATH`. All three were weighed and refused: each buys partial coverage
  in exchange for per-harness knowledge and a file outside chartr's own config root.
  This is also why the tracker-adapter offer surface is deleted rather than shrunk.
- **`domain-modeling`, ADR tooling, and `ideate`.** Culled as non-essential.
  Nothing replaces them; they simply stop shipping, and anyone who wants them
  registers a source that has them.
- **Redesigning the wayfinder method.** chartr drives wayfinder maps and restates
  the format; it does not change it.
- **A plugin or extension API for skills.** The source list *is* the extension
  point, and it is a directory of markdown. A second mechanism would be speculative
  bloat.
- **Redesigning what this effort keeps:** the spaces and sessions model, the PTY
  terminal layer, the star map, the wayfinder map format, the agent library and its
  per-spawn selection, and the one-session-per-space invariant.
- **A trust confirm before a git clone, and a changed-skills summary after a
  refresh.** Both were argued and declined in favour of the fewest steps. See the
  accepted cost below.
- **A migration report, a live warning on an inert workspace-skills directory, and
  an in-app removal action for either in-repo artifact.** All three were drafted and
  rejected; every first-run write this effort produces is quiet, and chartr does not
  delete files in a repo it does not own.
- **Whether a space can pin a source.** Global sources cannot express "this
  project's skills", which is the one thing the deleted committed layer did. Whether
  that need is real, and whether a per-space subset or ordering answers it without
  putting artifacts back in the repo, is unresolved and deliberately left so.
- **Supporting files inside a source's skills**, beyond the ruling that they are
  ordinary files addressed relative to their own `SKILL.md` and that chartr gains no
  generic attachment API. Whether they are separately addressable is downstream of
  what the conventions absorbed and was not settled.

### Suggested, not settled — surface deliberately if you want them

Neither of the following came up on the map. They are named here so they can be
pulled in on purpose rather than promoted quietly:

- **A "reset to shipped" action on the default source row.** Deleting the directory
  already does it, and the map's revisit trigger names this as the escape if an
  operator ever wants chartr's newer seed *after* having pinned. It is not a story.
- **Cleaning up the renamed-aside built-in directory on a later startup.** Named as
  a revisit trigger if the directory turns out to accumulate; a deletion the
  operator has had a full release cycle to see. Not scoped here.

## Further Notes

### Accepted costs, recorded rather than smoothed over

- **The only assertion of trust in a git source's entire lifetime is the moment its
  URL is typed.** After that a refresh can move the source arbitrarily far and the
  only visible evidence is a short sha. A registered source is executable text
  injected into agents that run with permissions skipped. This is the sharpest
  trade-off in the effort and it is a decision, not an oversight.
- **The free payload cannot compel the read.** An agent that never opens the
  conventions file writes whatever its own defaults say, and the payload has no
  sentence that would stop it — by construction. The consequence-framed sentence is
  the whole mitigation, backed by the applicability rule *inside* the file for the
  agent that does open it.
- **Preferences can defeat the format contract.** The operator chose maximum
  control; a contradictory preference can make an agent write a map chartr cannot
  read. This amends the map's own starting decision that a free session receives no
  behavioural instruction.
- **Cross-machine reproducibility is retired.** A `dir` source is a folder editable
  between two spawns that records no version, so "this exact prompt ran" is no
  longer verifiable off a teammate's machine. What replaces it is weaker and honest
  about being weaker: the claim trailer names the source and pins it where the
  source is a git checkout, and the payload hash still fixes the exact bytes for the
  machine that composed them. ADR 0017 carries this record, because the file that
  held the reversed rule is deleted by the same commit.
- **Two silent migrations.** An operator whose fork stops driving its old role, and
  one whose committed workspace skills go inert, both find out by noticing the
  behaviour changed. Both were drafted with notices and both notices were rejected
  in favour of uniform silence. The settings surface is what makes this survivable,
  which is why it is in scope rather than a follow-on.
- **The standing rule ships knowingly broken in one named place.** The map says
  chartr writes nothing into the operator's repo but `.plan/maps/`, and the
  per-session payload copy inside the repo keeps being written. It is byte-identical
  to the archived copy under the data root, so deleting it would have cost nothing
  functional, and the one thing that could have refuted the deletion — an adapter
  sandboxed to its working tree — checks out clean. The operator kept both copies
  anyway: the payload sitting next to the work is worth more than the rule being
  literally true, and the directory is gitignored so nothing is committed. **Anyone
  reading the rule later must find this paragraph rather than discover the exception
  in the code.**

### Amendments to resolved tickets, so they are not mistaken for drift

- **Ticket 05 amends ticket 01's file shape:** two scalars persist beside the
  default toggle, and the default row is git-kinded rather than dir-kinded. Inside
  the delegation ticket 01 wrote, but a change to a resolved model.
- **Ticket 08 amends ticket 07's disposal:** the untouched built-in directory is
  renamed aside rather than deleted. It was the only irreversible operation in the
  effort, it runs once per machine, it is silent, and it rested entirely on a
  byte-comparison being right. A stray directory nobody cleans up and a wrong delete
  of an operator's only copy are not comparable. Everything ticket 07 argued for
  survives — the renamed copy stays unregistered, so the shadowing papercut stays
  rejected, and downgrade still works because the old binary's materializer never
  overwrites an existing file. **Nothing in this effort destroys data.**
- **Ticket 08 also amends ticket 07's migrated-row order**, which contradicted
  itself in one lead sentence: `Legacy skills` first, `Migrated built-in skills`
  second. Two of the three statements in that ticket and the underlying resolution
  order agree, and the correction is already recorded in the ticket file.
- **Ticket 04 amends the map's decision that a free session receives no behavioural
  instruction**, by admitting `preferences.md`.

### Risk

**The widest blast radius is the map-discovery narrowing** — it changes discovery
for every registered space at once, and getting it wrong empties the star map
everywhere, this map included. It is cheaply bounded, since this repo's maps are
already under `.plan/maps/`, so the mistake is visible in the developer's own
cockpit immediately. **Land it alone.**

### Definition of done, as capabilities

1. Register a folder or a git repo of skills and have chartr resolve ticket
   sessions' role prompts out of it, in an order you control — from a screen, not a
   text editor.
2. Launch an agent shell from the space card with no ticket at all, told what
   chartr is and what skills exist, and nothing about how to behave.
3. Read one generated conventions file that is the whole of chartr's file-format
   contract, and append a preferences file chartr will never override.
4. Take chartr's role skills by upgrading, or pin them with one fetch and never be
   moved again.
5. Upgrade off the layer model without losing anything: a fork you edited is
   registered as a source, and one you never touched is set aside rather than
   removed.
