---
type: task
blocked_by: [06, 07, 08]
---

# The cut

## Question

Delete the layer model. **Strictly deletion and documentation — the moment this
ticket grows anything behavioural it has stopped being the cut.**

Every consumer has moved by now, which is why this lands last rather than first. The
`simplify` map's cut-first precedent supplies the stance and not the order: its cut
removed a *gate*, so the escape hatch was a terminal and `git`; this cut removes the
*engine*, and a tree with the layer model gone and the registry not yet wired cannot
compose a payload, so it cannot spawn the sessions that would do the remaining work.
There is a hard edge besides — ticket 06's migration consumes the very embed this
ticket deletes.

Delete: `prompt.Roots`, `RootsFor`, `Resolve`, `Names`, `Library`, `Materialize` and
the `LayerBuiltin`/`LayerUser`/`LayerWorkspace` tags; `hashFiles`, `ShippedHash`,
`Skill.Hash`, `Skill.Stale`, `ForkedFrom`, `staleWarning`, `LibraryWarnings`; the
`assets/skills` embed and `<configDir>/builtin-skills/` with its `readmeText`; the
`tracker-convention` skill and `glossary.md` if ticket 07 left anything behind;
`prompt.Launch`; `configsurface.go`'s `resolvedSkills`/`resolveSkillDir` and the
`layerSkillPrefix` hatch. Also the stale duplicate at the repo root (`/skills/`,
gitignored, already drifted from the embedded copy).

**Delete `docs/skill-sync.md` and write ADR 0017 in the same commit.** That file
carries a *"decided 2026-07-22, do not re-litigate"* block whose **no runtime
loading** rule this effort reverses, and no ADR records it — checked against all
sixteen — so deleting it before 0017 exists opens a window where a retired decision
has no home. Its *re-author, never wrap* rule survives, relocated to
`chartr-skills/CONTRACT.md`, where it now applies upstream.

**ADR 0017 — *Skills come from registered sources; chartr ships none*.** It carries
three things, and it is the *three* that make it an ADR rather than a sixth
amendment to 0009: the **model** (one ordered list of operator-owned sources,
first-hit resolution, explicit `[roles]` bindings, chartr shipping only two payloads
and a conventions ruleset — reaffirming ADR 0002 and ADR 0005); the **trust posture
and its cost** (a registered source is executable text reaching agents run with
permissions skipped, and the only assertion of trust in a git source's entire
lifetime is the moment its URL is typed); and the **reproducibility retirement**
(two machines no longer resolve identical bytes for the same ticket; what replaces
it is weaker and honest about being weaker — the trailer names the source and pins
it where there is a pin, and `Payload-SHA256` still fixes the exact bytes for the
machine that composed them).

**ADR 0009 gains a second banner beside its existing one**, not a sixth amendment.
All five of its amendments open by saying the mechanism is untouched, and here the
mechanism dies; its execution half is already superseded, so amending the content
half would leave the file deciding nothing while still reading, top to bottom, as
operative config policy. 0017 supersedes the content half; with the existing banner
covering the other, 0009 becomes wholly historical and the new banner says so in one
line.

Edit `CONTEXT.md`: **Skill library** and **Committed skills** die here, and
**Context bundle** loses any glossary or skill-library-manifest clause ticket 07 did
not already take.

## Done when

A grep for every symbol on the deletion list returns nothing, `internal/prompt` no
longer embeds `assets/skills`, and `<configDir>/builtin-skills/` is written by
nothing. ADR 0017 exists and ADR 0009 carries its second banner; `docs/skill-sync.md`
is gone in the same commit. The cockpit still spawns a ticket session and a free
session. `go vet`, `go test`, and the frontend `check`/`build`/`vitest` pass.
