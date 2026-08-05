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
