# Proposed next milestone: Chartr design reference v0

Status: proposed next step, not an adopted implementation plan. Prepared 10 September 2026 from the owner's design preferences and the discussion in Slopchan posts 51–61. The implementation remains at commit `5402a35`.

**Subsequent owner steering:** run Matt Pocock's `/prototype` skill first. A [three-variant browser comparison](2026-09-10-design-prototype.md) is now available on a separate throwaway branch. The native reference below becomes the next validation stage after the owner reviews those alternatives. This changes the sequence, not the requirement to prove native behavior and a shared plugin design contract.

The next move should produce a working native reference for Chartr's visual and interaction language. The question to settle is whether a realistic navigator, Git surface and web-plugin controls feel like the same application across themes and pane sizes. More competitor research or a broad feature rollout would not resolve that question as directly.

## The concrete deliverable

Build a development-only GPUI design reference screen using Chartr's existing Zed/GPUI stack and shared components. Populate it with fixture data so work can focus on layout, behavior and presentation without requiring a Git backend, new conversation persistence or agent execution.

The reference composition has a compact project/history navigator, a main Git surface based on the owner's sketch, and a small web-plugin specimen using the same ordinary controls and theme roles. Tree and history presentations can use the same fixture items; this does not commit us to a final navigation default. The Git fixture covers both All commits and Local changes, selected-commit details, file/diff presentation and a commit composer. The web specimen can represent Wayfinder's ticket inspector and action controls without redesigning the specialized map canvas.

The screen should support actual selection, focus, menus, text editing and resizing. Rendering a static picture alone cannot establish native interaction quality. The prototype should clearly identify simulated domain actions; it must not run agents, mutate a repository or imply that a fake commit action is functional.

## Work order

1. **Write a short design charter and inventory the existing controls.** Establish the visual direction: quiet surfaces, compact readable rows, consistent alignment, restrained color and predictable native behavior. Record the owners of window chrome, standard surface chrome and specialized content. Reuse the existing forms, selection controls and typography foundations. Exact dimensions remain prototype decisions.

2. **Build the reference composition and a compact component gallery.** The minimum vocabulary is surface header, toolbar/action group, tree/history row, tab/segmented control, text field, menu, status badge and empty/loading/error state. Show these in realistic arrangements as well as isolated states. Start with one direction; vary a few consequential choices such as density and separator strength instead of creating unrelated themes or mockup families.

3. **Prove a shared theme contract across native and web.** Define semantic roles for surfaces, text, borders, selection, focus, status and diffs, together with common typography and density inputs. Map existing theme data into those roles and demonstrate live updates in both specimens. A shared manifest or generated representation can avoid maintaining unrelated token definitions; the exact mechanism should follow the prototype's needs. Do not preserve a separately hardcoded dark palette for ordinary web-plugin controls.

4. **Review the populated screen under stress, then record v0.** Settle row geometry, spacing, type hierarchy, nesting, metadata priority and narrow-pane behavior from the reference. Record decisions with reference screenshots and working examples. Promote the proven components and contracts rather than freezing an untested general plugin API.

5. **Migrate real consumers in a controlled sequence.** Start with existing navigator chrome and shared native controls, then ordinary plugin chrome and Wayfinder's inspector controls. Use the Git prototype as the reference when implementing its real functionality. Track replacements explicitly so the old and new styling paths do not remain equally valid indefinitely.

## Acceptance criteria

The following sizes are test fixtures, not proposed hard minimums: approximately 320, 640 and 1100 pixels of available pane width. Include normal and enlarged interface text.

- Dark and light themes both look deliberately designed. A strongly different existing palette also preserves component hierarchy and state meaning.
- Selection, keyboard focus and agent/activity status remain distinguishable when present together.
- Long work titles, branch names, paths and mixed staged/unstaged file states remain understandable.
- Narrow panes reduce secondary metadata and change composition without losing selection, the visible work context or a draft message.
- Menus, keyboard traversal, text editing, drag feedback and focus restoration follow the same rules across specimens.
- A native row/field/button and its ordinary web-plugin counterpart share typography, sizing roles, states and spacing conventions.
- Common screens can be assembled from the approved vocabulary. Any custom surface has an explicit boundary rather than quietly reintroducing its own standard toolbar, field or status language.

The milestone is complete when the owner can interact with these compositions, choose the remaining consequential visual options, and the resulting reference can guide a new contributor without requiring them to invent ordinary UI conventions. “Looks good in one dark screenshot” is insufficient.

## Enforcement and scope boundaries

The host should own standard surface chrome and shared interaction behavior. Plugin content may still need specialized rendering. A component package and shared tokens help first-party/custom surfaces; host-rendered standard plugin layouts provide stronger structural enforcement where conformity is required. Arbitrary HTML remains an explicit limitation on any promise of complete internal visual conformity.

This milestone should reveal which standard surface contracts are useful. It should not redesign the whole plugin architecture in advance. It also does not include full Git behavior, a columns-mode implementation, a history database, rich CLI adapters or Companion hardening. Those features remain separate work and can consume the proven design system afterward.

Current evidence: `crates/chartr/src/components.rs`, `components/form.rs`, `components/selection.rs`, and `fonts.rs` already contain shared native foundations. `plugins/wayfinder/styles.css` defines an independent palette and control treatment. `docs/plugins.md` already demonstrates stronger host ownership in schema-rendered Settings. The proposed milestone connects and tests these existing ideas rather than replacing the framework.
