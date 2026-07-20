# The wayfinder adapter for harness-managed spaces

This is the adapter the `wayfinder` and `to-tickets` skills consult in a space
the harness watches. It **layers on top of** the local-markdown adapter
([`TRACKER-MARKDOWN.md`](https://github.com/rengwu/skills/blob/main/pocock/wayfinder/TRACKER-MARKDOWN.md),
vendored under `prompts/`) — every rule there still holds. The map format is
**untouched**: a vanilla wayfinder tool reads the same `.plan/<slug>/` map
unchanged. This file adds exactly one harness-side step, on one event.

## On creation, record the map's kind

The harness needs a map's **kind** — planning or implementation — before it
offers any action on it (ADR [0007](adr/0007-map-kind-declared-not-inferred.md)).
Kind is *declared, never inferred*: the harness will not run a lifecycle on a
guess. So a discovered map with no declaration is **inert** — readable, but
offering no sessions — until a human confirms its kind.

Recording the kind **at the moment the map is created** is what keeps that
confirm from ever surfacing. It is a one-time declaration the creating session
already knows the answer to: charting with `wayfinder` produces a planning map;
`to-tickets` produces an implementation map. Write it down then, and no one is
ever asked to.

**When you create a map, append its kind to the space's committed harness
config, `.wayfinder-harness/config.toml` (create the `.wayfinder-harness/`
directory if the space has none yet):**

```toml
[maps."<slug>"]
kind = "planning"       # wayfinder chart → a planning map
# kind = "implementation"   # to-tickets → the .plan/<slug>-impl/ map
```

- `<slug>` is the map directory's name under `.plan/` — the same slug the map
  lives at (`.plan/<slug>/`), quoted as a TOML basic-string key so a slug
  carrying a `.` never misreads as a dotted key.
- **Append; never rewrite.** The file is the operator's — their role bindings,
  comments, and formatting are theirs to keep. Add your one table at the end,
  separated by a blank line. If a `[maps."<slug>"]` table for this map already
  exists, leave it alone: it is already declared.
- **Commit it with the map.** The declaration is committed config, versioned and
  portable, so a teammate cloning the space inherits the kind rather than
  re-confirming it. It rides in the same commit as the `map.md` and tickets you
  are already committing for your own work.

Why here and not `kind:` in `map.md`: putting it in the map body would extend the
wayfinder markdown format past what the tracker spec defines, and a vanilla
wayfinder tool would meet a field it does not know (ADR 0007). The declaration
lives in harness-owned committed config precisely so the map format stays clean.

### Graduating a planning map to implementation

Running `to-tickets` on a finished planning map writes a new
`.plan/<slug>-impl/` map. Record **that** map's kind too —
`[maps."<slug>-impl"]  kind = "implementation"` — in the same commit. The
harness notices the new directory either way; recording the kind is what spares
it the classify prompt.

## The fallback, if this step is skipped

A map created without this step — hand-written, charted outside the cockpit, or
predating the adapter — arrives **unclassified and inert**. That is not an error:
the harness renders it, marks it unclassified in the sidebar (a quiet dashed
marker, no action), and surfaces a one-click **plan / impl** confirm inside the
star-map panel when the map is opened, with the convention guess pre-selected.
Recording the kind at creation is simply what keeps that fallback rare.
