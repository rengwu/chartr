package server

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"path/filepath"

	"github.com/coder/websocket"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/notify"
	"github.com/rengwu/chartr/internal/registry"
	"github.com/rengwu/chartr/internal/terminal"
)

// handleOpenTerminal opens an ad-hoc shell in the space's working tree (story
// 29). Opening is a plain HTTP action so a spawn failure — a shell that will not
// start — surfaces as a response (ADR 0010); the shell's bytes then flow on the
// terminal socket keyed by the returned id. The new tab also lands in the pushed
// model, so a second browser sees it appear.
func (s *Server) handleOpenTerminal(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	t, err := s.terms.Open(e.ID, e.Path)
	if err != nil {
		// A shell that will not start is the operator's environment to fix.
		httpError(w, http.StatusInternalServerError, "opening shell: "+err.Error())
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"id": t.ID})
}

// handleFree is the new-shell control's endpoint: it starts a **free session** —
// an agent chartr launched into a space with no ticket, no role, and no brief. It
// is an operator affordance, not a role, so it shares only the adapter's spawn
// primitive with a real session — no map or ticket is looked up, no claim is
// written, nothing is injected, and the tab it seats carries no Session, so it
// reads and ends exactly like an ad-hoc shell (never the session grammar, never
// the death halt).
func (s *Server) handleFree(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	// The menu sends the agent it was clicked on, and nothing else: a free session
	// takes no skill and no context line — the operator types their first message
	// into the live TUI, which is what the TUI is for.
	var body struct {
		Agent string `json:"agent"`
	}
	if r.Body != nil {
		_ = json.NewDecoder(r.Body).Decode(&body)
	}
	s.launchFree(w, e, body.Agent)
}

// launchFree is the free-session spine: settle the chosen agent, launch its TUI
// bare, and remember the agent. A free session injects nothing — no payload is
// composed, written, or typed in; the operator gets a live agent tab and types
// their first message themselves. Every refusal is the same one a spawn gives, in
// the same order (agent-selection ticket 04), and a refusal opens nothing.
func (s *Server) launchFree(w http.ResponseWriter, e registry.Entry, agent string) {
	// The same doorstep, the same refusals, in the same order a spawn gives them.
	spec, status, err := agentSpec(s.resolve(e), agent)
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	id := newSessionID()

	// A bare launch: no opener, so `Command` returns the plain interactive argv
	// and nothing to type in. The skill mirror and write contract are the
	// business of a real spawn's payload (spawn.go) — a free session reads
	// neither, so it syncs neither.
	launch := adapter.Command(adapter.Spawn{
		Adapter: spec.Adapter,
		Args:    spec.Args,
		Deliver: spec.Prompt,
	})
	// Titled by the agent's registered name — the tab is titled by the thing the
	// operator clicked, which is the only labelling rule that never needs
	// explaining. Three free sessions on one agent get three identical titles, as
	// every ad-hoc shell in a space is titled `zsh` today.
	t, err := s.terms.OpenFree(e.ID, e.Path, id, launch.Name, launch.Args, spec.Env, spec.Name, spec.Adapter)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "opening the free tab: "+err.Error())
		return
	}

	// The space remembers what it just spawned with, so the next free session or
	// spawn here is one click — the same rule a real spawn follows (spawn.go).
	if spec.Name != "" {
		if err := s.reg.SetLastAgent(e.ID, spec.Name); err == nil {
			s.rebuild()
		}
	}
	writeJSON(w, http.StatusOK, map[string]string{"id": t.ID})
}

// handleCloseTerminal ends an ad-hoc shell on the human's command — ad-hoc
// shells have no lifecycle and are ended only by the operator (spec, State
// model). The tab drops from the model once the process finishes exiting, on the
// same path a natural `exit` takes.
func (s *Server) handleCloseTerminal(w http.ResponseWriter, r *http.Request) {
	if err := s.terms.Close(r.PathValue("termID")); err != nil {
		httpError(w, http.StatusNotFound, err.Error())
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

// onRunFinished is the server's end of the one RunFinished seam: the run clock
// folded a tab's published states and decided a run worth reporting has ended
// (session-notifications spec, "The clock is a pure function"). Every consumer of
// that event hangs here, and they are independent — the dot below records it in
// the cockpit, and the OS notification announces it elsewhere; neither re-derives
// the rule, and neither knows about the other.
//
// It runs on the sampler's goroutine, so it does what the manager's own change
// callback does: mark, then push a fresh snapshot, which is what makes the dot
// appear with no refresh (and appear on reload, since the flag is server state).
// The OS notification goes last and off this goroutine, because it execs: the
// cockpit's own half must not wait on a notification daemon, and a daemon that has
// wedged must not stall the sampler that every tab's status is read from.
func (s *Server) onRunFinished(ev terminal.RunFinished) {
	s.terms.MarkFinishedUnseen(ev.TerminalID)
	s.rebuild()
	go s.announceRun(ev)
}

// announceRun tells the operating system that a run ended, so the operator hears
// about it with the cockpit's browser tab closed — the case that motivated the
// whole effort (spec, story 2). It is best-effort by contract: a missing binary, a
// non-zero exit, a headless box with no notification daemon each leave the cockpit
// exactly as it was, and only the first of them is logged. Nothing here is retried
// and nothing here reaches the model.
func (s *Server) announceRun(ev terminal.RunFinished) {
	// The event carries the space's id, because that is what the terminal manager
	// knows; the operator knows the space by the name the sidebar gives it, which is
	// the registry's to answer. A space deregistered between the run ending and this
	// call simply goes unnamed rather than unreported.
	space := ""
	if e, ok := s.reg.Get(ev.SpaceID); ok {
		space = filepath.Base(e.Path)
	}

	err := s.notifier.Notify(notify.Compose(notify.Run{
		Space:     space,
		MapSlug:   ev.MapSlug,
		TicketNum: ev.TicketNum,
		Reason:    ev.Reason,
		Duration:  ev.Duration,
	}))
	if err != nil {
		// Once per process, not once per notification: an operator whose machine can
		// never notify should learn that once, and an operator whose machine can should
		// never learn anything at all.
		s.notifyFailed.Do(func() {
			log.Printf("chartr: could not fire an OS notification (%v); "+
				"session notifications are best-effort and this is logged once", err)
		})
	}
}

// handleTerminalSeen clears one tab's dot: the operator focused it, which is the
// whole acknowledgement. Focusing a tab that carries no dot is an ordinary no-op
// — the client posts on focus without knowing whether there was anything to clear
// — so it answers 204 either way and pushes only when something actually changed.
// An id that names no live terminal is a 404, exactly as ending one is.
func (s *Server) handleTerminalSeen(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("termID")
	if _, ok := s.terms.Lookup(id); !ok {
		httpError(w, http.StatusNotFound, "no such terminal")
		return
	}
	if s.terms.MarkSeen(id) {
		s.rebuild()
	}
	w.WriteHeader(http.StatusNoContent)
}

// handleTerminal serves one attached terminal's binary socket. On attach it
// replays the server-side scrollback as a single binary frame, then streams raw
// PTY bytes down as binary frames while carrying keystrokes up as binary frames
// and resize requests up as a small text-JSON control message (ADR 0006, 0010).
// The socket closes when the browser leaves, the terminal ends, or this attach
// falls behind and is dropped to reattach and replay.
func (s *Server) handleTerminal(w http.ResponseWriter, r *http.Request) {
	t, ok := s.terms.Get(r.PathValue("termID"))
	if !ok {
		http.Error(w, "no such terminal", http.StatusNotFound)
		return
	}

	c, err := websocket.Accept(w, r, &websocket.AcceptOptions{
		// Scoped to the address chartr is listening on (as on the control socket).
		// Inbound binary frames go straight to the PTY below, so an unscoped
		// handshake is keystroke injection into the operator's live shells and
		// agent sessions, as them — and scrollback replay carries them back out.
		// The refusal happens here, before Attach, so a turned-away handshake
		// writes nothing at all.
		OriginPatterns: s.origins,
	})
	if err != nil {
		return
	}
	defer c.CloseNow()

	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

	att := t.Attach()
	defer att.Detach()

	// Replay scrollback first so a reattaching browser walks back into the
	// running shell rather than a blank pane (ADR 0006).
	if len(att.Scrollback) > 0 {
		if err := writeTerminal(ctx, c, att.Scrollback); err != nil {
			return
		}
	}

	// Up: keystrokes as binary straight to the PTY; resize as a text control
	// message. The goroutine unblocks when the handler returns and cancel fires.
	go func() {
		for {
			typ, data, err := c.Read(ctx)
			if err != nil {
				return
			}
			switch typ {
			case websocket.MessageBinary:
				_, _ = t.Write(data)
			case websocket.MessageText:
				applyResize(t, data)
			}
		}
	}()

	// Down: raw PTY bytes as binary frames until the socket, the request, or the
	// terminal ends.
	for {
		select {
		case <-ctx.Done():
			return
		case <-att.Done:
			c.Close(websocket.StatusNormalClosure, "terminal ended")
			return
		case b := <-att.Frames:
			if err := writeTerminal(ctx, c, b); err != nil {
				return
			}
		}
	}
}

// terminalResizer is the resize surface of a terminal — narrowed so applyResize
// stays testable and does not reach past what it needs.
type terminalResizer interface {
	Resize(cols, rows int) error
}

// applyResize parses a text control frame and, if it is a resize request,
// reflows the PTY. Unknown or malformed control frames are ignored — the up
// channel is otherwise keystrokes, and a stray text frame must never wedge the
// socket.
func applyResize(t terminalResizer, data []byte) {
	var msg struct {
		Resize *struct {
			Cols int `json:"cols"`
			Rows int `json:"rows"`
		} `json:"resize"`
	}
	if err := json.Unmarshal(data, &msg); err != nil || msg.Resize == nil {
		return
	}
	if msg.Resize.Cols > 0 && msg.Resize.Rows > 0 {
		_ = t.Resize(msg.Resize.Cols, msg.Resize.Rows)
	}
}

// writeTerminal writes one raw chunk as a binary frame, bounded by the same
// per-write timeout the control socket uses so one wedged browser cannot pin a
// goroutine forever.
func writeTerminal(ctx context.Context, c *websocket.Conn, b []byte) error {
	ctx, cancel := context.WithTimeout(ctx, writeTimeout)
	defer cancel()
	return c.Write(ctx, websocket.MessageBinary, b)
}
