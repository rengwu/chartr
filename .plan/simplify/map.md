# simplify — a leaner, more open chartr

## Destination

A decision set for cutting the chartr down to its working core and opening it up: the review pipeline (agent review + human hub) removed in favour of a direct ticket lifecycle, the prompt library repackaged as standard `SKILL.md` skills, configuration made legible through a transparency surface, and the cockpit wrapped in a mac-first native webview shell around the existing Svelte UI.

The map is done when every decision an implementation map needs is settled — nothing left to decide before `to-spec` and `to-tickets`. **Plan, don't do:** this map produces decisions, not code.

## Notes

**Read before choosing a ticket:** [`CONTEXT.md`](../../CONTEXT.md) for the vocabulary (several terms die on this map — proposed, review brief, human review, autopilot) and [`docs/adr/`](../../docs/adr/) for what is currently settled. This map **amends or retires ADRs** — 0004 (review lifecycle), 0008 (promotion/demotion commits), 0011 (release tiers) are directly in play; 0007 and 0009 survive but their surfaces shrink. Where a ticket's answer breaks an ADR's premise, the ticket must say which ADR it amends.

**The previous design map is precedent, not scripture.** `.plan/chartr-design/` settled the architecture being cut here. Its standing preferences still bind: the chartr is a **cockpit, not an autopilot**; the client is **hackable** — everything the chartr injects or reasons with is visible on disk as plain files, never sealed in the binary; deterministic code owns what must always be true, agents exist only where judgment is the product.

**Skills every session should consult:** `grill-me` / `grill-with-docs` for the grilling tickets; `domain-modeling` to keep `CONTEXT.md` and the ADRs honest as terms die and new ones crystallise. At the end of this map: `to-spec`, then `to-tickets`.

**What is being kept, and is not up for re-litigation:** the spaces/sessions model, the PTY terminal layer, the star-map and its renderer, the chrome/island split (ADR 0010), the design system (ADR 0012), the three-layer config *mechanism* (ADR 0009 — its surface changes, not its layering), and the one-session-per-space invariant (ADR 0003).

## Decisions so far

<!-- Settled with the operator on 2026-07-21, before the map was cut. Do not re-litigate. -->

- **Native means a webview shell around the current UI.** A mac-first native window hosting the existing Svelte cockpit (Linux second, Windows wherever it falls out). Rejected: a Go TUI (kills the star-map, one of the assets worth keeping), a SwiftUI rewrite (sacrifices Linux and ~6k lines of working frontend), staying browser-only (doesn't answer the complaint). Knowingly accepted: a webview shell will not *feel* truly native — it is the app in a real window, no more. **Revisit trigger:** if the shell ships and daily driving still feels wrong, the honest next step is a TUI companion, not another web layer.
- **Both review stages are cut; the extension point is the filesystem, not a plugin framework.** The lifecycle simplifies to: session writes `## Answer` and commits → resolved. A future review feature hooks the seams that already exist — file-derived ticket status, fsnotify events, git trailers — from inside or outside the process. Building a plugin API now is the speculative bloat this map exists to remove; the seam is documented convention, and a real second consumer is what would earn an interface.
- **Configuration becomes legible first, editable second.** Ship an effective-config surface showing every resolved value with its provenance layer and an open-in-editor escape hatch; the TOML files stay the interface. A real editing UI comes later, only for high-churn settings (role bindings), and must never become a second source of truth against the layering rules.
- **Everything the chartr injects becomes a `SKILL.md`.** Role prompts, the core "how to use this chartr" prompt, and the tracker convention (map format, kinds, statuses, ticket flow) are repackaged as standard skill directories — hackable, open, and usable by agents outside the chartr. Location, management, and injection mechanics are this map's ticket 02 to settle.
- **xterm.js stays.** The ghostty-web WASM terminal (xterm.js-compatible, libghostty-vt core) is banked as a revisit candidate if terminal UX still disappoints after the cut; server-side libghostty via cgo is rejected outright (kills the cgo-free supported artifact, drags in the Zig toolchain, pins an unstable API). If terminal bugs resurface, first suspect the chartr's own raw-byte scrollback replay, not the renderer.

## Out of scope

- **A plugin system, extension API, or hook framework.** Deliberately refused above; the filesystem and git conventions are the seam.
- **A TUI or true-native frontend.** The webview shell is the native answer this effort ships; anything further waits for the revisit trigger.
- **Redesigning the wayfinder method.** Same boundary as the design map: the chartr drives wayfinder maps; the tracker-convention skill restates the format, it does not change it.
- **Migrating or rewriting the old maps.** `chartr-design`, `chartr-design-impl`, and `reskin` are history; they stay readable, and whatever the new lifecycle does to their in-flight tickets is a ticket-01 question, not a rescue mission.
