# Syncing skills from upstream

How to take an upstream skill update and re-fit it for chartr. Point any agent
doing this work at this file instead of re-explaining the history.

**Upstream:** `github.com/rengwu/skills` (which itself follows
`github.com/mattpocock/skills`). If pocock's repo has newer changes than the
fork, update the fork first, then sync chartr from the fork.

**Shipped copy:** `internal/prompt/assets/skills/` — the only copy that matters
(`go:embed`ed into the binary). Edit nothing else.

## Why chartr owns its skills (decided 2026-07-22, do not re-litigate)

- **Re-author, never wrap.** A wrapper can add text but never retract a
  sentence the model also reads. Chartr's flow (injected payload, `## Answer`
  artifact, map-based tracking) contradicts too much of the upstream text for
  wrapping to work. Precedent: `grill`, `research`, `prototype` are chartr
  originals, not ports.
- **No runtime loading.** Skills ship embedded in the binary; payload hashes
  become provenance trailers in commits, so two machines must resolve identical
  bytes for the same ticket.
- **The test for any change:** if fitting it requires writing "instead" or
  "ignore", re-author that part from scratch instead of patching around it.

## The chartr contract — what every shipped skill must satisfy

- **Frontmatter:** `name` + `description` only. Strip Claude-Code-specific
  fields (`disable-model-invocation`, etc.) — chartr injects skills itself.
  **The description is agent-facing:** every session's context bundle carries a
  skill-library manifest (name + description + path, `skillManifest` in
  `internal/prompt/compose.go`), so keep the description one line that tells an
  agent *when to reach for this skill*.
- **Shape — two kinds.** Role skills (`core`, `grill`, `research`, `prototype`,
  `implement`, `ideate`) are short plain markdown (see `grill/SKILL.md`, ~20
  lines): opens with the role, names the product, a few load-bearing
  imperatives. Method skills (`wayfinder`, `domain-modeling`, `to-spec`,
  `to-tickets`) are longer reference documents an agent reads on demand via the
  manifest path — length is fine there, but they are still prose for agents:
  no Claude-Code framing, no relative links between skill directories (refer
  to another skill **by name** — `../wayfinder/TRACKER-MARKDOWN.md` means
  nothing inside the shipped library).
- **Artifact:** role skills end by telling the session to write its conclusion
  under `## Answer`. The ticket's Answer is the only output channel — no HTML
  reports, no files chartr has no surface for, no "synthesize the conversation"
  (chartr assembles a context bundle instead).
- **No Claude Code assumptions:** no slash commands, no skill loader, no hooks,
  no agent-CLI-specific features. Skills must work through any adapter (ADR
  0002).
- **Invariants live in shared skills, not per-skill:** ground rules go in
  `core/`; the wayfinder map format and glossary live only in
  `tracker-convention/`. Never ship upstream's `TRACKER-MARKDOWN.md` — that
  would put the format contract in two places, free to diverge. Method skills
  may carry *templates* that mirror the convention's section names (the
  wayfinder map body, the to-tickets ticket shape), but the contract itself —
  derived status, claim fields, numbering — is stated only in
  `tracker-convention/`; on any disagreement it wins and the sync fixes the
  method skill.
- **Vocabulary:** use the CONTEXT.md glossary (session, ticket, map, cockpit).
  The product is **chartr** — never "wayfinder-harness" or "Claude".

## The sync procedure

1. **Pin the upstream ref.** Clone or pull `github.com/rengwu/skills`; record
   the commit hash you are syncing to.
2. **Diff per skill.** For each upstream skill with a chartr counterpart, diff
   upstream-now against upstream-at-last-sync (the ref in `SourceCommit`,
   `internal/prompt/prompt.go`). This shows what *changed upstream* — do not
   diff upstream directly against the chartr skill, which is a re-authoring and
   will differ everywhere.
3. **Triage each change:** *adopt* (fits the contract as-is), *adapt* (rewrite
   to fit the contract), or *reject* (contradicts chartr's flow — note why in
   the commit message).
4. **Edit** `internal/prompt/assets/skills/`.
5. **Bump provenance:** set `SourceCommit` in `internal/prompt/prompt.go` to
   the pinned ref. This is what makes the next sync's step 2 possible.
6. **Verify:** `go vet ./... && go test ./...`; then run the app and open the
   payload preview in the cockpit to read exactly what a session would receive.
7. **Commit** with the upstream ref in the message.

## Never do

- Inject upstream skill text verbatim into a payload.
- Load skills from a local directory at runtime.
- Auto-merge a user's fork (whole-skill shadowing, ADR 0009): a fork with
  `forked_from` in its frontmatter is surfaced as behind in the cockpit, never
  overwritten.
- Duplicate the map format contract or the glossary outside `tracker-convention/`
  (templates mirroring its section names are the one allowed exception).
