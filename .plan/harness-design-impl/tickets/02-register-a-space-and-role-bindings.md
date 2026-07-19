---
type: task
blocked_by: [01]
---

# Register a space, and resolve role bindings

## Question

The two config-layer slices, delivered together. **The registry:** the empty registry is the first-run screen; registering a folder makes it a space (a non-repo gets an announced `git init`, never silent), listed in the sidebar pinned-then-recency with the always-present filter; removal is forget-not-destroy — nothing in the repository is touched; the registry lives in the operator's user config as a rebuildable index, never a source of truth. **Role bindings:** `{adapter, model, args?}` in TOML across built-in ‹ committed workspace ‹ local user layers, field-level merge resolving user-over-workspace (ADR 0009); the effective resolved binding is always rendered; adapters probe PATH for their binaries and absence surfaces as a registry badge; a committed autopilot flag is ignored with a warning.

Done when: process-boundary tests cover register (including the announced `git init`), forget-not-destroy removal, registry loss costing only re-adding folders, the binding merge matrix (field-level inheritance, user-over-workspace), and the ignored committed autopilot warning; in the browser, first run shows the register affordance, a registered space shows its row with ordering and filter working, and the effective bindings with any absence badge are visible.

## Proposed Answer

The registry and role-binding slices delivered together, both derived into the pushed model and driven by plain HTTP actions. Layout:

- **`internal/registry`** — the space registry as a rebuildable index in the user-config layer (`registry.toml` under the data dir), never a source of truth (ADR 0003). `Register` cleans the path, requires an existing directory, and runs `git init` when the folder is not yet a repo — reported in the action response, never silent (story 2). `Deregister` forgets the entry and its local pin/recency and touches nothing in the repository (story 4). `List` orders pinned-first then by recency; a stable id derived from the absolute path means a lost registry re-adds to the same identity. Writes are atomic (temp + rename).
- **`internal/config`** — role bindings `{adapter, model, args?}` across three layers, built-in ‹ committed workspace (`.wayfinder-harness.toml`) ‹ local user (`user.toml`, keyed by space path), merged **field by field** and resolving **user-over-workspace** (ADR 0009). The effective binding records the source layer of each field so inheritance is visible (story 39); an adapter absent from PATH becomes a badge naming the binding, its source layer, and the local-override fix (story 40); a committed autopilot flag is **ignored with a warning** (ADR 0009). The closed role set is enforced by surfacing — an unknown role warns rather than binding silently — and a malformed config file degrades to a warning plus the layers below, never a refusal.
- **`internal/server`** — register / deregister / pin are plain HTTP actions (ADR 0010); each recomputes the derived model from the registry and the config on disk and pushes the whole snapshot. The registry loads at startup so a restart restores spaces. `internal/model` grows `Space` (path, pinned, bindings, warnings) and `RoleBinding` (effective fields, per-field provenance, presence).
- **`web/`** — the empty registry is the first-run screen with one register affordance; a populated sidebar renders spaces in server order with an always-present filter, pin, and forget, plus an absence badge on a row whose adapter is missing; the detail pane shows each role's effective binding with per-field layer tags and the absence message. Rows are buttons (selection is keyboard-reachable) and no state is colour-only.

Against Done-when: `internal/server/spaces_test.go` extends the process-boundary rig and covers register with the announced `git init`, forget-not-destroy removal, registry-loss rebuildability, the binding merge matrix (field-level inheritance and user-over-workspace, with built-in fallback), and the ignored committed autopilot warning — plus the adapter presence badge and pin ordering. `go vet ./...`, `go test ./... -race`, `svelte-check` (0 errors), and the Vite build pass; `make build` serves the embedded cockpit and the register / forget / pin actions end to end. In the browser, first run shows the register affordance, a registered space shows its row with ordering and filter working, and the effective bindings with any absence badge are visible.

Scope notes for review: full keyboard-first space-switch / queue hotkeys and the cross-space "Needs you" queue are ticket 14's — this slice keeps selection keyboard-reachable via focusable rows. Discovery-by-notice of config changes is ticket 03's watch; bindings here resolve fresh on each action rather than on a file watch.

Review payload should carry this Done-when and the spec by assembly (spec, Prompts and payload).
