package terminal

import (
	"hash/fnv"
	"strconv"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// This file is the tab-side half of the auto-title feature: it decides *when* a
// tab is worth spending a cheap generation on, and holds the result. The spending
// itself — which registered adapter, which model, the process — is the Manager's
// injected generator (server side); here there is only scheduling and the string.
//
// The scheduler is the same shape as publish.go's hysteresis: a pure struct a test
// drives directly, with no clock or PTY of its own, so the "only while idle,
// unchanged-guarded, debounced" rule is exercised without launching anything.

// titleContextCap bounds how much of the reconstructed screen is fed to the model —
// the tail, which is where a TUI agent keeps its most recent exchange. Small on
// purpose: a title needs the gist of the last couple of turns, not the transcript,
// and a small context is a cheap context.
const titleContextCap = 1500

// titleDebounce is the minimum gap between generation attempts on one tab. A var so
// a test can shrink it. Set to the middle of the operator-chosen 5–10 minute band:
// a title tracks a session as it evolves without churning a paid model on every
// idle beat.
var titleDebounce = 7 * time.Minute

// titleStartupGrace is how long after a tab first appears its screen is off-limits
// for titling, whatever it is doing. It is the floor under the worked-gate below: a
// just-launched agent shows a splash and its own boot chatter, and a spinner frame
// during init can read as "working" before the human has said a word — so the very
// first title waits out this window, and titles the first real turn instead of the
// greeting. A var so a test can shrink it.
var titleStartupGrace = 45 * time.Second

// titleSched is the pure hysteresis behind one tab's title. Given the tab's
// activity and a hash of its current screen, due reports whether to generate now,
// and records the attempt so a caller must actually launch it. It is deliberately
// free of any clock or lock so publish-style tests can step it by hand.
//
// Four guards keep it from titling noise:
//
//   - The worked-gate (worked): a title summarises a *conversation*, which only
//     exists once a turn has run, so nothing generates until the tab has been seen
//     in the working state. worked is set by observe and cleared on a landed title,
//     so each title also waits for a *fresh* turn — titles track turns, not the
//     screen repaints and scrolls between them.
//   - The startup grace (firstAt + startupGrace): the first title waits out a
//     tab's opening window, so a boot splash or an init spinner mistaken for work
//     never becomes the title.
//   - The unchanged-guard (lastHash): regenerate only once the screen has changed
//     since the last title that landed. lastHash advances on success, never on a
//     mere attempt, so a transient failure retries the same screen after the
//     debounce rather than being written off.
//   - The debounce (lastAt): advances on every attempt, so a run of failures still
//     can't hammer a model.
type titleSched struct {
	debounce     time.Duration
	startupGrace time.Duration
	firstAt      time.Time // when this tab was first sampled — the startup-grace anchor
	worked       bool      // seen working since the last title landed: a real turn happened
	lastHash     string    // the screen hash the current title was generated from
	lastAt       time.Time // when the last attempt fired (success or not)
	inFlight     bool      // a generation is running; hold off until it resolves
}

// observe folds one sample's activity in: any observation of the tab working means
// a turn is under way, which arms the worked-gate.
func (s *titleSched) observe(working bool) {
	if working {
		s.worked = true
	}
}

// due reports whether a fresh title should be generated for this idle state and
// screen hash, and — when it returns true — records that an attempt is now in
// flight. It fires only when the tab is idle, a turn has run since the last title
// (worked), the startup grace has passed, the screen is non-empty and differs from
// the title already shown, no generation is running, and the debounce has elapsed.
func (s *titleSched) due(idle bool, hash string, now time.Time) bool {
	if !idle || !s.worked || hash == "" || s.inFlight || hash == s.lastHash {
		return false
	}
	if !s.firstAt.IsZero() && now.Sub(s.firstAt) < s.startupGrace {
		return false
	}
	if !s.lastAt.IsZero() && now.Sub(s.lastAt) < s.debounce {
		return false
	}
	s.inFlight = true
	s.lastAt = now
	return true
}

// succeeded records a landed title: the screen it summarised becomes the new
// unchanged-guard, and the worked-gate re-arms so nothing regenerates until both
// the screen changes and another turn has run.
func (s *titleSched) succeeded(hash string) {
	s.lastHash = hash
	s.inFlight = false
	s.worked = false
}

// failed clears the in-flight flag without advancing the guard or disarming the
// worked-gate, so the same turn's screen is retried after the debounce — a rate
// limit or a missing binary is a pause, not a permanent verdict on this tab.
func (s *titleSched) failed() { s.inFlight = false }

// titleDue reads the tab's live screen and activity and, if a title is due,
// returns the foreground agent to generate on, the context to summarise, and the
// hash that keys its result — arming the scheduler so the caller must launch the
// generation. The agent is returned so titling stays same-vendor: a tab's screen is
// only ever sent to the harness already running it, never rotated to another
// vendor. The context is the tail of the reconstructed screen (titleContextCap
// runes), the same grid the sampler reads; a tab with no grid (a bare test
// terminal) is never due.
func (t *Terminal) titleDue(now time.Time) (agent, context, hash string, ok bool) {
	if t.grid == nil {
		return "", "", "", false
	}
	text := tailRunes(t.grid.text(), titleContextCap)
	h := hashScreen(text)

	t.mu.Lock()
	defer t.mu.Unlock()
	if t.sched.firstAt.IsZero() {
		// First sample of this tab: seat the scheduler's config and anchor the
		// startup grace to now (≈ tab birth — agent tabs are sampled from launch).
		t.sched.debounce = titleDebounce
		t.sched.startupGrace = titleStartupGrace
		t.sched.firstAt = now
	}
	// Feed activity in first, so a working sample arms the worked-gate even on the
	// tick it is seen; then ask whether an idle sample is due.
	t.sched.observe(t.state == model.TerminalWorking)
	if t.agent == "" || !t.sched.due(t.state == model.TerminalIdle, h, now) {
		return "", "", "", false
	}
	return t.agent, text, h, true
}

// titleSucceeded stores a generated title and reports whether it changed what the
// tab shows — the caller pushes a fresh model only when it did.
func (t *Terminal) titleSucceeded(hash, title string) bool {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.sched.succeeded(hash)
	if title == t.autoTitle {
		return false
	}
	t.autoTitle = title
	return true
}

// titleFailed releases the in-flight hold after a generation that produced nothing,
// so the tab is eligible again once the debounce passes.
func (t *Terminal) titleFailed() {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.sched.failed()
}

// tailRunes returns the last n runes of s, rune-safe so a multibyte glyph at the
// cut is never split. Short strings return whole.
func tailRunes(s string, n int) string {
	r := []rune(s)
	if len(r) <= n {
		return s
	}
	return string(r[len(r)-n:])
}

// hashScreen is the unchanged-guard key: a cheap 64-bit hash of the screen text,
// rendered as a decimal string so it composes with the sched's string field. Empty
// text hashes to "" so a blank screen is never "due".
func hashScreen(s string) string {
	if s == "" {
		return ""
	}
	h := fnv.New64a()
	_, _ = h.Write([]byte(s))
	return strconv.FormatUint(h.Sum64(), 10)
}
