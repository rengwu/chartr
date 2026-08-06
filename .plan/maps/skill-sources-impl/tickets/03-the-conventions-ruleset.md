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
