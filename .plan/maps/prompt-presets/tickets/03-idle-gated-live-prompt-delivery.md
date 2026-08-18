---
type: task
blocked_by: [01]
undermined_by: []
claimed_by: s7d2252ec4007
claimed_at: 2026-08-18T03:39:26Z
---

# Deliver one live preset through the idle-gated PTY queue

## Question

Implement the narrow live-delivery contract from [the specification](../spec.md)
inside the terminal manager and server. A request names one catalog preset and one
active terminal. A live Chartr-launched agent receives it immediately when idle;
otherwise it holds one snapshotted pending preset until the next observed idle.
The operator can cancel that item, and a second activation is refused while it is
pending.

Reuse the existing agent identity, terminal status, and typed opener behavior.
Add only enough public terminal state for the pane to know that the tab is an
eligible Chartr-launched agent and which preset is pending. Serialize a submitted
preset's text and separate carriage return against other input writes, but do not
add persistence, history, retries, provider APIs, draft inspection, or delivery to
manually launched agents and shells.

## Done when

- The server refuses unknown presets, ordinary shells, manually launched agents,
  dead tabs, foreign-space terminals, and a second request while one is pending.
- An idle eligible agent receives the snapshotted body followed by a separate
  carriage return; another input write cannot interleave between them.
- Working, running, and blocked agents receive no bytes and expose one pending
  preset until cancellation, exit, or the next observed idle transition.
- The next idle transition submits the pending preset exactly once and clears it;
  cancellation and terminal exit submit nothing.
- The pushed terminal model contains only the small eligibility and pending shape
  the pane consumes, and the queue remains in-memory runtime state.
- Focused terminal-manager tests and server process-boundary tests cover the state
  matrix, write ordering, races at the supported seam, and cleanup, and pass.

## Answer

Live delivery is one new file on each side of the seam —
`internal/terminal/liveprompt.go` and two handlers at the end of
`internal/server/prompts.go` — plus three small edits to existing state. Nothing
persists, nothing retries, and nothing asks a provider anything.

**The queue is one field.** `Terminal.pending` is a `*pendingPrompt{id, body}`,
guarded by the lock the tab's state already uses. The body is snapshotted by the
handler out of the catalog, so a later edit or deletion cannot rewrite an
instruction the operator already sent. `Manager.SendPrompt(space, term, id, body)`
takes the whole decision under `t.mu` — refuse, queue, or submit — so a sample
landing mid-decision cannot split it, and returns `queued` so the pane knows
which of the two things it just did. `CancelPrompt` reports whether it cleared
anything; cancelling a row the sampler emptied a moment earlier is a no-op, not
an error.

**There is no idle *transition* to detect, and that is the point.** A preset is
only ever queued on a tab that was not idle, so the first idle that finds one is
the transition. `submitDuePrompt` runs on the sampler right after `sample`
publishes that tick's state, takes the item under the lock, and submits off the
goroutine — which is what makes "exactly once" true without a second notion of
"idle" that could disagree with the one the sidebar shows. A tab that goes idle
between the operator's click and the store is delivered on the very next sample,
which is the same promise, so that race needed no code. The pump's cleanup nils
`pending`, so exit submits nothing.

**One writer, two writes.** `Terminal.writeMu` serializes writes *into* the PTY.
`submitPrompt` holds it across the text, the beat, and the separate carriage
return; `Write` — the operator's keystrokes off the terminal socket — takes the
same lock, so a keystroke racing a delivery lands after the return and never
inside it. `typeOpener` now calls `submitPrompt` too: it needed the identical
two-write shape for the identical TUI reasons, and there is now one
implementation of it rather than two.

**Eligibility is who launched the binary, not what is running.** `PromptTarget`
is `launchedAgent != "" && alive`. An ad-hoc shell running `claude` reads the
agent grammar and shows claude's own status, and is still refused — a test
asserts exactly that, because it is the one case the rule exists for.
`Info`/`model.Terminal` gained only `PromptTarget` and `PendingPrompt` (the
catalog id), mirrored in `web/src/lib/model.ts` as two optional fields so ticket
04 starts typed.

**Actions.** `POST` and `DELETE /api/spaces/{id}/terminals/{termID}/prompt`,
behind the existing `repoSpace` guard (so Scratch is refused rather than
ignored). An unknown preset or a terminal that is not this space's is 404; an
ineligible tab and a second activation while one is pending are both 409 —
nothing is wrong with the request, the tab is simply not in a state that accepts
it. The manager's own `notify` pushes the queued and cleared rows, so neither
handler rebuilds.

**Tests.** `internal/terminal/liveprompt_test.go` drives a real PTY running the
raw-mode stub from `opener_test.go` and sets the gating state by hand — the
grammar that produces that state is `sample`'s business, covered where it lives —
across immediate send, all three busy states, next-idle-submits-once,
cancellation, the one-pending refusal, ineligible and foreign targets, exit
clearing the item, and a write deliberately raced against a submission.
`internal/server/liveprompt_test.go` runs the whole chain at the process
boundary against a stub `claude` that paints real OSC titles and goes idle on a
cue file: immediate delivery, queue → refuse second → idle → exactly one arrival
→ row cleared, cancellation, every refusal, the manually launched agent, and a
pinned dead session. All pass under `-race`.

**Omitted deliberately.** No delivery notification, history, or retry, and no
third endpoint to ask how a submission went — idle is inferred from the terminal
and the contract is best-effort. No draft inspection. One consequence worth
stating: a tab launched on an agent chartr ships no manifest for reads
working-while-alive by design, so it is an eligible target that will hold a
queued preset indefinitely. Refusing it would need a second eligibility rule the
pane would have to explain, and the honest reading is that the tab genuinely
never goes idle.

`make test`, `make vet`, and `make check` all pass, as do the web unit tests.

**Amended 2026-08-18: eligibility is what is in front of the tab, not who
launched it.** The operator asked for their own `claude` to be a target too, and
the rule above was the only thing refusing it — the sampler already identifies
that agent, reads its status from the agent grammar, and titles the tab from its
transcript, so chartr knows a TUI is listening there exactly as well as in a tab
it launched. `PromptTarget` is now `alive && titleAgentLocked() != ""`, which is
the same identification the title and status already run on: a launched tab
answers from its launch, an ad-hoc shell from whatever holds the PTY's
foreground. A shell at its own prompt is still refused, and that is the case the
rule exists for — there the preset would be run as a command.

Eligibility stopped being immutable in the process, so a pending item now
records the agent and pid it was aimed at, and `submitDuePrompt` re-checks that
seat before typing. Quitting an ad-hoc agent leaves a shell that reads *idle*,
which is precisely the state the queue was waiting for; a seat that is gone or
replaced drops the item instead, the same way an exit does. The spec's target
paragraph, user-visible item 9, and the failure-behavior bullet were updated to
match, along with `map.md`'s out-of-scope line.
