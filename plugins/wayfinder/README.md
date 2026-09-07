# Wayfinder

A bundled web plugin for folder spaces. Open **Wayfinder** from the plugin
picker. It reads existing `.plan/maps/<slug>/map.md` files and their numbered
Markdown tickets and launches ready tickets through Agent.

The opening screen is a grid of maps with resolution progress. Open a map to
explore its constellation; **Back** returns to the picker. The map title at the
top left opens its material, destination and ready frontier. Click a star to read
its rendered Markdown, blockers, assets, claim and warnings. The map itself is
the ticket picker.

The renderer carries over the original Wayfinder camera, palette, collision-aware
labels and star animations: layered parallax, breathing stars, selection flares,
claim rings and particles flowing along resolved dependency paths. Drag to pan;
scroll or pinch to zoom around the pointer. Both Ctrl-wheel pinch and macOS
WebKit gestures are supported, along with touch panning and two-finger pinch.
Camera moves ease with the original 120 ms time constant. Fog, including patches
without a valid ticket anchor, moves and scales in world coordinates.

Selecting a star centers it in the space beside the translucent detail pane.
Drag the shared seam to resize the pane, or focus the seam and use arrow keys
(Home/End set its limits; double-click resets its size). It docks right in wide
panes and below the map in narrow or tall panes, remembering each side's size.
Closing details preserves the camera. Use **F** or the recenter control to fit,
left/right arrows to select tickets, and Escape to close details or return to
the picker while the map is focused.

Files refresh every two seconds without resetting the camera, scroll position or
operator note. Status-only updates do not change star geometry. Cameras are
remembered when changing maps inside the open pane, but are not persisted across
application restarts. Animation pauses on the picker and in hidden documents;
reduced-motion settings remove continuous animation and camera easing.

## Prerequisites

Map browsing needs only a folder. Agent launching uses the following providers:

- **Agent**: enable it and register a launch definition under **Agents…**.
- **Skills**: enable it and register an enabled local or Git source under
  **Skill sources…**. For the supplied methods, register
  `https://github.com/rengwu/chartr-skills` through the Skills settings.

Dependencies are listed by feature in `chartr-plugin.toml` and Settings. Nothing
is installed, fetched or enabled automatically. If a provider is disabled,
Wayfinder keeps showing maps and explains why launch is unavailable. The setup
buttons open the provider's configuration, or the Plugins page if it is disabled.
The source repository and agent definitions remain owned by their providers.

## One launcher, automatic methods

There is no four-role binding table. A general dispatcher uses each ticket's
type: grilling interviews the human, prototype explores a throwaway artifact,
research gathers primary-source evidence, and task performs its stated work.
The automatic prompt includes the enabled `wayfinder` skill when available and
the matching `grill`, `prototype`, `research` or `implement` skill when available.
It does not require all four. A source with only `wayfinder` can drive the flow;
a ticket can also run with its matching method alone.

The method picker can pin any discovered `source/skill`. A missing pin refuses
launch rather than silently switching to another source. Bare automatic names
respect Skills' enabled source order. Supporting files are resolved from their
source directory; no skill mirror or generated `chartr.md` is written.

**Review & launch** shows the dispatcher, source provenance, selected methods,
tracker convention, map, ticket, resolved blockers and optional operator note.
The launch path rescans sources and revalidates the map to reject stale context.
An explicit method override changes the method, not the selected ticket's scope.

## Claims and sessions

The host prepares a real shell first. Wayfinder rechecks the on-disk frontier
under an OS lock, then atomically records the actual session ID in `claimed_by`
and an RFC 3339 `claimed_at`. Agent supplies the adapter-specific input; the host
delivers it to the normal terminal in the same space. No external agent runs
until these steps succeed. A failed delivery releases only the claim it created.
The prepared shell may remain open after a failure so the failure does not
silently destroy a terminal.

One claimed ticket is permitted per space, including across maps. Work is
explicitly launched; there is no automatic queue or worktree manager. A
non-empty Answer or Ruled out closes the ticket, so a leftover claim on a closed
ticket does not block the frontier. Claimed tickets show their session identity
in the detail pane.

The complete file contract is [TRACKER-CONVENTION.md](TRACKER-CONVENTION.md).
Ordinary agent sessions outside Wayfinder are not serialized by this plugin;
the advisory lock coordinates Wayfinder's own launches and claim updates.

## Implementation and validation

The frontend is plain HTML, CSS and two JavaScript modules, with no framework,
bundler, network calls or runtime dependencies. It loads through chartr's normal
web-plugin host. The old GPUI pane and native canvas are removed. The same plugin
ID preserves existing pane restoration and enablement settings.

Map parsing, safe Markdown rendering, prompts and atomic claims remain under
`src/`, compiled into the host as the narrow `wayfinder.*` bridge. Its explicit
`permissions.wayfinder` grant allows map/source reads, claim updates and
registered-agent launching (which can execute commands as the user). It does
not expose arbitrary native service calls, process commands or terminal bytes.
Project Markdown never executes as HTML. See [the API](../../docs/plugins.md#wayfinder-workflow).

Tests cover closing-heading parsing, legacy numbers, missing/ruled-out blockers,
cycles, duplicate IDs, stable layout, camera/input regression checks, source changes, claim-before-input ordering,
failed-delivery cleanup, document-bound previews and disabled providers. Browser
smoke tests cover responsive geometry and the complete preview/launch flow with
a fixture host. Real Herdr smoke
tests cover direct attachment and backend recovery. The original implementation
and the role simplification are discussed in
[ADR 0006](../../docs/adr/0006-native-plugin-services-and-wayfinder.md).

The tab and interface icons use Hugeicons' free Stroke Rounded collection.
`icons/ui.svg` packages the original named exports from
`@hugeicons/core-free-icons` 4.3.0 as a local sprite; geometry is unchanged and
no icon package or remote request is needed at runtime. Attribution and the
MIT license are in [icons/HUGEICONS_NOTICE.md](icons/HUGEICONS_NOTICE.md).

Interface controls share a 32 px height, 16 px icons, a 4 px spacing scale and
16 px detail-pane gutters. Body text is 13 px, controls and labels are 12 px,
and detail titles are 16 px. Blocker and frontier rows keep ticket numbers and
statuses in fixed columns while titles wrap. The legend has an opaque surface
to stay legible over map labels, and launcher actions stack in narrow panes.

Run `cargo test --workspace` and `node --test plugins/wayfinder/tests/layout.test.mjs`.
For browser QA, install Playwright in a separate dev environment and run
`node plugins/wayfinder/tests/browser.mjs`; `PLAYWRIGHT_MODULE` can point to its
`index.mjs` and `SCREENSHOT_DIR` to an existing output directory. These tests use
fixtures, not real agents or the user's project files. No frontend build is needed.

Set `BROWSER=webkit` to run the browser suite against WebKit as well as the
default Chromium. Screenshots include the picker, full map and responsive details.
