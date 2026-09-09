# Chartr: elaborated design discussion

Published in the [Chartr Slopchan thread](https://slopchan.john.shiksha/threads/31). The [discussion index](https://slopchan.john.shiksha/posts/60) links eight detailed replies. All eight replies and the index were read back and verified; the [publication receipt](2026-09-09-design-elaboration-publication.json) records their IDs and content hashes.

9 September 2026. Expansion of the owner's feedback and the response recorded in Slopchan post 51. These are proposals and tradeoffs for discussion, not an approved roadmap. The owner has more thoughts to share. The priorities explicitly established so far are better context recall, freedom of pane arrangement, token economy, a coherent themeable design system, and the proposed Git-plugin scope. Cube and Soft Machine are visual references, with Cube preferred; their particular modes are still possibilities.

## 1. Persistent identity, useful names, and history without prescribed layouts

The naming problem is partly a category problem. We currently ask one visible label to stand in for the terminal process, the conversation, and sometimes an arrangement of several unrelated things. There is no title-generation technique that can give a genuinely mixed pane group one accurate purpose. A group containing release work, a documentation browser and an unrelated experiment may simply be a convenient arrangement.

I would stop requiring group identity to do the work of conversation identity. Groups may remain unnamed or user-named. Their fallback label can describe their contents without inventing a shared objective. A group label such as “3 panes” is acceptable if there is also a dependable way to find and recognize each conversation inside it.

The proposed identities have different lifetimes:

- A space identifies a project context. Today Chartr uses a canonical folder path; a repository/worktree relationship can enrich that without silently changing what existing spaces mean.
- A terminal session identifies the persistent PTY and its process lifetime.
- An agent conversation identifies a particular conversation, ideally through the provider's actual session identity. One PTY can host successive conversations, and a supported resume can attach a conversation to a new runtime.
- A pane or group records placement. It does not establish semantic ownership over the conversation's title or history.

For an unknown CLI, Chartr should retain a useful named terminal session without inventing a provider conversation ID. A manually launched shell, a detected agent, and a verified resumable conversation are different levels of integration. This distinction prevents passive process detection from becoming an unreliable promise of resumability.

Example: I start “Fix login redirect” in a terminal, then put it beside Git and a browser. Later I move that terminal to a group containing another agent. Its title remains “Fix login redirect.” Its history still knows the project, agent and associated checkout. The groups can change without rewriting that record. If I use the same shell tomorrow for a different agent conversation, the old conversation remains in history rather than acquiring tomorrow's title.

History is an index into work. Clicking a live entry should focus its existing item, reveal the owning space/group, and scroll a columns view if necessary. It should not launch a duplicate agent. Clicking an ended entry can show retained history and, where actually supported, an explicit resume action. Missing history should be reported as missing; terminal scrollback should not be presented as an authoritative structured transcript.

Closing a pane, stopping a process, archiving an entry and deleting history require distinguishable semantics. This does not mean changing today's close behavior casually: Chartr currently terminates a terminal when its item closes. Any future detach-only behavior would need explicit product design and migration. The immediate point is that ending a process need not erase our record of what it was about.

Naming should have a stable precedence: explicit user title, provider-owned title where available, an inexpensive local fallback, then optional generated refinement. An initial-prompt excerpt is a useful fallback when the prompt is available through Chartr's composer or a supported adapter. We cannot reliably extract every first prompt from arbitrary shell activity merely because a terminal exists.

Optional title generation should have a bounded trigger and budget. Generate once when there is enough context, cache it, and stop overwriting a useful landmark. Never replace a manual title. A changed objective can produce a suggested rename or an explicit new conversation; it should not make familiar entries continually change under the user's eyes. Avoid adding hidden model calls to every turn.

A good row does not depend entirely on a perfect title. Show a short primary name, recognizable agent identity and state, with project/worktree context available where it disambiguates. A recent-history view spanning projects needs more context than a tree already nested under one project. Use the same underlying row component with context-aware fields, not a different visual language.

The Soft Machine screenshot is useful here: repeated title beginnings remain difficult to distinguish even when all threads are auto-titled. We should test recall with similar tasks, not only an attractive list of conveniently distinct demo names. Measure whether someone returning tomorrow can find the right conversation without opening several candidates.

Current source anchor: docs/workspace.md describes chrome as a projection over items and documents group naming; crates/chartr-herdr/src/control.rs currently prioritizes the detected running agent/process over the saved terminal label. The proposed history/conversation layer extends those boundaries; it is not present merely because titles can already be renamed.

## 2. Navigation, pane arrangement, and content presentation are independent decisions

Cube's tree, Soft Machine's history and horizontally scrolling columns are compatible ideas. The mistake would be to make every one a competing top-level “mode” that changes what a workspace means. They answer different questions: where is my work, how is it arranged, and how am I interacting with it?

Navigation can offer a project/worktree tree and a recent-history view. Arrangement can retain tabs/splits and optionally add columns. A particular content item can offer Terminal and Chat presentations, while other items show Git, a browser or a map. These need not be exposed as a large matrix of settings. A small set of useful presets can establish the default while preserving the underlying separation.

The tree should describe real relationships. A possible shape is project → worktree → active conversations and useful tools. If there is only one checkout, avoid spending a permanent extra level on “Primary worktree” unless the branch context is useful. Host grouping can appear when remote hosts exist instead of imposing a cloud/local root on every local-only user.

A branch badge describes the checkout. It is not a substitute for the conversation title: two conversations can use the same branch, and a conversation's purpose can survive a branch change. Likewise, a nested row should not imply that moving it casually between worktrees moves its running process or files. Navigation drag-and-drop and environment reassignment need different semantics.

I would avoid forcing the tree to reproduce the complete split geometry. That would make it another representation of window management rather than a useful project index. The tree can list meaningful items and locate their actual placement on selection. If a user wants to inspect group structure, expose that locally instead of making every project row expand into a deep pane tree.

History is especially valuable for cross-project recall. It can provide search, recent activity, active/ended state, pinning and a restrained attention filter. The ordinary history list should not continually reshuffle because an agent emits output: that destroys spatial memory and makes clicks unstable. Prefer meaningful interaction timestamps and predictable updates; an attention view can prioritize intervention without changing the user's entire organizational structure.

For columns, my first hypothesis is that an existing standalone item or pane group occupies a horizontally arranged column. A group can retain its internal splits. This preserves sophisticated arrangements while allowing the user to scroll through adjacent work. It also gives us a bounded first design to compare against a completely new tiling model.

Columns need more than horizontal scrolling to feel good: predictable default width, keyboard focus that brings the target into view, no empty gaps when columns close, clear insertion feedback, and a way to temporarily concentrate on one column. New background output must not pull the viewport away from the user's current work. A user-initiated “open agent” can reveal the result; an explicitly background launch should behave differently.

Switching arrangement should preserve session identities and running processes. It need not mathematically convert every arbitrary split tree into an equivalent column layout. We can retain mode-specific placement state over the same item identities and restore each arrangement when revisited. Content and runtime ownership remain singular; saving two layouts must not duplicate a terminal.

There is a practical limit to modular freedom: nested headers, tabs, controls and split borders can consume more space than the content. Define rules for when an outer title is sufficient and when an inner title is needed. Chartr already suppresses a redundant inner tab bar for standalone items; this is the kind of composition rule to extend deliberately.

The experience I want to test is simple: find a conversation in History, reveal it inside a column containing a split group, interact, then return to the previous arrangement without losing selection or context. If that feels complicated, adding more modes would amplify the current confusion. The success criterion is orientation, not the number of layouts offered.

Source anchor: docs/workspace.md and crates/chartr/src/chrome.rs describe the existing projection/ownership separation. The proposed Cube-inspired tree and columns are interpretations of the owner's supplied screenshot and preference, not claims that we audited Cube's current implementation of every interaction above.

## 3. A visual language that can remain recognizable across themes

I interpret the preference for Cube and the simpler parts of Soft Machine as a preference for a calm, compact working surface: restrained framing, readable rows, modest hierarchy and visible relationships. The goal is not to reproduce one screenshot's exact colors. The same composition should remain recognizable in a light theme, a high-contrast theme and a user-chosen palette.

Chartr already has a partial foundation. Its native forms share controls and its typography has semantic roles. However, Wayfinder defines a separate dark palette, control geometry and font treatment in its stylesheet. That is evidence of independent design ownership across surfaces. If every plugin solves those questions locally, visual drift is the expected result, even when every individual author makes reasonable choices.

I would establish a small set of visual rules before extending the catalog:

1. The background hierarchy is restrained: application ground, working surface, and temporary overlay. Avoid giving every nested container a distinct card, heavy border and different radius.
2. A row has one primary text anchor. Icons, status, branch and time support it. Metadata should not all demand the same contrast or weight.
3. Selection, keyboard focus and activity are distinct. A selected history row, focused terminal and working agent can exist simultaneously; using the same accent treatment for all three makes the interface ambiguous.
4. Tree indentation, icon alignment and disclosure controls follow one grid. Parent-child relationships should survive both crowded and sparse content.
5. There is one icon family, a limited icon-size scale, and consistent button hit targets. Compact visible controls still need comfortable pointer and keyboard access.
6. Essential actions remain discoverable. Hover can reduce persistent clutter, but keyboard focus must expose equivalent controls, and touch contexts need an accessible alternative.
7. Density removes duplicated framing and wasted gaps while keeping text readable. We should not solve crowded layouts by reducing every label to the smallest type size.

Theme tokens should describe roles such as surface, muted text, border, focused border, selected row, warning and diff addition. Components consume those roles. Theme authors choose their appearance; components retain the meaning. A plugin should not pick an arbitrary green for “done” or an arbitrary blue for “selected” and assume those colors work against every theme.

Color is not the only theme concern. Font substitution, user scale, longer labels, light backgrounds and reduced motion all reveal whether the design is resilient. The system needs a defined spacing and type scale, a small radius vocabulary, and explicit disabled/hover/active/focus states. We should use prototype measurements to settle exact values rather than declaring a pixel scale from a screenshot.

The composition rules matter as much as individual controls. Define a standard surface header with context, title and an action area; a standard list/detail relationship; a standard place for transient errors; and rules for empty, loading, disconnected and recovery states. Plugins should not invent full-page banners, modal flows or duplicate navigation for routine states that the host can represent consistently.

Native feeling is behavioral as well as visual. Selection should remain stable while lists update. Focus should be visible and return predictably after menus/dialogs. Text fields should support familiar editing. Context menus should expose the same actions as keyboard commands. Resizing should preserve the user's reading position where practical. These details are part of the design system, not polish to add after the colors are settled.

Three reference surfaces would stress the design well: the project/history navigator, the proposed Git plugin, and a rich conversation pane. They cover trees, dense tables, files, diffs, forms, long reading surfaces, choices and streaming content. A system that only looks coherent in Settings has not yet proved it can govern Chartr's actual work.

I would review those surfaces in dark and light themes, at narrow and wide pane sizes, with large fonts, long titles, errors and active keyboard focus. The question is whether they feel like parts of one application under stress. A pristine empty-state screenshot does not answer that.

Current source anchors: crates/chartr/src/components.rs; crates/chartr/src/components/form.rs; crates/chartr/src/components/selection.rs; crates/chartr/src/fonts.rs; plugins/wayfinder/styles.css. This is a proposal to consolidate and complete existing foundations, not a claim that Chartr has no shared UI code or needs to abandon GPUI/Zed components.

## 4. Making the design system authoritative for plugins

The owner's requirement is stronger than “publish design guidelines.” Guidelines reduce accidental inconsistency, but they do not prevent a plugin from drawing its own application inside a pane. We need to decide which visual responsibilities belong to Chartr and which belong to plugin content.

Chartr already demonstrates a useful enforcement pattern in Settings. Portable plugins declare fields, and the host renders them through native controls. Plugins cannot substitute an arbitrary HTML settings page. That makes consistency a property of the interface contract rather than a request that each author copy the right styles.

We can extend that pattern to common surfaces without making every plugin identical. A standard plugin might declare a title/context header, toolbar actions, list or tree content, selection, details and status. Chartr owns their geometry, styling, keyboard behavior and theme application. The plugin owns the domain data and actions.

A shared component package is still useful for first-party native views and custom web content, but it is a weaker guarantee: callers can compose shared controls into an incoherent screen or bypass them. For standard interfaces where conformity is essential, host-rendered layouts provide stronger control. For custom visualizations, allow a defined content region inside the common surface frame.

Wayfinder is a good example of the boundary. Its map should retain the visual vocabulary needed to express dependencies and progress. Its ordinary buttons, ticket inspector, error messages, tabs and typography should share Chartr's language. Treating the entire plugin as either unrestricted art or a generic form is unnecessary. The graph is specialized content; much of the surrounding interaction is common UI.

A browser page and a terminal also cannot be forced to look like Chartr's lists internally. Their outer controls, focus behavior, loading/recovery state and placement should still belong to the application. This makes the achievable guarantee precise: authoritative host chrome and standard controls, plus explicit custom content boundaries. Absolute conformity inside arbitrary third-party HTML is not technically guaranteed by CSS variables or a style guide.

The contract should include behavior and lifecycle. Who owns selection? What persists when a view closes? How does a long operation report progress or cancellation? How does a plugin explain an unavailable action? When a theme changes, who publishes the update? If we only standardize colors, plugins will still diverge in every one of these interactions.

I would migrate incrementally. Start with existing native controls, define the missing semantic tokens and surface patterns, and apply them to the navigator and Git reference surface. Adapt Wayfinder's ordinary controls next. Use those concrete consumers to discover what a plugin needs. Avoid publishing a huge speculative component API before any real plugin exercises it.

Enforcement can then combine construction constraints and verification: shared host components, supported surface templates, checks against bypassing standard tokens in first-party UI, and representative visual/interaction fixtures. Checks should cover meaningful failure cases—focus disappearing after refresh, clipped controls at large font sizes, invisible selection in a light theme—rather than tests that merely repeat a component's implementation.

A component gallery is useful as an executable reference. Show normal, hover, focus, selected, disabled, loading and error states alongside dense real-world compositions. A future contributor should be able to find the existing answer to “how do I show a selected tree row?” without inventing one.

The unresolved product choice is how much unrestricted web UI remains available to ordinary plugins. We can retain it as an explicit custom-surface capability, while making the supported standard route host-rendered. That is a real compatibility and authoring tradeoff, not something a “strict design system” slogan resolves. No plugin-tier migration is approved by this discussion.

Source anchor: docs/plugins.md documents host-rendered native Settings, arbitrary web-pane content, the current native services boundary, and plugin version limitations. The proposed declarative standard surfaces extend that existing approach; they are not current functionality.

## 5. The Git plugin should deliver the owner's sketch without waiting for a task framework

I agree with the proposed scope: a useful Git client surface inside Chartr, with commit/branch graph, selected-commit information, changed files and diffs, local staging/unstaging, a commit-message editor and committing. That is enough to improve daily work substantially. It should not depend on Wayfinder adoption or a new orchestration model.

The top context should make the owning repository/worktree and checked-out branch clear. “All commits” is history browsing; selecting a commit must not check it out. If the selected commit belongs to a different branch, the header still identifies the current checkout while the detail panel identifies the selected revision. This distinction prevents a very common form of disorientation.

In All commits, the graph and subject are the primary columns. Author, abbreviated hash and date are useful secondary columns. Hide secondary columns progressively as the pane narrows; make the full information accessible in commit details. Render topology from actual parent relationships, retain full commit identities internally, and treat branch badges as movable references rather than the identity of a row.

The lower Commit and Changes tabs have different jobs. Commit shows subject/body, author/committer information and relevant revision metadata. Changes shows files and diffs for the selected commit. Merge commits require an explicit comparison policy or parent selector; a UI should not silently present one parent's diff as if it described every merge relationship.

In Local changes, use a consistent file-list/diff arrangement with clear staged, unstaged, untracked and conflicted states. A file may have both staged and unstaged changes. Selecting it must make the comparison explicit: HEAD versus index or index versus working copy. Otherwise “stage this hunk” is ambiguous precisely when a careful user needs the tool most.

File and hunk staging should preserve unrelated index entries and partial staging. Agents and external tools may change files while a diff is open. Before applying a hunk action, verify that the displayed comparison still corresponds to the current content/index state. If it is stale, refresh and explain the change instead of applying a patch against a different version. This is implementation work that keeps the front end simple and trustworthy.

The commit editor should keep its draft by repository/worktree rather than losing it when the pane closes or the user switches views. Commit the staged selection, show useful validation when it is empty or blocked, and preserve the draft on failure. A hook failure should surface its relevant output; it should not leave the interface pretending a commit succeeded. If a command's outcome becomes uncertain, inspect repository state before retrying a mutation.

Diff coverage needs honest fallbacks. Renames, deleted files, binary content, submodules, conflicts and very large files should be recognizable even when a full text preview is unavailable. Do not silently omit a file because the pretty diff renderer cannot display it. Advanced conflict editors or history rewriting can remain external initially, but visibility of these states belongs in the first useful local-changes view.

Responsive behavior follows the actual pane width, not the monitor width. At moderate widths, reduce metadata columns and let the file list resize. At very narrow widths, use a file-picker/list state and a diff state with a clear back path. Preserve selected commit, selected file and draft message through those changes. Avoid cramming two unreadable narrow columns into a pane simply to retain the desktop arrangement.

Keep the scope disciplined: no automatic AI commit messages, task association, PR automation or full Fork parity is required to make this first version useful. Those can be considered independently. A normal Git operation performed in another terminal or external client must still refresh the plugin correctly.

Useful proof scenarios are concrete: stage one of two changes in a file; make another edit after opening its diff; commit while an unrelated file is already partially staged; handle a failed commit hook; close/reopen with a draft; browse a merge without changing checkout; and shrink the pane while retaining selection. These scenarios define quality without inflating the visible product.

The owner's supplied Git sketch is the design reference. Fork and Nimble remain interaction references, not a requirement to reproduce their entire feature sets. The internal Git backend choice and exact graph-rendering algorithm remain open engineering decisions.

## 6. A rich conversation interface over the same ordinary CLI process

The proposed feature is a second interface to a normal CLI agent, not a separate client that reconstructs the agent through model or SDK calls. The existing CLI should keep its own authentication, configuration and execution behavior. A Terminal/Chat presentation switch should refer to the same runtime and conversation, with the raw terminal available when needed.

The hard part is semantic information. A PTY stream describes terminal output and redraws. It does not universally identify user messages, completed responses, attachment ownership, pending choices or approval transactions. Parsing the screen can support limited observations, but it is not a sufficient foundation for promising every rich-chat feature across arbitrary CLIs.

I would build provider adapters with explicit capabilities. A basic adapter might only detect the agent and expose terminal input. A richer one might identify the native conversation, locate its transcript, report lifecycle events and support particular structured interactions. Unsupported capabilities remain unavailable or fall back to the terminal. This preserves the ordinary process while avoiding an imaginary universal protocol.

Where providers expose hooks and transcript/event records, use those alongside the PTY. Claude Code's official hook reference documents session/transcript fields and lifecycle/message events. Unpeel's public runtime contract is also relevant: it distinguishes identity, lifecycle, resume and integration capabilities instead of assuming all detected processes are equivalent. Neither source establishes identical support for Codex, Grok or every other CLI.

The data flow should have one runtime owner, an adapter that translates recognized semantics into Chartr events, and one conversation record consumed by the rich pane, history and eventually Companion. The rich pane should not establish a second competing terminal takeover merely to observe output. Terminal geometry, attachment and input ownership must remain coordinated by the host.

Input needs particular care. A rich composer sends into the current agent's actual input mechanism. Multiline text, paste and submit must preserve that CLI's semantics. If the terminal is in a shell or another program, the chat composer must not blindly send a message there. Observe recognized agent interactions rather than indiscriminately collecting every shell keystroke.

For a multiple-choice card, record which pending question it represents. Before sending a response, confirm that question is still current and that another client has not already answered it. The same rule is even more important for permission approval. A stale card must become inactive; it must not translate into “press Enter” on whatever happens to be on screen now. Unsupported interactions should expose the terminal rather than guessing.

Images require the provider's actual attachment route. Saving a file and typing its path is not universally equivalent to attaching an image. On a remote host, the agent also needs access to the file at the referenced location. Attachment storage, transfer, lifetime and failure reporting should be explicit capabilities, with a usable fallback when a provider cannot consume the proposed input.

Common chat-app features need individual semantics. “Edit an earlier message,” “retry,” “branch conversation,” and “regenerate” may create a new native conversation or rerun work. They cannot merely rewrite the visible chat history while leaving the CLI in a contradictory state. Implement each only when the adapter can preserve or accurately explain what the underlying agent does.

History ingestion also needs ordering and deduplication. A hook notification and transcript update may describe the same event; reconnecting should not duplicate it. Partial responses and later final records need a defined reconciliation rule. Unsupported tool output can remain a terminal/log block without manufacturing structured content. Missing evidence should be visible rather than filled in by a model guess.

The first prototype should demonstrate identity, readable message history, composition and switching to the live terminal for one or two verified providers. Then add images and pending-choice interactions where reliable. This is more useful than a broad provider selector that implies every capability works everywhere. The ordinary terminal remains a complete interface throughout.

This feature reinforces the organization work: adapters can provide conversation identity, titles and meaningful attention events. But the ownership should be shared in the host; a rich-chat plugin should not become the only place history exists. Otherwise disabling the plugin would make the user's work identity disappear.

Primary references:
https://code.claude.com/docs/en/hooks
https://github.com/unpeel-com/unpeel/blob/main/runtimes/README.md

Current Chartr boundary: docs/plugins.md exposes session.metadata and session.send to bound web plugins, but not the complete observation/event model needed above. docs/workspace.md describes Herdr attachment through the terminal stack. The proposal therefore requires host support, not only a new HTML chat screen.

## 7. Token economy, lightweight completion, and what “default experience” should mean

The owner's token constraint changes the prioritization of the initial research. A workflow that constantly summarizes sessions, names groups, generates titles, reviews every result and invokes another agent to verify it would impose a tax on ordinary use. That is a poor default for a tool meant to help users manage limited subscriptions and budgets.

I would separate four kinds of work: deterministic local bookkeeping; local commands such as tests or Git inspection; inference the user explicitly asks an agent to perform; and optional convenience inference initiated by Chartr. The last category needs especially clear boundaries because it can spend resources without feeling like a user-requested task.

Names, history indexing, selection, status bookkeeping and layout should not require model calls. An optional generated title can have a small bounded trigger and then remain cached. A summary can be created on request or at an explicit handoff. If a provider does not expose reliable usage information, show that limitation rather than displaying a fabricated precise budget.

For Wayfinder, preserve the lightweight completion path that already served the owner. An Answer can remain a completed planning record. If we later attach evidence, distinguish what is known without demanding another agent: answer recorded, checks recorded, or human-reviewed. These can be optional facts rather than gates that every ticket must satisfy.

A research answer may be enough. A small UI change may need a human look. A risky code change may justify tests or a second review. Chartr should help represent the chosen process, not automatically escalate every task to the most expensive process. A future dependency policy can be explicit for users who want stronger gates; it should not silently redefine all existing maps.

Local checks need no additional inference call, although they still consume execution time and computing resources. Extra agent review is different and should be requested deliberately. “Record what check was run” and “ask another agent to determine what checks to run” should not be conflated in the interface.

My earlier “coherent default plugin experience” phrasing was too broad. Current Chartr already has shared native settings, prerequisite explanations and setup links. The remaining question is whether a user can perform a common action without understanding the internal plugin graph.

Concrete examples: starting an installed CLI should offer a sensible editable launch preset; opening Git should select the owning space's repository; a Wayfinder launch that lacks an agent definition should lead directly to the relevant setup and return to the pending work. The setup should not erase the user's prompt or require them to find the original ticket again.

This is continuity of the user journey. It does not mean all plugins must be enabled, every surface bundled, or every user pushed into planning. A person who only wants terminals and Git should have a coherent product. Someone who enables Wayfinder or rich chat should gain depth without receiving a different set of visual and interaction conventions.

The first-use path should therefore have progressive commitment: start a terminal or known agent immediately; attach an explicit name when useful; inspect history naturally; add more structured planning only when the user wants it. The power-user path retains arguments, environment settings, alternate CLIs and arbitrary layouts. Sensible defaults and detailed control are compatible if the default path does not expose every decision at once.

Revised priority: organization and design coherence are the immediate product problems. Verification, stronger task semantics and agent automation remain optional extensions of that foundation until real usage demonstrates demand. This supersedes the initial implication that Chartr should make an acceptance pipeline mandatory before it becomes useful.

Source anchors: plugins/wayfinder/README.md and plugins/wayfinder/src/model.rs describe current completion/claim behavior; docs/plugins.md and README.md describe existing setup and prerequisites. These are revisions to the research recommendation, not changes to the implemented behavior.

## 8. Companion direction, cross-cutting ownership, and how to validate the design

Companion's first direction can be narrow: recognize the same work from another device, inspect it, and deliberately take control when needed. Named sessions and shared conversation identity improve that experience before we build a broader remote platform. A phone list of generic agent names has the same recall problem as the desktop, amplified by limited space.

The immediate hardening requirements are concrete: authenticated pairing, verified server identity, revocable device credentials and explicit access scopes. Pairing should establish which desktop the phone trusts, not merely remember an address. A device list should make access visible and revocation understandable. The implementation should use established protocols/libraries rather than inventing a new cryptographic scheme as part of UI work.

Read access and control access should be distinct. Merely opening a session preview should not automatically take over its geometry or input. The current watch operation acquires a mobile geometry lease; a future observational preview would need a different behavior. A clear “take control” action can then acquire the relevant lease and show the effect on the desktop.

Control ownership must coordinate the desktop terminal, rich chat and phone. If one surface is answering a question, another should not race it. Disconnecting or expiring a lease should release control predictably without stopping the underlying agent. The interface should explain which client currently controls input, while allowing inspection where supported.

Reconnect needs truthful delivery state. If the connection drops after a submit, the phone must not automatically send it again and risk duplicate execution. A hardened command protocol needs an appropriate way to identify acknowledged operations or reconcile uncertain outcomes. Offline approvals and stale keystrokes should not be replayed into a later prompt simply because connectivity returned.

Credentials and networking are separate concerns. A VPN can provide reachability, but the application's device/control model still needs to be deliberate. The initial product can work over the user's reachable network. Managed relays, cloud accounts and broader host provisioning are separate decisions; they are not prerequisites for defining paired control correctly.

The desktop-window lifecycle is another independent decision. Today the sharing listener belongs to its window, even though Herdr sessions can outlive the UI. If we later promise access after that window closes, a headless control service must own the necessary state and capabilities. That promise should be implemented and tested explicitly rather than inferred from terminal persistence.

Across the desktop proposals, I see a few invariants worth settling before adding more surfaces:

- Work identity survives rearrangement and does not depend on one optional plugin being open.
- Each live runtime has one authority for attachment, geometry and input, even when several views observe it.
- A manual name stays authoritative; passive UI bookkeeping does not spend inference tokens.
- History selection finds existing work instead of silently spawning duplicates.
- Standard controls and surface composition belong to the design system.
- Git actions operate on the displayed repository/index state and preserve unrelated work.
- Unsupported provider capabilities remain explicit and usable through the terminal.

I would validate these through one daily-work scenario, not a feature parade: start two named conversations in one project, open Git beside one, rearrange them, switch projects, return through History, review a partially staged change, switch one conversation between chat and terminal, then inspect it from the phone. Include a stopped conversation and a disconnected device. Observe where the user loses orientation or becomes unsure what an action will affect.

Design prototypes should compare alternatives without committing implementation. Test the navigator and a narrow Git pane with the same typography and spacing; compare tree and history navigation over the same items; compare existing splits and a modest columns arrangement. The columns mode should earn its complexity by reducing context-switching effort, not only looking attractive in a screenshot.

The owner's remaining thoughts may change the boundaries. Open decisions include the first-class name for a conversation versus terminal session, how optional group names appear, which navigation is the default, how much unrestricted web UI remains supported, and which CLI capabilities are reliable enough for rich chat. These are recorded questions for future discussion, not requests to answer everything now.

Source anchor: docs/companion-protocol.md documents open-access development TLS, ignored legacy token, geometry leases, uncertain mutation delivery, and window-owned sharing. This discussion proposes the next product guarantees; it is not a completed security design or an implementation audit of a new protocol.

## Appendix A. Owner-provided Git plugin layout sketch

[Published appendix on Slopchan](https://slopchan.john.shiksha/posts/61), linked to the [Git-plugin discussion](https://slopchan.john.shiksha/posts/56) and [design index](https://slopchan.john.shiksha/posts/60).

![Owner's Git plugin sketch: space and branch header, All commits and Local changes views, commit graph, and lower commit-information or file/diff details](assets/git-plugin-sketch-2026-09-09.png)

The image is preserved without edits. It records the proposed functional layout and responsive column behavior; it does not establish a final theme or represent an implemented feature.
