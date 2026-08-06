---
type: task
blocked_by: []
claimed_by: sddb7e4d7f359
claimed_at: 2026-08-06T15:48:07Z
---

# The conventions ruleset, and the fixed map root

## Question

Write chartr's file-format contract, embed it, materialize it — and make discovery
actually enforce the root it declares. The spec offered this fold by name: the
document *states* that `.plan/maps/` is fixed and non-configurable, and `mapscan`
currently contradicts it by walking `.plan/` recursively and accepting any nested
`map.md`. Stating a rule the parser does not keep is what both previous attempts at
this contract did wrong.

**The document is the largest writing job in the effort and it lands first**, while
the layer model is still live and nothing points at it yet — which is what makes
this ticket risk-free to land early. Write it to the spec's contents list: the
applicability rule that opens it (when work is asked to be charted into a map,
created as a wayfinder map, created as tickets, or equivalent wording, the output is
a chartr-compatible artifact using this convention); the fixed layout including the
sibling `-impl` map and the reserved `spec.md` slot; the filename and permanent-
identity rule with its zero-padding; `map.md`'s five sections in order with the
relative-link index syntax and the clearing-edge syntax; the recognized ticket
frontmatter with `status` forbidden and unknown keys tolerated; the exact structural
headings; the derived-status table; the frontier calculation; and the chartr-
specific format terms defined where they are used.

**Draw the format/method line hard.** Excluded, deliberately: interviewing, research
and prototyping technique, ticket decomposition and sizing, the wayfinder decision
process, role behaviour, commit discipline, spec templates. Putting any of those
here recreates the method skill chartr has just stopped shipping — and this is the
one file an operator cannot disable, so a method rule that leaks in is unshadowable.

**One document, not a directory.** Both writer types need the same small contract;
splitting buys no payload reduction and makes partial reading possible.

Materialization is the narrow exception to hackability that a parser contract
requires: embed the canonical bytes, write them atomically at startup, and
**reconcile again before every payload composition including a preview** — missing
or differing bytes are replaced. An upgrade therefore updates it automatically and
an operator edit lasts until the next composition. `preferences.md` is the opposite
and must not be confused with it: created empty on first run, **never rewritten or
merged**, recreated and treated as empty if later missing, and **failing composition
visibly** if it exists but cannot be read — silently dropping the operator's own
instructions is the one behaviour that is not acceptable here.

Then narrow `mapscan` to `.plan/maps/` and nowhere else. **This is the widest blast
radius in the effort** — it changes discovery for every registered space at once,
and getting it wrong empties the star map everywhere, this map included.
Compatibility with the old flat layout is knowingly dropped. It is cheaply bounded
because this repo's own maps are already under `.plan/maps/`, so the mistake shows
up in your own cockpit on the next launch; land it as its own commit inside this
ticket so it can be reverted alone. The existing lint stays unchanged, non-blocking
and advisory — this effort adds no lint command, no launch gate and no automatic
repair, and no session is ever told to run lint.

Write the new **Conventions** entry in `CONTEXT.md`.

## Done when

`conventions.md` and `preferences.md` appear under a throwaway config root on
startup; editing the conventions file and composing anything restores it; deleting
`preferences.md` recreates it empty; an unreadable `preferences.md` fails a
composition visibly rather than silently. `mapscan` finds maps under `.plan/maps/`
and nowhere else, and this repo's own star map still renders every map it renders
today. `go vet`, `go test`, and the frontend `check`/`build`/`vitest` pass.

## Answer

Two commits: the contract (`a221c34`) and the discovery narrowing (`90057af`),
split so the second can be reverted alone.

**The document** is `internal/prompt/assets/conventions.md`, embedded as a string.
It opens with the applicability rule (work asked to be charted into a map, created
as a wayfinder map, broken into tickets, or equivalent wording produces a
chartr-compatible artifact in this format), then: the fixed layout with the
sibling `-impl` map and the reserved `spec.md` slot and the explicit statement that
a `map.md` anywhere else is invisible; the filename/permanent-identity rule with
zero-padding below 100 and natural width above, never reused or renumbered, readers
tolerant of legacy widths; `map.md`'s five sections in order, with the
relative-link index bullet under `Decisions so far`, the bold-lead bullet and the
italic `*Anchored to [Title](tickets/NN-slug.md).*` clearing edge under `Not yet
specified`, and the rule that a ruled-out ticket is indexed under `Out of scope`;
the recognized frontmatter as a table with `status` forbidden, `claimed_by` /
`claimed_at` chartr-owned, and unknown keys tolerated and ignored; the four exact
structural headings; the derived-status table in evaluation order with the
bare-heading caveat; the frontier calculation; and the six format terms defined at
the end where the format uses them. Nothing about interviewing, research or
prototyping technique, decomposition, sizing, the decision process, role behaviour,
commit discipline or spec templates is in it — the format/method line is the reason
this file can be unshadowable.

**Materialization** is `prompt.ReconcileContract(configDir)` in
`internal/prompt/conventions.go`. It writes the conventions atomically (temp file
in the same directory, 0600, rename) whenever the bytes are missing or differ, and
returns a `Contract{ConventionsPath, Preferences}`. `preferences.md` is the
opposite: created empty when absent, never rewritten when present, and an existing
file that cannot be read returns an error naming the path. It runs at two points —
`server.New` after `prompt.Materialize`, and the top of `prompt.Compose`, which now
takes a `ConfigDir` (both call sites, spawn and preview, pass `s.opts.ConfigDir`).
So an upgrade updates the contract without a restart, and an operator edit lasts
exactly until the next composition, preview included.

**Deliberately not done in this ticket:** the returned `Contract` is not yet a
payload part. Ticket 07 owns where the conventions path and the preferences bytes
sit in the two payloads; here the value of the call is the reconcile and the
visible failure, and `Compose` discards the rest. That is a knowing, commented
loose end, not an oversight — wiring the parts now would have written ticket 07's
payload shape blind.

**The narrowing:** `mapDirs` is no longer a `filepath.WalkDir` over `.plan/` but a
single `os.ReadDir` of `.plan/maps/`, keeping the children that hold a `map.md`.
A `map.md` nested any deeper is not a map. The test fixtures moved with it —
`chartrtest.MapDir(slug)` names the root once, `WriteMap`/`WriteTicket` go through
it, and the four hard-coded `.plan/<slug>/…` paths in the halt, release and spawn
tests were re-pointed. `TestMapAppearsByNoticeBothLayouts` became
`TestMapAppearsByNoticeUnderTheFixedRootOnly`: it writes a flat map and one nested
a level too deep *first*, then waits for a map under the fixed root, and asserts
the two are still absent — so the negative is checked after discovery has had every
chance to see them. The stale layout-agnostic comments in `mapscan` and
`model.Map` were corrected to match.

**Verified against the Done-when:**

- Ran the real binary with `XDG_CONFIG_HOME=$(mktemp -d)`: `conventions.md` (byte
  identical to the embedded asset) and an empty `0600 preferences.md` appear on
  startup.
- `TestComposingRestoresTheConventionsFile` truncates the file and composes; the
  canonical bytes are back. `TestAnEditedConventionsFileIsRestored` covers the
  startup path.
- `TestADeletedPreferencesFileIsRecreatedEmpty`, and
  `TestPreferencesAreReadVerbatimAndNeverRewritten` for the never-rewritten half.
- `TestAnUnreadablePreferencesFileFailsComposition`: `chmod 000`, compose, expect
  an error naming `preferences.md`.
- Discovery: 19 of this repo's 19 `map.md` files are under `.plan/maps/<slug>/`, so
  the star map renders exactly what it rendered before — the narrowing costs this
  repo nothing.
- `go vet ./...`, `go test ./...`, and `web/` `check`, `vitest`, `build` all pass.

**Flagged, nothing acted on:** the existing lint is untouched, still advisory, and
no session is told to run it, as specified. `CONTEXT.md` gains **Conventions** and
**Preferences**; while writing them I removed "preferences" from **User config**'s
`_Avoid_` list, because the word now names a specific file and the avoid-list would
have forbidden the correct term. The `tracker-convention` skill, the
`issue-tracker.md` adapter template and the glossary are all still live and
untouched — they are ticket 09's to cut.
