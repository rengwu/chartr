package server_test

import (
	"bytes"
	"log"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/notify"
)

// The OS notification (session-notifications, ticket 03) at the process boundary,
// with a stub notifier substituted for the platform one — no test shells out to a
// real notifier on any platform, which is what keeps the suite honest on the OSes
// CI does not drive daily. A real tab works for a real span, the run clock folds
// the states it actually published, and the assertions are on what the operator is
// told: one notification, naming the space, the ticket, the reason and how long it
// ran. The argument vectors those strings become are `internal/notify`'s to prove.

// stubNotifier records what the operator would have been shown, and can be told to
// fail the way a machine with no notification daemon does.
type stubNotifier struct {
	mu   sync.Mutex
	err  error
	sent []notify.Notification
}

func (s *stubNotifier) Notify(n notify.Notification) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.sent = append(s.sent, n)
	return s.err
}

func (s *stubNotifier) all() []notify.Notification {
	s.mu.Lock()
	defer s.mu.Unlock()
	return append([]notify.Notification(nil), s.sent...)
}

// awaitNotification waits for the first notification and returns it. The notifier
// is fired off the sampler's goroutine — the cockpit must not wait on a
// notification daemon — so a test waits for it rather than reading it straight
// after the snapshot that carried the same event's dot.
func awaitNotification(t *testing.T, s *stubNotifier) notify.Notification {
	t.Helper()
	return awaitNotifications(t, s, 1)[0]
}

// awaitNotifications waits for at least n notifications and returns them all.
func awaitNotifications(t *testing.T, s *stubNotifier, n int) []notify.Notification {
	t.Helper()
	deadline := time.Now().Add(30 * time.Second)
	for {
		if sent := s.all(); len(sent) >= n {
			return sent
		}
		if time.Now().After(deadline) {
			t.Fatalf("only %d OS notifications fired for runs past the threshold, want %d",
				len(s.all()), n)
		}
		time.Sleep(20 * time.Millisecond)
	}
}

// startStubAgentRun stubs a `claude` binary on PATH, writes a one-ticket map into
// repo, registers it, and spawns a session against ticket 1 with it. The stub
// holds its PTY open saying nothing, so the tab reads working while it boots and
// settles to idle once the publisher's grace and absence confirmation pass — one
// real agent run, made of the states the tab published. It is the run clock's only
// driver now that a plain shell command reads `running` (never `working`) and so
// never opens a run at all; a test that needs a run the clock reports drives it
// through an agent, exactly as the operator's real notifiable runs are.
func startStubAgentRun(t *testing.T, h *chartrtest.Chartr, repo string) (spaceID, sessionID string) {
	t.Helper()
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)
	sp := mustSpawn(t, h, resp.ID, "widget", 1, "implement")
	return resp.ID, sp.SessionID
}

// A session that works past the threshold and settles tells the operator once,
// naming the space it ran in, the ticket it was bound to, how it ended and how
// long it took.
func TestFinishedSessionFiresOneOSNotification(t *testing.T) {
	stub := &stubNotifier{}
	h := chartrtest.Start(t, chartrtest.WithNotifier(stub))
	// A threshold and a settle small enough that the stub agent's own startup is a
	// qualifying run; the shipped 60s/10s would make this a minute long for no more
	// proof.
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", "after = \"10ms\"\nsettle = \"10ms\"\n")

	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	// The stub agent holds its PTY open saying nothing, so the tab reads working
	// while it boots and lands on idle once the publisher's grace and absence
	// confirmation are through — a real run, made of the states the tab published.
	chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	mustSpawn(t, h, resp.ID, "widget", 1, "implement")

	got := awaitNotification(t, stub)
	if got.Title != filepath.Base(repo) {
		t.Errorf("notification title = %q, want the space name %q", got.Title, filepath.Base(repo))
	}
	for _, want := range []string{"widget #01", "finished", "ran "} {
		if !strings.Contains(got.Body, want) {
			t.Errorf("notification body = %q, want it to name %q", got.Body, want)
		}
	}

	// Exactly one, not one per sample: the tab is idle and stays idle, so several
	// more sampler ticks are the whole of the proof that a settled run reports once.
	time.Sleep(time.Second)
	if sent := stub.all(); len(sent) != 1 {
		t.Errorf("one ended run fired %d notifications: %+v", len(sent), sent)
	}
}

// A plain process is silent. A command in an ad-hoc shell reads `running`, not the
// agent grammar's `working`, so it never opens a run on the clock — a dev server or
// a build finishing is not worth interrupting the operator, only an agent's turn is.
// However long the command holds the foreground, neither the dot nor the OS
// notification ever fires.
func TestPlainProcessFiresNoNotification(t *testing.T) {
	requireRunnableShell(t)
	stub := &stubNotifier{}
	h := chartrtest.Start(t, chartrtest.WithNotifier(stub))
	// A threshold so small any real run would qualify — so a silent tab proves the
	// process never opened a run, not merely that it ran too briefly to report.
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", "after = \"10ms\"\nsettle = \"10ms\"\n")
	resp := register(t, h, chartrtest.NewSpaceRepo(t))

	termID := h.OpenTerminal(resp.ID)
	tc := h.DialTerminal(dotCtx(t), termID)
	defer tc.Close()
	tc.Send(dotCtx(t), "sleep 2; echo done-$((6*7))\n")

	// The command really does hold the foreground — the tab reads `running`, the
	// shell grammar's state, which is the whole point: it is a run to the eye but not
	// to the clock.
	h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, resp.ID, termID) == model.TerminalRunning
	})
	tc.ReadUntil(dotCtx(t), "done-42")

	// Back at the prompt with no dot: the clock was never started, so there was no
	// run to end. Nothing is in flight — the notifier is only ever reached through an
	// event that was never created — so nothing here is a race.
	m := h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, resp.ID, termID) == model.TerminalIdle
	})
	if unseenDot(m, resp.ID, termID) {
		t.Fatal("a plain shell process raised a dot")
	}
	if sent := stub.all(); len(sent) != 0 {
		t.Errorf("a plain shell process fired %d notifications: %+v", len(sent), sent)
	}
}

// Turning notifications off turns the whole rule off at the source: no clock, so
// no event, so neither consumer fires. Driven by an agent run — one that would fire
// were the clock enabled — so the silence isolates the disabled flag rather than a
// process that was never notifiable in the first place.
func TestNotificationsDisabledFireNone(t *testing.T) {
	stub := &stubNotifier{}
	h := chartrtest.Start(t, chartrtest.WithNotifier(stub))
	chartrtest.WriteFile(t, h.ConfigDir,
		"notify.toml", "after = \"10ms\"\nsettle = \"10ms\"\nenabled = false\n")

	spaceID, sessionID := startStubAgentRun(t, h, chartrtest.NewSpaceRepo(t))

	// Let the agent's run complete: working while it boots, then idle once the
	// publisher's grace and absence confirmation pass. Were the clock live this run
	// would have fired; disabled, it created no event to fire.
	h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, spaceID, sessionID) == model.TerminalWorking
	})
	m := h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, spaceID, sessionID) == model.TerminalIdle
	})
	if unseenDot(m, spaceID, sessionID) {
		t.Error("a disabled clock raised a dot")
	}
	if sent := stub.all(); len(sent) != 0 {
		t.Errorf("a disabled clock fired %d notifications: %+v", len(sent), sent)
	}
}

// A notifier that fails is a fact about the machine, not about the run: the
// cockpit carries on, the model is exactly what it would have been, and the
// operator's dot is still there to find.
func TestFailingNotifierLeavesTheCockpitWorking(t *testing.T) {
	stub := &stubNotifier{err: errNoNotificationDaemon}
	h := chartrtest.Start(t, chartrtest.WithNotifier(stub))
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", "after = \"10ms\"\nsettle = \"10ms\"\n")

	spaceID, sessionID := startStubAgentRun(t, h, chartrtest.NewSpaceRepo(t))

	awaitNotification(t, stub) // it was attempted, and it failed

	if code, _ := h.Get("/api/health"); code != 200 {
		t.Errorf("health after a failed notification = %d, want 200", code)
	}
	// The snapshot is untouched by the failure: the dot the same event raised is
	// still there, and the tab is otherwise exactly as it was.
	m := h.SnapshotUntil(dotCtx(t), func(m model.Model) bool { return unseenDot(m, spaceID, sessionID) })
	if term := terminalIn(m, spaceID, sessionID); !term.Alive || term.Status == "" {
		t.Errorf("the failed notification changed the tab: %+v", term)
	}
}

// An operator on a machine that can never notify learns that once. Logging per
// notification would fill the log of the very operator the feature is failing,
// which is why the guard is per process rather than per event.
func TestFailingNotifierLogsOncePerProcess(t *testing.T) {
	logged := captureLog(t)

	stub := &stubNotifier{err: errNoNotificationDaemon}
	h := chartrtest.Start(t, chartrtest.WithNotifier(stub))
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", "after = \"10ms\"\nsettle = \"10ms\"\n")

	// Two runs, one after the other, so two notifications are attempted and both
	// fail — the second is what a per-notification log would have written twice. Each
	// runs in its own space, since the one-live-session-per-space gate allows one
	// session apiece.
	startStubAgentRun(t, h, chartrtest.NewSpaceRepo(t))
	awaitNotification(t, stub)
	startStubAgentRun(t, h, chartrtest.NewSpaceRepo(t))
	awaitNotifications(t, stub, 2)

	if n := strings.Count(logged(), "could not fire an OS notification"); n != 1 {
		t.Errorf("two failed notifications logged %d lines, want exactly 1:\n%s", n, logged())
	}
}

// captureLog redirects the standard logger for one test and returns a reader for
// what it collected. The logger is process-global, so this is why these tests do
// not run in parallel.
func captureLog(t *testing.T) func() string {
	t.Helper()
	buf := &syncBuffer{}
	log.SetOutput(buf)
	t.Cleanup(func() { log.SetOutput(os.Stderr) })
	return buf.String
}

type syncBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *syncBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *syncBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

// errNoNotificationDaemon stands in for the machine that cannot notify — a missing
// binary, a non-zero exit, a headless box.
var errNoNotificationDaemon = notifyError("no notification daemon on this machine")

type notifyError string

func (e notifyError) Error() string { return string(e) }
