package server

import (
	"encoding/json"
	"sync"

	"github.com/rengwu/chartr/internal/model"
)

// hub is the control-socket broadcaster. It holds the current model as
// pre-marshalled JSON and fans every change out to all connected browsers as a
// whole snapshot (ADR 0010). It is the one place model state is published; the
// walking skeleton only ever holds the empty model, but ticket 03 onward calls
// setModel to make maps and sessions appear live.
type hub struct {
	mu        sync.RWMutex
	modelJSON []byte
	subs      map[*subscriber]struct{}
}

// subscriber is one attached control socket. Snapshots are delivered through a
// small buffered channel; a browser that cannot keep up is killed rather than
// allowed to stall the broadcaster, and it re-syncs on reconnect (the snapshot
// is whole, so a dropped connection loses nothing).
type subscriber struct {
	ch   chan []byte
	dead chan struct{}
	once sync.Once
}

func (s *subscriber) kill() { s.once.Do(func() { close(s.dead) }) }

func newHub() *hub {
	h := &hub{subs: make(map[*subscriber]struct{})}
	h.setModel(model.Empty())
	return h
}

// setModel replaces the current model and pushes the new snapshot to every
// subscriber. Marshalling once here (not per subscriber) is deliberate: the
// snapshot is identical for every browser.
func (h *hub) setModel(m model.Model) {
	b, err := json.Marshal(m)
	if err != nil {
		// model is plain data with no custom marshalling; a failure here is a
		// programmer error, not a runtime condition.
		panic("server: marshalling model snapshot: " + err.Error())
	}

	h.mu.Lock()
	h.modelJSON = b
	subs := make([]*subscriber, 0, len(h.subs))
	for s := range h.subs {
		subs = append(subs, s)
	}
	h.mu.Unlock()

	for _, s := range subs {
		h.send(s, b)
	}
}

// subscribe registers a new socket and returns it together with the current
// snapshot captured under the same lock, so a change racing the registration is
// either included in this snapshot or delivered as a follow-up push — never
// dropped. A duplicate identical snapshot is harmless: the client replaces its
// state wholesale.
func (h *hub) subscribe() (*subscriber, []byte) {
	s := &subscriber{
		ch:   make(chan []byte, 8),
		dead: make(chan struct{}),
	}
	h.mu.Lock()
	snap := h.modelJSON
	h.subs[s] = struct{}{}
	h.mu.Unlock()
	return s, snap
}

func (h *hub) unsubscribe(s *subscriber) {
	h.mu.Lock()
	delete(h.subs, s)
	h.mu.Unlock()
}

// send is non-blocking: a subscriber whose buffer is full is a slow consumer,
// so it is killed and left to reconnect and resync rather than back-pressuring
// every other browser.
func (h *hub) send(s *subscriber, b []byte) {
	select {
	case s.ch <- b:
	case <-s.dead:
	default:
		s.kill()
	}
}
