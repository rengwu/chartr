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

## Answer

Shipped, frontend-only — find is transient UI, not config, so nothing crosses Seam
1. The search addon is lazily imported and bundled like the others; `TerminalFind.svelte`
is the widget — `Input` + `Button` primitives, Phosphor icons, `bg-popover` tokens,
no raw colour — floating at the island's top-right. It is pure chrome: it owns only
its input value and hands every action back to `Terminal.svelte`, which drives the
addon and feeds the live count/index back from `onDidChangeResults` (ADR 0010, never
inside the renderer). Cmd+F (meta only — Ctrl+F stays readline's) opens and focuses
it; Enter/Shift+Enter cycle; the "Aa" toggle re-runs case-sensitive; Esc closes and
clears decorations. The match-highlight colours resolve from design tokens at the
`tokens.ts` seam (`terminalSearchDecorations`), so the highlight tracks the theme
with no inlined colour.

Driven live end to end: Cmd+F opened the focused widget; typing `hello` over four
lines showed `1/4` with every match highlighted and the active one distinct; next
cycled `1/4 → 3/4`; the case toggle dropped it to `2/2` (only the lowercase hits);
Esc closed it and cleared the highlights. The drive caught the same class of bug as
ticket 05 — the addon's match **decorations** call `registerDecoration`, itself
xterm *proposed* API — so `allowProposedApi` is now on for every terminal (find is
universal), not gated to ligatures. A follow-up hardening moved Esc/Cmd+F onto the
widget container so Esc closes from a focused button too, not only the input.

Gist: a bundled search addon behind a token-and-primitive find widget hosted at the
island seam, with `allowProposedApi` promoted to universal for the match decorations.
