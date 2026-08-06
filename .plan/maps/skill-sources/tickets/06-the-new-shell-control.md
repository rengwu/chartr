---
type: grilling
blocked_by: [02]
assets: [new-shell-sketch.png]
claimed_by: s09f416f39a0f
claimed_at: 2026-08-06T09:04:45Z
---

# The new-shell control replaces the launcher

## Question

The skill launcher, its dropdown and the context-gathering modal are deleted. The space card grows a `new shell` split button instead: `empty shell` above a divider, then the registered agents, per the operator's sketch in [`assets/new-shell-sketch.png`](../assets/new-shell-sketch.png). Choosing an agent starts a free session; choosing `empty shell` spawns a shell with no agent, which is what `newTerminal` already does today.

This is a prototype ticket because the sketch settles the shape but not the behaviour, and the behaviour is where a tab's identity, title and lifecycle live.

Settle:

- **What a free session *is* in the model.** `Terminal.session` is non-nil for a ticket session and nil for an ad-hoc shell, and a great deal hangs off that (`internal/terminal/terminal.go`) — titling, the agent grammar's idle/working detection, the working-state seed at launch, what drops from the model on exit. A free session is an agent with a payload but no ticket. Which side of the existing split does it land on, or does it become a third thing?
- **Titling.** A ticket tab is titled by its ticket, an ad-hoc shell by its process. A free session has neither. The agent's name is the obvious candidate and the sketch's menu is built from exactly those names — confirm, and say what happens when three free sessions run the same agent.
- **The payload's delivery.** A ticket session writes a payload file and opens with a one-line read-this instruction (`adapter.Opener`). A free session's payload is smaller and never ticket-specific. Same mechanism, or is it small enough to ride the opener directly — and if it is a file, where does it live given `run/<sid>/` is keyed by session.
- **What the empty shell keeps.** Today an ad-hoc shell inherits chartr's environment exactly (`launchSpec.env` empty). It gets no payload and no agent. Confirm it stays exactly as it is, and that "empty" means empty — no conventions pointer, no sources list.
- **What is deleted, precisely.** `web/src/lib/SkillLauncher.svelte`, `web/src/lib/launchmenu.ts` and their tests, `OpenOnRamp` in the terminal manager, the `/ideate` route and its callers, `prompt.Launch` and `prompt.Ideate`, and the `on-ramp:` / `needs-context:` frontmatter with the `Skill.OnRamp` / `Skill.NeedsContext` fields. Walk the list and say what each is replaced by, or that it is replaced by nothing.
- **The sidebar's shape afterwards.** The launcher occupied space in the sidebar; the split button lives on the space card. Sketch what the card looks like with it, including the case where no agents are registered at all — the menu then has one live entry and a divider under it.

## Done when

A rough mockup exists showing the card with the split button in its populated and empty-library states, and the behavioural questions — free-session identity in the terminal model, titling, payload delivery, and the full deletion list — are answered against it.

## Answer

**A free session is today's on-ramp tab, renamed: `Terminal.session` stays nil, so
it never counts toward the one-session-per-space gate and never freezes dead.
But `session != nil` is doing two jobs today — "has a ticket" and "chartr knows
which agent runs here" — and the free session is the case that splits them. The
detection half moves to a new `launchedAgent` field; the lifecycle half stays on
`session`. Titling is the agent's registered name, duplicates allowed. The
payload rides the same file-plus-`Opener` mechanism, unchanged. The empty shell
is untouched — `Open` already does exactly the right thing and gets zero lines of
change. `OpenOnRamp` is *renamed, not deleted*, and the ticket's deletion list is
missing four items.**

**Retyped `prototype` → `grilling` before answering** (operator's call, 2026-08-06).
The Done-when asks for a mockup, and the operator's sketch already is one — the
empty-library state is a one-line inference from it, drawn below. Everything that
was actually unsettled is a decision read off existing code, not something a
throwaway slice could show. No prototype was built; the findings below are
grounded in the files named, not in a spike.

### The card, both states

The launcher and the `+` shell button collapse into one control, so the actions
row gets *shorter*, not longer — the branch chip (which doubles as the spacer,
`SpaceCard.svelte:598`) gains the width. That is the whole of the sidebar-shape
answer: nothing else moves.

```
populated                              empty library
┌────────────────────────────────┐     ┌────────────────────────────────┐
│ wayfinder-harness            × │     │ wayfinder-harness            × │
│ ▣ grill #06        claude   ●  │     │ ▣ zsh                          │
│ ▣ zsh                          │     │                                │
│  main         [ new shell │▾ ] │     │  main         [ new shell │▾ ] │
└────────────────────────────────┘     └────────────────────────────────┘
                ┌──────────────────┐                   ┌──────────────────┐
                │ empty shell      │                   │ empty shell      │
                ├──────────────────┤                   ├──────────────────┤
                │ claude-opus-yolo │                   │ no agents        │
                │ claude-sonnet-…  │                   │ registered   →   │
                │ gpt-5.6-sol-high │                   └──────────────────┘
                │ gpt-5.6-sol-med  │
                └──────────────────┘
```

- **The button body is `empty shell`; the caret opens the menu.** It is the
  zero-decision action, it is what today's `+` button does, and it is what keeps
  the static label "new shell" honest — a body that launched the last-used agent
  would make the label lie about what a click does.
- **Agents in registration order, no ranking, no model sublabels.** The menu is
  `[empty shell] ─── agents`. `chooseAgent`'s ranking and the remembered
  `lastAgent` are the *spawn* picker's machinery and stay there; a free session
  has no "confirm this and go" flow to optimise.
- **The empty-library state keeps the divider** (as the ticket says) with one
  disabled row routing to registration — the existing `onregister` prop and
  `emptyLibraryMessage` (`spawn.go:259`), reused verbatim.

### Free-session identity: nil `session`, plus one new field

Six behaviours branch on `session != nil`. Five of them want a free session on
the nil side, and the sixth is why nil alone is not enough:

| site | wants | why |
| --- | --- | --- |
| `isLiveSession` → `ErrSessionExists` | **nil** | the gate protects one working tree per *claim*; a free session claims nothing |
| `pinOnDeath` | **nil** | resume/respawn/release all act on a ticket; there is none to act on |
| `sampleGone` → `TerminalDead` | **nil** | `dead` is the halt state those three actions hang off |
| `sessionTitle` | **nil** | `"%s #%02d"` of role and ticket num — neither exists |
| `Info.Session` on the pushed model | **nil** | the tab renders no ticket row |
| **`sample()` identification** | **not nil** | ← the split |

`sample()` uses `session != nil` to mean *chartr launched this and recorded which
adapter ran, so skip the inspection*. That fact is equally true of a free session
— and today's on-ramp tab, which is the same shape, already pays for the mismatch
twice:

- **Every slow tick it runs `foreground()` + `procGroupNames(pgrp)` to rediscover
  an agent chartr chose itself** (`terminal.go:550`), where a session answers from
  its binding for free.
- **A free session on an agent with no shipped manifest reads permanently idle.**
  Its root process *is* the agent, so `pgrp == shellPID` forever, and
  `sampleShell` returns `TerminalIdle` for the tab's whole life
  (`terminal.go:666`). This is the exact failure `sampleUnknownSession` exists to
  prevent for sessions — and per [`codex-working-state-unverified`], kimi and
  friends are precisely the agents that would land here. It is a latent bug in
  today's on-ramp path that free sessions turn into the common case.
- Plus the boot flash: `newProc` seeds `TerminalWorking` only when
  `spec.session != nil` (`terminal.go:436`), so a free session reads idle for up
  to one slow tick while its agent draws its first frame.

**The fix, and it is small:** `launchSpec` gains `agent string` (the adapter
name), `Terminal` gains `launchedAgent`, and both `OpenSession` and `OpenFree`
set it. `sample()`'s identification branch and `newProc`'s working-state seed
read `launchedAgent != ""`; every other site above keeps reading `session != nil`.
`sampleUnknownSession` is renamed `sampleLaunchedAgent` and is reached by both.

**Not a third thing.** The model gains one field, not a third tab kind — there is
still exactly "is this bound to a ticket" and now "did chartr pick the binary".
Rejected: a `Kind` enum (`shell`/`free`/`session`), which is three names for two
independent booleans and would force every existing `session != nil` site to be
re-read and re-decided.

### Titling

**The agent's registered name** (`launchSpec.Name`) — confirmed. It is what the
menu row said, so the tab is titled by the thing the operator clicked, which is
the only labelling rule that never needs explaining.

**Three free sessions on one agent get three tabs titled `claude-opus-yolo`, and
that is fine.** The precedent is in the same list: every ad-hoc shell in a space
is titled `zsh` today and nobody has complained. A disambiguating counter is
per-space state the manager would have to allocate, recycle on close, and keep
stable across a model push — real machinery for a cosmetic gain, and YAGNI on
this map's terms. The tabs are ordered by creation and each renders its own
`Proc` and status beside the title, so they are told apart by the row, not the
word. Revisit if an operator actually runs three and reports losing one.

### Payload delivery: the file, unchanged

**Same mechanism — `writeSessionPayload` + `adapter.Opener(path)`.** Not inlined
into the opener line. Three reasons, in order of force:

1. **`typeOpener` cannot carry it.** That function's entire body is a workaround
   for how TUIs mishandle exactly this: it strips trailing newlines, sends `\r`
   not `\n`, and sleeps 150ms so the submit key lands in its own read *because a
   text-plus-return chunk looks like a paste and gets swallowed*
   (`manager.go:296–325`). A multi-line markdown document typed in as one line is
   the failure mode it was written to avoid.
2. **The free payload is not small, and its size is not chartr's to know.**
   `preferences.md` is operator-owned and unbounded (blocker 02), and the sources
   block grows with the registered library. "Small enough to ride the opener" is
   not a property this payload has.
3. **ADR 0005 wants it inspectable.** A file on disk is what the preview previews
   and what "what was this session told" answers word for word.

**Where it lives needs no new decision: `run/<sid>/` is already keyed by a
session id a free session already mints.** `terminals.go:113` calls
`newSessionID()` and `writeSessionPayload` for today's on-ramp launch; the id is
just not written into a claim, because there is no ticket. That is the whole
difference. `archivePayload` runs too, and should keep running — the archive is
the record.

**But blocker 02's surfaced doubt is now demonstrably real, and I am not closing
it here.** `.chartr/run/` sits inside the operator's repo, against this map's
standing rule that the repo carries nothing but `.plan/maps/`. What the code
shows is stronger than the doubt as blocker 02 stated it: **the in-repo payload
and the archived payload are byte-identical** (`spawn.go:311` and `:315` write
the same `payload.Markdown`), so the rule could be honoured by *deleting* the
in-repo copy and pointing `Opener` at `<data-root>/sessions/<sid>/payload.md`.
The one thing that could refute that is an agent sandboxed to its working tree,
which could read `.chartr/run/…` and not the data root — worth checking against
the registered adapters before anyone acts on it.

It is not ticket 06's to decide: it governs the ticket payload identically, and a
free session choosing a different home from a ticket session is the one outcome
that is certainly wrong. **A new ticket owns it**; free sessions use whatever
ticket sessions use.

### The empty shell: zero changes

Confirmed on all counts. `Open` → `newTerminal` → `launchSpec{name, args}` with
`env` empty, `session` nil, no opener typed, no payload written
(`terminal.go:464`). **Empty means empty: no conventions pointer, no sources
list, no payload of any kind.**

The reason is the map's own boundary restated, not a new one. chartr guarantees
what it launched; an empty shell launches a shell, and if the operator then types
`claude` into it, that is an agent in a terminal chartr did not launch — already
**Out of scope**. Injecting a conventions pointer into a shell on the chance an
agent appears later would be chartr writing instructions into a tree on
speculation, which is the thing the whole map is deleting.

Detection is unaffected either way: an ad-hoc shell running `claude` is still
identified by inspection and still reads the agent grammar (`terminal.go:550`,
and the comment at `:503` says that bug is exactly why). What it does not get is
a payload.

### The deletion list, walked

| item | fate |
| --- | --- |
| `web/src/lib/SkillLauncher.svelte` + its test | **deleted** → one new `NewShellButton.svelte` split button |
| `web/src/lib/launchmenu.ts` + `launchmenu.test.ts` | **deleted, replaced by nothing.** Its job was ranking agents × on-ramp skills into a `choice` state machine; the new menu is a static list plus one fixed row. `agentchoice.ts` survives — the spawn picker still uses it |
| **`OpenOnRamp`** | **renamed, not deleted** → `OpenFree`, body unchanged but for the `agent` field above. The ticket's premise is wrong here, and it is the good news on this ticket: free-session launch mechanics already exist and are already tested |
| `/ideate` route, `handleIdeate`, `actions.ts:ideate` | **deleted, nothing replaces.** The `ideate` skill is culled by the map's Out of scope |
| `prompt.Launch` | **deleted** → `prompt.ComposeFree` (blocker 02's seam) |
| `prompt.Ideate`, `prompt.IdeateSkill` | **deleted, nothing** |
| `on-ramp:` / `needs-context:` frontmatter, `Skill.OnRamp` / `Skill.NeedsContext` | **deleted, replaced by nothing.** They answered "which skills may launch cold"; cold launch is now agent-driven and no skill launches cold |
| the context modal | **deleted, nothing.** A free session takes no context line — the operator types their first message into a live TUI, which is what the TUI is for |

**Four items the ticket's list is missing.** Each is load-bearing and would be
found mid-implementation otherwise:

1. **`handleLaunch` / `launchOnRamp` and the `POST /api/spaces/{id}/launch`
   route** (`terminals.go:39–140`, `server.go:239`). This — not `/ideate` — is
   the actual spine, and `/ideate` is a thin delegate to it. `launchOnRamp`
   becomes `handleFreeSession`: same `agentSpec` refusals in the same order, the
   skill lookup and the `sk.OnRamp` allowlist check deleted, `prompt.Launch`
   swapped for `prompt.ComposeFree`. **Renamed, not deleted.**
2. **`model.Skill.OnRamp` / `NeedsContext` on the pushed model**
   (`model/model.go:119–120`) and their producer at `configsurface.go:144`.
3. **`web/src/lib/model.ts`'s `onRamp` / `needsContext`** (`:193–197`) — the
   frontend mirror of the same two fields.
4. **`App.svelte`'s `onlaunch` wiring** (`:441–453`) and `SpaceCard`'s
   `onlaunch` / `skills` props, which the split button does not need.

`prompt.Names()`'s `IdeateSkill` entry (`prompt.go:213`) and the `ideate/` line in
the embedded manifest (`:536`) die with `prompt.Names()` itself, which blocker 02
already deleted.

### Rejected

- **A third tab kind / `Kind` enum.** Two independent booleans, not three states;
  it would reopen every settled `session != nil` site.
- **Making a free session count toward the one-session-per-space gate.** The gate
  exists so two agents do not race on one claim in one tree. A free session writes
  no claim. Counting it would block the ordinary "spawn a ticket session while a
  chat tab is open" case for nothing.
- **Riding the opener with an inlined payload.** Refuted by `typeOpener`'s own
  reason for existing, and by `preferences.md` being unbounded.
- **Numbering duplicate free-session tabs.** Machinery for a cosmetic gain; ad-hoc
  shells set the precedent and duplicate freely.
- **Making the split button's body launch the last-used agent.** The static label
  "new shell" would then not describe what a click does, and it would put a
  behavioural dependency on `lastAgent` that the free-session path otherwise has
  none of.
- **Giving the empty shell a conventions pointer.** Speculative injection into a
  tree on the chance an agent appears later; the map already refused the whole
  category.
- **Deciding the payload's home here.** It governs both payload kinds identically
  and belongs to one ticket, not to whichever of the two happens to be answered
  first.

### Surfaced doubts

- **The in-repo `.chartr/run/` contradiction is unowned and now confirmed
  redundant** (above). It needs a ticket. If it stays unowned, implementation will
  make the choice silently by copying whatever `writeSessionPayload` does.
- **Today's on-ramp path has a live bug that this ticket's fix incidentally
  closes:** an on-ramp tab on an unmanifested agent reads idle for its whole life.
  Recorded here because someone may hit it before this map ships, and because it
  is the evidence for the `launchedAgent` field rather than an argument invented
  for it.
- **Nothing on this map says what the free-session tab's *close* semantics are
  when its agent is mid-turn.** It drops from the model on exit like an ad-hoc
  shell (`onExit`, `pinOnDeath` false), which is almost certainly right — but
  `manager.go:524` already records the accepted cost that a long on-ramp run
  ending on its own notifies nobody. That cost transfers to free sessions
  unexamined; flagging rather than deciding, since it is the notification map's
  ground.
- **The chartr payload handed to this session was truncated at 24 KB**, cut off
  mid-sentence inside blocker 02's sources block (`.chartr/run/s09f416f39a0f/payload.md`,
  248 lines, ends at an unclosed fence). The blockers were re-read off disk
  instead. This is a chartr bug, not a map question, but a session that did not
  notice would have grilled against half a premise.

