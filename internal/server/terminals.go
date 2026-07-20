package server

import (
	"context"
	"encoding/json"
	"net/http"

	"github.com/coder/websocket"
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
