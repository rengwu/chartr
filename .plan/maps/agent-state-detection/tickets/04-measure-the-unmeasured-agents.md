---
type: task
claimed_by: se193f9533252
claimed_at: 2026-08-02T15:57:26Z
---

# Four of the six agents were never measured

## Question

The Destination says done means "the six agents in the roster (claude, kimi,
codex, opencode, grok, pi) each read correctly through a full turn — idle,
working, and stopped-for-permission." Only **two** were ever measured. Tickets 01
and 02 captured claude and kimi and tested against their real bytes; the other
four manifests were written from herdr's data and from the braille-spinner shape
the captured agents share. All four say so in their own headers:

```
codex.toml    "only claude and kimi were captured as fixtures for this map, so
               codex's exact title strings are taken from herdr's manifest"
opencode.toml "NOT captured as a fixture … extrapolated … not verified against
               recorded opencode bytes"
pi.toml       "NOT captured as a fixture … extrapolated, not verified against
               recorded pi bytes"
grok.toml     "not captured as a local fixture … come from the described
               signals and herdr's manifest, not a recording here"
```

That was a reasonable place to leave it while the engine was being built. It is
not a place to call the map done from, and it has now produced a real symptom.

**The reported bug.** On 2026-08-02 the operator reported that state inference
while **codex is working** looks broken, with codex installed locally. The
mechanism is visible without a capture: both codex rules read `region =
"osc_title"` and nothing else. If codex's real title does not contain `Working` /
`Thinking` / `Running`, the working rule never fires, no other rule matches, and
the sampler falls through its absence path to **idle** — a working codex reading
as idle, which is exactly the class of bug this whole map exists to kill. Codex
ships no screen rules to catch it, unlike claude and kimi.

`opencode` and `pi` are the same shape one step further back: no title rules at
all, one extrapolated spinner pattern each, no symptom reported only because
nobody has watched them closely.

**What is newly possible.** All six roster agents are installed, available and
prompt-ready on this machine right now — claude, kimi, **codex**, **opencode**,
**grok**, **pi**. All four unmeasured agents can be captured today; there is no
blocker beyond someone driving each through a turn.

**Why one ticket and not four.** The expensive part is shared and paid once: a
PTY recorder in the capture format, and the judgement about what each agent's
evidence actually supports. Per agent after that it is a small TOML edit and one
table case. Splitting would multiply the ceremony over identical work.

**The recorder has to come back.** The map notes the two design spikes were
thrown away deliberately — "the *recordings* are worth keeping as test fixtures"
— so the thing that produced `rec-claude.jsonl` no longer exists. The format is
documented and trivial (`assets/README.md`): line 1 is `{"cols":N,"rows":M}`,
then `[elapsed_seconds, "<base64 chunk>"]` per PTY read. Rebuilding a recorder
that writes it is the first step and is a throwaway again — it does not ship.

**The test harness already generalises.** Nothing in `recording_test.go` is
claude- or kimi-specific: `loadRecording`, `recordingGeometry` and
`replayScreen(t, agent, name)` all take the agent and the filename. Adding an
agent is a fixture file plus a case, not new harness.

**A correction here is a data edit.** The rule DSL is three matchers — `all`,
`any`, `line_regex` — over the region seam, and `skip_state_update` vetoes a
sample. If an agent's evidence needs something none of those express, that is a
finding worth writing down before reaching for a new matcher; the DSL was just
cut down from six precisely because unused generality accumulated.

**Grok is part of the measurement pass.** Its `Action Required` title and OSC
9;4 progress rules came from herdr and described signals, not local evidence.
Now that the CLI is installed, capture the same idle, working and permission
states as the other three agents and correct the manifest against the bytes.

Done when: `codex`, `opencode`, `grok` and `pi` each have a real PTY capture in
`.plan/maps/agent-state-detection/assets/` covering idle, a working turn, and a
stopped-for-permission state where the agent has one; each manifest is corrected
against what those bytes actually say — including adding screen rules where the
title turns out to carry nothing, as kimi's did; `recording_test.go` asserts the
published transitions for each from its recording; the "not verified" notes are
gone from the four headers or replaced with what was measured; and
`go vet ./...` / `go test ./...` pass.
