# Chartr: competitor research and product direction

Published discussion: [Chartr on Slopchan](https://slopchan.john.shiksha/threads/31). Start with the [competitor watch index](https://slopchan.john.shiksha/posts/42). All 13 posts, including the [T3 citation correction](https://slopchan.john.shiksha/posts/44), were read back and verified after publication. Local companions: [competitor register](competitors.json), [source audit](2026-09-09-source-audit.md), and [citation ledger](2026-09-09-source-ledger.json).

Research snapshot: 9 September 2026. Chartr baseline: `rewrite/rust`, commit `5402a35`. Planning assumption: a focused 3–6 month product direction, with the first useful slice in 4–8 weeks; these are sequencing proposals, not delivery estimates.

My recommendation is to make Chartr the terminal-native workspace for carrying a piece of work from uncertain intent to accepted outcome. Preserve the purpose, decisions, dependencies, agent runs, changes, and evidence together. Let people keep their preferred CLI agents. Make the map, terminal, review screen, and phone different views of that same work.

Chartr has a credible foundation, but its current promise—an extensible AI-native workspace with spaces, panes, and plugins—is too easy for this field to match. Diri already contests the native Rust/GPUI position. Unpeel contests durable, agent-aware terminals. Several others already own a much more complete task-to-review journey. The opportunity is to turn Wayfinder's durable planning context into a dependable execution and acceptance workflow. That is a hypothesis to validate, not an uncontested category we can claim today.

This register includes every requested competitor:

1. [Superset][superset_site]
2. [Paseo][paseo_site]
3. [Emdash][emdash_site]
4. [Synara][synara_site]
5. [T3 Code][t3_site]
6. [bb][bb_site]
7. [DeepSeek Harness][deepseek_site]
8. [Moshi][moshi_site]
9. [Cube][cube_site]
10. [Soft Machine][soft_site]
11. [Superlogical][superlogical_site]
12. [Diri][diri_site]
13. [SSHHIP][sshhip_site]
14. [Unpeel][unpeel_site]
15. [Git Nimble][nimble_site]
16. [Fork][fork_site]

Superlogical tracking also includes public posts by [Alasdair Monk][almonk_profile] and [Mitchell Hashimoto][mitchell_profile].

Evidence distinguishes website/documentation claims, implementation inspected at a pinned commit, release evidence, and my judgments. I inspected selected implementation and test source in nine current product repositories, plus Cube's older public Collaborator implementation. Other public repositories were packaging or release feeds. I did not run competitor applications or their tests, benchmark performance, or conduct a complete security audit. Default-branch code is not proof of availability in a stable binary. A read-only look at the running Chartr Dev app informed the interaction critique; its build was not independently matched to the checkout.

## 1. Where Chartr actually stands

The rewrite is more substantial than a terminal skin. It has a native GPUI shell, its own workspace and pane model, persistent Herdr sessions, typed native plugin services, portable web panes, skills with source provenance, and Wayfinder's dependency-aware launch preparation. Those are useful ingredients. The current README describes source installation, pending production packaging, and legacy downloads that belong to the previous Go/Svelte product. Distribution is a competitive gap before feature comparisons even begin. [Chartr README][chartr_readme]

The strongest existing workflow is Wayfinder's launch boundary. It resolves a ready ticket, exposes the dispatcher, skills and context, rescans before execution, prepares a real session, and records the claim under a lock before delivering input. If delivery fails, it releases only its own claim. This is a good basis for trustworthy delegation: the user can inspect what will happen, and stale context does not silently slip through. Preserve it. [Wayfinder README][chartr_wayfinder]

There are six problems I would address before expanding the surface area further:

1. **The organizing unit is still mostly a location in the interface.** A space is a folder, containing tabs, groups, terminals, and plugin panes. That tells me where something lives, but not reliably what outcome it serves. In the running development app I saw repeated agent names and grouped tab counts. That is one configured workspace, not a usability study, but it illustrates the cost: a person must open terminals and reconstruct meaning. A persistent task title should lead; agent, branch, host, and current command should be secondary metadata. The title resolver currently prioritizes the detected agent or running process. [Session model][chartr_control]

2. **Session status and work status are disconnected.** Chartr already has Idle, Working, Blocked, Done, and Unknown states from Herdr, with chrome indicators. The missing piece is a durable answer to “what needs me, why, and since when?” Grouped outer tabs currently lack aggregate activity. A completed agent turn, a dead terminal, a claimed ticket, and accepted work must remain distinct. Add an attention view over existing signals, carrying reason, freshness, and uncertainty; do not invent a second competing detector. [Session model][chartr_control], [chrome activity][chartr_chrome]

3. **Wayfinder begins orchestration but stops short of owning its outcome.** One ticket can be claimed per space, including across maps. It has no automatic queue or worktree manager. A nonempty `Answer` marks a ticket resolved; this is body-derived status, not evidence that a change was tested or accepted. That convention is useful for research notes, but is too weak to certify engineering completion. Ordinary terminals can still run independently, so “Chartr cannot run parallel agents” would be an incorrect criticism. The real limitation is safe, coordinated parallel work. [Wayfinder rules][chartr_wayfinder], [ticket and claim implementation][chartr_wayfinder_model]

4. **Extensibility currently makes the user assemble part of the product.** Agent launch definitions and skill sources require configuration; Browser installation and restart are separate steps. Prompts provides a library and native service, but automatic injection/composer integration is not implemented. The pieces are present without one obvious default route from opening a repository to reviewing a useful result. Modularity should help maintainers and advanced users; the first useful workflow should arrive assembled. [Getting started][chartr_readme], [Prompts][chartr_prompts], [plugin contracts][chartr_plugins]

5. **There is no comparable integrated change-acceptance path in the reviewed baseline.** Launch context is carefully assembled, but an agent's output still needs a durable connection to its worktree, diff, checks, decisions, and acceptance. A Browser pane alone does not record verification. A future Git plugin can close this gap if it participates in the work model; an isolated Git window inside Chartr would leave the coordination burden intact. This is an architectural and product judgment from the reviewed workflow, not a claim that Git commands cannot be run in a terminal.

6. **Companion is explicitly a development protocol.** Starting sharing defaults to `0.0.0.0:9847`; connections are unauthenticated, the legacy token is ignored, and the phone does not verify the server certificate. A reachable client can use the exposed session/input operations. Sharing is opt-in, but these are concrete release constraints, not hypothetical concerns. Pairing, verified server identity, revocation, and deliberate control scopes must precede general distribution of remote control. The listener also belongs to a desktop window: persistent Herdr processes do not imply phone access after that window closes. [Companion protocol][chartr_companion_protocol], [Companion plugin][chartr_companion], [server implementation][chartr_companion_code]

My guess at the unspoken dissatisfaction is therefore: “I like the foundation, but I still spend too much effort arranging, finding, and supervising the work, and it doesn't yet feel like one opinionated product.” That is my inference, not a statement about the user's actual feelings. The current effort visible in pane ownership, chrome, and customization is necessary groundwork; it becomes strategically expensive if it keeps outranking the end-to-end work journey.

## 2. Superset, Paseo, and Emdash: the task-to-review baseline

**Superset** is competing to be the operating environment for parallel agent development: task-linked workspaces, isolated branches/worktrees, terminal agents, review, and remote execution. Its site distinguishes available remote-host workflows from managed cloud access being solicited through a design-partner program. Treat large concurrency claims as positioning, not measured capacity. [Product][superset_site]

The source shows depth beyond opening extra terminals. The host database ties workspaces to worktree paths, branches, HEADs, tasks, PRs, activity and archive reasons. Terminal records retain disposal intent; agent bindings distinguish different end reasons. The daemon manager handles attachment cancellation and recovery state, and its tests cover competing attach requests and stale dispatch. These are examples of making lifecycle transitions explicit; test presence does not establish a pass rate. [Workspace/session schema][superset_schema], [daemon manager][superset_daemon], [attach tests][superset_tests]

Compared with Chartr, Superset is farther along in making parallel output manageable and reviewable. Chartr should borrow the tight workspace–change–task connection and explicit recovery semantics. It should not chase every automation or remote-host feature immediately. Watch how Superset's default workflow balances speed against the growing number of controls. The reviewed release feed includes desktop 1.27.0, published 7 September. Its ELv2 repository is source-available; do not casually describe all inspected code as open source. [Repository/license][superset_repo], [release][superset_release]

**Paseo** is no longer accurately understood as just “use agents from your phone.” Its direction is a shared control plane across desktop, mobile, web, CLI and programmatic interfaces, with a headless daemon, workspaces, providers, and extensions. Its documentation describes direct and relay connections with different security properties; locality or encryption alone is not a blanket authentication guarantee. [Product][paseo_site], [security model][paseo_security]

The worktree service preserves a user's relative working directory, provisions branch/workspace metadata, and rolls back a newly created worktree when subsequent setup fails. A browser regression test specifically checks that archived worktree branch identity survives a daemon restart. Its client plugin registry has contribution points for commands, workspace panels, attachments, themes, and timeline transformations/renderers, not only rectangular panes. The registry also handles per-host catalogs and removal. The 0.8.0 beta and experimental plugin documentation are evidence of direction, not proof of stable API maturity. [Worktree service][paseo_worktrees], [restart test][paseo_tests], [plugin registry][paseo_plugins_code], [plugin docs][paseo_plugins_docs], [beta release][paseo_release]

Paseo's pressure on Chartr is architectural: the desktop should eventually be a client of work that retains identity elsewhere. In the near term, share one work model with Companion and provide useful remote supervision. Do not build a second phone-specific task system or a broad extension marketplace before the core experience is convincing.

**Emdash** presents a complete local/remote development cockpit: a task owns an isolated worktree; conversations, terminals, browser checks, diffs, PRs and CI surround it. Issue intake and reusable agent configuration reduce setup friction. This is a strong benchmark for “I arrived with work to do and left with a change I could ship.” [Tasks][emdash_tasks], [review workflow][emdash_review], [providers][emdash_providers]

Implementation details reinforce the product story. Task creation/provisioning and archive/delete are separate operations. Database records distinguish local and remote workspaces, observed Git state, script outcomes and deletion intent. Worktree creation has staged resolution/add/verification and rollback ownership, targeted fetching for missing refs, and protections for existing branches. Tests exercise a failed setup leaving no new debris and avoiding unnecessary network access. Provider plugins expose transport and lifecycle capabilities rather than relying on a launch string alone. [Task service][emdash_task_service], [database][emdash_schema], [worktree creation][emdash_worktrees], [creation tests][emdash_tests], [provider host][emdash_plugin_host]

Chartr should learn from Emdash's joined-up journey and failure handling. It can still prefer genuine terminal interaction and durable planning over a conversation-centric cockpit. The homepage and release feed were not identical in version labeling; the reviewed stable release is 1.2.4 on 7 September. Always compare the shipped version separately from the branch inspected. [Release][emdash_release]

## 3. Synara, T3 Code, bb, and DeepSeek: structured work and programmable execution

**Synara** brings provider conversations, handoffs, managed worktrees, review, browser verification and automation into a local-first application. Its direction overlaps heavily with a future Chartr that owns the full development loop. [Product][synara_site]

The handoff code is instructive: it constructs bounded text context, preserves recent messages more generously, summarizes older ones, and omits older material when budgets require it. Tests check omission behavior and character boundaries. This is useful continuity, but not lossless migration of a provider's hidden state. The managed-worktree retention code checks dirty state and uses non-forced removal. [Handoff implementation][synara_handoff], [handoff tests][synara_tests], [worktree retention][synara_worktrees]

For Chartr, a provider switch should produce an inspectable handoff packet: objective, decisions, current change, checks, unresolved risks and relevant transcript excerpts. Label what was carried and what was omitted. Do not promise that changing the CLI gives the next model the same native conversation state. Synara 0.8.3 fixes a packaged dependency problem affecting ACP launch—an example of why adapter and release quality matter as much as a provider logo grid. [Release][synara_release]

**T3 Code** is a serious interface competitor for people who want their coding subscriptions inside an integrated development flow. Its public surface now includes desktop, mobile and remote access, while the source models conversations, commands, checkpoints and thread settlement. [Product][t3_site], [repository][t3_repo]

The settlement policy distinguishes pending approval/input, live work, queued starts, inactivity and PR timestamps. A closed PR older than a later user request must not silently settle fresh work. Tests explicitly cover those cases. The orchestration engine consults persistent command receipts and rejects reuse against a different aggregate; this is substantially more than a chat renderer. Conversely, its plan-progress annotation explicitly lives in memory and is cleared around turn/session lifecycle. That is a narrower mechanism than a durable project dependency plan. [Settlement policy][t3_settlement], [policy tests][t3_tests], [engine][t3_engine], [plan-progress policy][t3_plan]

The implication is twofold: Chartr needs comparable lifecycle clarity, but Wayfinder can address the earlier question of what work should happen and why. Structured chat is not inherently superior to a terminal; the useful abstraction is work with reliable state. The sampled release feed was dominated by nightlies; this audit does not assert that every inspected feature is in the latest stable build. [Releases][t3_releases]

**bb** aims at a programmable IDE and software factory. Hosts, environments, provider sessions, threads and parent/child relationships are explicit entities. Its vision puts agent-driven operation and extension surfaces near the center. [Vision][bb_vision], [system model][bb_system], [schema][bb_schema]

The Git worktree environment is an actual plugin with host-side create/remove handlers. Its implementation has staged progress, ownership checks and Git coordination; tests cover retrying creation without replacing a valid checkout, recovering an earlier attempt, and leaving dirty earlier work alone. The SDK's experimental host contract carries schemas, cancellation, timeouts, resource retention and cleanup. This is an important counterexample to “plugins are tools in panes”: plugins can contribute execution environments and lifecycle behavior. [Environment plugin][bb_worktree_plugin], [worktree implementation][bb_worktrees], [tests][bb_tests], [host contract][bb_contract]

Chartr should borrow the distinction between a work item, a run and an environment, and eventually extend those contracts. It should avoid making “build a software factory” its first user experience. bb is evolving quickly; the reviewed desktop release is 0.42.1 on 5 September, with alpha/platform qualifications in the repository. [Repository][bb_repo], [release][bb_release]

**DeepSeek Harness** is a different layer of competition. It implements the agent runtime itself: scopes, tools, lifecycle, sessions, profiles and plugins. It is not simply another window around an existing CLI. Its developer-preview status and alpha releases matter. [Architecture][deepseek_architecture], [repository][deepseek_site], [alpha release][deepseek_release]

The core session is an append-only event model from which conversation state is derived; persistence is supplied by plugins. The agent inbox records mutations as replayable events and validates identities/coordinates during reconstruction. Scope construction checks ancestry and ties cleanup to lifecycle. Tests cover fork inheritance, projection ordering and cancellation. These are useful lessons in durable execution semantics, not evidence that Chartr should replace its users' agent harnesses. [Session implementation][deepseek_session], [inbox][deepseek_inbox], [inbox tests][deepseek_tests], [scope][deepseek_scope]

Treat DeepSeek Harness as both a watch item and a potential provider. Chartr's advantage should survive users switching to a better harness. Owning model calls, tool loops and an independent agent framework would add a large maintenance burden while moving away from the current bring-your-CLI strength.

## 4. Diri and Unpeel: the closest challenges to Chartr's foundation

**Diri** is the sharpest rebuttal to using “native Rust/GPUI agent workspace” as the differentiator. It combines a compact desktop, terminal agents, status, worktree review, remote access, history/resume and a beta phone companion. Its roadmap explicitly prioritizes continuity, useful integrations, compactness and release quality. It is not merely an Electron task dashboard with a different theme. [Product][diri_site], [roadmap][diri_roadmap]

Three source details deserve attention. First, its canonical status reducer distinguishes hook-primary, screen-primary and process-only evidence, including pending work and staleness. Second, detached holder processes can be adopted instead of relaunching agents; integration-test source simulates losing the session owner and recovering live sessions. Third, worktree cleanup in the app carries the expected HEAD from review into the operation and invalidates stale scans. Those are specific investments in trustworthy supervision and destructive-operation ownership. [Status reducer][diri_status], [holder manager][diri_holders], [holder tests][diri_tests], [worktree review/cleanup][diri_worktrees]

Chartr's Herdr integration is an asset, but session continuity is not exclusive. A useful comparison must distinguish closing the UI, restarting its service, upgrading the PTY owner, process failure and machine reboot. Existing CLI resume data after a crash is different from a live process surviving an application restart.

Diri's reviewed 0.6.3 release includes a fix for background work appearing finished. That does not establish poor reliability; it shows that accurate state is a contested, difficult product feature. Chartr can win on clarity and verification only by testing these cases, not by assuming native code makes them correct. Diri also already treats signed/notarized distribution as a priority. [Release][diri_release], [roadmap][diri_roadmap]

**Unpeel** is building an agent-first terminal system whose native clients attach to a headless Rust host. Its scope includes work beyond coding. The open repository documents runtime integrations, resumption, browser/artifact tools and scoped MCP capabilities; the operated Link service is separate from the open host/client code. [Repository][unpeel_site], [runtime integration][unpeel_runtime], [MCP design][unpeel_mcp]

The PTY owner and serving supervisor are deliberately separate. The supervisor adopts a live core rather than killing it on routine disconnect. The core supports takeover across binary changes, and a real-PTY test is written to verify continuous screens, journals and attached streams during transfer. That test was read, not run. Its MCP gate also checks that a tool is in the granted domain and attached to a valid hosted identity. Pairing code explicitly handles expiring bootstrap material and device credentials. [Supervisor][unpeel_supervisor], [PTY core][unpeel_core], [handoff test][unpeel_tests], [MCP gate][unpeel_gate], [pairing][unpeel_pairing]

Unpeel pressures Chartr on execution independence, reconnect behavior and practical agent controls. Chartr should not claim durability based only on saving layout or restoring scrollback. It should publish precise guarantees for live attachment, compatible upgrades and recovery. The distinctive opportunity remains project intent and acceptance around those terminals; “we also have persistent terminals and MCP” is insufficient.

This pair should be in the most frequent watch tier. They challenge the justification for installing Chartr even before its advanced planning workflow is involved. A user can reasonably prefer a smaller, dependable terminal product plus existing planning tools unless Chartr's integrated context creates a visible advantage.

## 5. Moshi, Cube, Soft Machine, and Superlogical: where work lives

**Moshi** demonstrates that mobile supervision is a product in its own right. Its phone experience emphasizes reliable transport, multiplexer integration, input conveniences and richer agent views through hooks. Current documentation also describes a desktop web interface shipped inside the hook binary, with a client that proxies remote hosts through SSH. It should not remain labeled mobile-only in the tracker. [Product][moshi_site], [desktop architecture][moshi_desktop], [security and sync][moshi_security]

The linked public GitHub repository is a Homebrew distribution tap, not the application's implementation. I found no public application source through the inspected official links and searches. Chartr's immediate lesson is to make the phone useful for understanding what needs attention, supplying input, inspecting a result and continuing later. A smaller copy of the desktop terminal misses much of that value. [Public tap][moshi_repo]

**Cube** combines a desktop surface for local and cloud work with provisioned Linux computers and worktree-oriented workflows. The cloud proposition matters: work on a remote machine can continue while the laptop sleeps. Local persistence by itself cannot make that promise. [Product][cube_site]

Source availability needs careful qualification. Cube links a public Collaborator repository whose inspected snapshot last changed on 16 June. That code has an Electron canvas, agent-accessible tile/terminal RPC and a local PTY sidecar with reconnect support. It demonstrates an earlier implementation and agent control over the interface. It does not substantiate the current Cube cloud platform. The separate Cube releases repository contains distribution material. [Older public implementation][cube_repo], [canvas RPC][cube_canvas], [sidecar protocol][cube_protocol], [release repository][cube_releases]

For Chartr, the lesson is optional execution environments behind a stable work identity. Building a managed-compute business now would introduce provisioning, isolation, billing and operational support before the local workflow is proven. Offer a path to a user's existing host first when demand warrants it.

**Soft Machine** makes the workspace itself a persistent cloud environment that people and agents can manipulate together. Its rendered official landing page describes dedicated Fly.io machines, persistent volumes, cloned environments, shared state, parent/subworkspace coordination and queued/scheduled work. It explicitly calls the product alpha and warns that environments may break or reset. [Product][soft_site]

Its desktop release repository explicitly says application source is private. The public material supports a product-direction comparison, not an implementation audit of those cloud or plugin claims. The useful lesson for Chartr is reproducible experiments and artifact continuity: a prototype should retain how it was created and what it established. Copying an entire cloud development environment and collaboration platform is a different business and should be deferred. [Release/source boundary][soft_repo]

**Superlogical** is the most ambitious long-term overlap: durable sessions across environments, composable interfaces and eventually production operation. Its initial terminal focus is a deliberate entry point into broader work infrastructure. The public site still invites beta signups; Mitchell's current public preview describes remote persistent sessions and remote directory browsing, while Alasdair's posts emphasize interface craft and customization. These are concrete previews, not proof that the whole proposed platform is generally available. [Product][superlogical_site], [remote preview][mitchell_demo], [interface preview][almonk_demo]

The founder's July announcement explains its relationship with the publicly available libghostty building block. No public Superlogical implementation repository was located. Do not infer the eventual product's reliability or architecture from the founders' previous work, and do not reduce the threat to reputation alone: the proposed durable-work model directly overlaps the direction Chartr might choose. [Founder announcement][superlogical_founder]

Chartr should not try to match “all work” or production operations in the next six months. It can earn a narrower position in deciding, executing and accepting development work, with portable planning records and a terminal-first interface. Superlogical's general availability, extensibility and structured work APIs are major reassessment triggers.

## 6. SSHHIP, Git Nimble, and Fork: narrow products worth learning from

**SSHHIP** is a focused iPhone/iPad SSH client for agent and multiplexer work. Its CommandDial makes commands, navigation, snippets and input reachable through a touch-specific interaction. The product describes multiplexer management through a separate SSH exec channel, rather than confusing management commands with typing into the foreground terminal. It supports tmux/Herdr workflows and direct SSH/Mosh transport. [Product][sshhip_site], [official App Store listing][sshhip_store]

No public application source was found through the inspected official links and search. The lesson is interaction economy: a phone needs a small number of reliable, discoverable controls, with clear separation between affecting an agent and affecting the multiplexer. Chartr should prototype attention → inspect → respond → return, and a deliberate handoff of keyboard/geometry ownership. Companion already has useful geometry ownership behavior; build a clear supervision experience around it. This is an adjacent interaction benchmark, not a desktop task orchestrator.

**Git Nimble** is a useful model for the entry point to Chartr's proposed Git plugin. Its native macOS client emphasizes a context-aware Quick Commit action, focused diff/staging, command access, and forge/project integrations. The appeal is reducing the time between an agent making a change and a human inspecting and committing the right material. Its optional AI messaging is secondary to that interaction. The linked public GitHub repository is an update feed, not app source. [Product][nimble_site], [update repository][nimble_repo]

Chartr can do better than merely opening the relevant repository: it can open the change set associated with the active work item, with the originating run, intent and verification already in view. A Quick Commit equivalent should show its repository and selection clearly, preserve partial staging, and never silently sweep unrelated modifications into an agent's commit. This is where the proposed plugin can reinforce Chartr's direction.

**Fork** supplies a much broader Git-client benchmark: graph/history, detailed staging, interactive rebase, conflict resolution, reflog, blame and other repository operations. Its Mac/Windows scope and mature breadth set expectations for users bringing an existing Git tool. The inspected site did not expose a public implementation repository. [Product][fork_site]

Do not make full Fork parity the first Git-plugin milestone. Start with the operations that complete Chartr's daily work loop, and make “open in Fork/Nimble/another client” easy for advanced history work. A thin diff viewer without staging and clear branch ownership will disappoint; a full Git client may consume the roadmap. The useful middle is a dependable, task-aware review and commit experience. No performance or reliability ranking between these clients is asserted here.

## 7. The direction I would choose

There are four plausible strategies. My recommendation is the second, using the first as its quality foundation.

| Strategic choice | Strongest competitive pressure | Assessment for Chartr |
| --- | --- | --- |
| Excellent native agent terminal workspace | Diri, Unpeel; Superlogical previews | Necessary baseline, weak stand-alone distinction. |
| Durable development work from intent through evidence and acceptance | Emdash, Superset, Synara, T3; parts of bb/Paseo | Best fit with Wayfinder and bring-your-CLI. Must prove context continuity is valuable. |
| Cross-device execution platform and plugin ecosystem | Paseo, bb, Unpeel | Design for it, expand only after the local workflow has pull. |
| General cloud and production multiplexer | Cube, Soft Machine, Superlogical's stated direction | Too broad for the proposed near-term focus. |

The initial user should be a developer or small technical team managing several related pieces of work with CLI agents, already feeling the cost of recovering context and reviewing output. Do not begin by promising unattended fleets, a universal IDE, or production incident tooling.

The proposed promise is: **“Keep the purpose, progress, and proof of agent work together.”** A concrete demonstration should be stronger than the slogan: reopen yesterday's work, see the decision that constrained it, inspect the resulting change, understand the failed check, send a targeted follow-up to a different agent, and accept the result without reconstructing the story from terminals.

The durable object model should be small:

- **Work item:** objective, scope, dependencies, decisions, owner and acceptance criteria. It may be a research question, experiment, implementation or review; not every useful outcome is a commit.
- **Run:** a particular provider/session attempt, its instructions and context provenance, observed status, attention events, environment and exit/recovery facts. One work item can have several runs.
- **Environment/change set:** repository identity, checkout/worktree, base revision, current changes and ownership. Several related runs may collaborate deliberately; independent write work should receive isolation.
- **Evidence and acceptance:** artifacts, checks, reviewed revision, result, unresolved concerns and the human acceptance decision. An answer is an artifact; a finished turn is a runtime event; acceptance is its own action.

The map should show what is known, what is blocked, and what evidence permits the next step. Keep a compact list and attention queue as equally capable views. A constellation earns its place when it exposes dependencies and uncertainty faster than a list; it should not require users to navigate space just to find the next blocked agent. Preserve the visual character, but spend the next design effort on comprehension and actions rather than additional atmosphere.

Execution should remain easy without a formal plan. A user can start a terminal or agent immediately, then promote useful work into a named item. For planned work, infer sensible defaults and expose the prepared launch packet when wanted. Avoid making users fill in project-management forms before asking a small coding question.

An evidence workflow must distinguish “the command passed” from “the desired behavior is established.” Record the command, environment, revision and output, and allow a human to judge sufficiency. Do not require every research item to produce tests or force every change through an artificial ceremony. Make the default lightweight and the provenance available when it matters.

This direction is not unique because competitors lack tasks, history, checkpoints or plans—they demonstrably have them. The potential distinction is a well-executed combination of portable decision/dependency records, real terminal workflows, transparent handoffs and acceptance evidence. If users do not value that combination enough to choose Chartr, narrow toward an excellent lightweight terminal product instead of accumulating more features around an unproven thesis.

## 8. Worktrees and the Git plugin must join the workflow

Do not simply remove Wayfinder's one-claim-per-space rule. That rule currently protects a shared checkout. A safe next step needs explicit checkout ownership and coordination. A Git worktree isolates tracked changes, but it does not automatically isolate ports, databases, secrets, caches or external side effects. Setup/teardown hooks should declare relevant shared resources, with bounded concurrency rather than an impressive agent-count claim.

Repository identity must be different from checkout identity. If each worktree becomes an unrelated space, dependencies, work history and user attention fragment. Likewise, copying `.plan` into every branch cannot create independent authoritative claim files: separate checkout-local locks would allow contradictory claims. Keep one local coordinator and ownership ledger per repository, while retaining portable Markdown intent and exporting the evidence needed to understand the work elsewhere. Define how external edits are reconciled and how a second host acquires or refuses ownership before adding distributed writes.

A first-party Git plugin should ship enabled as part of the default development experience. Its implementation can remain modular; responsibility for the core user journey remains Chartr's. The first version should cover:

1. Repository/worktree/base visibility and a live changes view associated with the current work item.
2. File and hunk review, staging/unstaging, conflict visibility, and explicit commit scope. Preserve users' unrelated and partially staged changes.
3. Check output and artifacts next to the exact change reviewed. If HEAD or relevant file contents change, mark the evidence/review stale.
4. Commit and PR handoff with human-readable summaries and links back to intent and runs. Allow ordinary terminal Git and external clients without losing synchronization.
5. Safe archive/cleanup that separates hiding work, stopping a run, deleting a worktree and deleting a branch. Revalidate ownership, current HEAD and dirty state at action time; preserve recoverable history.

Nimble suggests the fast doorway; Fork supplies expectations for Git correctness; Emdash, Diri and Synara provide concrete lifecycle and cleanup examples. Advanced rebase editing, broad forge parity and every repository-history tool can initially open externally. A review surface should expose enough detail to detect mistakes, rather than presenting an AI summary as proof of correctness.

This also changes the plugin roadmap. Current Chartr web plugins contribute panes with scoped host capabilities; native services already show how stronger contracts can work. Extend around stable work/run identifiers, environment provisioning, attention events, artifacts and review contributions when the first-party workflow needs them. Do not immediately expose an unbounded internal API. [Current plugin model][chartr_plugins]

Paseo's contribution registry and bb's environment/host contracts are useful references for breadth and lifecycle, respectively. Develop one or two concrete extensions—Git and verification/browser evidence—against the same contracts, then decide what is stable enough to publish. A third-party ecosystem needs version compatibility, cleanup, failure isolation and permission clarity, not just an install dialog. The current display-only plugin version and process/session grants must be described accurately; process permission carries the user's execution authority. [Paseo registry][paseo_plugins_code], [bb host contract][bb_contract], [Chartr permissions/versioning][chartr_plugins]

## 9. Sequence the next work around a complete demonstration

**First, make the baseline distributable and comprehensible.** Finish a signed/notarized macOS installation/update path and a tested Linux package path consistent with supported platforms. Clarify the legacy/rewrite download distinction. Detect installed CLI agents and offer editable presets; let users launch before understanding plugin setup. Give runs durable intent-based labels and add a compact attention view over Herdr's existing states. Gate general Companion distribution on authenticated pairing and verified identity. These steps improve adoption and trust independently of the larger strategy.

**Then build one vertical slice:** open a repository → name a work item → launch with inspectable context → use an owned worktree when needed → see an attention event → review its diff and evidence → accept or send a follow-up → reopen it tomorrow with its history intact. Use two well-supported provider integrations plus a generic terminal fallback. Publish which capabilities each adapter supports—launch, reliable status, native resume, structured approval, handoff—instead of implying uniform integration from a list of names.

**After that, connect Wayfinder to execution.** Add multiple runs per item, explicit verification/acceptance state, and a dependency queue whose items become runnable through accepted prerequisites. Keep manual launch and bounded concurrency. Add failure, cancellation and stale-claim recovery that a person can understand. Research outcomes can satisfy dependencies without a Git change; engineering outcomes should retain the reviewed change and relevant evidence. Only then expand coordinated parallel work.

**Next, make continuation across devices useful.** Companion should show the same named work, attention reasons, relevant output and evidence. Reconnect should explain whether a live process was reattached or a new run resumed prior context. If users need work to remain reachable after the desktop window closes, move the appropriate control service behind a headless boundary. Local host sleep and remote execution must remain different promises.

**Finally, open the proven extension contracts and optional remote-host path.** Use observed needs to choose which environment, provider and evidence contracts become public. Defer managed compute, production operations, a marketplace, a new model runtime, and full Fork parity. Do not schedule another broad shell rewrite unless the vertical slice demonstrates a concrete architectural blocker.

The most useful acceptance tests for this direction are user scenarios, not a feature total:

| Experiment | What to measure | Proposed decision rule, not an observed result |
| --- | --- | --- |
| Fresh install → first useful agent run | Completion without help; time spent configuring | At least 8 of 10 target users succeed unaided; investigate every setup failure. |
| Return to 5–10 mixed runs after a day | Time to identify purpose, blockers and next action | Compare with their existing tool; aim to halve context-recovery time. |
| Review a seeded change with known mistakes | Missed defects, unrelated staging, time to decision | Improve review time without increasing missed defects; any silent loss blocks release. |
| Switch providers midway through a task | Missing decisions, repeated investigation, time to useful follow-up | Show a material reduction in repeated work versus copy/paste, with explicit omissions. |
| Disconnect, close UI, restart service, upgrade, crash | Live continuity vs documented resume; duplicate launches, stale claims | No silent loss or duplicate execution in the release scenario suite; report each guarantee separately. |
| A two-week real-work pilot | Voluntary daily use and which workflow drives it | Continue the broader work model only if participants repeatedly use context/review, not just terminal tabs. |

The small pilot is directional evidence, not a population estimate. Recruit people already switching among terminal agents and Git tools. Observe existing workflows before prescribing ours. A competitive hands-on pass should concentrate on Diri/Unpeel for terminal and continuity work, and Emdash/Paseo/Superset or Synara for the complete task flow. Compare on the same repository and machine where practical.

Measure total resources across the app, daemon, sidecar and any webviews, along with input latency, idle load, reconnect time and a realistic active-session workload. Do not compare a single Rust process against an entire Electron process tree or treat language choice as a benchmark. No measurements of competitor speed, memory use or reliability were made in this research.

## 10. Tracking and review protocol

All 16 products remain in the register, including the adjacent mobile and Git tools. A lower immediate priority means a narrower reason to watch, not removal from the competitor set.

| Watch tier | Products | Main reason to revisit |
| --- | --- | --- |
| Closest substitution | Diri, Unpeel | Native terminal quality, state accuracy, continuity and review. |
| Core workflow | Superset, Paseo, Emdash, Synara, T3 Code | Task/environment lifecycle, provider integration, review, mobile and extension changes. |
| Architecture and broader direction | bb, DeepSeek Harness, Cube, Soft Machine, Superlogical | Execution contracts, autonomy, environment ownership and market expansion. |
| Focused interaction references | Moshi, SSHHIP, Git Nimble, Fork | Mobile supervision/input and efficient, dependable Git interaction. |

Proposed cadence: a short weekly release/source delta check for the closest and core-workflow tiers, and a monthly review of the remainder; check any major launch immediately. First proposed review: 16 September 2026. This is a recorded maintenance protocol, not an active background monitoring service.

Each update should record product ID, date checked, source URL, exact commit or release where applicable, what changed, its availability level (stable/beta/source/announced), confidence, and the implication for a Chartr decision. Avoid activity-only posts that merely count stars or commits. Link the previous product entry with a Slopchan `>>postId` reference. If an earlier claim was wrong, correct it explicitly in a reply; forum posts are immutable.

Key triggers: Diri or Unpeel materially deepening planning/review; Paseo stabilizing its broader plugin surface; Superset shipping managed cloud; Emdash or Synara linking verification to acceptance; T3 adding durable project planning; bb stabilizing execution extension contracts; DeepSeek changing provider integration requirements; Cube publishing current source; Soft Machine leaving alpha; Superlogical opening its beta, source or structured APIs; Moshi/SSHHIP improving actionable agent supervision; Nimble/Fork adding task-aware review or agent integration.

Reassess our recommendation if competitors deliver the proposed intent-to-evidence workflow with little friction, if users prefer unstructured terminal operation and ignore Wayfinder, or if maintaining provider semantics dominates engineering effort. Those outcomes would call for narrowing, interoperability or a different entry point—not a larger feature checklist.

The durable artifacts accompanying this report are a machine-readable competitor register and a pinned source-audit ledger. The Slopchan thread is the discussion and delta log. Neither implies that competitor tests were run, source was exhaustively audited, or ongoing checks have been scheduled.

<!-- Reference sources -->

[almonk_demo]: https://x.com/almonk/status/2097439320076403125
[almonk_profile]: https://x.com/almonk
[bb_contract]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/packages/plugin-sdk/src/host-contract.ts
[bb_release]: https://github.com/get-bb/bb/releases/tag/desktop-v0.42.1
[bb_repo]: https://github.com/get-bb/bb
[bb_schema]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/packages/db/src/schema.ts
[bb_site]: https://github.com/get-bb/bb
[bb_system]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/docs/system-overview.md
[bb_tests]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/plugins/environment-git-worktree/host.test.ts
[bb_vision]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/docs/VISION.md
[bb_worktree_plugin]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/plugins/environment-git-worktree/host.ts
[bb_worktrees]: https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/plugins/environment-git-worktree/host/worktree.ts
[chartr_chrome]: /Users/rengwu/Desktop/Projects/chartr/crates/chartr/src/chrome.rs:432
[chartr_companion]: /Users/rengwu/Desktop/Projects/chartr/plugins/companion/README.md
[chartr_companion_code]: /Users/rengwu/Desktop/Projects/chartr/crates/chartr-companion/src/lib.rs
[chartr_companion_protocol]: /Users/rengwu/Desktop/Projects/chartr/docs/companion-protocol.md
[chartr_control]: /Users/rengwu/Desktop/Projects/chartr/crates/chartr-herdr/src/control.rs:35
[chartr_plugins]: /Users/rengwu/Desktop/Projects/chartr/docs/plugins.md
[chartr_prompts]: /Users/rengwu/Desktop/Projects/chartr/plugins/prompts/README.md
[chartr_readme]: /Users/rengwu/Desktop/Projects/chartr/README.md
[chartr_wayfinder]: /Users/rengwu/Desktop/Projects/chartr/plugins/wayfinder/README.md:50
[chartr_wayfinder_model]: /Users/rengwu/Desktop/Projects/chartr/plugins/wayfinder/src/model.rs:240
[cube_canvas]: https://github.com/collabs-inc/collab-public/blob/476b8efc942ee5f430a9b8bf832b8560a8cf76c2/collab-electron/src/main/canvas-rpc.ts
[cube_protocol]: https://github.com/collabs-inc/collab-public/blob/476b8efc942ee5f430a9b8bf832b8560a8cf76c2/collab-electron/src/main/sidecar/protocol.ts
[cube_releases]: https://github.com/collabs-inc/cube-releases/blob/c765d8eebcde47069c4b02129abcf902f836db88/README.md
[cube_repo]: https://github.com/collabs-inc/collab-public/blob/476b8efc942ee5f430a9b8bf832b8560a8cf76c2/README.md
[cube_site]: https://cube.computer/
[deepseek_architecture]: https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/docs/architecture.md
[deepseek_inbox]: https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/agent-loop/src/inbox.ts
[deepseek_release]: https://github.com/deepseek-ai/deepseek-harness/releases/tag/dsh-v0.1.5-alpha.1
[deepseek_scope]: https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/scope/src/index.ts
[deepseek_session]: https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/session/src/index.ts
[deepseek_site]: https://github.com/deepseek-ai/deepseek-harness
[deepseek_tests]: https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/agent-loop/tests/inbox.spec.ts
[diri_holders]: https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-engine/src/holder/manager.rs
[diri_release]: https://github.com/cristicretu/diri/releases/tag/v0.6.3
[diri_roadmap]: https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/ROADMAP.md
[diri_site]: https://diri.sh/
[diri_status]: https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-engine/src/status/mod.rs
[diri_tests]: https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-engine/tests/holder_session.rs
[diri_worktrees]: https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-app/src/worktrees.rs
[emdash_plugin_host]: https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/packages/core/src/services/agent-plugins/api/plugins/plugin-host.ts
[emdash_providers]: https://emdash.com/docs/providers
[emdash_release]: https://github.com/generalaction/emdash/releases/tag/v1.2.4
[emdash_review]: https://emdash.com/docs/diff-view
[emdash_schema]: https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/apps/emdash-desktop/src/core/services/app-db/node/schema.ts
[emdash_site]: https://emdash.com/
[emdash_task_service]: https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/apps/emdash-desktop/src/core/features/tasks/api/node/task-service.ts
[emdash_tasks]: https://emdash.com/docs/tasks
[emdash_tests]: https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/packages/core/src/runtimes/workspace-registry/node/create-worktree.test.ts
[emdash_worktrees]: https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/packages/core/src/runtimes/workspace-registry/node/create-worktree.ts
[fork_site]: https://git-fork.com/
[mitchell_demo]: https://x.com/mitchellh/status/2097424868203758046
[mitchell_profile]: https://x.com/mitchellh
[moshi_desktop]: https://getmoshi.app/docs/install-desktop
[moshi_repo]: https://github.com/rjyo/homebrew-moshi/blob/a65d0c51e9664a34b088fff5591d40ba7e9761b5/README.md
[moshi_security]: https://getmoshi.app/docs/security-sync
[moshi_site]: https://getmoshi.app/
[nimble_repo]: https://github.com/RobSwish/nimble-updates/blob/33b43f462cc79038605c91c60eeb1cd9892fa00d/README.md
[nimble_site]: https://gitnimble.com/
[paseo_plugins_code]: https://github.com/getpaseo/paseo/blob/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0/packages/app/src/plugins/registry.ts
[paseo_plugins_docs]: https://paseo.sh/docs/plugins
[paseo_release]: https://github.com/getpaseo/paseo/releases/tag/v0.8.0-beta.1
[paseo_security]: https://paseo.sh/docs/security
[paseo_site]: https://paseo.sh/
[paseo_tests]: https://github.com/getpaseo/paseo/blob/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0/packages/app/e2e/browser/worktree-restore-after-restart.spec.ts
[paseo_worktrees]: https://github.com/getpaseo/paseo/blob/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0/packages/server/src/server/paseo-worktree-service.ts
[soft_repo]: https://github.com/Soft-Machine-io/desktop-releases/blob/6ec22ff3d91bc13fd67223179f1848d2f743e737/README.md
[soft_site]: https://soft-machine.io/landing
[sshhip_site]: https://sshhip.com/
[sshhip_store]: https://apps.apple.com/us/app/sshhip/id6785186457
[superlogical_founder]: https://mitchellh.com/writing/superlogical
[superlogical_site]: https://www.superlogical.com/
[superset_daemon]: https://github.com/superset-sh/superset/blob/397c4c2bcab279e694271f4311725cff46865118/apps/desktop/src/main/lib/terminal/daemon/daemon-manager.ts
[superset_release]: https://github.com/superset-sh/superset/releases/tag/desktop-v1.27.0
[superset_repo]: https://github.com/superset-sh/superset
[superset_schema]: https://github.com/superset-sh/superset/blob/397c4c2bcab279e694271f4311725cff46865118/packages/host-service/src/db/schema.ts
[superset_site]: https://superset.sh/
[superset_tests]: https://github.com/superset-sh/superset/blob/397c4c2bcab279e694271f4311725cff46865118/apps/desktop/src/main/lib/terminal/daemon/daemon-manager.test.ts
[synara_handoff]: https://github.com/Emanuele-web04/synara/blob/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be/apps/server/src/orchestration/handoff.ts
[synara_release]: https://github.com/Emanuele-web04/synara/releases/tag/v0.8.3
[synara_site]: https://www.trysynara.com/
[synara_tests]: https://github.com/Emanuele-web04/synara/blob/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be/apps/server/src/orchestration/handoff.test.ts
[synara_worktrees]: https://github.com/Emanuele-web04/synara/blob/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be/apps/server/src/managedWorktrees.ts
[t3_engine]: https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/Layers/OrchestrationEngine.ts
[t3_plan]: https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/ThreadPlanProgress.ts
[t3_releases]: https://github.com/pingdotgg/t3code/releases
[t3_repo]: https://github.com/pingdotgg/t3code
[t3_settlement]: https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/ThreadSettlementPolicy.ts
[t3_site]: https://t3.codes/
[t3_tests]: https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/ThreadSettlementPolicy.test.ts
[unpeel_core]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-core/src/pty_core.rs
[unpeel_gate]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-core/src/mcp_gate.rs
[unpeel_mcp]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/docs/agents/sessions-mcp.md
[unpeel_pairing]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-serve/src/pairing.rs
[unpeel_runtime]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/runtimes/README.md
[unpeel_site]: https://github.com/unpeel-com/unpeel
[unpeel_supervisor]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-serve/src/pty_core_supervisor.rs
[unpeel_tests]: https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-cli/tests/cases/pty_core_handoff.py
