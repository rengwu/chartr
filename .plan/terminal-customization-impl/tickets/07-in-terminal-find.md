---
type: task
blocked_by: [01]
---

# In-terminal find (Cmd+F)

## Question

An operator presses Cmd+F in a terminal and finds text in scrollback, with a match
count, next/previous, and match-case.

- Load the search addon (lazily, bundled).
- Host a find widget in the island wrapper (the chrome), built on design-system
  tokens and vendored primitives (ADR 0012) with Phosphor icons — an input, match
  count, next/previous, and a match-case toggle. It sits beside the island at the
  seam and drives the search addon; it never reaches inside the renderer (ADR
  0010). Its open/closed state is transient UI state, not config.
- Cmd+F opens it and focuses the input; Esc closes it.

Done when: Cmd+F opens the find widget, typing highlights matches with a live
count, next/previous cycles them, match-case works, and Esc closes it; the widget
uses tokens + primitives with no raw colour; frontend + Go checks pass.
