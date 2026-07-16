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
