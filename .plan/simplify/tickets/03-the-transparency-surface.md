---
type: grilling
blocked_by: [01, 02]
---

# The transparency surface

## Question

The operator's complaint: the harness is opaque. Role bindings resolve through three layers nobody can see; prompts materialise into a gitignored directory; map kinds live in a committed TOML file; small preferences have no home at all. The settled direction is **legibility first**: an effective-config surface showing every resolved value with its provenance layer and an open-in-editor hatch, with real editing later and only for high-churn settings. This ticket designs that surface — and takes stock of what "settings" even means after tickets 01 and 02 shrink the system.

This is the first ticket in the repo's history that designs a *screen* whose whole job is explaining the system. The pressure: a settings pane is where good tools go to grow a second config system. The layering rules (ADR 0009) are subtle — workspace wins content, user wins execution, autopilot (dying in ticket 01) was user-only — and a UI that writes TOML can silently violate the very provenance it displays. Legibility-first is a discipline, not a smaller feature.

Settle:

- **What the surface shows.** The full inventory after the cut: role bindings with per-field provenance (built-in/workspace/user) and PATH-probe status, map kinds, resolved skills with their layer, the prompt/payload assembly for a prospective spawn (the preview already exists), ad-hoc preferences. What is missing from that list, and what deliberately is not shown?
- **Where it lives in the cockpit.** A settings pane per space, a global one, or both? The cockpit has no router beyond a hash deep-link and no settings screen at all — does this earn a first-class surface, and what is the navigation model?
- **The read path.** `config.Resolve` already computes provenance — is exposing it a handler away, or does the resolution need restructuring to be reportable? Skills and map kinds need the same treatment; is there one "effective state" endpoint or several?
- **The edit boundary.** Which settings are high-churn enough to earn real editing later (role bindings are the named candidate), and what is the write discipline when that lands — structured TOML edits preserving comments and operator formatting, like `DeclareMapKind` already does? What must the UI *never* write (committed workspace config from a local UI is the obvious trap)?
- **Explaining the flow.** The operator's deeper complaint is that *how the whole thing works* is invisible. Does the surface carry explanation (inline "why" text, a diagram, links into docs), or is that a docs problem the UI should stay out of? Where does a confused user's first five minutes actually go?
