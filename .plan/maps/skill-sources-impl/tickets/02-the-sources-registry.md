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
