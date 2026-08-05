---
type: grilling
blocked_by: [01, 02]
claimed_by: s209205d6ec3e
claimed_at: 2026-08-05T19:03:13Z
---

# Claim and release commits from a subdirectory

## Question

Define claim and release commit behavior when a ticket file is under the Space
path but Git commands run at the Git root.

The current behavior stages and commits one ticket file with a path relative to the
Space path. Decide how the path is computed from the Git root, how the existing
single-ticket `--only` behavior is preserved, and what happens when the ticket
path is not inside the Git root or a Git command fails. The contract must protect
unrelated changes in a shared Git root.

Build this answer on:

- [01 — Git root discovery and fallback errors](01-git-root-discovery-and-fallback-errors.md);
- [02 — Space path and shared Git state](02-space-path-and-shared-git-state.md).

## Done when

The answer gives the path and command contract for both claim and release. It
states the failure behavior and proves that a shared Git root does not cause an
unrelated file to enter a chartr commit.

## Answer

### Decision

Use the two-path boundary from Tickets 01 and 02. Discover or pass the internal
Git root for the Space. Keep reading and writing the ticket at its ticket path
under the Space path. Run every Git command with the Git root as its working
directory. Do not persist or expose the Git root.

Before changing the ticket, normalize both paths to absolute, clean paths and
compute:

```text
ticketRel = filepath.Rel(gitRoot, ticketPath)
```

The action must reject the path when this computation fails, returns an absolute
path, returns `.` (the root itself), or returns `..` or a path that starts with
`..` and a path separator. This is a strict descendant check. It must not use a
raw string prefix check. A ticket outside the Git root is an internal error. The
action returns before it reads or edits the ticket, stages a path, or runs a
commit.

If Git root discovery fails during a claim or release, including an explicit
no-repository result, the action fails. Claim and release never run `git init`.
They never use the Space path as a fallback Git root.

### Claim commit

The claim keeps its current file operation and commit message content. It uses
the root-relative ticket path for both Git commands and for the path in the
claim message:

```text
git -C <Git root> add -- <ticketRel>
git -C <Git root> commit --only -m <claim message> -- <ticketRel>
```

Running these commands with `<Git root>` as the process working directory is
equivalent to using `git -C`. `--` ends the option list. `--only` remains
required. It makes the commit contain the current content of only
`<ticketRel>`, including a ticket that was not tracked before the `add`.

Chartr stamps the claim in the ticket file under the Space path, stages that
one root-relative path, and commits it. It launches the session only after the
claim commit succeeds. The existing claim trailers remain unchanged.

### Release commit

The release uses the same `ticketRel` calculation, Git root, and pathspec:

```text
git -C <Git root> add -- <ticketRel>
git -C <Git root> commit --only -m <release message> -- <ticketRel>
```

Chartr removes the claim keys from the ticket under the Space path, stages only
that root-relative ticket path, and commits the release. The release message
names `ticketRel` and keeps the existing session and `Chartr-Write` values.

### Failure behavior

- A path validation failure returns before any ticket or Git change.
- A root discovery failure, a ticket read or write failure, `git add` failure,
  or `git commit` failure returns an error. The error identifies the operation
  and includes Git's combined output when Git produced output.
- A claim error returns a server error and does not write the session payload or
  launch a session. A release error returns a server error and does not report
  success, discard the dead tab, or rebuild the model.
- Chartr does not retry with another path, run `git init`, reset, revert, or
  unstage after an error. If the ticket file was changed before a Git command
  failed, that change and any index state left by Git remain visible for the
  operator to repair. This accepts a visible partial lifecycle write so chartr
  does not overwrite operator or agent edits during automatic cleanup.

### Shared-root safety

For Git root `/repo`, Space path `/repo/one`, and ticket
`/repo/one/.plan/maps/demo/tickets/03-commit.md`, the pathspec is:

```text
one/.plan/maps/demo/tickets/03-commit.md
```

An unrelated file such as `/repo/two/work.go` is not named by either command.
`git add -- <ticketRel>` does not stage it. `git commit --only ... --
<ticketRel>` does not copy it from the index into the commit, even when the
unrelated file was already staged. The unrelated change remains in the working
tree or index after the chartr commit. A Space that shares the root still sees
the root-wide dirty state from Ticket 02; shared dirty state is not shared
commit content.

This guarantee is at file-path level. An edit to the same ticket file is part of
the one path and cannot be separated by `--only`. That remains a concurrent
same-file edit risk.

### Rejected alternatives

- **Use the Space path as the Git base.** Rejected because it breaks the
  two-path contract and makes the path in a subdirectory Space different from
  the path Git actions use at the root.
- **Use an absolute ticket path or allow `..` pathspecs.** Rejected because it
  removes the strict repository boundary and can address a path outside the
  selected Git root.
- **Use `git add -A`, `git commit -a`, or a commit without `--only`.** Rejected
  because a shared Git root can then put another Space's staged or working file
  into a chartr commit.
- **Treat a Git error as no repository and initialize.** Rejected by Ticket 01.
  It can replace or hide a broken repository and is not valid for a later Git
  action.
- **Restore the ticket automatically after a Git failure.** Rejected because
  restoration can overwrite a new operator or agent edit, and the append-only
  contract leaves cleanup to the operator. The accepted cost is a visible
  uncommitted lifecycle change.

### Revisit trigger

Reopen this decision if Git root discovery cannot provide a usable root for a
claim or release, if a lifecycle action must commit more than one file, if
same-ticket edits need field-level isolation, or if chartr must provide atomic
rollback for a failed lifecycle commit.
