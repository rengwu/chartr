# Chartr design discussion: owner feedback

Follow-up to the [competitor research](https://slopchan.john.shiksha/threads/31), 9 September 2026. Discussion remains open; the owner has more thoughts to share. These notes do not authorize implementation or settle the proposals below.

[Forum discussion record](https://slopchan.john.shiksha/posts/51), posted and read back for verification.

## Owner's constraints and preferences

- Context switching is the immediate organization problem. Named/context-sensitive sessions or threads should remind people what they were working on. Arbitrary pane/tab groups often have no single purpose, so requiring meaningful group names would be a poor fit.
- Wayfinder's existing completion behavior was adequate in prior use. Cater to people carefully managing tokens; additional agent verification must not become mandatory overhead.
- The earlier phrase “coherent default plugin experience” needs a concrete explanation before it can support a product decision.
- Proposed Git plugin: space and branch header; All commits with a commit/branch graph; Local changes with diff previews, file/hunk stage and unstage, commit-message authoring and committing. Selected-commit details split between Commit information and Changes. Adapt visible columns to pane width. The supplied sketch is the scope reference.
- Companion is early development and needs hardening. Its longer-term direction is not decided.
- A coherent, themeable, sleek, native-feeling design system is a central requirement. Future plugins/extensions should follow authoritative shared UI rules rather than accumulating independent styles.
- Preferred references, in order: Cube and Soft Machine. Cube's branched repository/worktree tree and horizontally scrolling Niri-like columns are particularly appealing. These are possible additional presentations, not approved replacements for existing modes.
- Soft Machine's thread list is appealing; its potential clutter is not. Paseo/T3-style historical sidebars provide useful simplicity, but Chartr should preserve richer pane and power-user workflows.
- Proposed rich chat plugin: the ordinary CLI agent remains running in a terminal; a richer interface sits between the user and that process. It should support features such as images and multiple-choice interactions where possible. This is not a request to replace the CLI with SDK/model calls.

## Revised interpretation, for discussion

The strongest candidate boundary is persistent conversation/session identity independent of pane placement. A history entry can locate an existing item or reopen supported history without constraining how its panes are arranged. A shell/PTY can host successive agent conversations, so terminal identity and provider conversation identity must not be assumed identical. Group naming stays optional.

Titles should prioritize explicit user names, provider-owned titles where available, and inexpensive local fallbacks such as an initial prompt excerpt. Additional model-generated titles would be optional, bounded, and stable rather than continuously refreshed. Moving panes must not rename or reclassify conversations.

Navigation (tree/history), arrangement (tabs/splits/scrolling columns), and content presentation (terminal/chat/diff) are separate choices. This extends the existing distinction between chrome and item ownership described in [workspace.md](../workspace.md). A history list does not need to become the parent of every visible pane.

The design-system gap is concrete: [shared native controls](../../crates/chartr/src/components.rs) and [semantic text sizes](../../crates/chartr/src/fonts.rs) already exist, while [Wayfinder](../../plugins/wayfinder/styles.css) defines an independent dark palette, sizing and controls. A common palette alone cannot establish common layout and interaction rules. Shared components and host-rendered standard plugin surfaces provide stronger enforcement than optional CSS guidance; unrestricted custom HTML cannot guarantee internal visual conformity.

The earlier Wayfinder recommendation was too prescriptive as a near-term priority. Completion should remain cheap and explicit. Optional manual review and ordinary local checks can be recorded without extra model inference; extra agent review requires a separate user choice. Likewise, the proposed Git plugin can begin as a useful repository tool without depending on a new task system.

A terminal-backed rich UI is feasible as a capability-specific integration, not a universal conversion of terminal pixels into chat. Prefer available hooks and transcript/event records, retain the live terminal, and expose unsupported interactions there. Claude Code's [official hook reference](https://code.claude.com/docs/en/hooks) documents structured lifecycle/message events and session/transcript fields. [Unpeel's runtime contract](https://github.com/unpeel-com/unpeel/blob/main/runtimes/README.md) is another relevant primary reference for provider-specific identity, lifecycle and capability limits. This does not establish equivalent capabilities for every CLI named in the discussion.

Companion's immediate proposed direction is paired, authenticated access to the same sessions, verified server identity, revocable devices, clear read/control permissions and ownership. Broader cloud access or headless operation is a separate product decision. The existing [development protocol](../companion-protocol.md) makes its current limitations explicit.
