package server

import (
	"context"
	"encoding/json"
	"net/http"

	"github.com/coder/websocket"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
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

// handleLaunch is the skill launcher's endpoint: it runs any *on-ramp* skill on a
// chosen agent as a live, ticketless tab, with an optional line of context. It
// generalises the ideate on-ramp (ticket 15) rather than adding a parallel path —
// ideate is just the `skill=ideate` case, and `handleIdeate` below delegates
// here. It is the one opinionated nudge toward charting grown into a picker: an
// operator affordance, not a role, so it shares only the adapter's spawn primitive
// with a real session — no map or ticket is looked up, no claim is written, and
// the tab it seats carries no Session, so it reads and ends exactly like an ad-hoc
// shell (never the session grammar, never the death halt).
func (s *Server) handleLaunch(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	// The picker sends the agent, the on-ramp skill it chose, and — for a skill
	// that offers it — an optional line of context. An empty context is valid and
	// launches the skill bare.
	var body struct {
		Agent   string `json:"agent"`
		Skill   string `json:"skill"`
		Context string `json:"context"`
	}
	if r.Body != nil {
		_ = json.NewDecoder(r.Body).Decode(&body)
	}
	s.launchOnRamp(w, e, body.Agent, body.Skill, body.Context)
}

// handleIdeate keeps the `/ideate` route working as a thin delegate to the launch
// spine with `skill=ideate` and no context, so nothing mid-flight breaks while the
// frontend moves to `/launch`. It names its agent explicitly, exactly as a session
// does (ticket 03) — the operator sees and chooses what their on-ramp runs.
func (s *Server) handleIdeate(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	// An empty body is still a well-formed request that named no agent, which
	// agentSpec refuses with the same message as any other nameless one.
	var body struct {
		Agent string `json:"agent"`
	}
	if r.Body != nil {
		_ = json.NewDecoder(r.Body).Decode(&body)
	}
	s.launchOnRamp(w, e, body.Agent, prompt.IdeateSkill, "")
}

// launchOnRamp is the shared launch spine: settle the chosen agent, refuse a skill
// the resolved library does not mark on-ramp, compose that skill's payload alone
// (with the optional context riding inside it), write it to the gitignored run
// directory, launch the agent's TUI with the read-this-file opener, and remember
// the agent. It is the generalisation of the old ideate handler — every refusal is
// the same one a spawn gives, in the same order (ticket 04), and a refusal opens
// nothing and writes nothing.
func (s *Server) launchOnRamp(w http.ResponseWriter, e registry.Entry, agent, skill, context string) {
	// The same doorstep, the same refusals, in the same order a spawn gives them.
	spec, status, err := agentSpec(s.resolve(e), agent)
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	// The pushed library is the allowlist: the server launches only a skill it
	// resolves as `on-ramp`, never one the client merely named (as spawn refuses a
	// non-role). This is what keeps an augmentative or second-step skill — core,
	// to-tickets, implement — off the launcher even if a stale client asks for it.
	roots := s.skillRoots(e.Path)
	sk, ok := prompt.Resolve(skill, roots)
	if !ok || !sk.OnRamp {
		httpError(w, http.StatusBadRequest, "skill "+skill+" is not an on-ramp skill")
		return
	}

	id := newSessionID()
	promptPath, err := s.writeSessionPayload(e.Path, id, string(prompt.Launch(roots, skill, context)))
	if err != nil {
		httpError(w, http.StatusInternalServerError, "writing the launch prompt: "+err.Error())
		return
	}

	launch := adapter.Command(adapter.Spawn{
		Adapter: spec.Adapter,
		Args:    spec.Args,
		Prompt:  adapter.Opener(promptPath),
		Deliver: spec.Prompt,
	})
	t, err := s.terms.OpenOnRamp(e.ID, e.Path, id, launch.Name, launch.Args, launch.TypeIn, skill)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "opening the launch tab: "+err.Error())
		return
	}

	// The space remembers what it just spawned with, so the next launch or spawn
	// here is one click — the same rule a real spawn follows (spawn.go). There is no
	// remembered *skill*: the launcher is always a dropdown the operator picks from.
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
		// Single-operator localhost tool reached through the Vite dev proxy; the
		// cross-origin Host check would only get in the way (as on the control
		// socket).
		InsecureSkipVerify: true,
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
