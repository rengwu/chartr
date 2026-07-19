---
type: task
blocked_by: [03]
---

# Classify a map's kind

## Question

Kind is declared, never inferred (ADR 0007). A discovered map with no declaration is inert — rendered and readable in the snapshot, but offering no session actions — until classified. Classification is one action over HTTP: the convention heuristics (`-impl` suffix, all-`task` tickets) pre-fill the guess for a one-keystroke confirm, and the confirmed kind is written to the committed workspace config, keyed by map slug. A renamed map directory dangles its entry and resolves into unclassified-and-inert, never an error. The map format itself is untouched — a vanilla wayfinder tool reads the same map unchanged.

Done when: process-boundary tests assert an undeclared map's snapshot offers no session actions; classifying writes the committed config and the actions appear; the pre-filled guess matches the conventions on fixtures for both kinds; renaming a classified map's directory returns it to unclassified-and-inert; the inline confirm renders in the sidebar.
