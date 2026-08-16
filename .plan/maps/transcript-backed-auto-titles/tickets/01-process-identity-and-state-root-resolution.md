---
type: task
blocked_by: []
undermined_by: []
claimed_by: sb56d83d49cb1
claimed_at: 2026-08-15T18:07:40Z
---

# Foreground process identity and allowlisted state-root resolution

## Question

Transcript binding needs to know which process a tab is actually running and
where that process persists its sessions. Today chartr reads only enough about a
tab's foreground process group to identify the agent by name: the group's argv
tokens and the leader's executable name. That is not enough to match a live tab
to one persisted conversation.

Resolve the rest of the process facts binding depends on: the foreground
process's identity, when it started, its working directory, and the state root it
persists sessions under.

The state root must come from the running agent's own environment. Each adapter
declares an allowlist of the environment variables that select its state root,
and an unset variable resolves to that provider's documented default. chartr must
not scan similarly named directories to find a root, and must not introduce a
configuration surface for registering one — an operator running an alias with a
custom configuration directory gets a working feature without telling chartr
about it.

A process environment is sensitive material. Only allowlisted variables may leave
the reader; the raw environment is discarded immediately and never logged,
serialized, or made reachable from the browser model.

This is the foundation for two later consumers: matching a tab to its persisted
session, and running a title generation under the same account or state root as
the live agent. It ships no operator-visible behavior of its own.

Platform support is macOS and Linux, matching the existing foreground-process
seam. Elsewhere the resolver reports unavailable, and the cross-platform build
must continue to compile without acquiring an implicit Unix dependency.

## Done when

- Given a tab whose foreground holds a known agent, chartr resolves that
  process's identity, start time, working directory, and resolved state root, or
  reports unavailable.
- Each adapter declares its own allowlist of state-root environment variables and
  the documented default that stands when none is set. Adding a provider is a
  data entry, not a change to the resolver.
- Two agents of the same adapter running concurrently with different
  configuration-directory values resolve to distinct state roots, even when they
  share an executable, a working directory and an adapter name.
- Relative and user-relative values are normalized before the root is used, and a
  resolved root is validated before anything reads from it.
- An unreadable or inaccessible process environment resolves to unavailable
  rather than to a guessed root, and surfaces nothing to the operator.
- Variables outside the active adapter's allowlist never leave the process
  reader. No value carrying a raw environment is logged, serialized, or present
  in the browser model.
- On a platform with no foreground-process or process-environment lookup, the
  resolver reports unavailable and the build still compiles.
- Tests cover default roots, two simultaneous custom Claude roots, relative and
  user-relative values after normalization, an inaccessible process environment,
  and the guarantee that non-allowlisted variables never escape the reader.
  Platform-specific tests run on macOS and Linux and skip explicitly elsewhere.

## Answer

Built as two pieces in commit `bc4a9a0` (`feat(proc): foreground process
identity and allowlisted state roots`), deliberately unwired — the ticket ships
no operator-visible behavior, and ticket 02 is the first caller.

- `internal/adapter/stateroot.go` is the adapter package's fourth per-agent data
  table, beside prompt delivery, preflight and headless generation. A row names
  the environment variables that select the state root, in precedence order,
  plus the provider's documented default; `StateRootVars` returns a defensive
  copy of the allowlist, `AllowedEnv` is the chokepoint that narrows a raw
  `KEY=VALUE` environment to it, `StateRoot` resolves and normalizes the root
  (tilde against home, relative against the process's own working directory,
  `~user` refused), and `StateRootEnv` renders the already-normalized root back
  into the single `KEY=VALUE` a same-profile title generation needs. Only
  `claude` (`CLAUDE_CONFIG_DIR`, default `~/.claude`) and `kimi`
  (`KIMI_CODE_HOME`, default `~/.kimi-code`) carry rows — both verified
  first-hand, and preflight's former private copy of the kimi rule now reads the
  same table so the two cannot drift. Codex, OpenCode, Pi and Grok are
  deliberately absent until ticket 04 measures their stores; an adapter without
  a row asks for no variables and resolves to unavailable.
- `internal/proc` resolves a tab's foreground process group (`Foreground`, with
  the detection engine's `Identify` as the identification seam) or a pid chartr
  launched itself (`Launched`) to an `Agent`: adapter, pid and process group,
  start time (stable against pid reuse), working directory, and the validated
  state root. Group selection fails closed: two vendors in one group, two
  indistinguishable same-adapter peers with no leader, or a wrapper that merely
  names the agent all resolve to nothing, while a runtime-launched agent
  (`node /opt/bin/claude`) still resolves. macOS reads identity and start from
  `kern.proc.pid`, the environment from `kern.procargs2` (treating the
  argv-only answer the kernel gives SIP-protected platform binaries as a
  withheld environment, never as "no variables set"), and the working directory
  from `lsof`; Linux reads all three from `/proc`, converting the tick-based
  start time against `/proc/stat`'s btime. Every other platform reports
  `ErrUnsupported` and every resolution is unavailable, with the build
  compiling everywhere (verified for windows, linux and darwin).

The privacy boundary is structural: the raw process environment is discarded
inside the platform reader before it returns, `Agent` carries no environment at
all, and a resolved root is validated as an existing directory before anything
can read from it. An inaccessible or withheld environment resolves to
unavailable rather than to the provider's default, because a process that hides
its environment may be the one running under a custom root.

Excluded: no wiring into the terminal manager or titler (ticket 03's), no
native-title or transcript reading (ticket 02's), and no rows for the four
unmeasured providers (ticket 04's). Verified with `make test`, `make vet`, and
`GOOS=windows`/`GOOS=linux` builds; the pure selection/normalization rules run
on every platform through the host seam, and the platform readers are proved
against real processes on macOS (and Linux via the same test file), skipping by
name with an asserted-unavailable replacement elsewhere.

## Answer

Built two things: a per-adapter state-root data table in `internal/adapter`, and
a process reader in the new `internal/proc` that resolves a tab's foreground
agent to validated facts. Nothing is wired into the cockpit — the package ships
no operator-visible behaviour, as the ticket intends. Ticket 02 is its first
caller.

**The data table** (`internal/adapter/stateroot.go`) is the package's fourth kind
of per-agent knowledge, beside prompt delivery, launch preflight and headless
generation, exactly where the map said new per-adapter knowledge belongs. A row
is a list of environment variables in precedence order plus a home-relative
default. `StateRootVars` hands out the allowlist, `StateRoot` normalizes a value
into an absolute cleaned path, and `AllowedEnv` is the chokepoint that narrows a
`KEY=VALUE` environment to the allowlist. Adding a provider is a row.

Only **claude** (`CLAUDE_CONFIG_DIR`, `~/.claude`) and **kimi**
(`KIMI_CODE_HOME`, `~/.kimi-code`) are listed. Both were already verified
first-hand in this repository; Codex, OpenCode, Pi and Grok are deliberately
absent because ticket 04 owns measuring their stores and their allowlists, and a
guessed root is the expensive failure this specification exists to avoid. An
adapter with no row asks for no variables and resolves to unavailable. On the way
through, `preflight.go`'s own `KIMI_CODE_HOME`-else-`~/.kimi-code` logic now
reads the shared table instead of its own copy, so the two cannot drift.

**The process reader** (`internal/proc`) widens the seam the activity sampler
already had. `Foreground(pgid, identify)` resolves an ad-hoc tab, `Launched(adapter,
pid)` a tab chartr started itself, and both return an `Agent` carrying adapter,
pid, pgid, start time, working directory and a validated state root. `Lookup` is
the platform reader and takes the allowlist as an argument, so the raw
environment is discarded inside it and never reaches a caller. `Agent` carries no
environment at all — only the root the environment selected — and a subprocess
that must run under the same profile gets there through the new
`adapter.StateRootEnv` instead. That is what makes "never logged, serialized, or
in the browser model" structural rather than a rule to remember.

Picking *which* process in a group is the agent needed a rule the ticket did not
settle, so I chose one and tested each clause: candidates are members whose argv
names an agent; two different adapters in one group is ambiguous and yields
nothing; members whose *first* argv token names the agent win over ones that only
mention it (`sh -c claude` is not claude); and among what remains the group
leader wins, so an agent running a copy of itself cannot hand the tab its child's
identity. A runtime-launched agent (`node /opt/bin/claude`) still resolves,
because the exec test only narrows when something passes it.

Everything unreadable is the same outcome: unavailable. That includes an
environment chartr may not read, which never falls through to the documented
default.

**Platforms.** Linux reads `/proc` (stat for pgid and start ticks, `btime` for
the boot clock, `environ`, `cwd`). macOS reads `kern.proc.pid` and
`kern.procargs2` via sysctl and gets the working directory from `lsof`, which is
base-system — libproc would answer all three but is a C library a cgo-free binary
(ADR 0011) cannot reach. Group listing stays `ps`-based and `!windows`, so the
BSDs keep the agent identification they had; only the facts reader is
macOS/Linux. Everywhere else `Lookup` returns `ErrUnsupported` and every
resolution is unavailable. Verified compiling and vetting for linux/amd64,
linux/arm64, darwin/amd64, windows/amd64 and freebsd/amd64.

**One finding worth recording.** macOS answers `kern.procargs2` for a
SIP-protected platform binary with its argv and *nothing after it* — no
environment, and none of the dyld strings that always follow a real one. Read as
"no variables set" that would resolve to the provider's default root and could
bind a tab running under a custom root to the wrong conversation, so the reader
tells the two apart and treats the withheld case as an error. It costs nothing in
practice: agent CLIs are node scripts and Homebrew binaries, never platform
binaries. It did mean the tests had to spawn a *copy* of `sleep` rather than
`/bin/sleep`, which is noted where they do it.

**Tests.** Pure tests drive selection, normalization and validation through an
injected host seam, so they run on every platform including ones with no reader.
Real-process tests (macOS and Linux) cover default roots, two concurrent custom
Claude roots sharing a binary, argv and working directory, relative and
user-relative values after normalization, a root that is not a directory, an
inaccessible environment, a dead process, an empty allowlist, and a sentinel
variable proving nothing outside the allowlist escapes the reader. A
`!darwin && !linux` file skips those by name with a reason and asserts the
unavailable contract instead. The macOS and Linux buffer/file parsers have their
own fixture tests, including the withheld-environment case, an argument shaped
like a variable, and a `/proc` comm containing spaces and parentheses.

`go vet ./...` and the full `make test` suite pass on macOS and, in a container,
on Linux as a non-root user. `make check`'s web half was not run: no web source
was touched.

**Omitted.** No wiring into the terminal manager, no transcript reading, no
discovery or binding — all ticket 02. No Windows support, per the map. The four
unmeasured providers, per ticket 04.
