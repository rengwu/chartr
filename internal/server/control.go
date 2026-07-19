package server

import (
	"context"
	"net/http"
	"time"

	"github.com/coder/websocket"
)

// writeTimeout bounds a single snapshot write so one wedged browser socket
// cannot pin a goroutine forever; a timed-out write drops the connection and
// the browser resyncs on reconnect.
const writeTimeout = 10 * time.Second

// handleControl serves one browser's control socket. On connect it sends the
// current model snapshot immediately, then streams a fresh whole snapshot on
// every change (ADR 0010). The socket is push-only — the client never writes
// state through it — so a dropped or reconnecting browser simply gets the whole
// snapshot again, losing nothing.
func (s *Server) handleControl(w http.ResponseWriter, r *http.Request) {
	c, err := websocket.Accept(w, r, &websocket.AcceptOptions{
		// The cockpit is a single-operator tool bound to localhost and reached
		// through the Vite dev proxy in development, so the cross-origin Host
		// check would only get in the way here.
		InsecureSkipVerify: true,
	})
	if err != nil {
		return
	}
	defer c.CloseNow()

	// The control socket carries no client-authored messages, but we still must
	// drain incoming frames to process close/ping and notice disconnects.
	// CloseRead does exactly that and cancels ctx when the peer goes away.
	ctx := c.CloseRead(r.Context())

	sub, snapshot := s.hub.subscribe()
	defer s.hub.unsubscribe(sub)

	if err := writeSnapshot(ctx, c, snapshot); err != nil {
		return
	}

	for {
		select {
		case <-ctx.Done():
			return
		case <-sub.dead:
			// The hub killed us as a slow consumer; close and let the browser
			// reconnect for a fresh snapshot.
			c.Close(websocket.StatusPolicyViolation, "snapshot backlog")
			return
		case b := <-sub.ch:
			if err := writeSnapshot(ctx, c, b); err != nil {
				return
			}
		}
	}
}

// writeSnapshot writes one already-marshalled model snapshot as a text frame.
func writeSnapshot(ctx context.Context, c *websocket.Conn, snapshot []byte) error {
	ctx, cancel := context.WithTimeout(ctx, writeTimeout)
	defer cancel()
	return c.Write(ctx, websocket.MessageText, snapshot)
}
