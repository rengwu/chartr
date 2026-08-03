---
type: task
claimed_by: s22fa74c60137
claimed_at: 2026-08-03T09:04:13Z
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

## Answer

**chartr's own artefacts are now owner-only; the operator's repository is
untouched.** Commit `f536d10`.

The rule lives in one place — `internal/server/filemodes.go` — as
`ownerFileMode`/`ownerDirMode` plus `writeOwnerFile`, with the reason and the
deliberate exclusions written above it, so the next person to add a write has
the policy in front of them rather than a literal to copy.

**Walked, and what I did with each.**

Tightened, all of it chartr's own state:

- `writeSessionPayload` (`spawn.go`) — the per-session directory is now `0700`
  and `payload.md` `0600`. The single `MkdirAll(runDir/sessionID)` became two
  calls so the run directory and the session directory can differ.
- `archivePayload` (`spawn.go`) — `<data>/sessions/<id>` `0700`, `payload.md`
  `0600`. `sessions/` itself lands `0700` too, which is what stops another login
  even listing which sessions ran.
- `writeFileAtomic` (`spaces.go`) — the config root `0700`, the temp file
  `0600`. Its callers are the agent library and the terminal/notify config,
  never a repository (its doc comment claimed otherwise — it described a
  committed `.chartr/` config that no longer exists; corrected in place).
  `user.toml` carries an agent's `env`, the one place in chartr's config an
  operator may keep a secret.
- `registry.saveLocked` — `spaces.toml` `0600` under a `0700` root. It is the
  absolute path of every repository the operator works in.
- `server.New` — creates the config root `0700` before anything writes into it.
  Without this the tightened modes above were half-real: `prompt.Materialize`
  runs at every startup and created the root at `0755` on a fresh install, so
  the root's mode was whichever writer got there first. An existing root is left
  alone (`MkdirAll` never chmods), which also keeps the fallback case honest —
  `ConfigRoot` can return the runtime root when there is no home directory, and
  that is not chartr's directory to re-permission.
- `cmd/webview/lock.go` — `shell.lock` `0600`. It names the loopback address the
  running shell is serving on. Its directory keeps `0755`: the data root may be
  a repository checkout.

Left alone, deliberately — every one of these is a file in the operator's
repository, belonging to git and to them:

- `.chartr/run/.gitignore` and the run directory that holds it (`spawn.go`, and
  the same marker re-asserted on resume at `halt.go:263`). Git reads the marker,
  and a checkout shared with another login has to keep working.
- The ticket files a claim rewrites under `.plan/` (`claim.go:62,162`) —
  git-tracked and human-edited.
- The tracker adapter installed under `docs/agents/`
  (`internal/tracker/adapter.go:109`) — written for the operator to review and
  commit.
- `prompt.Materialize`'s `builtin-skills/` (`prompt.go:432-445`). This one is
  under the config root rather than a repository, so it is the judgement call:
  its content is the skill library embedded in a public binary, identical on
  every machine and says nothing about the operator, and the point of
  materializing it is that they read and edit it. It sits inside a `0700` root
  now, so it is not exposed either way.
- Test fixtures (`internal/chartrtest`).

**`os.WriteFile` does not chmod an existing file — where that mattered.** For
payloads it does not: session IDs are 48 random bits, and the one path that
rewrites a payload (`ensureSessionPayload`, restoring from the archive) only runs
when the file is absent. It *does* matter for the two atomic writers, and not
theoretically: a crash mid-save leaves `spaces.toml.tmp` behind, the next save
writes it without creating it, and the rename would carry that stale `0644` onto
the real file. Both chmod their temp file explicitly, as does the lock's
stale-takeover path. `TestAnOlderWorldReadableRegistryIsRepairedOnSave` pins the
happy consequence: an install upgrading from a world-readable `spaces.toml` is
repaired by its next save, because the rename lands the temp file's mode.

**Not migrated: payloads already on disk.** Session directories and archives
written by an earlier build keep their `0755`/`0644`. Only new writes are tight.
Walking the data root to chmod it is a migration, not this ticket, and I did not
want a security fix quietly rewriting modes across a tree it did not create — but
it does mean that on a shared machine every *existing* payload stays readable
until it is deleted. Flagging it rather than deciding it.

**Tests** — `internal/server/filemode_test.go`, at the process boundary like the
rest: spawn a session through the normal path (registered space, frontier ticket,
stub agent) and stat what it wrote. Five tests cover the payload and its archive,
the registry, the agent library, the config root, and the repair-on-save case;
`TestTheRunMarkerStaysARepositoryFile` pins the line that is *not* crossed, so a
later blanket chmod fails a test rather than passing one. All are skipped on
Windows.

Verified as regressions: with the source changes stashed, every assertion about a
tightened file fails reporting `0644`/`0755`, and the run-marker test passes both
before and after — which is what says it is pinning behaviour rather than
describing the fix.

The untightened assertions compare against a probe write rather than a literal
`0644`, because the umask has the last word on a create and a machine testing
under a tight one would otherwise fail an assertion about chartr's own choice.

`go vet ./...` and `go test ./...` pass, and again under `-tags chartrdev`. No
frontend change, so nothing to run there.

**One flag for a human**, same as tickets 01–04: the map's disclosure note covers
this commit. It is on `main`, unpushed; nothing here pushes.
