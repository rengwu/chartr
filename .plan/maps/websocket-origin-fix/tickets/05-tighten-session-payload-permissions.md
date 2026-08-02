---
type: task
---

# Session payloads are world-readable

## Question

Session payloads and their archived copies are written `0644` under `0755`
directories:

```go
// internal/server/spawn.go
os.MkdirAll(filepath.Join(runDir, sessionID), 0o755)
os.WriteFile(filepath.Join(runDir, ".gitignore"), []byte("*\n"), 0o644)
os.WriteFile(path, []byte(markdown), 0o644)
// …and the archive copy
os.MkdirAll(dir, 0o755)
os.WriteFile(filepath.Join(dir, "payload.md"), []byte(markdown), 0o644)
```

(`spawn.go:381-403`; the registry save at `internal/server/spaces.go:482-486` and
the halt path at `internal/server/halt.go:263` write the same way.)

A payload is the composed prompt for a session: ticket text, map notes, and
whatever skill content was assembled into it. On a single-user machine this is
uninteresting. On a shared box — a build host, a dev server, a machine with more
than one login — every local user can read what every session was asked to do, and
the containing directories are traversable.

This is a **local** exposure only. It is not reachable by any of the remote paths in
this map, needs an existing account on the machine, and is the least urgent thing
here. It is also cheap to fix and there is no argument for the current mode.

**The fix.** Write session payloads `0600` under `0700` directories. Do the same
for the registry file and anything else under the data directory that records what
the operator is working on. Two things to get right rather than blanket-changing
every mode in the tree:

- **Do not tighten what is meant to be shared.** Files written into the operator's
  *repository* — the `.gitignore` markers, anything under `.plan/` that a human
  edits or git tracks — should keep normal modes. The target is chartr's own
  session and data artefacts, not the repo it is pointed at. Walk the call sites
  and say in the answer which ones you changed and which you deliberately left.
- **`os.WriteFile` does not chmod an existing file.** A payload path that already
  exists keeps its old mode, and `MkdirAll` leaves an existing directory alone.
  Decide whether that matters here (it mostly does not — session IDs are fresh) and
  note the conclusion rather than assuming it.

Umask already narrows this on many machines, which is why it has gone unnoticed;
that is not a reason to rely on it.

Tests lead: create a session payload through the normal path and assert the file
mode is `0600` and its directory `0700`. Guard the test for non-POSIX platforms if
the suite runs on Windows.

Done when: session payloads and chartr's own data files are written `0600`/`0700`;
files written into the operator's repository are unchanged and the answer says
which those are; the test above exists; and `go vet ./...` / `go test ./...` pass.
