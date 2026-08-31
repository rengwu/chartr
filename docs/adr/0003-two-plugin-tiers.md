# 0003 — Two plugin tiers

## Decision

A plugin is `kind = "native"` (a `cdylib` whose GPUI view is mounted directly in
zeddy's element tree) or `kind = "web"` (a manifest and an entry document in a
webview). Both contribute the same thing: a pane. Nothing above
`zeddy-plugin-host` asks which tier a pane came from.

## Why two

The brief was "plugins anyone can author" *and* "a star map plugin". Those are
different requirements and one runtime cannot honestly serve both.

- Anyone can author a web plugin: a manifest and an HTML file, no toolchain, no
  ABI, and a sandbox. Its pane is composited rather than painted, so it runs a
  frame behind the terminal next to it. For a clock, invisible. For a star map
  being panned, not.
- A native plugin is on zeddy's own frame path — the same scrolling, resizing,
  focus, input, and painting as a built-in view, because it *is* an ordinary
  view. The cost is that authoring means Rust against a pinned GPUI ABI, and
  installing one is installing native code.

Shipping only the native tier would have meant "authorable by anyone" was not
true. Shipping only the web tier would have meant "a star map" was not true.

## What was rejected

**WASM components with a host-drawn UI contract.** Language-agnostic and
sandboxed, and it needs a display-list or UI-RPC layer between the plugin and
the renderer. That layer is the thing that makes a plugin pane feel unlike the
rest of the window, and it would have to exist even for plugins that do not need
a sandbox. Two honest tiers beat one dishonest one.

## Consequences

- `native_abi` in the manifest must equal zeddy's **exactly**. There is no
  compatibility range and there is not going to be one: a mismatch is a vtable
  from a different compilation, and the failure mode is a crash rather than a
  wrong answer.
- Native libraries are **never unloaded**. A plugin's views hold vtables that
  live in its library, and there is no moment at which zeddy reliably knows the
  last one is gone. `zeddy-plugin-host` leaks the `Library` on purpose; a reload
  brings a new generation up and swaps it in. Memory is the cost, and it is the
  cheap side of that trade.
- Plugin data lives outside the plugin directory and survives replacement,
  because a reload replaces the directory.
