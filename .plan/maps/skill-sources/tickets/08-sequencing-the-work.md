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

## Answer

**One implementation map, **build-then-cut**, opening on a four-ticket frontier and
closing on one deletion-only ticket. The `simplify` map's cut-first precedent does
**not** transfer, and the reason is precise: cut-first won there because 02 would
otherwise have ported review code 01 was about to delete. That migrate-then-delete
trap is absent here — the sources registry is a *new package*, not a port of the
layer resolver, and the layer resolver is deleted un-ported. What does transfer is
`simplify`'s stance (one map, small independently-green commits, self-hosting as the
binding constraint), and the deciding argument against cutting first is sharper than
a preference: `simplify`'s cut removed a *gate*, so the escape hatch was "a terminal
and `git`". This cut removes the *engine*. A tree with the layer model deleted and
`internal/sources` not yet wired cannot compose a payload, so it cannot spawn the
sessions that would do the remaining work — chartr would be unable to drive its own
implementation map.**

**The tracer bullet is a milestone, not a ticket**, and it completes at the payloads
ticket. **One thing ships genuinely alone, and it is a fix to a bug that exists at
HEAD today.** **A hard release gate falls out of ticket 07's trigger analysis: no
release may be cut between the migration ticket and the cut.** **A new ADR 0017, not
a sixth amendment to 0009.** **The settings section for sources is in scope** and is
what makes ticket 07's silence survivable.

Two decisions here **amend resolved tickets**, both confirmed with the operator
(2026-08-06) and both recorded as amendments rather than smoothed over:

- **Ticket 07's `builtin-skills/` disposal becomes a rename, not a delete.** Details
  and reasoning below, under "The one act that cannot be undone — and now can be."
- **The in-repo payload copy stays.** Ticket 06 handed the question forward; the
  operator's answer is to keep writing both copies, so this map's standing rule
  ("nothing in the operator's repo but `.plan/maps/`") **ships knowingly broken** in
  one named place. Recorded below under "The rule that ships broken."

### The shape of the effort

The ticket's premise — "there may be no honest half-way state" — is half right. There
is no half-way state in which *the new model is useful*: a registry with no bindings
resolves nothing, and the map's own destination is not reached until the cut lands.
But there are four pieces of work whose value does not depend on the new model at
all, and putting them first buys a wide frontier and one real bug fix before the
effort's risk begins.

The effort is also bigger than its seven tickets. **Three pieces of work no ticket on
this map owns** are in the order below and are not optional: the `chartr-skills` repo
itself, the sources settings section, and the documentation pass.

### The order

Forced edges, named at the file. `→` means "must land after".

| # | ticket | must land after | why the edge is forced |
| --- | --- | --- | --- |
| **01** | **`chartr-skills`, the repo** | — | prose, not Go; longest lead; no chartr code depends on it compiling |
| **02** | **The sources registry, `dir` only** (`internal/sources`) | — | new package; nothing consumes it yet |
| **03** | **`conventions.md` + `preferences.md`** (embed, materialize) | — | writes two files nothing points at yet |
| **04** | **The `launchedAgent` split** | — | `internal/terminal` only; fixes a HEAD-level bug |
| **05** | **Seed and vendoring** | 01, 02 | vendors 01's commit into 02's default-row path |
| **06** | **Migration + the tracker-adapter surface** | 02 | needs `sources.toml`'s writer; **must precede every other `sources.toml` writer and the cut** |
| **07** | **Role bindings + the `Skill:` trailer** | 02, 05 | `[roles]` must point into a source that exists |
| **08** | **The two payloads** | 02, 03, 07 | ← **tracer bullet completes here** |
| **09** | **The new-shell control** | 04, 08 | needs `ComposeFree`; needs `launchedAgent` |
| **10** | **Git sources: clone, refresh, pin** | 02 | free after 02; lifted off the critical path (below) |
| **11** | **The cut** + ADR 0017 + delete `docs/skill-sync.md` | 06, 08, 09 | every consumer of the layer model must have moved |
| **12** | **Discovery narrows to `.plan/maps/`** (`mapscan`) | 03 | free; placed late because it is behavioural |
| **13** | **The sources settings section** | 07, 10, 11 | renders 10's git rows and 07's bindings, on 11's smaller screen |
| **14** | **Documentation pass** | all | narrative prose describing the end state |

**01–04 are all frontier on day one.** Sizing is `to-tickets`' call — 12 may fold into
03 and 10 into 02 — but **every edge in that table must survive the folding.**

Three of those tickets are not on this planning map and must be written anyway:

- **01, `chartr-skills`** is the effort's longest-lead item and it is *not code*: seven
  skills re-authored against ticket 05's two-clause test, the placeholder method
  guidance re-authored as prose, `CONTRACT.md`, MIT plus the attribution that travels
  with the three Pocock-derived skills. It blocks 05 and nothing else, so it runs
  concurrently with 02–04 rather than gating them. **Confirmed with the operator:**
  the code path waits for real bytes rather than shipping a placeholder seed.
- **13, the settings section.** Its stated precondition — "cannot be drawn until the
  registry's shape and the binding's failure modes are settled" — is now met, so the
  map's "not yet specified" entry graduates into the implementation map rather than a
  new planning map. **Confirmed with the operator: it is in scope**, and it is what
  ticket 07's silence depends on. See "The settings surface is load-bearing" below.
- **14, the documentation pass**, below.

### The tracer bullet

**One `grilling` ticket session spawns, and the body between its core and its
conventions pointer came out of `<configDir>/sources/chartr-skills/grill/SKILL.md`,
resolved through `sources.Resolve("chartr-skills/grill")`.** Demonstrated, not
asserted: spawn it on this repo's own next map and read the composed payload in the
cockpit's preview.

That path runs when ticket 08 lands. What it deliberately leaves broken at that
moment:

- **No free sessions.** The skill launcher still stands; `prompt.Launch` still
  composes its cold-launch prompt. `ComposeFree` exists and only the preview calls it.
- **No git sources.** `dir` only. No clone, no refresh, no pin, no
  `default_commit`/`default_fetched` — so the default row reads *"shipped with this
  build"* and cannot be moved off it.
- **No settings surface.** A second source is registered by hand-editing
  `sources.toml`, which is exactly what ticket 01's "a missing file is the first-run
  state" makes safe to do.
- **Both models in the tree.** `prompt.Roots`, `Names()`, the embedded
  `assets/skills`, `configsurface`'s skill rows and `<configDir>/builtin-skills/` are
  all still live and still compiling. Nothing reads them on the spawn path.
- **`.plan/maps/` not yet enforced** — `mapscan` still walks `.plan/` recursively.

**Rejected: making the tracer bullet a *ticket* by slicing five tickets in half.**
It is the textbook move and it is wrong here. Every half-ticket leaves a "finish the
registry", "finish the seed", "finish the payloads" stub, and stub tickets are where
scope quietly dies. The bullet's value is as a **verification gate** — the point at
which the effort is proven end-to-end before the cut is attempted — and it delivers
that as a milestone without fracturing the work. The one split that *does* earn its
place is 02/10 (`dir` versus `git`), because it falls on a seam ticket 01 drew itself
(`kind` is declared, never inferred) and it moves the clone/refresh/PATH-gate
machinery off the critical path entirely.

### What ships independently — and what only looks like it

**Genuinely alone:**

- **The `launchedAgent` split (04).** Ticket 06 found it as evidence for a field; it is
  independently a bug fix to today's tree. An on-ramp tab on an agent with no shipped
  manifest reads idle for its whole life (`terminal.go:666`), and the boot flash at
  `terminal.go:436` is the same root cause. `internal/terminal` only, testable alone,
  and landing it away from ticket 09's UI churn is what keeps a subtle detection
  change out of a big diff.
- **`conventions.md` + `preferences.md` (03),** in the sense the ticket meant:
  embedding a document and materializing two files while the layer model is still
  live is risk-free and front-loads the map's largest *writing* job.

**What only looks separable:**

- **The new-shell control as a whole.** Its payload is `ComposeFree`, which is ticket
  08's. Shipping it early on today's `prompt.Launch` output means writing throwaway
  wiring *and* leaving its deletion list half-executed — the `on-ramp:` allowlist
  cannot die while `prompt.Launch` still takes a skill name. Only the `launchedAgent`
  half is free.
- **The conventions ticket as a whole.** Materializing is free; the `mapscan`
  narrowing (12) and the `tracker-convention`/`glossary.md` deletions (11) are not.
  Ticket 04 decided one thing; it lands in three places.
- **The registry alone (02).** It compiles, it is fully tested, and it delivers the
  operator no capability whatsoever. It is a foundation, not a ship — worth saying
  because a green test suite on a new package reads like progress and is not, on its
  own, a half-way state anyone can use.
- **Git sources (10)** look like part of the registry and are the cleanest thing to
  lift out of it.

### The deletion order

**Build alongside, then cut, in one deletion-only ticket (11)** — confirmed with the
operator. Three arguments, in order of force:

1. **Self-hosting.** The cut removes the payload composer's only path to a role
   prompt. Cut first and the cockpit cannot spawn a ticket session against its own
   implementation map until the registry, the seed, the bindings and the payloads all
   land — four tickets of blindness, worked from an ad-hoc shell with the context
   assembled by hand. `simplify` could cut first because its cut *made* the escape
   hatch; this one removes it.
2. **Ticket 06 consumes the embed the cut deletes.** The migration's
   `builtin-skills/` disposal is a byte-comparison against `prompt.ShippedHash` /
   `hashFiles` over the embedded `assets/skills` — "the migration just gets to use
   them once before they go". Cut first and it must either reinvent the differ or drop
   the preserve-an-edited-fork behaviour it was argued into. **This is a hard edge,
   not a preference: `06 → 11`.**
3. **There is no migrate-then-delete trap to avoid.** `internal/sources` shares no
   code with `internal/prompt`'s layer half; nothing is ported, so nothing is ported
   twice. The pressure that produced `simplify`'s order is simply absent, and copying
   the order without the pressure is the same mistake ticket 01 refused when it
   declined to clone `internal/registry`'s machinery.

The intermediate cost — "every intermediate ticket carries both models" — is smaller
than it sounds, and the table above is the proof: exactly one ticket (08) touches
both, and it touches the old one only to stop calling it. Tickets 02, 03, 05, 07 and
10 never mention the layer model at all.

**What ticket 11 deletes,** consolidated so `to-spec` has one list: `prompt.Roots`,
`RootsFor`, `Resolve`, `Names`, `Library`, `Materialize`, `LayerBuiltin`/`User`/
`Workspace`; `hashFiles`, `ShippedHash`, `Skill.Hash`, `Skill.Stale`, `ForkedFrom`,
`staleWarning`, `LibraryWarnings`; the `assets/skills` embed and
`<configDir>/builtin-skills/` with its `readmeText`; the `tracker-convention` skill
and `glossary.md`; `prompt.Launch`, `prompt.Ideate`, `prompt.IdeateSkill` and the
`/ideate` route; `on-ramp:`/`needs-context:` and their model and TypeScript mirrors;
`configsurface.go`'s `resolvedSkills`/`resolveSkillDir` and the `layerSkillPrefix`
hatch. Plus `docs/skill-sync.md` and ADR 0017, for the reason in the ADR section.
Strictly deletion and documentation — the moment it grows anything behavioural it has
stopped being the cut.

### What ships before the whole effort does

**A release may be cut after any of 01–05, and after 12. No release may be cut
between 06 and 11** — confirmed with the operator. This falls straight out of ticket
07's own trigger: migration fires on *the absence of `sources.toml`*, exactly once per
machine, and it is silent. An operator who upgrades to a build in that window burns
their one migration on a binary where sources do not yet drive role resolution —
their fork is registered into a list nothing reads, and the trigger is gone when the
real build arrives.

Rejected: **making the migration re-runnable** (an explicit re-scan action) so
releases are unconstrained. It is a decision ticket 07 did not make, and it is a
permanent control bought for a five-ticket window. Also rejected: **releasing freely
and accepting the burn** — on this repo the operator inside the window is the
operator, and they would find out the way ticket 07's accepted silence already
guarantees, by noticing behaviour changed.

The same fact is an operational rule for whoever implements it: **develop and test
every first-run path against a throwaway root** (`XDG_CONFIG_HOME=$(mktemp -d)`,
which `server.ConfigRoot` already honours), because your own real config root gives
you exactly one migration and testing 02 by hand will spend it.

**The first-run sequence is one function assembled across four tickets** (06's steps 3
and 2, 05's reconcile, 07's `[roles]` seed, 03's `conventions.md`) **in ticket 07's
stated order.** Each ticket adds its step in place rather than inventing its own
startup hook.

### Documentation

**Vocabulary follows its ticket; narrative gets one pass.**

- **`CONTEXT.md` is edited by the ticket that makes an entry false** — not per-ticket
  on principle, and not all at the end. Every session on this map is told vocabulary
  comes from `CONTEXT.md`, so a stale one actively misinforms the next session. In
  practice: **Skill library** and **Committed skills** die with the cut (11);
  **Context bundle** loses its glossary and skill-library-manifest clauses at the
  payloads ticket (08); **Role** loses "it selects a skill" at the bindings ticket
  (07); **Settings surface** is rewritten by 13. New entries — **Source**, **Free
  session**, **Conventions** — are written by 02, 09 and 03 respectively.
- **`docs/skill-sync.md` is deleted by the cut (11), not by the vendoring ticket.**
  Ticket 05 fixed *that* it dies; this is *when*. It carries a "do not re-litigate"
  block whose **no runtime loading** rule is reversed by this effort and recorded in
  no ADR — checked against all sixteen — so deleting it before ADR 0017 exists opens a
  window where a retired decision has no home. Same commit, both files.
- **`docs/getting-started.md` and `CLAUDE.md` go to one documentation ticket (14).**
  Both are narrative prose describing an end state; every intermediate version is
  wrong regardless of when it is written, so writing them seven times is churn. 14
  also does a final coherence read of `CONTEXT.md` — the per-ticket edits keep it
  honest, not necessarily elegant.
- **`docs/design-system.md` is untouched**, but ticket 09 is UI and CLAUDE.md binds it.
  Checked: the split button is the existing `button` + `dropdown-menu` primitives
  composed, so **no `shadcn-svelte add` step is needed** and no new token is required.

### The ADR

**A new ADR 0017 — *Skills come from registered sources; chartr ships none* — written
by the cut (11), with ADR 0009 gaining a second banner beside its existing one.**
Confirmed with the operator.

The sixth-amendment option is genuinely tempting and it fails on the file's own
pattern. Every one of 0009's five amendments opens by saying *the mechanism is
untouched* and then retires a consequence. Here **the mechanism is what dies**. And
0009's execution half is already superseded by `agent-selection`: delete the content
half by amendment and the file decides nothing at all while still reading, top to
bottom, as operative config policy. The banner is the file's own idiom for exactly
this, used once already.

Rejected: **0017 supersedes 0009 entirely.** The execution half's supersession has its
own provenance in the `agent-selection` effort and 0017 was not part of it. 0017
supersedes the *content* half; with the existing banner covering the other, 0009
becomes wholly historical, and the new banner says so in one line.

0017 carries three things, and it is the *three* that settle it — an amendment
carrying three unrelated records is a chapter:

1. **The model** — one ordered list of operator-owned sources, `resolve(name)` as first
   hit, explicit `[roles]` bindings, chartr shipping only two payloads and a
   conventions ruleset. Reaffirms **ADR 0002** (chartr composes its own payload) and
   **ADR 0005** (context assembled fresh).
2. **The trust posture and its cost** — a registered source is executable text reaching
   agents run with permissions skipped, and the only assertion of trust in a git
   source's lifetime is the moment its URL is typed. Ticket 01's sharpest accepted
   trade-off has no other durable home once it leaves a ticket file.
3. **The reproducibility retirement** — `docs/skill-sync.md`'s **no runtime loading**
   rule, reversed knowingly: two machines no longer resolve identical bytes for the
   same ticket. Ticket 05 handed this to "ticket 07 or 08"; 07 did not take it, so it
   lands here.

### What is verified before it is called done

The standing bar is unchanged and applies per commit: `go vet ./...`, `go test ./...`,
and in `web/` the `check` and `build` scripts plus `vitest`. This effort adds five
things, and the first three exist because the behaviours they cover are **silent** —
no UI reports them, so a test is the only place they are visible at all.

1. **Clean-root first run.** An empty config root: `conventions.md` and
   `preferences.md` written; `sources/chartr-skills` materialized from the seed;
   `sources.toml` written with `default_enabled = true`; `[roles]` seeded with four
   qualified rows; and `ComposeTicket` for a `grilling` ticket returning a payload that
   contains the seed's `grill` body. One test, asserted end to end.
2. **Offline first run** — the same, with `PATH` carrying no `git`. The seed exists for
   exactly this case, and the test doubles as ticket 01's "`git` absent from PATH" row.
3. **Migration, all three cases** — a pre-populated old root: a `<configDir>/skills/`
   with one skill becomes a `Legacy skills` row; a `builtin-skills/` byte-identical to
   shipped is **renamed to `builtin-skills.migrated/` and not registered**; a
   `builtin-skills/` with one edited byte survives in place and becomes a `Migrated
   built-in skills` row.
4. **Payload goldens, both payloads.** The free payload's golden is the *only*
   mechanical guard on ticket 02's ignore test — chartr's own voice is four sentences
   and nothing else will notice a fifth. A diff on that file is the review.
5. **Refuse-the-spawn** — an unresolvable binding aborts before `writeClaimCommit`,
   with the error naming the role, the recorded binding string, and which of the three
   unresolvable shapes it hit.

Two manual gates, neither automatable: **read both payloads in the cockpit's
preview**, and **spawn a real grilling session on this repo's own next map** — the
self-hosting acceptance, and the tracer bullet's own demo. Ticket 11 adds one grep:
the dead symbol list returns nothing and `internal/prompt` no longer embeds
`assets/skills`.

### Self-hosting, and the one act that cannot be undone — and now can be

Every intermediate commit must leave a chartr that builds, derives ticket status, and
spawns.

**Ticket 06's `builtin-skills/` disposal becomes a rename** — `builtin-skills/` →
`builtin-skills.migrated/`, left unregistered, for the operator to remove or ignore.
**This amends ticket 07**, which chose a delete on ownership grounds ("chartr wrote
it, chartr owns it"), and the operator changed it once the consequence was stated
plainly. The reasoning, recorded so the amendment is not mistaken for drift:

- The disposal is the **only irreversible operation on this map**, it runs exactly
  once per machine, it is silent, and an operator's edited fork lives nowhere but that
  directory — no git, no backup. It rests entirely on a byte-comparison being right.
- A rename costs a stray directory nobody cleans up. A wrong delete costs work that
  cannot be recovered. Those are not comparable.
- **It preserves everything ticket 07 argued for.** The renamed copy is *not*
  registered, so the "two rows shipping the same seven names" shadowing papercut
  ticket 07 rejected stays rejected. Chartr still stops carrying an untouched copy
  with a stale README describing a dead model; it just stops carrying it somewhere
  recoverable.
- **Downgrade still works, for ticket 07's own reason.** `Materialize`
  (`internal/prompt/prompt.go:415`) never overwrites an existing file, so an old
  binary recreates `builtin-skills/` from its own embed on the next startup exactly as
  it would after a delete. The renamed directory is inert to it.
- The three-case test still lands in the same commit.

With that, **nothing in this effort destroys data.** The remaining risk is width, not
depth:

- **Widest blast radius: ticket 12 (`mapscan` narrowing).** It changes discovery for
  every registered space at once; get it wrong and the star map is empty everywhere,
  this map included. Cheaply bounded, though — this repo's maps are all under
  `.plan/maps/` already, so the mistake is visible in the developer's own cockpit
  immediately. Land it alone.

**The escape hatch is weaker than `simplify`'s and worth naming as such.** If the
cockpit cannot spawn mid-effort, the fallback is an empty shell plus the last good
payload, readable at `.chartr/run/<sid>/payload.md` in the tree (see below) or at
`<dataDir>/sessions/<sid>/payload.md`. That is a workaround, not a lifecycle —
`simplify` could honestly say "a terminal and `git` finish any ticket" because it had
removed a gate. This effort is rebuilding the engine, so the discipline that replaces
it is: **land 06, 08 and 11 as small, independently-green commits, and never leave a
red tree overnight.** A stated discipline the operator owns, backed by the build/test
tripwire — not a new gate.

### The rule that ships broken

**`.chartr/run/<sid>/payload.md` keeps being written inside the operator's repo.**
Confirmed with the operator, against my own recommendation, and it is the one place
this map's standing rule — *the operator's repo carries no chartr operating
artifacts* — ships knowingly unhonoured.

What was established before the call, so the trade is on the record:

- The in-repo copy and the archive under the data root are **byte-identical**
  (`spawn.go:311` and `:315` write the same `payload.Markdown`), so deleting the
  in-repo copy would have cost nothing functional.
- Ticket 06 named the one thing that could refute the deletion — "an adapter sandboxed
  to its working tree" — and **it checks out clean**: `internal/adapter` models exactly
  one thing about a CLI, prompt delivery, and says so in its package doc; sandboxing
  is entirely operator-supplied `args`. Deleting it was safe.
- Had it been deleted, `.chartr/` would have stopped being created in a registered
  space at all — the lock file lives at the *data* dir (`cmd/webview/lock.go:25`), not
  the space, and ticket 07 already stops chartr reading `.chartr/skills/`.

The operator kept both copies anyway: the payload sitting next to the work is worth
more than the rule being literally true, and the whole directory is gitignored, so
nothing is ever committed. **What this costs, stated plainly:** the map's Decisions
list says chartr writes nothing into the operator's repo but `.plan/maps/`, and after
this effort that sentence is still false in exactly one way. Anyone reading the rule
later must find this paragraph, not discover the exception in the code. `halt.go:261`
and `writeSessionPayload` are untouched, and ticket 09's `OpenFree` inherits them
unchanged — so **this decision removes a ticket from the sequence rather than adding
one.**

### Done, as capabilities

1. Register a folder or a git repo of skills and have chartr resolve ticket sessions'
   role prompts out of it, in an order you control — from a screen, not a text editor.
2. Launch an agent shell from the space card with no ticket at all, told what chartr
   is and what skills exist — and nothing about how to behave.
3. Read one generated `conventions.md` that is the whole of chartr's file-format
   contract, and append a `preferences.md` chartr will never override.
4. Take chartr's role skills by upgrading, or pin them with one `git fetch` and never
   be moved again.
5. Upgrade from the layer model without losing anything: a fork you edited is
   registered as a source, and one you never touched is set aside rather than removed.

### Blockers under pressure

- **Ticket 07 contradicts itself on migrated-row order, and implementation will hit
  it.** Its §`<configDir>/builtin-skills/` says *"registering an edited copy is placed
  before the migrated `<configDir>/skills/` row"* — then justifies it with the old
  resolution order, which is workspace › **user** › built-in (confirmed at
  `internal/prompt/prompt.go:323-325`), and that justification says the opposite. Its
  own step 3 says *"migrated-user row before migrated-builtin row"*. **Two of the
  three statements and the underlying code agree: `Legacy skills` first, `Migrated
  built-in skills` second.** Implementation should follow that reading; the lead
  sentence is a slip, and a human should confirm rather than let a coin land in code.
- **The settings surface is load-bearing for three resolved tickets**, which is why it
  is ticket 13 rather than a follow-on map. Ticket 01 puts the orphaned-checkout
  warning and the "this checkout is chartr's" caveat on it; ticket 03's only recovery
  for a deleted role binding is a control on it; and **ticket 07's entire acceptance
  of a silent migration rests on "it shows up the moment the operator opens
  Settings."** Ship without it and 07's silence has nothing to be discoverable *in*.
  The one call it still needed is made here on ADR 0014's existing boundary:
  **editable = the actions 01/03/05 already designed** (register, remove, enable
  toggle, reorder, refresh, restore a role binding); **open-the-file = everything
  else** (`sources.toml`, `user.toml`, `conventions.md`, `preferences.md`). Layout and
  placement stay the operator's call when the ticket is worked.

### Rejected

- **Cut first.** Argued above: no migrate-then-delete trap to justify it, ticket 07
  needs the embed it would delete, and it strands the cockpit that has to drive the
  rest of the work. `simplify`'s precedent supplies the stance, not the order.
- **Delete last, after the settings surface and the docs.** The other extreme. Rejected
  because the dead symbols are exactly what a session grepping for "how does skill
  resolution work" finds first, and every ticket after 11 would be written against a
  tree with two answers to that question.
- **Slicing five tickets in half to make the tracer bullet a ticket.** Rejected above:
  it manufactures five "finish the X" stubs, and the bullet's value is as a
  verification gate, which a milestone delivers for free.
- **Splitting into two implementation maps** (the new model, then the deletion and
  surfaces). Rejected on `simplify`'s own evidence: `CONTEXT.md` and the ADR are
  co-edited across the boundary, and one map is the only structure that serialises
  those writes. The independence that *is* real — 01, 03, 04, 10, 12 — is expressed as
  unblocked tickets, which needs no second map.
- **Shipping the new-shell control early on today's `core`.** Rejected: throwaway
  wiring plus a deletion list that cannot fully execute. Only its `launchedAgent` half
  is genuinely free, and that half ships first.
- **A sixth amendment to ADR 0009.** Rejected on the file's own pattern — five
  amendments all open with "the mechanism is untouched", and here the mechanism dies —
  and because 0009 would be left deciding nothing while still reading as operative.
- **Writing ADR 0017 first, before the code.** Rejected: this repo's ADRs are written
  by the ticket that realizes them (0013 by `simplify-impl` 01, 0014 by 05). An ADR
  describing a system nine tickets away is a plan, not a record.
- **Bootstrapping the vendored seed from today's four role skills** so ticket 05 need
  not wait on the `chartr-skills` repo. Genuinely tempting — it decouples every code
  ticket from days of prose. **Rejected by the operator**: it puts un-re-authored bytes
  (skills that name chartr, and section skeletons ticket 05 banned) inside the binary,
  and a bootstrap that works is a bootstrap nobody replaces. Making the repo ticket 01
  and running it concurrently costs nothing: it blocks 05 and only 05. The narrower
  variant — **re-author `grill` alone to unblock the tracer bullet** — was rejected
  with it, because offline first run would then chart nothing and spawn only grills.
- **Shipping without the settings section**, or shipping a read-only version of it.
  **Rejected by the operator** in favour of the full surface. Read-only was the honest
  middle — it restores exactly what ticket 07's silence needs and costs about a third
  as much — but it leaves a `git` source unregisterable without hand-writing a commit
  sha and cloning by hand, which is not a file anyone should author.
- **Making the migration re-runnable** to lift the release freeze, and **releasing
  freely and accepting the burn.** Both rejected above.
- **Deleting the in-repo payload copy.** Rejected by the operator; recorded as an
  accepted broken rule rather than a closed question, under "The rule that ships
  broken".

### Revisit triggers

- **The `06 → 11` edge** is revisited only if ticket 07's `builtin-skills/`
  comparison is dropped or replaced. The edge exists solely because the migration
  consumes the shipped embed; remove that consumer and 11 floats free.
- **The no-release window between 06 and 11** closes early if the migration is ever
  made re-runnable. That is a different decision, not a loosening of this one.
- **The build-then-cut order** is revisited if carrying both models turns out to cost
  more than the table predicts — concretely, if a ticket other than 08 finds itself
  editing `internal/prompt`'s layer half to keep the tree green. That would mean the
  two models are coupled somewhere this grill did not find, and the cut moves earlier
  by exactly one ticket.
- **The settings section's editable/open-the-file line** is revisited the first time
  an operator hand-edits `sources.toml` for something the surface was supposed to do.
  The escape is widening the editable set, never a second config store (ADR 0014).
- **The renamed `builtin-skills.migrated/`** is revisited if the directory turns out to
  accumulate — the escape is chartr removing it on a later startup once a full release
  cycle has passed, which is a deletion the operator has had a chance to see.
- **The in-repo payload copy** is revisited if `.chartr/` ever causes real trouble in a
  space — a tool that walks it, a watcher that thrashes on it, a teammate who commits
  it past the gitignore. The deletion is a small, already-argued change; nothing here
  forecloses it.
