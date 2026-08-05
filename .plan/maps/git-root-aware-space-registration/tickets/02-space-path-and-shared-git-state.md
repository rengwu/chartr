---
type: grilling
blocked_by: [01]
claimed_by: se451ad1e7690
claimed_at: 2026-08-05T18:59:50Z
---

# Space path and shared Git state

## Question

Define the boundary between the Space path and the Git root after
[01 — Git root discovery and fallback errors](01-git-root-discovery-and-fallback-errors.md)
settles root selection.

The selected directory must remain the Space path and the Git root must remain
internal. Decide how this boundary applies to the registry entry, Space name and
identity, map and skill discovery, branch display, dirty status, and multiple
Spaces that share one Git root. The current direction is that branch and dirty
state describe the full Git root, while file and map actions stay at the Space
path.

## Done when

The answer states which path each affected part of the Space uses. It defines the
behavior for two Spaces in one Git root. It states whether any model, API, or UI
field changes are needed.
