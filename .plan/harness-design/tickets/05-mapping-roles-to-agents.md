---
type: grilling
blocked_by: [01]
---

# Mapping roles to agents, and keeping the reviewer honest

## Question

Config layers: workspace defaults are committed and shared with everyone in the repo; user preferences override them locally and are never committed. Design that surface, starting with **role → agent** resolution.

Settle the schema — what a role binds to (a bare command? an agent name plus a model? arbitrary argv? an adapter name plus options?), where each layer's file lives, how the two merge, and what is even *legal* to commit into a repo other people work in.

The load-bearing part is **heterogeneity**. A model reviewing its own work is marking its own homework, and the only real mitigation is that `implement` and `review` resolve to different models. Decide how hard the harness pushes: silently allow, warn loudly, or refuse outright. Note the limit of what it can actually know — the harness cannot verify that two commands are different models, only that they differ *as configured*, so this is a default worth defending rather than an invariant it can enforce.

Also settle:

- What happens when a configured agent is **absent from the machine** — a committed default naming a CLI the operator has never installed is the ordinary case, not the exotic one.
- Whether **autopilot** (both reviews disabled, non-default, disclaimed) may be turned on by a *committed* config for everyone who clones the repo, or is strictly a local choice. Committing "no human reviews this project's code" is a very different act from choosing it for yourself.

## Answer

Role→agent resolution is **structured config in two layers**, resolving **user-over-workspace** — the deliberate mirror of [ticket 04](./04-the-prompt-library-the-harness-injects.md)'s space-wins prompts. The reconciling rule: **content the project ships wins; execution choices the operator makes win.** A committed *prompt* is shared project content (space wins); a committed *binding* names a concrete CLI and model — an execution/environment fact that must yield to the operator's actual machine and wallet (user wins). Recorded as [ADR 0009](../../../docs/adr/0009-config-layers-execution-vs-content.md).

**The schema.** A role binds to **`{adapter, model, args?}`**. The adapter is the named thing from [ticket 01](./01-the-agent-adapter-contract.md), so the harness stays able to *reason* about a binding — compare models for heterogeneity, probe the binary for presence, know which token-stream to parse. `args` is a deliberate escape hatch for flags the adapter doesn't model; **using it knowingly forfeits that introspection** on that binding. Rejected: bare command strings and raw argv arrays — both blind the harness to the model and leak argv assembly out of the adapter (ADR 0002) into config, and a free-form command is also the easy path for smuggling machine-specific junk into a shared repo.

**Layers, homes, merge.** Three layers: shipped **built-in defaults** ‹ **committed workspace** ‹ **local user**, in TOML for hand-editing (the hackable preference). The committed layer is the second tenant of the harness config ADR 0007 already put in the space's repo (map-kind is the first); role bindings are **space-global** — one `[roles]` block, not per-map. The local layer is uncommitted, under `~/.config/wayfinder-harness/`, keyed by space. Merge is **field-level (deep)**: a user override may set just `model` and inherit the committed `adapter`/`args`. Because that inheritance can surprise, the harness always renders the **effective resolved binding**, so what will actually run is visible rather than guessed.

**What is legal to commit.** Adapters, models, and portable `args` — yes. Machine-specific/absolute paths — no; that is exactly what the user layer is for. And **autopilot has no committed meaning at all**: a committed autopilot flag is ignored with a warning. Committing "no human reviews this code" for everyone who clones is a categorically different act from choosing it for your own machine, and the standing "cockpit, not autopilot" preference refuses it — autopilot is **strictly a local-user setting**.

**Heterogeneity — the load-bearing part.** `implement` and `review` on different models is the only real mitigation for marking-your-own-homework, but the harness **cannot enforce it as an invariant** and does not pretend to: two model strings can alias one backend, and the `args` hatch can swap the effective model, so a config-equality check would give false confidence. So the harness **never guards at config time — it always allows, and surfaces at the one gate.** [Ticket 10](./10-the-human-review-hub.md)'s review brief carries an **observed**-model heterogeneity line (what actually ran, from the adapter's `observe` — stronger than what was merely configured), and the human weighs the verdict there. No un-overridable rule; judgment lives at the human gate.

**Absent agent — the ordinary case.** A committed default naming a CLI the operator never installed is normal, and user-over-workspace layering makes the fix and the mechanism the same thing: a local override. **At spawn**, the harness hard-blocks *that one role* with a specific message — *"review is bound to codex (committed default), which isn't on your PATH — install it, or set a local override"* — never a silent failure and never a whole-map error (other roles may be fine). **Before spawn**, absence is surfaced by a doctor check (ties to [ticket 13](./13-shipping-the-harness-cross-platform.md)) and a badge at classify-time in the space registry ([ticket 11](./11-the-space-registry-and-switching.md)), so it is seen up front, not discovered mid-drive.
