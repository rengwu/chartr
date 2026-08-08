# One global settings route shows what the layers resolve, and edits only role bindings into the user layer

> **Superseded in full by the `agent-selection` effort. Nothing here is
> operative.** What replaced it is small enough to need no ADR of its own.

**What was decided:** that chartr's three config layers resolved invisibly, and
the fix was one global settings route rendering every effective value beside the
layer it came from and the file that layer lives in — legibility first, and
**never a second config store**. It edited exactly one thing, role bindings, and
only into the user layer.

**Why it is gone:** the surface was built on per-field provenance across three
layers, and those layers no longer exist (see
[0009](0009-config-layers-execution-vs-content.md), superseded alongside it).
Role bindings and the committed execution layer are deleted; execution is chosen
per spawn from the operator's agent library. The read-of-bindings, the inline
binding editor (`config.SetUserBinding`) and the layer/provenance badges are all
removed. Only the open-this-file action survives, now resolving skill
directories and the agent-library file.

**Where the live surface is:** the settings route today is the agent library,
the registered skill sources in resolution order, the four role→**skill**
bindings 0017 introduced (not the role→*agent* bindings this record edited,
which are gone), and the paths of the files behind all of it —
read-value-plus-open-file, holding the original "never a second config store"
constraint. See `CONTEXT.md` → *Settings surface*.

*Original record in `git log -p -- docs/adr/0014-the-effective-config-surface.md`.*
