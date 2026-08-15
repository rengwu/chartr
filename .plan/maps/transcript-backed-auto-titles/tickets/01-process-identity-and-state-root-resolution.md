---
type: task
blocked_by: []
undermined_by: []
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
