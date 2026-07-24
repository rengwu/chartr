---
type: task
blocked_by: [01]
claimed_by: s63f4ee253a9d
claimed_at: 2026-07-24T04:10:38Z
---

# The screen grid, kimi, and blocked

## Question

Reconstruct each terminal's screen server-side and let ticket 01's rule engine
read it. This is where the states that matter most actually arrive: Claude
signals working-vs-idle in its title but **never** signals blocked there — a
permission prompt reads as `✳`, byte-identical to idle — and kimi, opencode and
pi put nothing in the title at all. Everything past "is Claude thinking" needs
the screen.

**Add the grid.** Feed PTY bytes through `github.com/charmbracelet/x/vt` in
`Terminal.pump`, alongside the existing scrollback append, and expose a
`detectionText()` returning the rendered viewport as plain text
(`Emulator.String()`). This is read for *detection only* — the browser keeps
rendering through xterm.js from the raw scrollback exactly as today (ADR 0010).

Two things were measured and must not be re-litigated:

- **Cost is not a concern.** 0.70 MiB of real kimi output — a whole five-minute
  session — replays through the emulator in ~58ms including decode. Do not add
  sampling tricks to avoid emulator cost; do keep herdr's skip-rescan-when-idle
  optimisation only if it falls out naturally.
- **The emulator deadlocks unless its reply channel is drained.** Agents query
  the terminal (Claude opens with `OSC 11;?` asking the background colour); the
  emulator answers by writing into an internal pipe, and with no reader that
  write blocks forever and wedges the whole emulator mid-stream. Drain
  `Emulator.Read` from a goroutine. This is not hypothetical — it is how the
  spike first failed.

Resize the grid with the PTY so regions anchored to the bottom stay meaningful.

**Add the screen regions** behind ticket 01's region seam: `whole_recent`,
`bottom_non_empty_lines(n)`, `after_last_horizontal_rule`, `prompt_box_body`.
They are the point of the design — rules slice the screen structurally instead of
grepping it, which is what keeps a keyword in transcript prose from being read as
live chrome.

**Ship manifests for kimi, opencode and pi**, and add the screen rules that give
claude its `blocked`. herdr's shipped manifests were verified to fire unmodified
against real recordings on this machine:

- kimi **working** — `⠋ thinking...` matches an anchored
  `^\s*[⠀-⣿]+\s*(thinking\.\.\.|working\.\.\.|using )`. The anchor is
  essential: kimi's status bar reads `K2.7 Coding thinking ~` on *every* screen,
  so an unanchored `contains "thinking"` is always true. This is the trap the
  region-and-anchor design exists to avoid.
- kimi **blocked** — `↵ confirm` + `▶ Run this command?` + ` choose` +
  `Approve`/`Reject`.
- claude **blocked** — `Do you want to proceed?` + `❯ 1. Yes` +
  `Tab to amend`/`ctrl+e to explain`, scoped to `after_last_horizontal_rule`.
- claude **idle** — the `❯` prompt box, which must rank *below* the blocked
  rules so a title saying idle never overrules a screen showing a permission
  prompt.

**Anchor rules on structure, never on prose.** Reconstruction is not perfect and
the imperfection is specific: `charmbracelet/x/vt` garbles a prose line inside
Claude's permission dialog (spaces clobbered) while rendering every structural
line — the horizontal rule, the numbered options, the footer hints — cleanly. A
second emulator (`danielgatis/go-headless-term`) got that prose line right but
merged `Do you want to proceed?` into the horizontal rule above it and left stale
duplicate status lines, which is why it was rejected. Rules that match on menu
options, footer hints, box borders and rules are reliable; rules that match on
sentences are not.

**Extend the fixtures.** Ticket 01's captured recordings gain kimi's approval
panel and claude's permission dialog; the pure-engine table test grows cases that
assert each new rule fires at the recorded moment it should and stays silent
before and after.

Tests lead: extend the pure-engine table test with the screen regions and the new
manifests, driven by the recordings; add a regression asserting kimi's
always-present `thinking` status bar does **not** read as working; add a
process-boundary test that a stub agent painting a permission-prompt screen reads
`blocked` in the snapshot and returns to `idle` once answered.

Done when: kimi reads `idle` / `working` / `blocked` correctly through a real
turn; claude reads `blocked` while sitting on a permission prompt and `idle` once
answered; opencode and pi read working-vs-idle; the emulator survives a full
session without wedging (the OSC-query drain is tested, not assumed); a tab with
no known agent still reads the shell grammar; `go vet ./...` and `go test ./...`
pass; frontend `check`, `build` and `vitest` pass with no amber in the built CSS.
