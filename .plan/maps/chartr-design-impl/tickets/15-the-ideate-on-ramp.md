---
type: task
blocked_by: [09]
claimed_by: s75c291ca53ca
claimed_at: 2026-07-21T09:29:12Z
---

# The ideate on-ramp

## Question

The one opinionated nudge toward charting (planning ticket 07). An "ideate" affordance spawns an open-ended, ticketless session from a chartr-provided starter prompt — on-disk hackable markdown filed explicitly as a non-role on-ramp, so the five-role set stays closed — that prods the user on what's on their mind and, if the idea proves big, suggests escalating to `/wayfinder` in prose only: escalation is advice, never a chartr transition or tracked state. Like an ad-hoc shell the session is ticketless, live, un-reviewed, and ends only when the human ends it — chartr never derives finished for it, and no quiet badge applies. It shares only the adapter's spawn primitive with real sessions: no claim commit, no lifecycle.

Done when: the button spawns a live TUI session opened with the starter prompt's payload; process-boundary tests assert no claim commit is written, no ticket is bound, no lifecycle state ever derives for it, and editing the on-disk starter prompt changes what the next ideate session is told.

## Answer

**What I built:**

- **`internal/prompt/assets/ideate.md` + `prompt.Ideate(dataDir)`** — the starter
  prompt as a sixth, explicitly non-role part (`IdeatePart = "ideate"`). It
  materializes to `<dataDir>/prompts/ideate.md` alongside the five role prompts
  (`Materialize` now also writes it), so it is on-disk hackable the same way, but
  `Ideate` resolves it alone — no core, no role, no context bundle, and
  deliberately outside `resolvePart`'s workspace `replace`/`append` overlay and
  the payload preview's provenance machinery, since an ideate session is
  ticketless and mapless: there is nothing to inject it against. The prompt
  itself prods the human on what's on their mind and, in prose only, suggests
  `/wayfinder` when an idea turns out big — it never claims to chart or transition
  anything itself.
- **`terminal.Manager.OpenIdeate`** — a new sibling to `Open` (ad-hoc shell) and
  `OpenSession` (real session): it runs `newProc` with the resolved agent's argv
  instead of the operator's shell, types the opener in once the PTY is up (same
  mechanic as `OpenSession`), but — this is the load-bearing choice — **passes no
  `Session`**. The tab it seats therefore has `t.session == nil`, so it is,
  structurally, an ad-hoc shell running an agent: `isLiveSession`/`HasLiveSession`
  never count it (it neither blocks nor is blocked by the one-live-session-per-
  space limit a real spawn enforces), `pinOnDeath` is false (a death just ends the
  tab, exactly like a closed shell, never the death halt), and `sample` reads it
  on the ad-hoc shell's idle/working/exited grammar rather than the session
  grammar's working/quiet/dead — which is also *why* no quiet badge ever applies,
  for free, rather than as a special case threaded through the server's
  AFK-role logic.
- **`internal/server/terminals.go` — `handleIdeate`**, routed at
  `POST /api/spaces/{id}/ideate`. It does not look up any map or ticket (there is
  none to look up): it resolves the `grill` binding (the closest existing
  analogue — a live, human-in-the-loop conversation — rather than opening a sixth
  entry in the closed role set), hard-blocks on an absent agent exactly as a real
  spawn does, composes `prompt.Ideate`, writes it through the existing
  `writeSessionPayload` into the same gitignored `.chartr/run/<id>/`
  directory a real session's payload uses (reusing the mechanism, not the
  session-ness — the constant's doc comment now says so), and calls
  `OpenIdeate`. No claim commit, no archive to `<dataDir>/sessions/`, no map or
  ticket ever read.
- **Frontend**: `actions.ts` gained `ideate(id)`; `App.svelte`'s sidebar footer
  gained a second icon button (Lightbulb, beside the existing shell "+") that
  calls it and makes the new tab active; `SpacePane.svelte`'s empty-terminal state
  gained a matching "Ideate" button next to "Open a shell", per planning ticket
  07's "no map → shells/ideate" framing. No new chrome component: since the tab
  carries no `Session`, the sidebar's existing ad-hoc-shell row (`t.proc` /
  `t.status`) renders it as-is, titled "ideate" until the agent takes the
  foreground. Tokens/primitives/Phosphor only; `svelte-check` (0/0), the Vite
  build, and `vitest` (45) all pass, and the built CSS carries no amber.

**How each Done-when clause is met** (`internal/server/ideate_test.go`, 4 new
process-boundary tests, plus a live smoke run against the real `claude` CLI —
see below):

- *the button spawns a live TUI session opened with the starter prompt's
  payload* — `TestIdeateOpensLiveTicketlessTab` opens ideate in a **mapless**
  space, asserts the tab is alive, byte-matches the gitignored prompt file
  against `prompt.Ideate`, and waits for the stub agent's stdin to receive the
  read-this-file opener naming that exact path.
- *no claim commit is written* — `TestIdeateWritesNoClaimCommit` commits a
  baseline first (so "no commit" is a real assertion, not "no commits exist
  yet"), calls ideate, and asserts `git rev-list --count HEAD` is unchanged.
- *no ticket is bound* — the same test asserts an unrelated ticket's status stays
  `open`; `TestIdeateOpensLiveTicketlessTab` additionally asserts `tab.Session ==
  nil` on the wire, and runs against a space with no `.plan/` at all to prove the
  on-ramp needs no ticket to exist.
- *no lifecycle state ever derives for it* — `TestIdeateHasNoDeathHalt` kills the
  ideate agent (`StubDyingAgent`) and asserts the tab **drops** from the model
  rather than pinning dead — the concrete, testable difference from a real
  session's death halt (ticket 10).
- *editing the on-disk starter prompt changes what the next ideate session is
  told* — `TestIdeateStarterPromptIsEditable` overwrites the materialized
  `<dataDir>/prompts/ideate.md`, opens a fresh ideate tab, and asserts the edit
  reached the composed prompt file.

**Tested:** `go vet ./...`, `go test ./...` (all packages), `svelte-check`
(0 errors/warnings), the Vite build, `vitest` (45) — all pass; no amber in the
built CSS. Beyond the process-boundary suite, I ran the real binary against a
throwaway git-repo space with the real `claude` CLI on `PATH` (no stub): `POST
.../ideate` returned a tab id, `ps` showed a live `claude --model opus` process,
the gitignored prompt file on disk matched the shipped `ideate.md` verbatim, and
`DELETE .../terminals/<id>` cleanly killed it — the same close path an ad-hoc
shell uses. I could not click through the actual browser UI (no Chrome
extension connection in this session), so the two new buttons are unverified by
eye — flagged below, same precedent tickets 12/17 set for a headless session.

**Deliberately left / flagged for review:**

- **Which binding ideate runs on.** The Done-when doesn't say, and the five-role
  set is closed by design, so I reused the `grill` binding rather than adding a
  sixth config surface. This is a judgement call, not a spec-derived answer: it
  couples ideate's agent/model to whatever the operator has bound for grilling,
  and an absent-agent block on ideate reads `"claude" isn't on your PATH (grill →
  built-in config)…"` — technically correct (that *is* the binding it used and
  the fix is the same override) but names a role the operator didn't ask for.
  Worth a human call on whether that's acceptable or ideate deserves its own
  (still non-role) binding key.
- **`OpenIdeate` allows unlimited concurrent ideate tabs, and lets them run
  alongside a live real session.** This mirrors ad-hoc shell semantics
  deliberately (the spec's own frame: ideate is one of the two on-ramps that are
  "not sessions," sharing only the spawn primitive) rather than session semantics
  — not stated explicitly as a requirement, so flagging the choice.
  Ideate tabs are also uncounted by `HasLiveSession`, so ideate can never block
  (or be blocked by) a real spawn.
- **No archived copy under `<dataDir>/sessions/`.** A real session's payload is
  archived there so `ensureSessionPayload` can restore it after a resume; ideate
  has no resume path (death just ends the tab), so I did not archive it. The
  gitignored copy in the space is the only record, and it is exactly as
  ephemeral as an ad-hoc shell's scrollback.
- **Browser click-through did not run** (no Chrome extension connection this
  session) — the API-level and stub-agent tests, plus the real-`claude` smoke run
  above, cover the wiring; the two new buttons' placement and feel are unverified
  by eye, same precedent as tickets 12 and 17.
