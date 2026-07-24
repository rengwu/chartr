---
type: task
blocked_by: [03]
claimed_by: s644781a7b16e
claimed_at: 2026-07-22T04:08:25Z
---

# Everything is a skill

## Question

Repackage every injected prompt as a standard `SKILL.md` directory on disk while
chartr keeps composing the payload — the format opens up, the injection path
does not (reaffirms **ADR 0002**). Blocked by the vanilla-lifecycle revert (03) so
the composer is rewritten once, review-free and with the `## Proposed Answer`
semantics already settled, rather than migrating code tickets 02–03 then change.
The prompt-resolution / composition unit test leads (lands red before the
rewrite) — `internal/prompt` has no test file today, so this establishes that
seam without regressing the server's `payload_test.go`, which exercises the
composer through the payload preview.

- **Resolution.** Move `internal/prompt` from `<part>.md` files and the
  `replace` / `append` overlay to skill-directory resolution with **whole-skill
  shadowing**: the most-specific layer defining a skill of a given name wins its
  whole directory across built-in (`<dataDir>/skills/`) ‹ user
  (`~/.config/chartr/skills/`) ‹ workspace
  (`<space>/.chartr/skills/`). Drop the bespoke `replace` / `append`
  convention. Fork provenance moves from the `<!-- forked from <hash> -->` comment
  to a `forked_from:` frontmatter field; a skill's content hash covers its
  `SKILL.md` plus supporting files in a stable order.
- **The skills.** Convert `prompts/` and `internal/prompt/assets/` into **seven**
  `SKILL.md` directories — `grill`, `prototype`, `research`, `implement`,
  `ideate`, `core`, and `tracker-convention` (with the glossary as its supporting
  file `glossary.md`). Each `SKILL.md` carries the standard `name` / `description`
  frontmatter contract. `ideate` keeps its special injection (composed alone, no
  core, no context bundle).
- **Composition.** At spawn chartr reads the resolved `core` + role
  `SKILL.md` **bodies**, strips `name` / `description` / `forked_from`
  frontmatter, and assembles them with a freshly-built context bundle (map body,
  ticket, blockers' answers, and the glossary *sourced* from `tracker-convention`)
  into the one gitignored payload in `run/<sid>/`; the one-line PTY opener is
  unchanged, and supporting files stay on disk (not inlined) for on-demand zooming
  and external reuse. The claim commit's provenance trailers re-key from parts to
  skills (which layer won each composed part, plus the content hash).
- **Vendoring & drift.** Keep vendoring from the upstream skills repo, recording
  the upstream commit per sync (the existing `SourceCommit` record, bumped each
  sync). Surface the **stale-fork warning** over the directory hash — a shadowing
  skill whose `forked_from` is behind the built-in's current hash shows "behind"
  on the cockpit (via `prompt.LibraryWarnings`), never auto-merged.
- **Docs.** In `CONTEXT.md` rename **Prompt library → Skill library** (its
  `_Avoid_: skills` line inverts — it now *is* skills) and turn the Workspace /
  User config "prompts" content-half references into "skills."

Done when: `internal/prompt` unit tests assert whole-skill shadowing picks the
most-specific layer's whole directory, the composed payload carries the resolved
`core` + role bodies with frontmatter stripped and the context bundle appended,
and `forked_from` drift is detected over the directory hash; the server's
`payload_test.go` still passes; a spawn injects the resolved role skill's body; a
shadowing skill behind the shipped hash surfaces "behind"; `go vet ./...` /
`go test ./...` and the frontend gates are green; and `CONTEXT.md` reads Skill
library.

## Answer

The prompt library is a **skill library**: seven standard `SKILL.md` directories
under `internal/prompt/assets/skills/`, resolved by whole-skill shadowing and
still composed by chartr. The format opened; the injection path did not.

**Resolution.** `internal/prompt` no longer knows about `<part>.md`,
`.replace.md` or `.append.md`. `Resolve(name, Roots)` walks workspace ‹ user ‹
built-in and the **first layer that contains a `<name>/SKILL.md` wins its whole
directory** — body and supporting files together, nothing merged per file; the
embedded copy is the floor beneath the materialized built-in, so a deleted
`<dataDir>/skills/` still resolves. A directory without a `SKILL.md` does not
define a skill and resolution falls through it. Fork provenance is a
`forked_from:` frontmatter field, and a skill's hash covers every file in the
directory in sorted order (path and bytes), so a supporting-file edit moves it.
`Roots` is derived in one place, `Server.skillRoots`.

**The skills.** `core`, `grill`, `prototype`, `research`, `implement`, `ideate`,
and the new `tracker-convention` — which restates the map format (layout,
frontmatter, the derived-status table, the frontier) and carries `glossary.md`
as its supporting file. Each carries the `name` / `description` contract; the
bodies are the old prompts verbatim. `ideate` still composes alone: `Ideate` is
one `Resolve` and nothing else.

**Composition.** `Compose` reads the resolved `core` + role **bodies** with
frontmatter stripped, tags each with the layer that won it, and appends the
context bundle — glossary *sourced from the resolved `tracker-convention`
skill*, then map, ticket, blockers. Supporting files stay on disk. The payload
now also carries `Skills []Skill` (name, layer, hash), which the claim commit
writes as one `Skill: <name>=<layer>:<hash>` trailer each — the provenance
re-keyed from parts to skills. The wire shape of `Part`/`Segment` is unchanged,
so the preview needed no rework: a resolved skill is simply one segment now.

**The layer roots needed a name.** The ticket names the user layer
`~/.config/chartr/skills/`, but in this repo `<dataDir>` doubles as
the user-config root (`user.toml`), so built-in and user would have collided in
one directory. Materialization must land in the built-in layer (else every skill
would forever resolve "user"), so the user layer became the literal path the
ticket names, via a new `server.Options.ConfigDir` defaulting to
`os.UserConfigDir()/chartr`. **Flagged for ticket 05:** *bindings*
still read their user layer from `<dataDir>/user.toml` while *skills* read
`~/.config/chartr/skills/` — one "user layer" with two homes, which
the transparency surface will have to render honestly (CONTEXT.md's User config
entry already claims the `~/.config` path). The test rig points `ConfigDir` at a
temp dir so no run reads the developer's own library.

**Docs.** `CONTEXT.md` reads **Skill library** (its `_Avoid_` line inverted) and
Workspace config's content half reads "skills". **ADR 0002** gains a
reaffirmation — what it chose was never a file format but that chartr wires
the session itself — noting that its model-heterogeneity clause lapsed with the
gate. `docs/wayfinder-adapter.md` points at the vendored `tracker-convention`
skill instead of a `prompts/` path. **ADR 0012** is amended for a live
consequence, below.

**One thing beyond the ticket's letter, and it was load-bearing.**
`.chartr/prompts/implement.append.md` is **tracked in this repo** —
it is the design-system guardrail ADR 0012 injects into every chartr-spawned UI
session (it is in this session's own payload). Retiring `.append.md` would have
dropped it silently on the next spawn. It is migrated to a committed workspace
`implement` skill: a whole-skill fork of the shipped one recording
`forked_from: c231b077`, with the same section at the end of its body; ADR 0012
now names the new home. This is the forfeited append affordance costing a full
fork for one section — the spec's named revisit trigger, met on day one, in this
repo. Worth a human's eye if it recurs.

**Tested.** New `internal/prompt/prompt_test.go` (the seam this ticket
establishes) asserts: shadowing picks the most-specific layer's whole directory
and a shadowed layer's supporting file does *not* survive; an empty directory
does not define a skill; the composed payload carries both resolved bodies with
frontmatter stripped, the glossary sourced from `tracker-convention`, the bundle
after the prompts, and skills recorded with layer + hash; drift over the
directory hash (stale, not-stale on the current hash, no `forked_from` never
warns, a supporting-file edit moves the hash); ideate alone; the shipped library
is seven skills with descriptions and matching hashes; and Materialize preserves
edits. `payload_test.go` still passes and still covers composition through the
preview — its two overlay tests were **rewritten**, not kept: they asserted the
`replace`/`append` convention this ticket deletes, so they now assert shadowing
and `forked_from` drift at the same seam. `spawn_test.go` gained the `Skill:`
trailer assertions. `go vet ./...` / `go test ./...` green; frontend `check` /
`build` / `vitest` green (53 tests); no amber in the built CSS (every hue is the
~107 monochrome ramp or `--destructive`).

**Verified against the real binary**, second instance on port 8811 with its own
data dir and `HOME` so the operator's cockpit and config were untouched: the
preview for this ticket resolved `core` + `implement` from built-in with
frontmatter stripped and the glossary in the bundle; a user-layer `implement`
skill shadowed it whole and, carrying `forked_from: deadbeef`, surfaced *"behind
the shipped default"*; a workspace skill then won over the user one; and with
the migrated overlay in place `implement` resolves **workspace**, warning-free,
with the design-system rules still injected.

**Deliberately not done.** No migration of the untracked local `prompts/`
materialization at the repo root or `.chartr/prompts/`'s now-empty
directory — untracked operator state, and the housekeeping ticket's business,
not mine. No `Space.Skills` model field or settings UI (ticket 05's; the
resolver it needs is exported as `prompt.Library`). No per-skill `append`
affordance and no auto-merge of a stale fork — both refused by the spec. The
`ideate` skill is materialized and layered like the rest but still composed
alone, unchanged.
