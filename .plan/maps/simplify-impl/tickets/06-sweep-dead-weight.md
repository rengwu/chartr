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

## Answer

**Three of the four sweep items turned out to already be done, so the whole
effort is one deletion commit and a set of confirmations.** The sweep landed as
`1760135`, and it touches exactly two files.

**`Probe.svelte` is gone** (`git rm`). It was the throwaway ticket-01
design-system verification surface — a card proving token utilities, the radius
scale, IBM Plex Sans and a Phosphor icon all resolved — reached only through the
`#probe` URL hash and self-documented as "deleted once the reskin proper
(tickets 02–04) exercises these for real". That day arrived: the real chrome now
exercises every one of those. Its single consumer was `main.ts`, whose hash swap
(`const Root = location.hash === '#probe' ? Probe : App`) went with it, so `App`
mounts unconditionally. Nothing else in the tree referenced it — no test, no
route, no doc.

**`sessions/`, root `node_modules` and `.DS_Store` needed no `.gitignore`
lines.** All three rules are already committed at HEAD (`.gitignore:37`, `:12`,
`:51`) and `git ls-files` tracks nothing matching any of them, so the honest work
was to confirm rather than to change:

- `sessions/` is not a stale archive — it is *live* chartr state, the
  per-session payload audit trail from ADR-era ticket 09, and it holds this very
  session's payload. Correctly ignored; deliberately left on disk. Deleting it
  would have destroyed the audit trail chartr is currently writing.
- The stray root `node_modules` held only a `.vite` cache directory (Vite run
  with the repo root as cwd). Removed from disk; `/node_modules/` already ignores
  its return.
- No `.DS_Store` is tracked anywhere despite nine sitting on disk, because the
  rule has been in place since before this map.

**`make webview` is untouched**, as the ticket directs.

**Gates, all green:** `go vet ./...` clean; `go test ./...` passes (`server`
12.1s, plus `config`, `prompt`, `terminal`, `wayfinder`, `cmd/webview`); frontend
`check` 3727 files / 0 errors / 0 warnings; `vitest` 60 tests in 7 files; `build`
succeeds; zero amber hits in the built CSS.

**One honest deviation from "purely deletions".** The diff is 30 deletions and
**one** insertion: `mount(Root, …)` → `mount(App, …)` in `main.ts`. There is no
deletion-only way to remove the `Probe` import, because the `Root` binding that
selected between the two dies with it. The change is mechanical and behaviour-
preserving for every URL except `#probe`, which no longer swaps in a component
that does not exist. I judged this inside the spirit of "deletion-and-ignore-
only" — nothing behavioural rides along — and am flagging it rather than
pretending the diff is pure.

**Deliberately not done:** no ADR is touched; no dependency pruning, no dead-code
hunt beyond the named items, and no `web/dist` or lockfile churn. The moment the
sweep would have grown anything behavioural — as the ticket instructs — it would
have split out; it never had to.
