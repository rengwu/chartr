# Handoff — validating Chris Abernethy's security report

**Date:** 2026-08-03
**Repo state at handoff:** `37f79bb`, working tree clean, nothing committed by this session.

## What this session was

John received a private report from Chris Abernethy (CISO, Tradeverifyd), who had
read the chartr source while evaluating it for internal use. Two wayfinder maps
were charted and implemented off it — `.plan/maps/localhost-trust-boundary/`
(the grill: where the trust boundary actually is) and
`.plan/maps/websocket-origin-fix/` (the fix: six tickets, all implemented).

This session's job was to **validate the current state and verify every reported
finding is resolved**. It was verification only — no production code was changed.

## Outcome

All seven findings are resolved. Full evidence, per finding, is in:

> **`.plan/maps/websocket-origin-fix/verification.md`**

That document has the code references, the live-probe results (status codes for
forged Origins, forged Hosts, CORS-simple POSTs), the reproduction script, and
the regression-test inventory. Do not re-derive it — read it.

Three follow-ups are recorded there and are the only open items:

- **A — payloads already on disk.** The 0600/0700 fix is forward-only, scoped out
  on purpose in ticket 05. On John's machine 83/83 existing session payloads are
  still 0644, `~/.config/chartr` is 0755, and the running `chartr.app` (started
  2026-07-27) keeps writing 0644 until restarted on a build with the fix.
  Suggested close: `chmod -R go-rwx ~/.config/chartr` + restart the app.
- **B — the `[::1]` origin pattern is inert.** `path.Match` reads `[::1]` as a
  character class, so that pattern can never fire. Fails closed and every real
  path is covered by Origin-equals-Host, so it is a comment-vs-reality defect in
  `internal/server/origins.go`, not a hole. Drop the pattern or fix the comment.
- **C — scheme ignored on the library's same-host fast path.** Informational;
  upstream coder/websocket behaviour, not exploitable here.

## What was not done

- **Nothing was committed.** `verification.md` and this handoff are untracked.
- **No reply to Chris was drafted.** If that is the next task, follow-ups A and B
  are the two things worth telling him; the rest is a clean "all resolved".
- **Follow-up B was not fixed** — it is a one-line decision (drop the pattern vs.
  correct the comment) that belongs to John, not to a verification pass.
- **Frontend checks were not run** (`web/` was untouched; CLAUDE.md only requires
  them before committing frontend changes). `go vet ./...` and `go test ./...`
  were both run and pass.

## Gotchas for the next session

- **Registering a space through the API writes to the real
  `~/.config/chartr/spaces.toml`**, not to a sandbox — `-data-dir` only moves the
  session/runtime root; user config always resolves to `~/.config/chartr`. This
  session registered a test space and deregistered it afterwards (verified gone).
  Deregister anything you create.
- **Port 8799 on loopback is held by a stale `harness` process** from an earlier
  session, serving an older build. An initial probe run hit it instead of the
  freshly built binary and produced completely misleading 200s. Always confirm
  the bind succeeded (`grep "listen tcp" server.log`) before trusting probe output.
- **A successful websocket upgrade hangs curl** — use `--max-time`.
- `internal/server` tests take ~57s; budget for it.

## Suggested skills

Only if available in the next agent's environment:

- **`review-code`** — if follow-up B is fixed, to check the change against the
  surrounding `internal/server` conventions before it lands.
- **`wayfinder`** — if the follow-ups grow past one-liners and deserve their own
  map rather than being patched ad hoc.
- **`to-tickets`** — to graduate such a map into tickets, matching how the
  original two maps were run.
