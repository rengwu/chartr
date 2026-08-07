---
type: task
blocked_by: [04, 07]
claimed_by: s4aac98c813a3
claimed_at: 2026-08-06T19:29:34Z
---

# The new-shell control

## Question

Replace the skill launcher with one split button on the space card. The button body
is `empty shell` — the zero-decision action, and what keeps the static label honest,
since a body that launched the last-used agent would make "new shell" lie about what
a click does. The caret opens a menu: `empty shell`, a divider, then the registered
agents **in registration order, no ranking, no model sublabels**. The spawn picker's
ranking and remembered last-agent stay where they are; a free session has no
"confirm this and go" flow to optimise. The empty-library state keeps the divider
with one disabled row routing to registration, reusing the existing message and
callback. The launcher and the `+` shell button collapse into this one control, so
the actions row gets *shorter* — the branch chip, which doubles as the spacer, gains
the width. Nothing else in the sidebar moves.

Choosing an agent starts a **free session**: `Terminal.session` stays nil, so it
never counts toward the one-session-per-space gate and never freezes dead, while
`launchedAgent` from ticket 04 carries the identification. Title it by the agent's
registered name — the tab is titled by the thing the operator clicked, which is the
only labelling rule that never needs explaining. **Three free sessions on one agent
get three identical titles and that is fine**; every ad-hoc shell in a space is
titled `zsh` today. A disambiguating counter is per-space state to allocate, recycle
and keep stable across a model push, for a cosmetic gain.

The payload rides the **same file-plus-`Opener` mechanism, unchanged**. Do not
inline it into the opener line: `typeOpener`'s entire body is a workaround for how
TUIs mishandle exactly that — it strips trailing newlines, sends `\r` not `\n`, and
sleeps so the submit key lands in its own read, because a text-plus-return chunk
looks like a paste and gets swallowed. The free payload is also not small and its
size is not chartr's to know, since preferences are operator-owned and unbounded.
`run/<sid>/` already applies — a free session already mints a session id — and
archiving keeps running, because the archive is the record. **The in-repo copy stays
by decision**, not by oversight; see the spec's "rule that ships broken".

**The empty shell gets zero changes. Empty means empty**: no conventions pointer,
no sources list, no payload of any kind. If the operator types an agent name into
it, that is an agent in a terminal chartr did not launch — already out of scope, and
injecting a pointer on the chance one appears later would be chartr writing
instructions into a tree on speculation.

Deleted: `SkillLauncher.svelte` and its test, replaced by one `NewShellButton.svelte`;
`launchmenu.ts` and its test, **replaced by nothing** — its job was ranking agents ×
on-ramp skills into a choice state machine, and the new menu is a static list plus
one fixed row (`agentchoice.ts` survives, the spawn picker still uses it); the
`/ideate` route, its handler and its client action; `prompt.Ideate` and
`prompt.IdeateSkill`; the `on-ramp:`/`needs-context:` frontmatter with
`Skill.OnRamp`/`Skill.NeedsContext`, their fields on the pushed model, their producer
in `configsurface.go`, and their mirrors in `web/src/lib/model.ts`; `App.svelte`'s
`onlaunch` wiring and `SpaceCard`'s now-unused `onlaunch`/`skills` props; and the
context modal — a free session takes no context line, the operator types their first
message into a live TUI, which is what the TUI is for.

**Renamed, not deleted:** `OpenOnRamp` becomes `OpenFree`, body unchanged but for
the agent field. `handleLaunch`/`launchOnRamp` and the launch route become the
free-session handler — same `agentSpec` refusals in the same order, with the skill
lookup and the on-ramp allowlist check removed and `prompt.Launch` swapped for
`prompt.ComposeFree`. This route, not `/ideate`, is the actual spine; `/ideate` is a
thin delegate to it. Free-session launch mechanics already exist and are already
tested, which is the good news on this ticket.

No new frontend tests: the deleted launch-menu module takes its own test with it,
and the split button is existing `button` + `dropdown-menu` primitives composed with
no logic in a testable module. **No `shadcn-svelte add` step and no new token
should be needed** — if you reach for one, flag it. Write the new **Free session**
entry in `CONTEXT.md`.

## Done when

The space card shows one `new shell` split button in both the populated and
empty-library states; its body opens a plain shell with nothing injected; picking an
agent opens a tab titled with that agent's name, running against the free payload,
which never counts toward the one-session-per-space gate. Every item on the deletion
list is gone and every rename is done. `go vet`, `go test`, and the frontend
`check`/`build`/`vitest` pass.

## Answer

One commit. `NewShellButton.svelte` is a `Button` + `DropdownMenu.Trigger` pair
sharing an outline (`rounded-r-none border-r-0` / `rounded-l-none`), no new
primitive and **no new token** — the whole control is `outline` variant plus the
existing `text-destructive` for an absent agent's reason. No `shadcn-svelte add`
step was needed.

### Each Done-when clause

- *One split button in both library states* — the populated menu is `empty shell`,
  `DropdownMenu.Separator`, then `agents` in the order the model pushes them, with
  no ranking, no remembered default and no model sublabel; an agent absent from
  PATH is a `disabled` row under its own `missing` line. The empty state keeps the
  divider and reuses SkillLauncher's exact message and callback (see the flag
  below on "one disabled row").
- *The body opens a plain shell with nothing injected* — `onshell` is the
  card's existing `onopenshell`, unchanged, straight to `POST /terminals`. Nothing
  new touches that path, so empty stayed empty by not being edited.
- *Picking an agent opens a tab titled with that agent's name, on the free payload*
  — `launchFree` composes `prompt.ComposeFree(s.opts.ConfigDir, s.srcs)`, writes it
  through the same `writeSessionPayload` → `run/<sid>/payload.md` → `adapter.Opener`
  → `typeOpener` path (the file-plus-`Opener` mechanism, untouched — nothing was
  inlined into the opener line), and calls `OpenFree` with `spec.Name` as the title
  and `spec.Adapter` as ticket 04's `launchedAgent`.
  `TestFreeSessionOpensATabTitledByTheAgent` asserts the on-disk bytes are
  byte-identical to `ComposeFree`'s markdown, the title is the registered name, and
  the opener the stub received says "Read the file".
- *Never counts toward the one-session-per-space gate* —
  `TestFreeSessionIsOutsideTheOneSessionGate` is new and is the one assertion the
  old suite never made: it opens a free session, spawns a real session in the same
  space (which a second spawn would refuse 409), and opens a second free session
  with that session live.
- *Every deletion done, every rename done* — a grep over `internal/`, `web/src/`
  and `cmd/` for `OnRamp|onRamp|on-ramp|OpenOnRamp|IdeateSkill|prompt.Ideate|
  launchmenu|SkillLauncher|onlaunch|NeedsContext|needs-context|handleLaunch|
  launchOnRamp|/ideate` returns only prose in comments, no live symbol. The stale
  prose I could fix cheaply I did (`terminal.go`, `manager.go`, `spawn.go`); what
  is left sits in doc comments on symbols ticket 09 deletes outright
  (`prompt.Launch`, `readmeText`), so rewording them would be churn.
- *All checks* — `go vet ./...`, `go test ./...`, and `check`/`build`/`vitest` in
  `web/` all clean.

### A settled decision the operator reopened, after the fact

**The body is no longer fixed to `empty shell`.** The ticket settles this
explicitly — "the button body is `empty shell` — the zero-decision action, and
what keeps the static label honest, since a body that launched the last-used agent
would make 'new shell' lie about what a click does". The operator looked at the
built control and asked for the opposite: *"the dropdown should act like a
selector, not a launcher. once selected, the shell/model-injected session can be
launched with the main button."* That is a human's call on a design decision, so
it is taken, and it is recorded here rather than quietly absorbed — a later reader
comparing the ticket to the code needs to find this paragraph.

What it changes: the menu is one `DropdownMenu.RadioGroup` spanning both halves —
`new shell` and the agents are the same kind of choice, so exactly one is checked
and the divider groups rather than separates. Picking a row only *chooses*.
The body then **relabels itself to the choice** and runs it on click.

The ticket's objection is answered rather than overruled: the label lies only if
it stays static while the action moves, and here it does not — the button reads
`new shell` or `claude-opus`, and that is exactly what a click does. What is
genuinely given up is the guarantee that the leftmost control in the actions row
is *always* a plain shell, one click, no reading required. The selection is
component state defaulting to the plain shell, so a reload restores that; it is
not seeded from the server's `lastAgent`, which keeps the ticket's "no remembered
last-agent for free sessions" rule intact. An agent that is deregistered or leaves
PATH while selected falls back to the plain shell rather than leaving the body
pointed at something that cannot run.

### What moved beyond the ticket's list, and why

**`SpacePane.svelte` also hosted a `SkillLauncher`.** Its empty-state pane ("No
shell open in this space") carried a `New Shell` button and a `Skills ▾` menu side
by side — the same pair the card had. Deleting the component forced the same
collapse there, so it now renders the identical `NewShellButton` at `size="sm"`.
The ticket says "nothing else in the sidebar moves", and nothing did; this is the
stage, not the sidebar, and it was not optional.

**`SpaceCard` gained `onfreesession`, it did not merely lose `onlaunch`.** The old
prop's `(agent, skill, context)` signature is what was unused; the agent still has
to reach `App.svelte`. Same for `SpacePane`'s `onLaunch` → `onFreeSession`.

**Test files.** `launch_test.go` and `ideate_test.go` are replaced by one
`free_test.go` rather than edited: between them they held eleven tests on two
routes that are now one, most of which were the same doorstep assertion written
twice. The seven that survive cover the tab shape and title, the payload bytes,
the gate, both refusal ladders, the remembered agent, no claim commit, and no
death halt. Frontend tests: none added, per the ticket — the deleted
`launchmenu.test.ts` went with its module and the split button has no logic in a
testable module.

### Flagged

- **"One disabled row routing to registration" is self-contradictory**, so I did
  not implement it literally: a `DropdownMenu.Item` that is `disabled` does not
  fire its callback, which would make the empty state a dead end — exactly what
  the ticket says it must not be. I reused SkillLauncher's empty branch verbatim
  instead: a muted `DropdownMenu.Label` carrying "No agents registered yet." above
  an enabled `Register an agent…` item calling `onregister`. That honours "reusing
  the existing message and callback" and the divider; it is two rows, not one.
- **`prompt.Names()` lost `ideate`, so the shipped library is nine skills, not
  ten.** `TestShippedLibraryIsTenSkills` is now `…IsNineSkills`. The `ideate/`
  directory under `internal/prompt/assets/skills/` is *left on disk* — unreachable
  through `Names()`, so dead bytes in the binary for one ticket. I did not take
  ticket 07's precedent of deleting the directory, because ticket 09 deletes the
  whole `assets/skills` embed anyway and 07's case was different: nothing else was
  coming for `tracker-convention`.
- **`prompt.Launch` survives with no production caller.** It is explicitly on
  ticket 09's deletion list, so I left it and its test rather than pull a symbol
  out from under that ticket. It is dead code at HEAD.

### Deliberately not done

I did not run the built app by hand: the diff's two interesting seams are the
payload bytes on disk and the gate, and both are asserted at the process boundary
against a real PTY and a real registry, which is a stronger signal than eyeballing
a dropdown — and the session was asked to be economical. **What that leaves
unverified is the split button's appearance**: that the two halves' radii and
border actually join at the seam, and that the caret's popover is not clipped by
the sidebar's overflow at `size="xs"`. Both are one `make build webview` away and
neither can break a test. The ticket's own note that no new token should be needed
held, so there is nothing to flag on the palette.
