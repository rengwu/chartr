---
type: task
blocked_by: []
claimed_by: s543787ab2ad4
claimed_at: 2026-08-05T19:19:54Z
---
<!-- triage: ready-for-agent -->

# Register a subdirectory Space with root-wide Git state

## Question

Make the main monorepo path work end to end.

When an operator registers a directory below a normal Git worktree, chartr must
keep that directory as the Space path. It must find and use the Git root without
creating a nested repository. The registration response must keep the selected
absolute path and must not expose the internal Git root.

The Space snapshot must show branch and dirty state from the Git root. Maps,
skills, and other Space files must remain scoped to the selected directory. Two
Space paths in one Git root must remain separate Spaces.

Add the shared internal Git-root resolution seam needed by registration and the
existing branch and dirty Git actions. Keep the normal non-repository behavior
available for the later edge-case ticket.

## Done when

- A repository child directory registers successfully.
- Registration does not create `.git` in the child directory and reports
  `gitInited` as false.
- The response keeps the selected directory as the Space path.
- Branch and dirty state use the Git root, including a dirty change outside the
  selected directory.
- Maps and skills remain scoped to the selected directory.
- Two Spaces in one Git root keep separate paths and identities while showing
  shared branch and dirty state.
- No public registry, API, model, or UI field exposes the Git root.
- Tests prove the behavior through the existing chartr process boundary.
