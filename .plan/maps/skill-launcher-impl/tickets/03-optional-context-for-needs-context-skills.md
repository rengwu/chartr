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
