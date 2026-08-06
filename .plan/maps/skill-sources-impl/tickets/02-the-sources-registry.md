---
type: task
blocked_by: []
claimed_by: s86c8778648d2
claimed_at: 2026-08-06T15:38:37Z
---

# The sources registry

## Question

Build `internal/sources`: the ordered list of skill sources that replaces the
layer model's resolver. It is a new package and nothing consumes it yet, so this
ticket delivers no operator-visible capability — it is a foundation, and a green
test suite on it is not, by itself, progress anyone can use. Say so when you resolve
it rather than overclaiming.

**This ticket carries both kinds of source.** The spec split `dir` from `git` to
keep clone machinery off the critical path; that split was folded back in, so the
clone, the refresh and the missing-`git` gate land here. Nothing downstream needs
them, so if the ticket runs long, land `dir` green first and `git` in a second
commit — the seam between them is `kind`, which the spec fixed as declared and
never inferred.

Build to the spec's registry section: `sources.toml` under the config root, an
array of tables whose **position in the file is resolution order** — no order
field, and therefore no densification, no duplicate-order case, no legacy decode.
Identity is the operator's `name` (trimmed, 1–64 chars, letters/digits/space/
hyphen/underscore, **no `/`**, unique case-insensitively). `enabled` sits on the
row and is written only when false. The `chartr-skills` row is **synthetic** —
never written as a row, always last, not removable, not reorderable — with only its
scalars persisted; what lands at its path is ticket 05's. Write atomically at
`0600` under a `0700` root, temp-then-rename. A missing or unparseable file is the
default row alone: **the first-run state, not an error.**

Borrow `internal/registry`'s *stance* — atomic writes, missing-is-first-run, losing
the file costs re-registration and never work — and deliberately none of its
machinery. The hashed id, the dense `order` int, the densify-on-save and the
legacy-row second decode all exist there because `spaces.toml` serialises its rows
sorted by path and keys per-space state by id. Neither pressure exists here, and
copying the mechanism without the pressure imports three failure modes that file
position makes unrepresentable.

Discovery is the bounded uncached walk: a skill is any directory at depth 1–3
holding `SKILL.md` at its top level; never descend into one that has it; skip
dot-entries and `node_modules`; a skill's name is its directory's basename.
Resolution takes both forms — a bare name searching enabled sources top-down then
the default row, and a `Source/skill` reference addressing one source exactly. A
**qualified miss never falls through**, and a qualified reference into a disabled
source reads as not-found *and names the source as disabled*, because that is the
one failure fixed in a click.

Git sources are shallow single-branch clones under `<configDir>/sources/`, keyed by
a hash of the URL so renaming a source is a pure metadata edit. Clone into a temp
directory and rename into place; **write the row only after that rename**, so
nothing half-cloned is ever a source and a failed clone leaves neither row nor
directory. No confirm gate before the clone. Refresh is explicit and quiet — fetch,
hard-reset, record commit and timestamp, report the new short sha — and nothing
fetches unattended.

The public surface is the seam: load, list in file order, walk one source, resolve,
register (refusing duplicate names), remove, set-enabled, reorder as a whole list,
refresh. Tests go here rather than through the server, because the failure-mode
table is combinatorial and unreachable over HTTP without absurd fixtures — this is
exactly where `internal/prompt/prompt_test.go` sits today, and `registry_test.go`
is the model for driving a TOML file under a temp root through a public API with
hand-written malformed files for the degradation cases.

## Done when

Every row of the spec's failure-mode table has a test asserting what it reads as:
vanished dir path, `git` absent from `PATH` at registration, clone failing partway,
a source yielding zero skills, a duplicate skill name across sources, a duplicate
basename inside one source, a duplicate source name at registration and in a
hand-edited file, an unknown `kind`, a row named `chartr-skills`, and a missing or
unparseable file. Both resolution forms are tested including the never-fall-through
rule; the discovery walk is tested at its depth bound and its descend-stop rule.
`go vet` and `go test` pass.

## Answer

`internal/sources` exists: `sources.go` (the file, the walk, resolution, the
`dir` half of the surface) and `git.go` (clone, refresh, the PATH gate), with
`sources_test.go` driving all of it through the public API against real
directories under a temp config root. **Nothing consumes it.** No operator can see
or do anything new at this commit — this is a foundation, and its green suite is
not progress anyone can use until ticket 05 seeds the default row and ticket 07
resolves role bindings through it.

**What it is.** `Load(configDir)` reads `sources.toml`, an array of tables whose
position is resolution order, and returns the operator's rows plus the synthetic
`chartr-skills` row last. `Warnings()` carries what the load dropped. `List`,
`Get`, `Walk`, `States`, `Resolve`, `RegisterDir`, `RegisterGit`, `Remove`,
`SetEnabled`, `Reorder` and `Refresh` are the seam. Saves are temp-then-rename at
`0600` under a `0700` root. No hashed id, no order int, no densify, no second
decode — position makes those failure modes unrepresentable, which was the whole
argument for not copying `internal/registry`'s machinery.

**Each Done-when clause.** Every row of the failure-mode table has a test, named
after what it reads as: vanished dir path (`unavailable`, zero skills, row never
auto-removed), `git` absent from PATH (registration refused at the gate, an
existing checkout still resolves, only refresh fails), clone failing partway (no
row, no directory, git's own output in the error), zero skills (`empty`),
duplicate skill name across sources (not an error — the lower one is marked
shadowed and stays reachable qualified), duplicate basename inside one source
(sorted walk order wins, loser named on the row), duplicate source name at
registration (`ErrDuplicateName`, nothing written) and in a hand-edited file
(first row wins, the later one warned by name), unknown kind and a row with
neither path nor url (dropped, rest of the list stands), a row named
`chartr-skills` (dropped, and the name refused at registration), missing file and
unparseable file (default row alone; missing is silent, unparseable warns). Both
resolution forms are tested, including a qualified miss into an empty source while
a *lower* source has that skill — it stays not-found — and a qualified reference
into a disabled source whose error contains the word `disabled`. The walk is
tested at depth 1/2/3 with a depth-4 skill excluded, a `SKILL.md` below a skill
not descended into, and dot-entries and `node_modules` skipped. `go vet ./...` and
`go test ./...` pass; both source kinds landed in one commit rather than two.

**Decisions inside the delegation, worth reading.**

- The default row's path is `<configDir>/sources/chartr-skills`, exposed as
  `DefaultPath` — a named directory beside the hash-keyed git checkouts, which
  cannot collide with it. Ticket 05 owns what lands there.
- `default_commit` and `default_fetched` are already persisted beside
  `default_enabled`, so ticket 05's amendment (two scalars beside the toggle,
  default row git-kinded) needs no file reshape.
- **`Refresh` refuses the default row**, with an error saying so. The
  seeded→pinned conversion needs the seed's URL constant, which is ticket 05's;
  making the registry guess it would have been that ticket's decision taken here.
- Walk order is sorted depth-first, so a subdirectory named `a` yields before a
  top-level skill named `one`. It is deterministic, which is all the
  duplicate-basename rule needs, and it is asserted rather than left implicit.
- `Remove` deletes a git source's checkout (chartr's own, under the config root)
  and never touches a `dir` source's folder.
- Registration takes a source that is empty or absent today: that is a status on
  the row, not a refusal, which is the same reading the failure-mode table gives a
  path that vanishes later.

**Deliberately not done:** no server wiring, no HTTP surface, no settings model —
those are tickets 07, 08 and 10. The `Skill` type carries no body or frontmatter;
composition reads the file when it needs it. Nothing in `internal/prompt` was
touched, so both models coexist exactly as the sequencing intends.

`CONTEXT.md` gains the **Source** entry, per the spec's rule that vocabulary
follows its ticket.
