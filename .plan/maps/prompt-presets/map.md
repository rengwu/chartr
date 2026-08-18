# Space prompt presets

## Destination

An operator keeps short behavioral prompts in one Chartr-owned catalog, chooses
which ones apply when a registered space launches an agent, and can send or queue
one catalog prompt to that space's active agent tab. The feature
uses Chartr's existing payload, terminal-state, and pane machinery and introduces
no provider-specific registration or speculative extension system.

## Notes

- The settled behavior is [spec.md](./spec.md). Read it before taking any ticket;
  it is authoritative when a tempting generalization conflicts with the narrow
  first version.
- This is an implementation map. It explicitly overrides the wayfinder default
  to plan but not build: tickets here implement the settled specification.
- YAGNI is a product constraint, not just an implementation preference. Prefer
  the fewest new types, files, endpoints, and UI states that express the specified
  behavior, and reuse the existing payload composer, registry, terminal manager,
  activity state, and auxiliary-pane presentation.
- Live delivery through a PTY is deliberately best-effort. Do not add provider
  RPC integrations or claim a stronger guarantee than the terminal can provide.
- `make test`, `make vet`, and `make check` are the repository-level verification
  commands.

## Decisions so far

<!-- The specification is the settled premise. Implementation decisions land here. -->

- [Persist the prompt catalog and per-space launch selection](./tickets/01-prompt-catalog-and-space-selection.md)
  — `internal/prompts` owns an ordered `prompts.toml` catalog (id/name/body,
  creation order, atomic writes); a space's `At launch` ids live in
  `spaces.toml` as `Entry.Prompts`. Any defect in the catalog file yields zero
  presets, one warning, and refused mutations, so operator bytes are never
  overwritten. The model carries `Model.Prompts` (catalog) and `Space.Prompts`
  (that space's ids, already in catalog order); warnings ride the existing
  per-space surface. Actions: `POST/PUT/DELETE /api/config/prompts[/{id}]` and
  `PUT /api/spaces/{id}/prompts`. Deleting a preset clears it from every space;
  Scratch is unchanged and refused.

- [Compose selected prompts into ticket and free launches](./tickets/02-selected-prompts-in-launch-payloads.md)
  — `prompt.ComposeInput.Prompts` carries the space's selection (catalog order,
  resolved once in `Server.launchPrompts`); each preset composes as its own
  operator prompt part between `preferences` and the context region, so preview,
  spawn, and the existing `Payload-SHA256` see one composition. A free launch in a
  space with a selection writes those presets with the spawn path's own
  `writeSessionPayload` and opens with `adapter.Opener`, so argv, flag, and typed
  delivery need no new mechanism (`OpenFree` gained the `opener` parameter
  `OpenSession` already had); an empty selection composes the empty string and
  leaves the launch bare. Scratch is unreachable through `repoSpace` and
  `CHARTR.md` is untouched.

## Not yet specified

<!-- Nothing. The settled specification and four implementation tickets cover the destination. -->

## Out of scope

- Installing or synchronizing prompts with any agent provider or harness.
- Changing `CHARTR.md`, provider-native instruction files, registered skill
  sources, or `preferences.md`.
- Prompt groups, profiles, tags, variables, templating, reordering controls,
  import/export, sharing, or marketplace behavior.
- A generalized pane or plugin framework; Prompts is only a second concrete tool
  beside Map.
- Provider RPC, SDK, transcript, hook, or MCP delivery paths.
- Delivery to a shell with no agent in front of it, dead tabs, or Scratch.
- Persistent delivery queues, delivery history, audit logs, retries, or an
  attempt to retract instructions an agent already saw.
- Detecting or preserving an unsent draft already present in an agent's composer.

