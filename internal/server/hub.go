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
	mu sync.RWMutex
	// The current snapshot, held both as the struct it was built from and as the
	// JSON every socket is sent. The struct is what lets a pure rearrangement be
	// republished without deriving the model from disk again (reorderSpaces).
	model     model.Model
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
	h.publish(m)
}

// reorderSpaces republishes the snapshot it already holds with its spaces
// permuted into `ids`, deriving nothing from disk. A reorder changes the
// sequence and nothing else: the branch, the dirty flag, the maps, the sessions
// and the resolved config in the last snapshot all still describe the same
// spaces, so rebuilding them is work whose only possible result is the spaces we
// already have in a different order.
//
// That work is not small. A rebuild forks `git status --porcelain` and re-reads
// `.git/HEAD` per space, walks each `.plan/` tree twice (the watch reconcile and
// the map scan, the latter reading every ticket), and re-resolves every config
// layer — all serially, all before the reorder's response returns. Paying it to
// permute a slice is what made the sidebar visibly lag the operator's drop.
//
// It reports whether it could. The snapshot must hold exactly the set `ids`
// names — a rebuild racing this reorder, or a space registered between them,
// makes the permutation undefined — and the caller falls back to a full rebuild
// when it does not. Correctness never depends on this path being taken.
func (h *hub) reorderSpaces(ids []string) bool {
	h.mu.RLock()
	spaces := h.model.Spaces
	h.mu.RUnlock()

	if len(spaces) != len(ids) {
		return false
	}
	by := make(map[string]model.Space, len(spaces))
	for _, sp := range spaces {
		by[sp.ID] = sp
	}
	ordered := make([]model.Space, 0, len(ids))
	for _, id := range ids {
		sp, ok := by[id]
		if !ok {
			// An unknown or repeated id: the second lookup of a repeat still hits,
			// but the length check above means a repeat implies an omission, which
			// misses. Either way the permutation is not a permutation.
			return false
		}
		ordered = append(ordered, sp)
	}

	h.mu.RLock()
	next := h.model
	h.mu.RUnlock()
	next.Spaces = ordered
	h.publish(next)
	return true
}

// publish marshals one model, retains it as the current snapshot, and fans it
// out. The struct is kept alongside its JSON so a change that is a pure
// rearrangement of what is already published — reorderSpaces — can be answered
// from it rather than derived from disk again.
func (h *hub) publish(m model.Model) {
	b, err := json.Marshal(m)
	if err != nil {
		// model is plain data with no custom marshalling; a failure here is a
		// programmer error, not a runtime condition.
		panic("server: marshalling model snapshot: " + err.Error())
	}

	h.mu.Lock()
	h.model = m
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
