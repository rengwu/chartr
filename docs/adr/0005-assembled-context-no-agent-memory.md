# Context is assembled per spawn; agents accumulate no memory

A freshly spawned session is handed a **context bundle** built at launch out of artifacts that already exist — the map body, its ticket, the `## Answer`s of that ticket's blockers, and the glossary — and zooms on demand for anything else. Nothing is persisted between sessions; there is no store agents write learnings into.

The rejected alternative was a memory-system-style accumulating store. It was rejected because a place where agents write unverified "learnings" is exactly where drift and marking-your-own-homework re-enter through the back door: a claim no human ever gated becomes gospel for every session after it. The map and its resolved answers already *are* the shared and truthful memory, and a session's whole job is to add one blessed answer to it.
