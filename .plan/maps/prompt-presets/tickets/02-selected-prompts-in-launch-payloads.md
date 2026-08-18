---
type: task
blocked_by: [01]
undermined_by: []
claimed_by: s66b3f035f5d3
claimed_at: 2026-08-18T03:23:34Z
---

# Compose selected prompts into ticket and free launches

## Question

Make the selection settled in
[Persist the prompt catalog and per-space launch selection](./01-prompt-catalog-and-space-selection.md)
take effect on future Chartr-launched agents using the existing payload and adapter
delivery seams.

Ticket payloads carry each selected preset as its own operator prompt part after
global preferences and before context, so preview, spawn, and payload hashing stay
one path. A registered space's free launch creates and opens a small run payload
only when at least one preset is selected; an empty selection preserves today's
bare launch exactly. Do not change `CHARTR.md`, Scratch, skill resolution, global
preferences, claim trailers, or sessions already running.

## Done when

- Ticket preview and spawn compose the same selected preset bytes in catalog
  order, at the specified position, with useful per-part identity and provenance.
- The existing payload hash covers the selected bytes without a parallel audit
  mechanism.
- A free agent with selected presets receives a gitignored owner-only run payload
  through the existing adapter opener, under both argv/flag and typed delivery.
- A free agent with no selection still launches bare and creates or injects
  nothing new.
- Changing the space selection affects only later compositions, and Scratch and
  `CHARTR.md` are demonstrably untouched.
- Composition, payload process-boundary, and spawn/free-session tests pass.


## Answer

Selected presets compose through the payload seam that already exists, and a free
launch reuses the opener seam that already exists. No new file, no new endpoint,
no new audit path.

**Composition.** `prompt.ComposeInput` grew one field, `Prompts []prompts.Prompt`
— the space's selection, already in catalog order from ticket 01, so composition
does no resolving of its own. `presetParts` renders one part per preset between
`contractParts` and the context region: `Kind: "prompt"`, `Origin: operator`,
`Name: "preset <id>"`, `Label: <the operator's name for it>`, `Text: <body>`. They
sit after `preferences` because they are the same kind of thing — the operator's
standing instructions in their own voice — and before context because instruction
precedes data, the payload's one ordering rule. One part each rather than one
merged block so the preview can attribute a sentence to the preset it came from;
named by id rather than by name because the id is unique, is what the selection
stores, and is what keeps the preview's part list keyable when two presets share a
human name.

Because it is one composition, the ticket preview and the spawn produce the same
bytes and the existing `Payload-SHA256` covers the presets with nothing added: a
test asserts the spawned `payload.md` is byte-identical to the preview and that
its sha256 is the `payloadSha` the spawn reported.

**Free launches.** `prompt.ComposePresets` renders the same parts through the same
`renderMarkdown`, and returns the empty string for an empty selection — which is
what keeps a bare launch bare. `launchFree` writes that document with the spawn
path's own `writeSessionPayload` (gitignored run directory, 0600 payload under a
0700 session directory) and passes `adapter.Opener(path)` to `adapter.Command`, so
argv, flag, and typed delivery all work with no second mechanism. `OpenFree` grew
the `opener` parameter `OpenSession` already had and calls the same `typeOpener`;
with no selection it receives the empty string and behaves exactly as before.
Tests drive all three delivery shapes and assert the opener line arrives tagged
`argv:` or `stdin:` accordingly.

**Resolution.** `Server.launchPrompts(entry)` is the one place both paths resolve a
selection, and it skips an id the catalog no longer holds without comment — the
snapshot already surfaces that on the space (ticket 01's `spacePrompts`), and a
launch is the wrong moment to discover it.

**Omitted deliberately.** No archive copy of a free session's presets: the archive
answers "what was this session told" for a *session*, and a free session is not
one. Nothing was added to the claim trailer — the payload hash already covers the
bytes, which is the ticket's own instruction. Scratch needed no code: `repoSpace`
already refuses both `/launch` and the spawn path there, so neither composition is
reachable, and `SetPrompts` no-ops it besides. `CHARTR.md` is composed by
`ComposeStanding`, which takes no space and was not touched; a test asserts its
bytes are identical across a selection change.

One drive-by correction: `CONTEXT.md`'s **Free session** entry still claimed the
free session is told "the free payload", which stopped being true before this map
existed — it launched bare. The entry now states the bare launch and the one thing
that can now ride with it. Two `gofmt` misalignments left in `internal/server/server.go`
and `internal/registry/registry.go` are formatted.

`make test`, `make vet`, and `make check` all pass.
