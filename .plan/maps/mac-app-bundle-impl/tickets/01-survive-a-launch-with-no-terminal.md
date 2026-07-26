---
type: task
blocked_by: []
---

# Survive a launch with no terminal

## Question

Make the webview shell start correctly when it is launched with no terminal
attached — the launch a bundle will perform, and the one the shell has never been
built for. Today a Finder launch inherits `/` as its working directory, derives
its runtime root from that, and dies claiming the single-instance lock before a
window exists, writing the reason to a stream nobody will read. This ticket is
the reason the packaging tickets come after it: every one of them would otherwise
be verified against an app that cannot start.

Three behaviours, all in the shell package's **tag-free half** — beside the
single-instance lock, which is tag-free for exactly this reason and is the prior
art for the shape:

- **Bundle detection.** A path predicate over the shell's own executable path:
  is it inside a `Contents/MacOS` directory whose grandparent carries the `.app`
  extension? It **takes the path as an argument** so a test drives it with a
  constructed path and never needs a real bundle.
- **Runtime-root resolution.** When bundled and the operator passed no explicit
  runtime root, resolve to the **same home-anchored root the config root already
  resolves to** — not to an Apple-conventional application-support directory, and
  not to the working directory. The spec's reasoning is binding here: two roots
  chosen by how the app was started would give an operator two registries, two
  session archives and two locks, and the split would be invisible until it
  confused them. An explicitly passed root always wins.
- **Argument tolerance.** When bundled, an unrecognised argument is ignored
  rather than fatal, because the window server can inject its own. A terminal
  launch's parsing behaviour is unchanged.

Confirm rather than change two things the spec expects to already work: the
registry resolves through the config root, so a bundled launch should find the
operator's spaces with no work; and login-shell `PATH` hydration already runs at
startup, added precisely because a Finder launch inherits a `PATH` carrying
neither Homebrew nor the operator's own bin directory. Say in the answer whether
both held.

Tests lead here: they are table tests over path resolution in the cgo-free suite,
and they are the whole reason this logic is tag-free. Write them against the
lock test file's shape.

## Done when

`go test ./...` passes at `CGO_ENABLED=0` with new table tests covering: a path
inside a bundle detected as bundled; a plain path not; an explicit runtime root
winning over the bundled default; and the bundled default landing on the
home-anchored root rather than the working directory. The shell binary built with
`make webview` and launched with `/` as its working directory claims its lock and
opens a window instead of exiting — verified by running it that way on a real
Mac. `go vet ./...`, the frontend `check` / `build` / `vitest`, and the no-amber
check are green.
