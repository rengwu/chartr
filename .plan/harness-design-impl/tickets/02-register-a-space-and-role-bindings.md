---
type: task
blocked_by: [01]
---

# Register a space, and resolve role bindings

## Question

The two config-layer slices, delivered together. **The registry:** the empty registry is the first-run screen; registering a folder makes it a space (a non-repo gets an announced `git init`, never silent), listed in the sidebar pinned-then-recency with the always-present filter; removal is forget-not-destroy — nothing in the repository is touched; the registry lives in the operator's user config as a rebuildable index, never a source of truth. **Role bindings:** `{adapter, model, args?}` in TOML across built-in ‹ committed workspace ‹ local user layers, field-level merge resolving user-over-workspace (ADR 0009); the effective resolved binding is always rendered; adapters probe PATH for their binaries and absence surfaces as a registry badge; a committed autopilot flag is ignored with a warning.

Done when: process-boundary tests cover register (including the announced `git init`), forget-not-destroy removal, registry loss costing only re-adding folders, the binding merge matrix (field-level inheritance, user-over-workspace), and the ignored committed autopilot warning; in the browser, first run shows the register affordance, a registered space shows its row with ordering and filter working, and the effective bindings with any absence badge are visible.
