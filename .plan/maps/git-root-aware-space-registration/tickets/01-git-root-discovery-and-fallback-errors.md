---
type: grilling
blocked_by: []
claimed_by: s61b5dbba8898
claimed_at: 2026-08-05T18:55:26Z
---

# Git root discovery and fallback errors

## Question

Define the repository setup contract for a Space path.

The current registration code checks only `Space path/.git`. It must instead find
the Git root from the selected path and must initialize Git only when no repository
exists. Settle the exact behavior for:

- a normal Git worktree;
- a linked worktree whose `.git` is a file;
- a nested repository or submodule;
- a path that is not inside a repository;
- a missing Git command or another Git failure; and
- a path that cannot be registered.

The contract must state which root is selected, which failures allow `git init`,
what path is used when initialization succeeds, and what the registration response
reports.

## Done when

The answer gives one root discovery and fallback rule. It distinguishes no
repository from another Git error. It states the expected result for every listed
path type and names the acceptance cases that prove the rule.

## Answer

### Decision

Use Git to discover the root from the normalized absolute Space path:

```text
git -C <Space path> rev-parse --show-toplevel
```

When this command succeeds, its output is the Git root. Normalize the output to
an absolute, clean path. The command handles both forms of `.git`: a directory
in a normal worktree and a file in a linked worktree. Registration must not use
`Space path/.git` as the repository test.

The discovery operation has three outcomes:

1. **Git root found.** Use the reported root. Do not run `git init`.
2. **No repository found.** Allow fallback only when Git gives the explicit
   no-repository result for the Space path. An equivalent typed result is also
   valid. Run `git init` with the Space path as its working directory. After a
   successful init, the Git root is the Space path.
3. **Git failure.** A missing Git command, a permission failure, a malformed or
   unusable `.git`, a bare repository, an invalid worktree, or any other result
   that is not the explicit no-repository result is an error. Do not run
   `git init`.

The implementation must not use `if probeError != nil { git init }`. In
particular, an existing but broken `.git` entry must not be replaced by a new
repository. The no-repository classification must distinguish that case from a
directory with no repository in its ancestor chain.

### Required path results

| Selected path | Git root | `git init` | Registration result |
| --- | --- | --- | --- |
| Normal worktree root or subdirectory | The containing worktree root | No | Success; `gitInited` is `false` |
| Linked worktree root or subdirectory | The linked worktree root, not the common Git directory | No | Success; `gitInited` is `false` |
| Nested repository or submodule | The innermost repository root, not the outer repository or superproject | No | Success; `gitInited` is `false` |
| Valid directory outside every repository | The selected Space path after init | Yes, in the Space path | Success; `gitInited` is `true` |
| Missing Git command or another Git failure | None is selected | No | Failure; no new or updated registration |
| Missing path, inaccessible path, or a path that is not a directory | None is selected | No | Failure; no new or updated registration |

For a nested repository, the parent repository remains the parent of the
selected Space path only when the path is outside the nested repository. A
submodule is treated as its own repository because Git reports the submodule
worktree as the root. This accepts separate Git state for the submodule.

### Registration response

On success, `POST /api/spaces` returns HTTP 200 with the existing response
shape:

```json
{"id":"...","path":"<absolute Space path>","gitInited":false}
```

`gitInited` is `true` only when this registration ran a successful `git init`.
The response always reports the selected Space path. It does not report the
internal Git root.

On failure, the endpoint returns an error JSON body and no success fields. An
invalid path is a client error (HTTP 400). A missing Git command, another Git
failure, a failed `git init`, or a registry persistence failure is a server
error (HTTP 500). The error text must identify the failed operation and preserve
Git output where it helps the operator correct the problem. Discovery and init
failures must not create or update a registry entry.

Path validation happens before Git discovery, so an invalid path never causes a
Git action. Registration is not atomic with repository initialization: if
`git init` succeeds but saving the registry entry fails, the response is still
an error and chartr does not delete the new `.git` directory automatically.
This avoids a destructive cleanup decision and must remain visible to the
operator.

### Rejected alternatives

- **Check only `Space path/.git`.** Rejected because a subdirectory of a
  repository would cause an unwanted nested `git init`. It also fails to find
  the root above the selected path.
- **Walk for a `.git` entry without asking Git.** Rejected because it accepts a
  stale or broken marker and duplicates Git's worktree rules. It cannot safely
  decide whether a bare or malformed repository is usable.
- **Treat every discovery error as no repository.** Rejected because it can
  turn a missing Git command, a permission problem, or a damaged repository into
  an unexpected write. The safe cost is to refuse registration until the Git
  problem is fixed.

### Acceptance cases

The later implementation must prove these cases at the registration boundary:

- Register a normal repository subdirectory. No `.git` appears in the selected
  directory; the response keeps that directory as `path` and reports
  `gitInited: false`.
- Register a linked worktree and a subdirectory in it. The `.git` file remains
  valid, no init runs, and the selected linked worktree root is used for a Git
  action.
- Register a nested repository and a submodule path. Neither case initializes
  the outer repository, and a Git action uses the innermost root.
- Register a plain directory. `.git` is created only in the selected directory,
  the new root is that directory, and the response reports `gitInited: true`.
- Run registration with Git unavailable. Run it with an unusable `.git` or a
  bare repository. Each case returns an error, creates no new repository, and
  creates no new registry entry.
- Register a missing path and a regular file. Each case fails before any Git
  command and creates no `.git` entry.
- Assert the success JSON fields and the error JSON shape. The Git root is not
  exposed in the response.

### Revisit trigger

Reopen this decision if a new Git worktree type cannot be represented by Git's
root discovery, if chartr must support bare repositories, if an operator needs
the superproject root instead of the innermost root, or if registration must
roll back a successful `git init` when registry persistence fails. A new Git
action that cannot consume this internal root also reopens the decision.
