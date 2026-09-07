# 0003 — Plugin runtimes

## Decision

A plugin is `kind = "web"` (a manifest and entry document in a sandboxed
webview), `kind = "hosted"` (a declarative package activating a reviewed
Chartr-owned surface), or `kind = "native"` (a GPUI module linked into Chartr at
build time). All contribute the same thing: a pane. Nothing above
`zeddy-plugin-host` asks which runtime a pane came from.

## Why three

The brief was "plugins anyone can author" *and* "a star map plugin". Those are
different requirements and one runtime cannot honestly serve both.

- Anyone can author a web plugin: a manifest and an HTML file, no toolchain, no
  ABI, and a sandbox. Its pane is composited rather than painted, so it runs a
  frame behind the terminal next to it. For a clock, invisible. For a star map
  being panned, not.
- A build-time native module is on zeddy's own frame path — the same scrolling, resizing,
  focus, input, and painting as a built-in view, because it *is* an ordinary
  view.
- A hosted package carries no code and activates an explicit surface kept in
  Chartr. It fits first-party integrations that need native windowing or OS
  services but should still be installed separately.

Web packages provide the open extension path. Build-time native modules retain
the full GPUI path, and hosted packages let first-party integrations be optional
without duplicating the GUI runtime.

## What was rejected

**Separately compiled GPUI dylibs.** Real Browser launch testing showed that an
ABI number cannot make Rust `TypeId`, crate globals, entity state, or window
internals identical across two linked copies of GPUI. The result was repeatable
process crashes in theme lookup, async scheduling, and layout. The loader and
installer reject this format.

**WASM components with a host-drawn UI contract.** Language-agnostic and
sandboxed, and it needs a display-list or UI-RPC layer between the plugin and
the renderer. That layer is the thing that makes a plugin pane feel unlike the
rest of the window, and it would have to exist even for plugins that do not need
a sandbox. Explicit runtimes keep that tradeoff visible.

## Consequences

- Plugin installation is validation plus an atomic directory copy. It never
  runs a compiler, package script, or plugin code.
- Hosted surface names form a small allowlist in Chartr; unknown names are
  rejected at discovery.
- Native GPUI modules are part of the Chartr build rather than its installer.
- Plugin data lives outside the plugin directory and survives replacement,
  because a reload replaces the directory.
