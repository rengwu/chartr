---
type: task
blocked_by: [03]
---

# Everything is a skill

## Question

Repackage every injected prompt as a standard `SKILL.md` directory on disk while
the harness keeps composing the payload — the format opens up, the injection path
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
  (`~/.config/wayfinder-harness/skills/`) ‹ workspace
  (`<space>/.wayfinder-harness/skills/`). Drop the bespoke `replace` / `append`
  convention. Fork provenance moves from the `<!-- forked from <hash> -->` comment
  to a `forked_from:` frontmatter field; a skill's content hash covers its
  `SKILL.md` plus supporting files in a stable order.
- **The skills.** Convert `prompts/` and `internal/prompt/assets/` into **seven**
  `SKILL.md` directories — `grill`, `prototype`, `research`, `implement`,
  `ideate`, `core`, and `tracker-convention` (with the glossary as its supporting
  file `glossary.md`). Each `SKILL.md` carries the standard `name` / `description`
  frontmatter contract. `ideate` keeps its special injection (composed alone, no
  core, no context bundle).
- **Composition.** At spawn the harness reads the resolved `core` + role
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
