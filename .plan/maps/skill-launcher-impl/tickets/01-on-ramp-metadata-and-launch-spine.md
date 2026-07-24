---
type: task
blocked_by: []
---

# On-ramp metadata and the launch spine

## Question

Stand up the whole backend once so the frontend picker (02) has a seam to hang
off: skills declare themselves launchable, the resolved library carries that to
the browser, and one endpoint launches any on-ramp skill on a chosen agent with an
optional line of context. This generalises the existing ideate on-ramp rather than
adding a parallel path — `handleIdeate` becomes the `skill=ideate` case of a
`launch` handler, and `prompt.Ideate` becomes `prompt.Launch(roots, skill)`.

Cut it in three moves:

- **Metadata (`internal/prompt`).** Two new frontmatter keys parsed by the existing
  `splitFrontmatter` onto the `Skill` struct: `on-ramp` (bool — this skill shows in
  the launcher) and `needs-context` (bool — it offers the optional context box).
  Both ride whole-skill shadowing unchanged: a shadowing layer's `SKILL.md` carries
  its own flags, so a user or workspace skill declares its own on-ramp status. Tag
  the shipped self-drivers `on-ramp: true` — **ideate, wayfinder, grill, research,
  prototype** — and mark `needs-context: true` on the ones that read a subject
  (grill, research, prototype; wayfinder and ideate open cold). Leave **core,
  tracker-convention, domain-modeling, to-spec, to-tickets, implement** untagged.

- **Wire to the snapshot.** Widen `Skill` → `ResolvedSkill` (Go `internal/model`
  and `web/src/lib/model.ts`) with `onRamp` / `needsContext`, so the resolved
  library the config surface already pushes now also tells the browser which skills
  are launchable and which want context. No new endpoint to *list* skills — they
  are already on the snapshot per space.

- **The launch endpoint.** `prompt.Launch(roots Roots, skill string) []byte`
  composes the named on-ramp skill **alone** — no core, no context bundle — exactly
  as `prompt.Ideate` does today (keep `Ideate` as `Launch(roots, IdeateSkill)`, or
  inline it). A new `POST /api/spaces/{id}/launch` with body
  `{ agent, skill, context? }` generalises `handleIdeate`: same `agentSpec`
  doorstep and refusals, but it also **refuses a skill that is not `on-ramp`** (a
  404/400 the way spawn refuses a non-role — the pushed library is the allowlist,
  so the server never launches a skill the client merely named), and threads the
  optional `context` line into the launch. Keep remembering the **agent**
  (`SetLastAgent`, as `handleIdeate` does today) so the dropdown's agent section
  opens on the last choice — but there is **no remembered skill**: the control is
  always a dropdown the operator picks a skill from each time, so no `lastSkill`
  state is needed. Keep the `/ideate` route working as a thin delegate to
  `launch(skill=ideate)` so nothing mid-flight breaks; the frontend (02) moves to
  `/launch`.

  The optional context reaches the agent by riding **in the composed payload**: when
  `context` is non-empty, append it under a short trailer (e.g. `## Your task`) to
  the bytes `Launch` writes, so the agent reads its brief from the same on-disk
  payload it already opens — not a fragile typed-in second line. An empty context
  writes the skill body unchanged.

Tests lead. `internal/prompt` (its `prompt_test.go` is the established seam):
`on-ramp` / `needs-context` parse off frontmatter and survive whole-skill
shadowing (a shadowing layer sets its own); `Launch` composes the named skill's
body alone and appends the context trailer only when context is present; the
shipped library tags exactly the five on-ramp skills. Server (`ideate_test.go`
generalised or a new `launch_test.go`): the handler launches an on-ramp skill,
**refuses a non-on-ramp one**, threads context into the written payload, remembers
the agent, and the `ideate` path is unchanged end to end.

Done when: a skill's `on-ramp` / `needs-context` flags reach the browser on the
snapshot; `POST /launch` runs any on-ramp skill on a named agent, refuses the rest,
carries the optional context into the payload, and remembers the agent;
`/ideate` still works; `go vet ./...` / `go test ./...` and the frontend gates are
green.
