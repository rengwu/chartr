# Build fresh, reusing wayfinder-maps' model layer

wayfinder-maps is a solid read-only viewer, but its architecture is deliberately stateless — no build step, vanilla JS inlined by Go, re-read-on-refresh, no state file — and the harness needs the opposite: live PTYs, background sessions, and pushed state. Rather than bolt a stateful runtime onto a codebase whose charm is having none, wayfinder-harness is a separate project that lifts wayfinder-maps' model layer (`Load`, `Layers`, `Frontier`, derived `Status`, `lint`) and its star-map renderer, copying freely wherever copying elevates the result.

## Considered Options

- **Extend wayfinder-maps in place** — one codebase, but its statelessness is load-bearing for the star-map's spatial-memory design, and a multiplexer would corrupt it.
- **Two tools sharing one model library** — essentially what this is, minus a constraint that the viewer stay pristine.
