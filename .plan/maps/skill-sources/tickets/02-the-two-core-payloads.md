---
type: grilling
blocked_by: [01, 04]
claimed_by: s2ec1358a7a62
claimed_at: 2026-08-06T04:39:26Z
---

# The two core payloads

## Question

chartr composes one payload today: core + role skill + a context bundle (`internal/prompt/compose.go`). It now composes two shapes, and neither is what it composes today.

The **free-session** payload is the new artifact and the harder one, because it is defined by a restraint: it tells the agent what chartr *is* and what skills exist, and nothing that would change how the agent behaves. The **ticket** payload keeps today's `core` body, but the role skill it used to concatenate now resolves out of a registered source, and the context bundle it carries sourced its glossary from a skill that may no longer exist.

Settle:

- **What "capabilities" means, concretely.** The free-session payload names the wayfinder map, the tickets, the folder structure. Is that a static paragraph, or does it carry live facts — this space's path, whether a map exists, which maps exist, the frontier? Live facts make the session useful immediately; they also make the payload a second reader of the map with its own staleness. Where is the line, and does an empty space (no `.plan/`) get a different payload from a space mid-effort?
- **How sources appear in it.** Name and location only, so the agent can lazily look one up. Does it list the *skills* discovered in each source too, or only the sources? Listing skills is what makes "use Matt Pocock's prototype skill" resolvable without a filesystem walk mid-turn; not listing them is what keeps the payload from growing with the operator's library. Where a source is git-backed, is the URL shown or only the local cache path?
- **The restraint, stated as a test.** "Don't provide any more instructions that may change the agent's behavior" needs an operational form, because the conventions pointer and the sources list are both, strictly, instructions. What is the rule that admits those two and rejects the next thing someone wants to add?
- **What the ticket payload keeps.** Today's `core` stays. Does the role skill's body still get concatenated ahead of the context bundle — now read from the resolved source — or does the payload point at it the way it points at the conventions? Concatenating preserves the guarantee that the role prompt was actually read; pointing keeps the payload small and treats sourced skills uniformly.
- **The context bundle's glossary.** It is inlined today from `tracker-convention`'s supporting file. If the conventions ruleset absorbs that vocabulary, the bundle sources it from the ruleset instead — or stops carrying it, because the payload already points at the ruleset. Decide, and say what `Bundle` and the `ctxPart` list look like afterwards.
- **The payload preview.** The cockpit renders a payload with per-segment provenance (`Segment.Layer`). Layers are gone; segments now come from sources, the embedded cores, and the bundle. What provenance does the preview show, and does the free-session payload get a preview at all — there is no ticket to preview it against.

## Done when

Both payloads are specified as concrete documents — what each section is, where its text comes from, and what varies per space or per ticket — together with the restraint rule for the free-session payload, the ruling on concatenate-versus-point for role bodies, and what becomes of the glossary part and the provenance the preview renders.

## Answer

**Two payloads built from five interchangeable parts, four of which are shared.
The free-session payload carries no live facts about the space at all — it is
the same bytes in an empty tree and a tree mid-effort — and chartr's own voice
in it is held to the *ignore test*: every sentence chartr writes must still be
true if the agent does nothing about it. The ticket payload keeps today's shape
— core, then the role body concatenated, then the tail — with the glossary part
gone and the skill-library manifest replaced by a sources block that both
payloads render identically.**

### The parts

Five parts. The free payload uses four of them; the ticket payload uses all
five and adds the map, the ticket and the blockers.

| part | kind | origin | free | ticket |
| --- | --- | --- | --- | --- |
| `core` | prompt | `chartr` (embedded) | free variant | ticket variant |
| `role` | prompt | the source's registered name | — | the bound skill's body |
| `conventions` | prompt | `chartr` (embedded) | ✓ identical bytes | ✓ identical bytes |
| `preferences` | prompt | `operator` | ✓ | ✓ |
| `sources` | context | `context` | ✓ identical rendering | ✓ identical rendering |
| `map`, `ticket`, `blocker #NN` | context | `context` | — | ✓ |

Order is ticket 04's, unchanged: core, role, conventions, preferences, then the
context region. The `sources` part is *context*, not instruction, so in both
payloads it renders below the `# Context` rule that `renderMarkdown` already
draws — which is why preferences remain the last instruction bytes in both. One
rule holds the whole document together: **instructions, then data.**

### The free-session payload

`core`, the free variant — embedded, and the only genuinely new prose on this
ticket:

```markdown
# chartr

chartr launched this shell. It is a cockpit for driving agent sessions in a git
working tree — this one, at your current working directory.

- chartr reads wayfinder **maps** under `.plan/maps/` in this tree: one
  directory per map, holding a `map.md` and `tickets/NN-slug.md`. It reads maps
  nowhere else, and `.plan/maps/` is the only path it writes in this tree.
- A ticket is one question or one unit of work. chartr spawns a session against
  exactly one ticket, in one working tree, and hands it that ticket, its map,
  and its blockers' answers.
- The skills the operator registered are listed below, by name and location.
```

`conventions` — embedded, identical in both payloads:

```markdown
chartr parses the files under `.plan/maps/`. A file there is read only where it
follows the format stated at `~/.config/chartr/conventions.md`, which also
states when the convention applies.
```

`preferences` — the raw bytes of `~/.config/chartr/preferences.md`, no heading,
no wrapper, exactly as ticket 04 fixed them. **Omitted entirely when the file is
empty**, so the payload never carries a heading over nothing.

`sources` — rendered from ticket 01's registry, enabled sources in file order,
the default row last:

```markdown
## Skill sources

Skills the operator registered. A skill's `SKILL.md` sits under the path shown.

- **Matt Pocock Skills** — `~/.config/chartr/sources/3f2a1b9c4d7e`
  grill-me, handoff, prototype, research, to-spec, to-tickets, wayfinder
- **chartr-skills** — `~/.config/chartr/sources/chartr-skills`
  grill, implement, prototype, research

A name in two sources resolves to the first listed; the other is addressed as
`Source name/skill`.
```

**What varies per space: nothing.** Capabilities are static structure, not live
facts — no map list, no frontier, no branch for a space without `.plan/`. The
agent is already cwd'd in the tree and can list `.plan/maps/` in one tool call,
while a list composed at spawn is wrong the moment another session resolves a
ticket, and a free session outlives that snapshot. This also deletes the
empty-space-versus-mid-effort question the ticket raised: there is one document.

**What varies per operator:** the sources block and the preferences bytes. That
is the whole of it.

### The sources block, in detail

- **Skill names, comma-joined, no descriptions.** Names fall out of ticket 01's
  walk, which stats directories and reads no file; descriptions would cost one
  `SKILL.md` read per skill at every compose and are the thing that would make
  this block grow without bound. The names are the short tokens a free session
  says out loud and a `[roles]` binding writes, so they are also the addressable
  form.
- **The local checkout path only — never the git URL.** The path is what is
  readable; the URL is a fetchable address, and this map's standing decision is
  that nothing fetches unattended. Not printing it is the cheapest way to keep
  that shut. The operator's own source name already says whose skills these are.
- **Disabled sources do not appear.** *Disabled* means one thing (ticket 01);
  a source the agent cannot resolve through has no business being listed.
- **The shadowing sentence is a fact, not advice** — it states how `resolve`
  behaves and names the qualified form, which is the map's answer to
  "naming collisions in free-session lookup".

### The restraint, as a test

> **Every sentence chartr writes in its own voice must still be true if the
> agent does nothing about it.**

A fact about the machine survives being ignored; an instruction's entire content
is what the agent should do, so ignoring it is the only way it can be false.
That admits the two cases the ticket named — a location (`conventions.md`) and
an inventory (the sources block) — and rejects the next addition, which will
almost always arrive phrased as a *should*.

```
PASSES                                  FAILS
"chartr reads maps under .plan/maps/"   "Read the conventions first."
"The format is stated at <path>."       "Prefer grill-me for design work."
"Matt Pocock Skills is at <path>:       "Commit in focused commits."
 grill-me, prototype, …"
```

Two consequences worth stating plainly:

- **The test forces consequence-framing, and that is the technique, not a
  loophole.** chartr may not say "read the conventions"; it *may* say "a file
  under `.plan/maps/` is read only where it follows the format stated at
  `<path>`". Both point at the same file; only the second is still true when
  ignored, and it carries the entire reason to open it. Where a fact needs to
  motivate an action, state the consequence, never the command.
- **`preferences.md` is exempt**, by ticket 04. It is the operator's voice, not
  chartr's, and it is unwrapped, unlabelled and unranked. The restraint here
  governs what *chartr* injects; it makes no claim about the payload as a whole.

**Knowingly accepted: the free payload cannot compel the read.** An agent that
never opens `conventions.md` writes whatever its own defaults say, and this
payload has no sentence that would stop it — by construction. The consequence
sentence is the whole mitigation, backed by ticket 04's applicability rule
*inside* the file for the agent that does open it. This is the sharpest cost on
this ticket and it is recorded rather than smoothed over; its revisit trigger is
below.

### The ticket payload

- **`core` keeps today's body with exactly one edit.** The sentence listing what
  was assembled — "the map, this ticket, the answers of the blockers it depends
  on, and the glossary" — drops its last clause. The glossary is gone (below);
  a core that still promises it is lying to every session.
- **The role body is concatenated, not pointed at.** It runs between `core` and
  the conventions pointer, exactly where ticket 04's instruction order puts it —
  that order lists the bound role skill as an instruction and the conventions as
  a *path*, and the distinction was deliberate. The role prompt is what makes a
  ticket session that role; concatenating is what makes the claim trailer's skill
  record mean *this text ran* rather than *this text was suggested*. Pointing was
  the live alternative and it loses that guarantee for a payload-size saving that
  nothing on this map needs.
- **No origin line above the body.** The payload prose does not say which source
  the role came from; the preview badge and the claim trailer both do, and the
  payload is what the agent reads, not an audit record. A role skill's supporting
  files stay reachable — the sources block gives the source's path and the skill's
  name, which is that skill's directory (ticket 01's basename rule).
- **The tail is identical to the free payload's:** the same conventions bytes,
  the same preferences bytes, the same sources block, then the map, the ticket
  and each blocker's answer as today.

### The glossary, `Bundle`, and the `ctxPart` list

Ticket 04 settled it and this ticket does not reopen it: **`glossary.md` is
deleted and the bundle stops carrying a glossary part.** Format vocabulary lives
in `conventions.md` where it is used; method vocabulary lives in the sourced
method skill. There is no second glossary and no replacement `Support()` path.

Afterwards:

- **`Bundle` is unchanged.** It never held the glossary — `Compose` resolved
  `tracker-convention` and read its supporting file inline. That whole block
  goes, along with `TrackerSkill`, `GlossaryFile` and `Skill.Support`.
- **The `ctxPart` list becomes:** `sources`, `map`, `ticket`, `blocker #NN…` for
  a ticket payload, and `sources` alone for a free one. `skill-library` is
  renamed `sources` and rendered from the registry rather than `prompt.Names()`;
  `skillManifest(Roots)` dies with `Roots`.

### The preview

**Provenance survives, as the source's own name.** `Segment.Layer` becomes
`Segment.Origin`, a free string:

| origin | what it tags |
| --- | --- |
| `chartr` | the embedded cores and the conventions pointer |
| *the source's registered name* | a resolved skill body |
| `operator` | `preferences.md` |
| `context` | every assembled part |

The badge answers the one silent failure source order can cause — you dragged a
source up the list and every grilling ticket now gets a different prompt — and
it separates chartr's bytes from the operator's, which matters more after ticket
04 let preferences contradict the conventions. The frontend's `layerVariant`
lookup keeps its `?? 'outline'` fallback, so an open string set degrades to the
neutral badge rather than breaking.

**The free payload gets a preview, and it is space-independent.** Same seam,
same modal, same part list — four parts instead of eight, and no ticket or role
selector. It has no per-space input now that capabilities are static, so one
preview is truthful everywhere. It is also the only place the operator sees
their own `preferences.md` land in an assembled document, which is the file
ticket 04 permits to defeat the format contract. It hangs off the sources
settings surface rather than a ticket; **exactly where is the settings surface's
call** (still "not yet specified" on the map), and it is deliberately not put on
the space card, whose shape is ticket 06's.

### The seam

```go
func ComposeTicket(in TicketInput) (Payload, error)
func ComposeFree(in FreeInput) (Payload, error)

type TicketInput struct {
    ConfigDir string
    Role      string
    Skill     sources.Skill   // the bound role skill; ticket 03 resolves it
    Sources   *sources.Registry
    Bundle    Bundle          // unchanged
}

type FreeInput struct {
    ConfigDir string
    Sources   *sources.Registry
}

type Payload struct {
    Kind      string   `json:"kind"` // "ticket" | "free"
    Role      string   `json:"role,omitempty"`      // ticket only
    TicketNum int      `json:"ticketNum,omitempty"` // ticket only
    Parts     []Part
    Skills    []Skill
    Warnings  []string
    Markdown  string
}

type Segment struct {
    Origin string `json:"origin"`
    Label  string `json:"label,omitempty"`
    Text   string `json:"text"`
}
```

- **Both entry points reconcile `conventions.md` before composing**, including
  on a preview — ticket 04's rule, and the reason `ConfigDir` is an input.
- **Unreadable `preferences.md` fails the composition visibly** (ticket 04);
  a missing one is recreated empty and the part is omitted.
- **`Warnings` survives with one named case:** a registered source whose status
  is `unavailable` (ticket 01), because a source that vanished is precisely what
  silently changes what a payload says. The stale-fork warnings go with the fork
  model; what remains of provenance is ticket 05's.
- **`Resolve(name, Roots)` is not called from here any more.** Ticket 03 hands
  `ComposeTicket` an already-resolved role skill, so composition never decides
  which source wins.

### Rejected

- **Live space facts in the free payload** (map list, or map list plus frontier).
  Rejected: the payload becomes a second reader of the map with its own
  staleness, composed once and outlived by the session that holds it, duplicating
  a star map the operator is already looking at. The agent can list a directory.
- **Skill counts instead of skill names.** Cheaper and never grows. Rejected
  because an agent that does not know a skill exists never looks for one, and the
  names cost nothing — ticket 01's walk already has them.
- **Names with descriptions**, closest to today's manifest. Rejected: one
  `SKILL.md` read per skill at every compose, and unbounded growth with the
  operator's library, for a one-line hint the agent can read on demand.
- **Printing a git source's URL.** Rejected: it is a fetchable address in a
  payload handed to an agent running with permissions skipped, on a map whose
  standing decision is that refreshing a source is always an explicit human act.
- **Pointing at the role body** instead of concatenating. Rejected above: it
  makes "did this session actually run as a grill?" unknowable, and contradicts
  ticket 04's instruction order.
- **A size budget as the restraint rule** (chartr's prose ≤ N lines). Rejected:
  mechanically checkable and substantively empty — "always read the map first"
  is four words and is exactly what the restraint exists to keep out.
- **The relocation test** (behaviour allowed, but only inside a pointed-at file).
  Rejected: it licenses growing `conventions.md`, which ticket 04 fenced off as
  format-only, so it would leak method teaching back into the one file chartr
  cannot let an operator disable.
- **A fixed four-word provenance vocabulary** (`chartr`/`source`/`operator`/
  `context`) with the name in the label. Rejected: it keeps a styling lookup
  exhaustive at the cost of burying the one fact the badge is read for.
- **Dropping the badge.** Rejected: it deletes the shadowing check the modal is
  opened for.
- **No free-session preview.** Rejected: three of the four parts are files on
  disk, but the assembled order — and how a preferences file reads once it lands
  after the conventions pointer — is visible nowhere else.
- **Hanging the free preview off the space card's new-shell menu.** Not wrong,
  but it constrains a control ticket 06 owns, and the payload has no per-space
  input to justify living there.

### Surfaced doubts

- **`Part` now always holds exactly one `Segment`.** It did under whole-skill
  shadowing too; the multi-segment machinery was built for a per-field merge that
  never shipped. Collapsing `Segment` into `Part` is a real simplification and it
  touches the wire format and the frontend, so it is flagged for the
  implementation map rather than decided here — the `Origin` rename lands in the
  same file either way.
- **`.chartr/run/<sid>/payload.md` sits inside the operator's repo**, gitignored,
  while this map's standing rule is that the operator's repo carries no chartr
  operating artifacts and the decisions list names only `docs/agents/issue-tracker.md`
  and `.chartr/skills/` as deleted. A free session's payload delivery is ticket
  06's call and the ticket session's location is nobody's on this map; someone
  should own the discrepancy before implementation, because the rule as written
  does not survive contact with `run/`.

### Revisit triggers

- **The ignore test's accepted cost** is revisited the first time a free session
  demonstrably writes maps outside `.plan/maps/` or in an unparseable shape. The
  escape is one more consequence-framed fact, and if that fails, admitting a
  single imperative and recording it as an exception rather than quietly widening
  the rule.
- **Skill names in the sources block** are revisited the first time a registered
  source's name list is long enough to be a visible share of the payload. The fix
  is a per-source cap falling back to a count, not a model change.
- **Static capabilities** are revisited if free sessions habitually burn their
  first turns rediscovering what maps exist. What would change is one live line
  (the map slugs), never the frontier.
- **The origin badge** gains importance if ticket 03 binds roles by bare name
  rather than qualified: the badge would become the only place a shadowed role
  prompt is visible. That raises the stakes on the decision made here; it does
  not change it.
