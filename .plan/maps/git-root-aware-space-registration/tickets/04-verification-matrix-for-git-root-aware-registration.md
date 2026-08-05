---
type: grilling
blocked_by: [01, 02, 03]
claimed_by: s4529e7818655
claimed_at: 2026-08-05T19:08:12Z
---

# Verification matrix for Git-root-aware registration

## Question

Turn the decisions in:

- [01 — Git root discovery and fallback errors](01-git-root-discovery-and-fallback-errors.md);
- [02 — Space path and shared Git state](02-space-path-and-shared-git-state.md); and
- [03 — Claim and release commits from a subdirectory](03-claim-and-release-commits-from-a-subdirectory.md)

into a complete acceptance matrix. Include registration from a repository root,
from a subdirectory, from a non-repository directory, and from the supported
special repository forms. Include branch and dirty state, two Spaces sharing one
Git root, claim and release commits, the `gitInited` response, and the unchanged
non-repository behavior.

## Done when

The answer lists the required test scenarios and their observable results. It
identifies the smallest test seams that prove the contract without testing Git's
own behavior. It also states which old tests must remain unchanged.
