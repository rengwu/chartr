# Source audit: Chartr competitor research

Snapshot: 9 September 2026. Chartr commit: `5402a35454c17bea0dead3437b264e7e2801dd2e`.

This is a targeted source review, not an exhaustive audit. Nine current public product implementations and one historical implementation were examined. Test source was read for all nine current repositories; no competitor tests or applications were executed. A test documents an intended invariant, not an independently verified passing result. Documentation and release evidence are kept distinct from implementation.

The linked implementation files below are the subset used for the report’s conclusions. Larger files were read through selected relevant paths. Repository trees were retrieved at the recorded commits without truncation. Release/packaging repositories do not count as application source.

## Superset

Source boundary: **current-source-available**. source-backed selected implementation.

Repository: [superset](https://github.com/superset-sh/superset). Branch `main`; commit [`397c4c2bcab2`](https://github.com/superset-sh/superset/commit/397c4c2bcab279e694271f4311725cff46865118); commit date 2026-09-09T00:55:05Z.

[Release evidence](https://github.com/superset-sh/superset/releases/tag/desktop-v1.27.0). Stable/beta/source distinctions are recorded in the report.

- [packages/host-service/src/db/schema.ts](https://github.com/superset-sh/superset/blob/397c4c2bcab279e694271f4311725cff46865118/packages/host-service/src/db/schema.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [apps/desktop/src/main/lib/terminal/daemon/daemon-manager.ts](https://github.com/superset-sh/superset/blob/397c4c2bcab279e694271f4311725cff46865118/apps/desktop/src/main/lib/terminal/daemon/daemon-manager.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [apps/desktop/src/main/lib/terminal/daemon/daemon-manager.test.ts](https://github.com/superset-sh/superset/blob/397c4c2bcab279e694271f4311725cff46865118/apps/desktop/src/main/lib/terminal/daemon/daemon-manager.test.ts) — Selected test cases and assertions inspected; tests not run.

## Paseo

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [paseo](https://github.com/getpaseo/paseo). Branch `main`; commit [`da8c1b5c94e7`](https://github.com/getpaseo/paseo/commit/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0); commit date 2026-09-08T11:40:21Z.

Inspected 0.8.0-beta.1, published 2026-09-08; stable 0.7.2 observed 2026-09-02. Broader plugin API is experimental/beta.

[Release evidence](https://github.com/getpaseo/paseo/releases/tag/v0.8.0-beta.1). Stable/beta/source distinctions are recorded in the report.

- [packages/server/src/server/paseo-worktree-service.ts](https://github.com/getpaseo/paseo/blob/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0/packages/server/src/server/paseo-worktree-service.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [packages/app/e2e/browser/worktree-restore-after-restart.spec.ts](https://github.com/getpaseo/paseo/blob/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0/packages/app/e2e/browser/worktree-restore-after-restart.spec.ts) — Selected test cases and assertions inspected; tests not run.
- [packages/app/src/plugins/registry.ts](https://github.com/getpaseo/paseo/blob/da8c1b5c94e752b01d451645e5fa52aba2c1b2f0/packages/app/src/plugins/registry.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.

## Emdash

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [emdash](https://github.com/generalaction/emdash). Branch `main`; commit [`9eb050ca7a18`](https://github.com/generalaction/emdash/commit/9eb050ca7a184b1411a7f3a1e9705fb1632a7253); commit date 2026-09-08T10:27:31Z.

[Release evidence](https://github.com/generalaction/emdash/releases/tag/v1.2.4). Stable/beta/source distinctions are recorded in the report.

- [apps/emdash-desktop/src/core/features/tasks/api/node/task-service.ts](https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/apps/emdash-desktop/src/core/features/tasks/api/node/task-service.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [apps/emdash-desktop/src/core/services/app-db/node/schema.ts](https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/apps/emdash-desktop/src/core/services/app-db/node/schema.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [packages/core/src/runtimes/workspace-registry/node/create-worktree.ts](https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/packages/core/src/runtimes/workspace-registry/node/create-worktree.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [packages/core/src/runtimes/workspace-registry/node/create-worktree.test.ts](https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/packages/core/src/runtimes/workspace-registry/node/create-worktree.test.ts) — Selected test cases and assertions inspected; tests not run.
- [packages/core/src/services/agent-plugins/api/plugins/plugin-host.ts](https://github.com/generalaction/emdash/blob/9eb050ca7a184b1411a7f3a1e9705fb1632a7253/packages/core/src/services/agent-plugins/api/plugins/plugin-host.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.

## Synara

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [synara](https://github.com/Emanuele-web04/synara). Branch `main`; commit [`4bb3dccfa2cf`](https://github.com/Emanuele-web04/synara/commit/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be); commit date 2026-09-08T20:56:35Z.

[Release evidence](https://github.com/Emanuele-web04/synara/releases/tag/v0.8.3). Stable/beta/source distinctions are recorded in the report.

- [apps/server/src/orchestration/handoff.ts](https://github.com/Emanuele-web04/synara/blob/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be/apps/server/src/orchestration/handoff.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [apps/server/src/orchestration/handoff.test.ts](https://github.com/Emanuele-web04/synara/blob/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be/apps/server/src/orchestration/handoff.test.ts) — Selected test cases and assertions inspected; tests not run.
- [apps/server/src/managedWorktrees.ts](https://github.com/Emanuele-web04/synara/blob/4bb3dccfa2cfdafb432bc4cdbc474921a379f6be/apps/server/src/managedWorktrees.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.

## T3 Code

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [t3code](https://github.com/pingdotgg/t3code). Branch `main`; commit [`08463e2c401c`](https://github.com/pingdotgg/t3code/commit/08463e2c401ce87858aaaebcb70ed86fb002fb5f); commit date 2026-09-09T02:27:05Z.

Sampled top three releases were nightlies; latest stable version not established by this audit.

[Release evidence](https://github.com/pingdotgg/t3code/releases). Stable/beta/source distinctions are recorded in the report.

- [apps/server/src/orchestration/ThreadSettlementPolicy.ts](https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/ThreadSettlementPolicy.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [apps/server/src/orchestration/ThreadSettlementPolicy.test.ts](https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/ThreadSettlementPolicy.test.ts) — Selected test cases and assertions inspected; tests not run.
- [apps/server/src/orchestration/Layers/OrchestrationEngine.ts](https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/Layers/OrchestrationEngine.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [apps/server/src/orchestration/ThreadPlanProgress.ts](https://github.com/pingdotgg/t3code/blob/08463e2c401ce87858aaaebcb70ed86fb002fb5f/apps/server/src/orchestration/ThreadPlanProgress.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.

## bb

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [bb](https://github.com/get-bb/bb). Branch `main`; commit [`a3ac7a5025f1`](https://github.com/get-bb/bb/commit/a3ac7a5025f1e7fac927313c928d5add5e90f800); commit date 2026-09-09T01:22:31Z.

[Release evidence](https://github.com/get-bb/bb/releases/tag/desktop-v0.42.1). Stable/beta/source distinctions are recorded in the report.

- [packages/db/src/schema.ts](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/packages/db/src/schema.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [plugins/environment-git-worktree/host.ts](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/plugins/environment-git-worktree/host.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [plugins/environment-git-worktree/host/worktree.ts](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/plugins/environment-git-worktree/host/worktree.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [plugins/environment-git-worktree/host.test.ts](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/plugins/environment-git-worktree/host.test.ts) — Selected test cases and assertions inspected; tests not run.
- [packages/plugin-sdk/src/host-contract.ts](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/packages/plugin-sdk/src/host-contract.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [docs/VISION.md](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/docs/VISION.md) — Repository documentation or distribution boundary inspected.
- [docs/system-overview.md](https://github.com/get-bb/bb/blob/a3ac7a5025f1e7fac927313c928d5add5e90f800/docs/system-overview.md) — Repository documentation or distribution boundary inspected.

## DeepSeek Harness

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness). Branch `master`; commit [`5dda764ed3aa`](https://github.com/deepseek-ai/deepseek-harness/commit/5dda764ed3aa172535a7967b06ff95d9cbfe536a); commit date 2026-09-08T15:25:45Z.

Developer preview; dsh-v0.1.5-alpha.1 prerelease observed.

[Release evidence](https://github.com/deepseek-ai/deepseek-harness/releases/tag/dsh-v0.1.5-alpha.1). Stable/beta/source distinctions are recorded in the report.

- [packages/core/session/src/index.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/session/src/index.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [packages/core/agent-loop/src/inbox.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/agent-loop/src/inbox.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [packages/core/agent-loop/tests/inbox.spec.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/agent-loop/tests/inbox.spec.ts) — Selected test cases and assertions inspected; tests not run.
- [packages/core/scope/src/index.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/packages/core/scope/src/index.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [docs/architecture.md](https://github.com/deepseek-ai/deepseek-harness/blob/5dda764ed3aa172535a7967b06ff95d9cbfe536a/docs/architecture.md) — Repository documentation or distribution boundary inspected.

## Moshi

Source boundary: **packaging-only**. official product claims; current implementation not audited.

Repository: [homebrew-moshi](https://github.com/rjyo/homebrew-moshi). Branch `main`; commit [`a65d0c51e966`](https://github.com/rjyo/homebrew-moshi/commit/a65d0c51e9664a34b088fff5591d40ba7e9761b5); commit date 2026-09-08T16:55:48Z.

- [README.md](https://github.com/rjyo/homebrew-moshi/blob/a65d0c51e9664a34b088fff5591d40ba7e9761b5/README.md) — Repository documentation or distribution boundary inspected.

## Cube

Source boundary: **historical-public-implementation**. official product claims; current implementation not audited.

Repository: [collab-public](https://github.com/collabs-inc/collab-public). Branch `main`; commit [`476b8efc942e`](https://github.com/collabs-inc/collab-public/commit/476b8efc942ee5f430a9b8bf832b8560a8cf76c2); commit date 2026-06-16T12:01:14Z.

June 16 Collaborator snapshot is historical evidence only; separate cube-releases repo contains distribution material, not current cloud app source.

- [collab-electron/src/main/canvas-rpc.ts](https://github.com/collabs-inc/collab-public/blob/476b8efc942ee5f430a9b8bf832b8560a8cf76c2/collab-electron/src/main/canvas-rpc.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [collab-electron/src/main/sidecar/protocol.ts](https://github.com/collabs-inc/collab-public/blob/476b8efc942ee5f430a9b8bf832b8560a8cf76c2/collab-electron/src/main/sidecar/protocol.ts) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [README.md](https://github.com/collabs-inc/collab-public/blob/476b8efc942ee5f430a9b8bf832b8560a8cf76c2/README.md) — Repository documentation or distribution boundary inspected.
- [README.md](https://github.com/collabs-inc/cube-releases/blob/c765d8eebcde47069c4b02129abcf902f836db88/README.md) — Repository documentation or distribution boundary inspected.

## Soft Machine

Source boundary: **private-app-release-feed**. official product claims; current implementation not audited.

Repository: [desktop-releases](https://github.com/Soft-Machine-io/desktop-releases). Branch `main`; commit [`6ec22ff3d91b`](https://github.com/Soft-Machine-io/desktop-releases/commit/6ec22ff3d91bc13fd67223179f1848d2f743e737); commit date 2026-09-09T01:33:45Z.

Rendered official landing page inspected in browser; release README says app source private. Alpha claims not independently tested.

- [README.md](https://github.com/Soft-Machine-io/desktop-releases/blob/6ec22ff3d91bc13fd67223179f1848d2f743e737/README.md) — Repository documentation or distribution boundary inspected.

## Superlogical

Source boundary: **no-public-implementation-located**. official product claims; current implementation not audited.

Site invites beta signups; public preview post text inspected. Embedded demo video not independently evaluated.

- [Official product material](https://www.superlogical.com/); no app implementation claim is made.

## Diri

Source boundary: **current-open-source**. source-backed selected implementation.

Repository: [diri](https://github.com/cristicretu/diri). Branch `main`; commit [`c564784199cf`](https://github.com/cristicretu/diri/commit/c564784199cfbeabd29f011bac28467a2b12fccf); commit date 2026-09-08T19:05:28Z.

[Release evidence](https://github.com/cristicretu/diri/releases/tag/v0.6.3). Stable/beta/source distinctions are recorded in the report.

- [diri/crates/diri-engine/src/status/mod.rs](https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-engine/src/status/mod.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [diri/crates/diri-engine/src/holder/manager.rs](https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-engine/src/holder/manager.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [diri/crates/diri-engine/tests/holder_session.rs](https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-engine/tests/holder_session.rs) — Selected test cases and assertions inspected; tests not run.
- [diri/crates/diri-app/src/worktrees.rs](https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/diri/crates/diri-app/src/worktrees.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [ROADMAP.md](https://github.com/cristicretu/diri/blob/c564784199cfbeabd29f011bac28467a2b12fccf/ROADMAP.md) — Repository documentation or distribution boundary inspected.

## SSHHIP

Source boundary: **no-public-implementation-located**. official product claims; current implementation not audited.

- [Official product material](https://sshhip.com/); no app implementation claim is made.

## Unpeel

Source boundary: **open-host-clients-closed-link-service**. source-backed selected implementation.

Repository: [unpeel](https://github.com/unpeel-com/unpeel). Branch `main`; commit [`a058275f1ff4`](https://github.com/unpeel-com/unpeel/commit/a058275f1ff4433e73e477109a135276bacaf17f); commit date 2026-09-08T22:08:58Z.

GitHub releases API returned no entries; this is not evidence of no distribution. Core source and docs describe 0.4.4-era behavior.

- [crates/unpeel-serve/src/pty_core_supervisor.rs](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-serve/src/pty_core_supervisor.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [crates/unpeel-core/src/pty_core.rs](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-core/src/pty_core.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [crates/unpeel-cli/tests/cases/pty_core_handoff.py](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-cli/tests/cases/pty_core_handoff.py) — Selected test cases and assertions inspected; tests not run.
- [crates/unpeel-core/src/mcp_gate.rs](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-core/src/mcp_gate.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [crates/unpeel-serve/src/pairing.rs](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/crates/unpeel-serve/src/pairing.rs) — Selected implementation paths inspected; not a whole-file or whole-repository audit.
- [runtimes/README.md](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/runtimes/README.md) — Repository documentation or distribution boundary inspected.
- [docs/agents/sessions-mcp.md](https://github.com/unpeel-com/unpeel/blob/a058275f1ff4433e73e477109a135276bacaf17f/docs/agents/sessions-mcp.md) — Repository documentation or distribution boundary inspected.

## Git Nimble

Source boundary: **packaging-only**. official product claims; current implementation not audited.

Repository: [nimble-updates](https://github.com/RobSwish/nimble-updates). Branch `main`; commit [`33b43f462cc7`](https://github.com/RobSwish/nimble-updates/commit/33b43f462cc79038605c91c60eeb1cd9892fa00d); commit date 2026-06-26T03:09:14Z.

- [README.md](https://github.com/RobSwish/nimble-updates/blob/33b43f462cc79038605c91c60eeb1cd9892fa00d/README.md) — Repository documentation or distribution boundary inspected.

## Fork

Source boundary: **no-public-implementation-located**. official product claims; current implementation not audited.

- [Official product material](https://git-fork.com/); no app implementation claim is made.

## Chartr evidence

The local checkout, documentation, Wayfinder model/launch code, Herdr status model, chrome, plugin contracts, and Companion protocol/server were inspected. The running development UI was observed read-only. Screenshots and unrelated terminal contents are not included in the published research.

- [README.md](/Users/rengwu/Desktop/Projects/chartr/README.md) — local commit `5402a35454c1`.
- [plugins/wayfinder/README.md](/Users/rengwu/Desktop/Projects/chartr/plugins/wayfinder/README.md:50) — local commit `5402a35454c1`.
- [crates/chartr-herdr/src/control.rs](/Users/rengwu/Desktop/Projects/chartr/crates/chartr-herdr/src/control.rs:35) — local commit `5402a35454c1`.
- [crates/chartr/src/chrome.rs](/Users/rengwu/Desktop/Projects/chartr/crates/chartr/src/chrome.rs:432) — local commit `5402a35454c1`.
- [plugins/wayfinder/src/model.rs](/Users/rengwu/Desktop/Projects/chartr/plugins/wayfinder/src/model.rs:240) — local commit `5402a35454c1`.
- [plugins/prompts/README.md](/Users/rengwu/Desktop/Projects/chartr/plugins/prompts/README.md) — local commit `5402a35454c1`.
- [docs/plugins.md](/Users/rengwu/Desktop/Projects/chartr/docs/plugins.md) — local commit `5402a35454c1`.
- [docs/companion-protocol.md](/Users/rengwu/Desktop/Projects/chartr/docs/companion-protocol.md) — local commit `5402a35454c1`.
- [plugins/companion/README.md](/Users/rengwu/Desktop/Projects/chartr/plugins/companion/README.md) — local commit `5402a35454c1`.
- [crates/chartr-companion/src/lib.rs](/Users/rengwu/Desktop/Projects/chartr/crates/chartr-companion/src/lib.rs) — local commit `5402a35454c1`.

## Reproducibility and limits

- Code links use pinned commits. Dynamic product pages and release feeds can change after the snapshot.
- Public absence means no implementation was located through inspected official links and searches; it does not prove none exists anywhere.
- Superlogical public post text was inspected in the native browser; the embedded video was not benchmarked or fully evaluated.
- Soft Machine’s rendered landing page was inspected because static extraction yielded its JavaScript shell.
- Source architecture does not establish end-user reliability, security certification, feature parity, adoption, revenue or performance.
- The companion JSON ledger contains citation IDs, URLs, repository commits and reviewed file paths. The competitor register records watch triggers and proposed review dates; no automated monitoring has been installed.
