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

## Answer

All four agents were captured and all four manifests corrected. **Three of the
four were wrong**, each in the same direction: a rule that could never fire, so a
working agent fell through the sampler's absence path and read as idle. The
reported codex bug was not a one-off — it was the first of three instances of the
same defect, and the extrapolation was the common cause.

### What each agent actually broadcasts

| agent | idle | working | blocked |
|---|---|---|---|
| codex 0.146.0 | `agentcwd` | `⠹ agentcwd` | `[ . ] Action Required \| agentcwd` |
| grok 0.2.118 | `<summary> - grok` | `⠼ - Waiting for response… - grok` | `⚠ Action Required - ⠙ - … - grok` |
| opencode 1.2.27 | *(title says nothing)* | `⬝⬝⬝■■■■■  esc interrupt` on screen | `Allow once / Allow always / Reject` panel |
| pi 0.78.0 | *(title says nothing)* | `⠇ Working...` on screen | **none — no approval gate exists** |

**codex** — the reported bug, mechanism confirmed exactly as the ticket predicted.
Its working rule looked for `Working` / `Thinking` / `Running`; codex broadcasts
none of them. It *does* draw `Working (2s • esc to interrupt)`, but on the screen,
never in the title — the title is the directory basename with a braille frame
prefixed, the same signal claude uses. `Action Required` survived measurement
unchanged, making codex the one roster agent whose title carries blocked outright.

**grok** — the map had written grok off as unverifiable ("not installed on this
machine"); it is installed and it measured fine. `Action Required` is real. But
grok emits **no OSC 9;4 progress at any point** — zero writes across the whole
capture — so *both* progress rules were dead: working never fired, and the
progress-derived positive idle never fired either. herdr's progress claim was
simply never true here. Working now reads the braille frame; the phantom idle rule
is deleted rather than left as decoration. One ordering fact the bytes forced:
grok keeps spinning *while* it waits on the operator, so a blocked title carries a
braille frame too and blocked must outrank working.

**opencode** — extrapolated to a braille spinner; it does not use braille. Its
spinner is eight square cells (U+2B1D / U+25A0) filling left to right, trailed by
the `esc interrupt` affordance. The old pattern could not have matched a single
frame. Its title is written twice a session and names the conversation, so like
kimi it reads entirely off the screen — which is where its `△ Permission required`
panel lives too, so it gained the screen blocked rule the Done-when asks for.

**pi** — the one extrapolation measurement *vindicated*: it really does draw
`⠇ Working...`, braille at the head of a line near the foot. The pattern was right;
the **region** was not, and only measuring it showed why. The spinner sits five
non-empty lines above the foot with an empty input box. The capture deliberately
types a follow-up into the box mid-turn — the ordinary thing an operator does
while an agent works — which inserts a line and pushes the spinner to exactly six,
the last slot `bottom_non_empty_lines(6)` could see. A two-line message would have
pushed it out and a working pi would have read as idle with nothing to catch it.
Widened to 8. `TestPiWorkingSurvivesATypedInputBox` pins the headroom.

### What I deliberately did not do

- **pi has no blocked rule, because pi has no blocked state.** It ships no
  approval gate: `pi --help` offers only allow/deny *lists* of tool names, never
  an interactive confirmation, and the capture shows it running `rm -f` outside
  the workspace without pausing. `pi list` confirms no extensions installed. This
  is an absence in the agent, not a gap in the manifest — and it is the concrete
  case that makes the map's *Hook-reported state* fog worth clearing, since pi's
  herdr lifecycle extension is the only thing that could ever report it. Flagged
  rather than faked with a rule that matches nothing.
- **No new matchers or regions.** Everything measured fits `all` / `any` /
  `line_regex` over the existing regions, so the DSL that was just cut from six to
  three stays at three. opencode needed positional scoping rather than the
  framed-region helpers (its chrome is `┃`/`▀`, not flat U+2500 rules), which
  `bottom_non_empty_lines(n)` already covers.
- **No screen rules for codex or grok.** The Done-when says to add them "where the
  title turns out to carry nothing"; both titles carry everything, and adding a
  redundant second path is the unused generality this map keeps pruning.
- **The recorder does not ship.** Rebuilt as ~150 lines around `creack/pty` that
  replays a `wait`/`type`/`key` script into a PTY, used, and discarded — as the
  ticket says it should be. Its format and behaviour are documented in
  `assets/README.md` so the next capture does not start from nothing.

### Two things a human should look at

1. **The three agents that auto-approve by default.** codex, opencode and grok all
   ran a shell command without pausing under their shipped defaults; reaching
   blocked needed `-a untrusted -s read-only` (codex), a project `opencode.json`
   with `permission.bash = "ask"` (opencode), and a command outside the trusted set
   (grok). The blocked rules are therefore measured against a *configuration* an
   operator has to opt into. That is genuine — it is how you hit the state at all —
   but "blocked works" means "blocked works when the agent is configured to ask".
2. **The map edit that was already in the working tree.** `map.md` arrived with the
   *Grok's rules cannot be verified here* fog note deleted. Measurement makes that
   deletion correct, so I committed it with this work rather than leaving it
   dangling — noting it here because I did not write it.

`go vet ./...` and `go test ./...` pass. `recording_test.go` asserts the published
transitions for all four from their own recordings; the hand-written table cases in
`detect_test.go` that encoded the old guesses were rewritten against the captures —
four of them were failing the moment the manifests became true.
