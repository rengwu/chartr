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
