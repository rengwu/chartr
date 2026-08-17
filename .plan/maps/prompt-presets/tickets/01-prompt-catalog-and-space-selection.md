---
type: task
blocked_by: []
undermined_by: []
claimed_by: sb5ea0076d97e
claimed_at: 2026-08-17T19:24:36Z
---

# Persist the prompt catalog and per-space launch selection

## Question

Implement the smallest server-authoritative data spine described by
[the specification](../spec.md): one ordered global `prompts.toml` catalog, CRUD
for its stable entries, and a remembered `At launch` selection for each registered
space in the existing registry. Carry both through the pushed model and expose
only the HTTP actions the Prompts pane and later launch work need.

Keep the feature separate from skills and `preferences.md`. Catalog creation order
is its only order, selections compose in that order, deletion cleans the deleted
id from registered spaces, and Scratch remains unchanged. A malformed operator
file must be surfaced without being silently overwritten by a mutation.

## Done when

- A missing, valid, and malformed `prompts.toml` resolve according to the spec,
  with stable unique ids, non-empty names and bodies, and deterministic order.
- Create, edit, and delete actions atomically persist the catalog; editing keeps
  the id and deleting removes its launch selection from every registered space.
- A whole-list per-space action atomically persists valid selected ids without
  changing another space, and the selection survives registry reload.
- The complete model exposes the ordered catalog, each registered space's selected
  ids, and actionable warnings without introducing provider or skill concepts.
- Process-boundary tests cover the model and every mutation's successful and
  refused behavior, and the relevant Go tests pass.

## Answer

The data spine is `internal/prompts` (the global catalog), three fields
(`registry.Entry.Prompts`, `model.Model.Prompts`, `model.Space.Prompts`), and
four HTTP actions. No new concept beyond a preset: no provider, no skill, no
group, no order field.

**The catalog.** `internal/prompts/prompts.go` is a `Catalog` backed by
`<configDir>/prompts.toml`, built like `sources.Registry` and the space registry:
loaded once at startup, mutations persist the whole file atomically (temp file,
0600 under a 0700 root, rename). A row is `id`/`name`/`body` and nothing else;
file order is creation order and the only order there is. `Create` derives the
lower-kebab-case id from the name once and uniquifies it with a numeric suffix,
so two presets named the same thing are two presets; `Update` rewrites name and
text in place, keeping both the id and the row's position, which is what makes an
edit change every space that selected it and nothing more.

**Malformed is all-or-nothing.** A missing file is an empty catalog. *Any* defect
— a parse failure, a row with no id, a blank name or body, a duplicate id — yields
zero presets plus one warning naming the file, and every mutation is refused with
`ErrMalformed` (HTTP 409) until the operator fixes it. I chose whole-file
strictness over per-row salvage deliberately: salvaging rows would mean the next
click rewrites the file without the operator's unreadable content, which is
exactly the silent discard the ticket forbids, and "yields no executable presets"
is the spec's own phrasing. A test asserts the bytes on disk are byte-identical
after three refused mutations.

**The selection.** `Entry.Prompts` is a `prompts` key beside the space's existing
local state in `spaces.toml`. `SetPrompts` is a whole-list write (repeats
dropped, empty list legitimate, Scratch a no-op); `RemovePrompt` is the deletion
cleanup, called by the delete handler so removing a preset takes its references
with it in the same action. Ids are stored exactly as given: resolution against
the catalog happens at read time, in `Server.spacePrompts`, which returns the
selection in catalog order and a warning for any id the catalog no longer holds —
skipped, named, never substituted.

**The wire.** `Model.Prompts` is the ordered catalog (id, name, body — the body
rides because the pane shows and edits it and these are short); `Space.Prompts`
is that space's selected ids already in catalog order, so the pane and ticket
02's composition read one sequence. Catalog and missing-selection warnings join
the existing per-space config-warnings surface rather than inventing a second
one. Scratch derives an empty selection and is otherwise untouched. The TypeScript
mirror in `web/src/lib/model.ts` grew the same two fields (three test fixtures
updated), so ticket 04 starts from a typed model.

**Actions.** `POST/PUT/DELETE /api/config/prompts[/{id}]` (global, like the agent
library) and `PUT /api/spaces/{id}/prompts` (repo-scoped through the existing
`repoSpace` guard, so Scratch is refused rather than silently ignored). The
per-space write refuses a list naming a preset that is not in the catalog: that
can only come from a stale client, and storing it would record a choice the
operator never made — as against a *persisted* id that has gone missing, which is
tolerated and surfaced.

**Excluded**, as out of this ticket: any payload composition or launch behaviour
(ticket 02), live delivery (03), and the pane (04). Also deliberately omitted —
`prompts.toml` is not added to the settings surface's config-layer list, because
ticket 04 gives the catalog in-pane CRUD and a second editing route is surface
without a use. One consequence worth stating: like `sources.toml`, the catalog is
read at startup, so a hand-edit to `prompts.toml` needs a restart to be seen.

Tests: `internal/prompts` (round trip, every malformed shape, refusal without
overwrite, id derivation, catalog-order selection), `internal/registry`
(per-space isolation, reload, deletion cleanup, Scratch), and
`internal/server/prompts_test.go` at the process boundary (CRUD through the
snapshot, every refusal, malformed surfaced and untouched, selection per space
and across a restart, deletion clearing every space, a hand-edited missing id
surfaced). `make test`, `make vet`, and `make check` all pass, as do the web unit
tests.

