# Documentation audit — 17 September 2026

Reviewed the current `rewrite/rust` working tree, including its pre-existing
uncommitted changes. The scope was 44 existing Markdown documents: the root
guide, current references, plugin/example/tool/asset guides, hidden workspace
plan, ADRs, research notes, and maintained vendor guidance (including the retained
upstream terminal README). Publisher license texts and historical JSON receipts
were preserved. The new [documentation index](../README.md) separates current
guidance, design contracts, and historical evidence.

## Corrections

| Area | Corrected guidance and source evidence |
| --- | --- |
| Views and history | The selector is Tabs / Spaces / Chats. Chats opens Inbox beside the original terminal; rich chat and conversation rename are removed. Sidebar and Inbox show all spaces. Checked `app/window_chrome.rs`, `mode.rs`, `conversations/`, and the history store. |
| Settings | One shared native Settings window replaces the old embedded-workspace description. Skill sources and Saved Prompts have settings/services but no workspace surface. Checked their registrars and `settings_window.rs`. |
| Markdown Prompt | Provider notifications refresh the palette. Only explicit Apply changes → Save writes project content; no background worker resumes saved compositions. Checked `plugins/markdown-prompt/src/lib.rs` and `persistence.rs`. |
| Wayfinder prerequisites | Empty registries allow browsing, but disabling a required provider disables Wayfinder itself. The claim-release bridge works without provider services; that does not make the bundled UI available while its prerequisites are disabled. Checked the manifest, catalog activation/cascade logic, and bundled-plugin tests. |
| Architecture | Native plugins are compiled in; external GPUI libraries are rejected. The complete terminal/editor graph now includes Zed workspace dependencies transitively. chartr still owns its workspace model. ADRs and the planning spec now reflect these boundaries. |
| Terminal integration | The maintained view patch covers alignment, padding, overlay scrollbars, and resize pausing, replacing the old one-extension claim. Launch queue acceptance does not prove agent startup. |
| Companion | The protocol and retained plugin are explicitly marked inactive in the desktop build. Removed old Companion verification totals from the current status-bar guide. |
| Navigation and provenance | Fixed the Clock installation directory, theme-playground working directory, font-catalog links, a deleted `sync.rs` reference, and machine-specific historical citations. Added all recent audits to the research index. |
| Historical material | Old design/chat reports now carry dated-snapshot notices and current-guide links. Their screenshots, proposed review dates, experiment results, and publication receipts are not presented as current verification. |

Current guides were cross-checked against package manifests, SDK signatures,
storage/path resolution, keymap labels, host operation limits, CI, packaging
scripts, and the corresponding implementation owners. The acceptance checklist
now tests the current surfaces, modals, explicit save policy, and recovery flow.

## Validation

- Formatting and shell-script syntax checks passed.
- All 81 interface-font files match their manifest SHA-256 hashes: 23 added
  families, 3.83 MiB. The code catalogs contain 25 interface choices and 14
  terminal families, matching the asset guide.
- All eight JSON files under `docs/` parse. The Wayfinder manifest embeds the
  same tracker convention as its standalone Markdown file.
- The theme playground production build and standalone native Hello example
  check passed. All 13 literal native theme palettes and all 15 sidebar palettes
  match the playground's preset data.
- Nine JavaScript bridge/layout tests passed after repairing the bridge's stale
  mock document. Its request, timeout, and document-isolation assertions remain intact.
- All 170 local links/anchors and eight TOML/JSON examples passed validation
  across the resulting 46 Markdown documents. Ten pinned historical Chartr source
  paths were verified against their recorded Git commit.
- The locked workspace suite passed: 405 tests, zero failures, five ignored
  opt-in checks/doctests.

The first workspace run found 403 passing tests, two failures, and five ignored
checks. One failure was an outdated expected Skills heading. The other caught
four icon buttons from the earlier UI changes bypassing the shared semantic
component. The heading fixture now matches the documented formatter; those
buttons use `icon_action`, with tooltip support added to that shared helper.
The icon sizes, labels, tooltips, and actions are preserved.

Commands used include `cargo fmt --all --check`,
`cargo test --workspace --locked --no-fail-fast`,
`cargo check --manifest-path examples/plugins/hello/Cargo.toml --locked`,
`node --test crates/chartr/tests/plugin_bridge.cjs plugins/wayfinder/tests/layout.test.mjs`,
`sh -n scripts/build-dev-dmg.sh vendor/herdr/fetch.sh`, and `npm run build` in
`misc/theme-playground`. A temporary Python audit checked local Markdown targets,
heading anchors, TOML/JSON examples, and asset hashes without adding a new test
framework to the repository.

## Limits and follow-up

This is a documentation/source audit on macOS, not a fresh release acceptance
run on every platform. It does not rerun the older research experiments, launch
paid agents, exercise every installed CLI, rebuild a DMG, or perform Linux/native
pointer and accessibility acceptance. The current guide retains the live-provider
verification limits for Cursor, Antigravity, and OMP.

External competitor snapshots and citations remain dated evidence; the audit
did not recertify current competitor features or the availability of every remote
page. Local publication receipts were preserved without posting anything.

Wayfinder's disabled-provider UI limitation is now documented rather than being
concealed by a bridge-only test. The small code/test corrections above repair
validation drift; no product workflow was changed to make a documentation
assertion true.
