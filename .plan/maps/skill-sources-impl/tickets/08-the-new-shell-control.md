---
type: task
blocked_by: [04, 07]
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
