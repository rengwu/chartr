---
type: task
blocked_by: [02]
---

# Optional context for needs-context skills

## Question

Give the skills that read a subject their one line. When the operator picks a skill
whose snapshot flag is `needsContext` (01), the launcher offers a single **optional**
one-line input before it launches; a skill without the flag launches immediately,
with no box, exactly as 02 already does. This is the "or after given some context"
half of what makes a skill self-driving — `grill` wants to know *what* to grill,
`research` *what* to research — without turning the launcher into a form.

- **Optional means optional.** An empty box is a valid launch: it sends no
  `context` and the skill opens bare (the payload is its body unchanged, per 01).
  A typed line rides as the `context` on the `/launch` call and lands in the payload
  under 01's trailer. There is no required field and no validation gate — Enter or
  the launch button fires either way, Esc dismisses without launching.

- **Where it appears.** A token + primitive affordance hung off the dropdown at the
  moment a `needs-context` skill is chosen — a small `Popover`/inline field with one
  `Input` and a launch button, not a modal `Dialog` (this is a quick line, not a
  task). Placeholder text comes from the skill (a short hint; reuse the skill's
  `description` if no dedicated hint is worth adding). Phosphor icon, tokens only,
  vendored primitives — no bespoke chrome, no raw colour (ADR 0012). Keyboard-first:
  the field autofocuses, Enter launches, Esc closes.

- **Self-driving skills are untouched.** ideate and wayfinder (`needsContext` false)
  never show the box — picking them launches on the click, preserving 02's flow.

Tests lead on the pure branch: the helper that, given the picked skill's
`needsContext`, decides *box* vs *launch-now*, and that assembles the `/launch`
payload with `context` present only when the line is non-empty — a `vitest` unit
beside 02's selection helper. The popover rendering and focus are trusted like the
rest of the chrome.

Done when: picking a `needs-context` on-ramp skill opens an optional one-line box
that launches with the typed context (or bare when empty), picking a self-driving
skill launches with no box, the context reaches the agent through `/launch` and its
payload, and Esc dismisses cleanly; `check` / `build` / `vitest` and `go vet` /
`go test` pass; no amber in the built CSS.

## Answer

Shipped the optional box. Frontend only — 01's `/launch` already took `context`
and already wrote the `## Your task` trailer, so this ticket just gave the
operator a way to type the line.

**The decision, in `launchmenu.ts` (unit-tested).** `launchClick` no longer
returns a bare `(agent, skill)`; it returns a `LaunchStep` — `{kind: 'context',
agent, skill}` when the picked skill's snapshot flag is `needsContext`,
`{kind: 'launch', agent, skill}` otherwise, and still `null` when no agent is
ready (an unchosen agent opens no box either). `launchPayload(agent, skill, line)`
assembles the call: the trimmed line rides as `context`, and a blank or
whitespace-only line **omits the key entirely**, so an empty box is a real launch
that opens the skill bare. There is no validation gate anywhere on the path.
`contextHint(skill)` is the placeholder — `What should grill work on?`, named from
the skill that will read it; the skill's `description` says what the skill *does*
rather than what the line should say, so it stays on the menu row above and the
field asks the question instead.

**The box (`SkillLauncher.svelte`).** A vendored `Popover` (added with
`npx shadcn-svelte@latest add popover`; no lucide, no new dependency — bits-ui was
already there), not a `Dialog`: this is a quick line, not a task. The dropdown has
closed by the time it opens, so it hangs off the same trigger via `customAnchor`,
with the menu's `onCloseAutoFocus` suppressed while a launch is pending so the
closing menu does not pull focus back off the field. One `Input` in a `<form>`
with a `Launch` submit — Enter launches for free, Esc closes on the escape layer
without launching, and clicking away is the same dismissal. Autofocus is
`onOpenAutoFocus` preventDefault + focus. Tokens and primitives throughout.
Deliberately *outside* the `DropdownMenu` content: a menu's typeahead and roving
focus would eat the keystrokes and steal focus on pointer-move.

**Both mounts.** `onrun` / `onLaunch` / `launchSpace` gained a trailing optional
`context`, threaded into the existing `launch()` action. Nothing else moved.

**Verified on the real binary** (isolated `XDG_CONFIG_HOME` + data dir, a scratch
space, an `echo` agent): `research` opened the autofocused box, a typed line
landed under `## Your task` in the written payload; `grill` with an empty box
launched a payload with no trailer; `ideate` launched on the click with no box;
Esc dismissed opening nothing. Frontend `check` (0/0), `vitest` (148 pass), and
`build` are green with no amber in the built CSS; `go vet ./...` and
`go test ./...` pass.
