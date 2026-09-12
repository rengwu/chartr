# Native provider services and a web Wayfinder launcher

Wayfinder is the first pane that needs configured data and behavior owned by
other plugins. Its map remains useful without an agent or skill source, so
prerequisites belong to features rather than preventing the whole pane from
loading. Manifests list provider IDs and the features needing them. Settings
shows those dependencies; the consumer explains missing configuration and links
to the provider's setup. There is no automatic installation or enablement.

Trusted native providers export typed services through `Plugin::services`.
Each catalog owns a live directory passed through `InstanceContext`. Disablement
removes the provider's exports; reenablement publishes them again. A consumer
looks up the live provider again before a consequential action. Native code is
already fully trusted, so this is a lifecycle/API boundary, not a security sandbox.
Web plugins do not receive this directory. Wayfinder uses a narrow,
permission-gated workflow bridge rather than a general native-service RPC layer.

Agent exports registered names and adapter-aware input preparation. Skills exports
ordered, enabled skill discovery, full method text and provenance. Their private
storage layouts stay private. Provider registry entities are shared across app
windows for the same data directory, so configuration and consumers see one
registry. A typed service can be added when another concrete plugin needs it;
arbitrary RPC, dependency downloads and version solving are unnecessary here.

## Why there is no four-role settings table

The original Go chartr was inspected at local revision `ac94f06`, particularly
`internal/config/binding.go`, `internal/prompt/rolebinding.go`, the prompt cores,
the map parser and scanner, and `web/src/lib/starmap/`. The source methods were
inspected in [chartr-skills](https://github.com/rengwu/chartr-skills), revision
`35a4d93`, including `wayfinder`, `grill`, `prototype`, `research`, and `implement`.
The local uncommitted `to-chain` addition is unrelated to this map plugin.

The original roles are prompt selection, not four different scheduling engines.
`grilling → grill`, `prototype → prototype`, `research → research`, and
`task → implement` already follow the ticket's own `type`. Original chartr's
[ADR 0015](https://github.com/rengwu/chartr/blob/ac94f06/docs/adr/0015-map-kind-removed-role-comes-from-the-ticket.md)
also eliminated map-level classification for precisely this redundancy.

The [Wayfinder method](https://github.com/rengwu/chartr-skills/blob/35a4d93/skills/wayfinder/SKILL.md)
already describes the branch choices and ticket lifecycle. A general dispatcher
can therefore carry the routing instructions, while specific source methods
retain their useful depth. This is a design judgment from the code and method
contracts, not a claim that every external agent will follow a prompt perfectly.

The resulting UI has one agent picker and one method picker: automatic by ticket
type, with an optional exact source/skill override. Automatic mode includes the
general Wayfinder method and the applicable specific method when available;
either can satisfy the ticket's method prerequisite. An operator does not have to configure
four bindings or register four role skills before the first map is usable.

## Web surface, small host boundary

The first native pane was replaced at the operator's request. The package is now
`kind = "web"`: plain HTML/CSS, one UI module and one canvas module. Its design
follows the original `MapCard.svelte` and `DetailPane.svelte`: a full constellation,
floating controls, and a flush translucent reading drawer, bottom-docked in a
narrow pane. There is no retained GPUI Wayfinder view or native canvas.

The package ships directly, without Svelte, a bundler or runtime dependencies.
The initial reduced canvas omitted interaction details that made the original
usable. The frontend now ports the original star-map renderer's palette, layout,
label solver, timed camera easing, parallax layers, pulses, claim rings, selection
flares and dependency flow. A separate progress-card map picker and floating
Back/title controls restore its navigation. The measured detail seam is resizable
and uses the original width/aspect docking rule with hysteresis.

Wheel, Ctrl-wheel pinch, WebKit cumulative gestures and touch pointers share the
camera target. Fog has world-space anchors even when its clearing ticket is
missing. Selection seats a star in the remaining viewport; closing the drawer
leaves the camera alone. Animation runs only in the visible map, with demand-only
rendering and immediate camera changes for reduced motion. Session liveness
orbits are not inferred from a claim: the bridge does not provide that evidence.
Scheduling and a second role configuration system remain outside this surface.

The tested Rust parser, prompt composer and claim handling stay in the plugin's
`src/` as the host implementation of `wayfinder.*`. The explicit `wayfinder`
permission authorizes this workflow (including registered-agent execution).
It is checked for installed and bundled web packages alike. JavaScript receives
map data and sanitized Markdown, and requests previews/launches, claim release,
session navigation, project-file opening and provider setup. It cannot send raw
terminal input or arbitrary source-registry commands through this API.

Preview IDs bind launch to the exact host-generated context and originating
document. The existing bounded web request queue dispatches the workflow on the
UI thread only when necessary, with filesystem scans and Markdown rendering on
the background executor. A detached worker can finish its claim rollback after
the pane closes; weak view references and a navigation generation prevent a
closed or replaced document from starting an agent.

## The file and terminal boundaries

The parser retains the original fixed `.plan/maps/<slug>/` discovery and
file-derived statuses. A non-empty Answer resolves; Ruled out closes outside the
route; otherwise a claim marks in-flight work. Missing or ruled-out blockers
do not clear an edge. Empty headings and examples inside fences do not resolve
tickets. Geometry uses only identities and edges, never statuses.

One prompt contains host workflow rules, the file contract, selected source
methods and provenance, then map/ticket/blocker context. The tracker convention
is embedded in the prompt and packaged with the plugin; this implementation
does not overwrite a project's existing convention or mirror source folders.

The terminal host now supports prepare-then-send: an attached shell returns its
real session identity before any agent input. Wayfinder revalidates the preview
and the frontier under an advisory lock and writes the claim atomically. Agent
prepares correctly quoted input using its current definition. Delivery failures
release the matching claim; a changed claim is never cleared as cleanup.

Only one claimed ticket runs per space. This keeps the original shared-working-
tree serialization rule; prompt routing does not bypass it. An operator may
explicitly release abandoned claims or focus the existing terminal. There is no
automatic completion inference from a process exit and no automatic next-ticket
launch. General sessions outside Wayfinder remain outside this coordination.

Revisit this boundary if a second workflow needs a broader service API, or a
workflow needs actual parallel execution. Those need a concrete shared contract
or an isolation model, respectively, not more role dropdowns.
