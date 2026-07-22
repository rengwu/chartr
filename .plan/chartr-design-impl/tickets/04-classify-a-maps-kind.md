---
type: task
blocked_by: [03]
---

# Classify a map's kind

## Question

Kind is declared, never inferred (ADR 0007). A discovered map with no declaration is inert — rendered and readable in the snapshot, but offering no session actions — until classified. Classification is one action over HTTP: the convention heuristics (`-impl` suffix, all-`task` tickets) pre-fill the guess for a one-keystroke confirm, and the confirmed kind is written to the committed workspace config, keyed by map slug. A renamed map directory dangles its entry and resolves into unclassified-and-inert, never an error. The map format itself is untouched — a vanilla wayfinder tool reads the same map unchanged.

Done when: process-boundary tests assert an undeclared map's snapshot offers no session actions; classifying writes the committed config and the actions appear; the pre-filled guess matches the conventions on fixtures for both kinds; renaming a classified map's directory returns it to unclassified-and-inert; the inline confirm renders in the sidebar.

## Answer

A discovered map is now inert until a human declares its kind, and classification is one HTTP action that writes the declaration into committed workspace config. The lifecycle is declared, never inferred (ADR 0007): the conventions survive only as a pre-filled guess. Layout:

- **`internal/model`** — `Map` gains `Kind` (the empty `KindUnclassified`, or `KindPlanning` / `KindImplementation`) and `KindGuess`, plus a `ValidKind` helper. The empty kind is a genuine third state — undeclared and **inert** — and is the gate the session actions of later tickets hang off a map. `KindGuess` is set only while unclassified and blanked once declared, so the heuristic never lingers as gospel.
- **`internal/mapscan`** — `GuessKind` proposes a kind from the two breakable conventions ADR 0007 keeps alive only as a one-time guess: the `-impl` directory suffix and every ticket typed `task`, defaulting to planning. `deriveMap` computes it on every map. It is never authoritative — a human confirms it, and a committed declaration always overrides it.
- **`internal/config`** — the committed workspace layer grows a `[maps."<slug>"]` table with a `kind` field; kind is committed-layer only, so teammates agree rather than each re-classifying (story 15). `Resolve` surfaces recognised kinds as a slug→kind map and, for an unrecognised value, warns-then-drops it so the map stays unclassified — adoption is never gated on config lint. `DeclareMapKind` is the write half: it **appends** the table rather than decode-and-re-encode the file, so the operator's own bindings, comments, and formatting survive untouched (the hackability stance), and it refuses an already-declared slug rather than write a duplicate table or silently rewrite the operator's bytes. `WorkspaceConfigName` moves here — config owns the file's shape.
- **`internal/server`** — `POST /api/spaces/{id}/maps/{slug}/classify` validates the kind, appends via `DeclareMapKind`, writes atomically (temp + rename), and rebuilds-and-pushes. `deriveSpace` overlays committed declarations onto the discovered maps and clears the spent guess; a declaration whose slug matches no discovered map dangles harmlessly — a renamed directory resolves to unclassified-and-inert, never an error.
- **`web/`** — the sidebar map-row renders the inline confirm while a map is unclassified: a dimmed name and a `kind? plan / impl` pair with the convention guess pre-emphasised for a one-keystroke confirm, swapping back to the frontier count once the kind is declared.

Against Done-when: `internal/server/classify_test.go` extends the process-boundary rig and covers an undeclared map inert with a guess (kind unclassified, `KindGuess` set); classify declaring the kind into committed config keyed by slug while an existing role binding survives the append; the guess matching the conventions for **both** kinds on fixtures without ever auto-classifying; a renamed directory dangling its entry back to unclassified-and-inert **by notice** (rename while watching, wait for the push); classify rejecting a bogus kind (400), a missing space (404), and an already-declared map (400); and an unrecognised committed kind surfaced-and-inert. `go vet ./...`, `go test ./...`, `svelte-check` (0 errors), and the Vite build pass.

One decision for review to weigh: **classification writes `.chartr.toml` but does not `git commit` it.** The spec enumerates the chartr's own commits as exactly claim / promotion / demotion (ADR 0008), none of which machinery exists yet; "committed config" is read here as the version-controlled *layer* (as ticket 02's bindings are — the operator commits them), and leaving the write in the working tree matches "operator owns the tree" and dirty-tree-is-a-badge. If ADR 0007's "recorded in chartr-owned config committed to the space's repo" is meant as *the chartr commits it*, this is the line to change — the write path is isolated in `handleClassify`, so adding a pathspec-limited commit later is local.

Scope notes for review: this slice declares kind and gates on it; the session actions the gate governs (spawn, review) are tickets 09+, so "offers no session actions" is asserted through `Kind` — the gate that will govern them — rather than through actions that do not yet exist. Re-classifying a declared map is deliberately not an affordance (a classified map is not inert); the idempotence guard only catches a race or a hand-edited entry.

Review payload should carry this Done-when and the spec by assembly (spec, Prompts and payload).
