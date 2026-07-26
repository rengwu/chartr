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

## Answer

All three behaviours live in one new tag-free file, `cmd/webview/bundle.go`,
beside the lock and for the same reason — it compiles and tests at
`CGO_ENABLED=0`, and every function in it takes its inputs as arguments, so
`bundle_test.go` drives them with constructed paths and never touches a real
bundle, display or home directory.

- **`isBundled(exePath)`** is a pure path predicate: a `MacOS` directory, a
  `Contents` above it, a `.app` extension above that. No stat, no environment, no
  platform call — the shape is what the bundle format guarantees, and it is what
  ticket 02 will assemble. The table covers the near-misses that matter: a
  `MacOS` directory with no `Contents` over it, and the executable parked
  elsewhere inside a real bundle.
- **`runtimeRoot(explicit, exePath, configRoot)`** returns `explicit` when there
  is one, `""` (the working directory — the terminal launch, unchanged) when the
  path is not a bundle, and `configRoot` when it is. `configRoot` is passed in
  rather than resolved, which is what keeps the function testable *and* what
  keeps there being one definition of the home-anchored root: **`server`'s
  `userConfigRoot` was exported as `server.ConfigRoot`** and `main` calls it. The
  shell needs the answer before it can construct a `Server`, and duplicating the
  `XDG_CONFIG_HOME`-then-home logic in the shell is exactly the drift that would
  fork an operator's state. Its no-home fallback comes along for free: no home →
  empty config root → the working directory, degrading the way the config root
  itself already does.
- **`parseFlags(args, bundled)`** replaces the package-level `flag` calls with a
  `FlagSet` on `ContinueOnError`. A terminal launch is unchanged — usage printed,
  exit 2, verified against the built binary. A bundled launch keeps whatever
  parsed before the offending argument and returns no error, so an injected
  `-psn_0_…` is ignored rather than fatal; its usage output goes to `io.Discard`,
  since there is no terminal reading it and the argument was never the
  operator's.

`main` reads `os.Executable` once, and everything below it is a function of that
string; an unreadable path is simply "not a bundle", which is the safe half of
the guess.

**Both confirmations held.** The registry resolves through the config root
(`registry.Load(opts.ConfigDir)`) — the simulated bundled launch came up with
this operator's real registered spaces in the sidebar, no work needed. Login-shell
`PATH` hydration still runs first thing in `run()`, before anything can resolve a
binary.

**Verified on a real Mac.** No bundle exists yet, so the `make webview` binary
was copied to `…/chartr.app/Contents/MacOS/chartr` and launched with `/` as its
working directory *and* a `-psn_0_123456` argument. It started, wrote its lock to
`~/.config/chartr/.chartr/shell.lock` (nothing was created at `/.chartr`), served
`/api/health` 200, and drew a cockpit window. `SIGTERM` closed the window and
released the lock.

**One deviation, raised not quiet, and outside this ticket's scope:**
`go test ./...` was already red on `main` — commit `7ea529a` lowered the terminal
default font size to 13 and left `terminal.scaffold.toml` at 14, breaking
`TestScaffoldMatchesDefault`. The gate cannot be met without it, so the scaffold
moved to 13: a one-line fix to a commented template, touching nothing this ticket
owns. No ADR was touched — ticket 02 owns the one new ADR.
