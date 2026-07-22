---
type: task
blocked_by: []
claimed_by: saba3c5ff2ada
claimed_at: 2026-07-22T05:50:05Z
---

# Sweep dead weight

## Question

Clear the dead weight that is not part of any other ticket, as a **deletion-and-
ignore-only** change isolated from every functional diff — so the sweep never
hides inside a behavioural change, and the moment it would grow anything
behavioural it splits out a functional ticket instead.

- `git rm web/src/lib/Probe.svelte` — orphan debug component.
- Remove the stale `sessions/` archives from the tree, or confirm they are
  gitignored run-time debris and leave them ignored.
- Delete the stray root `node_modules` from disk (deps belong under `web/`) and
  confirm the repo root ignores it — it is already untracked, so this is disk +
  `.gitignore`, not a `git rm`.
- Add a `.DS_Store` entry to `.gitignore` — none are tracked, so the honest task
  is an ignore rule, not a `git rm`.
- **Leave `make webview` untouched** — it is ticket 01's scaffolding, not debris;
  sweeping it would delete a target one ticket from being real.

Done when: `git ls-files` shows no `Probe.svelte`, no `sessions/` archives, and no
`.DS_Store`; the root `node_modules` is gone from disk and ignored; `make webview`
is still present; `go vet ./...` / `go test ./...` and the frontend `check` /
`build` are green; and the diff is purely deletions plus `.gitignore` lines.
