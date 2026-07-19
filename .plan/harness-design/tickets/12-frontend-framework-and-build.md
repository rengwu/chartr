---
type: grilling
blocked_by: []
claimed_by: claude-fable-5
claimed_at: 2026-07-19T14:01:25Z
---

# Frontend framework and build

## Question

xterm.js is an npm dependency, so a build step exists whether or not anyone wanted one, and wayfinder-maps' no-build, vanilla-JS-inlined-by-Go ethos does not survive (ADR 0006). Given that it is already lost, choose the frontend stack deliberately rather than by accident.

- **Framework, or not?** The cockpit is live, stateful and multiplexed — panes, terminals, pushed map state — which is a real step up from a read-only viewer. Svelte is the obvious light choice; plain TypeScript over the build we already need is the obvious austere one. The question is whether this UI's state is complicated enough to buy a framework, or whether the complexity is mostly *terminals*, which xterm.js already owns.
- **The star-map is canvas** and framework-agnostic, and should port largely as-is (ADR 0001). Whatever is chosen must not fight that — and must not tempt anyone into rewriting the renderer as components, which would throw away the one part of the UI that is already designed and proven.
- **How the frontend reaches the Go binary:** embedded in it (`embed.FS`, as wayfinder-maps does) so distribution stays a single file, or served separately? Ticket 13 hangs on the answer.
- **Websocket state sync:** hand-rolled or a library. Terminal streams and map pushes have very different shapes — one is a firehose of bytes, the other is a small derived model — and pretending they are one transport may be a mistake.
