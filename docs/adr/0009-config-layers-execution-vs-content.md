# Config layers: workspace commits shared content and defaults; the local user layer wins for execution

> **Superseded in full — both halves. Nothing here is operative policy.**
> Execution by the `agent-selection` effort; content by
> [0017](0017-skills-come-from-registered-sources.md).

**What was decided:** that chartr config lives in two layers — committed
workspace config in the space's repo, never-committed user config under the
operator's home — and that which one wins is deliberately *not* uniform.
Role→agent bindings resolved user-over-workspace, because a committed binding
names a CLI and model that may not exist on this machine; prompts and skills
resolved space-over-user, because those are project content. The reconciling
rule was **content the project ships wins; execution choices the operator makes
win.**

**Why it is gone, in two steps:**

- *Execution half* — role→agent bindings and the committed execution layer were
  deleted. There is no committed execution config left, so the layering question
  has nothing to apply to: an agent is chosen per spawn from the operator's
  global, never-committed library. The safety property this record cared about
  survives in a stronger form — with no committed execution config at all,
  nothing about how an agent runs can arrive by `git pull`.
- *Content half* — [0017](0017-skills-come-from-registered-sources.md) removed
  the skill layers. chartr ships no library and resolves none; skills come from
  an ordered list of operator-registered sources, and `.chartr/skills/` is inert
  (the whole `.chartr/` directory is now gitignored).

**Where the live decisions are:** [0017](0017-skills-come-from-registered-sources.md)
for skills; `internal/config/agents.go` for the agent library, which is the only
execution config there is.

*Original record in `git log -p -- docs/adr/0009-config-layers-execution-vs-content.md`.*
