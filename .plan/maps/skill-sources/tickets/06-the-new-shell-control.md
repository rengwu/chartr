---
type: prototype
blocked_by: [02]
assets: [new-shell-sketch.png]
claimed_by: sc10c52b60df9
claimed_at: 2026-08-06T09:04:17Z
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
