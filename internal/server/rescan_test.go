package server_test

import (
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// Ticket 19 at the process boundary: opening the control socket is itself a
// discovery notice, so the degraded watch — the state chartr falls back to when
// fsnotify cannot start — still shows an operator a map written from outside.

// With no watch at all, nothing fires a notice when a map lands in a registered
// space. The map still has to be in the very first snapshot a browser receives,
// with no operator action behind it: the connect ran the scan.
func TestConnectRescansWhenWatchIsDead(t *testing.T) {
	h := chartrtest.Start(t, chartrtest.WithoutWatch())
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	// Registration itself rebuilt, so this snapshot is the stale one the next
	// browser would otherwise be handed: the space is here, the map is not.
	if got := findSpace(t, h.Snapshot(ctx(t)), resp.ID).Maps; len(got) != 0 {
		t.Fatalf("space starts with %d maps, want 0", len(got))
	}

	chartrtest.WriteMap(t, repo, "unwatched-map", mapBody)
	chartrtest.WriteTicket(t, repo, "unwatched-map", "01-first.md", ticket(1, "First", "[]", "task", ""))

	cc := h.DialControl(ctx(t))
	defer cc.Close()

	// The *first* snapshot, not one waited for: no notice is ever coming.
	first := cc.ReadSnapshot(ctx(t))
	if !hasMap(findSpace(t, first, resp.ID), "unwatched-map") {
		t.Errorf("connect did not rescan; maps = %v", mapSlugs(findSpace(t, first, resp.ID)))
	}
}

// The live watch is unregressed: a browser already connected still receives a
// map by notice, without reconnecting to provoke the scan.
func TestWatchStillDiscoversForAConnectedBrowser(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	cc := h.DialControl(ctx(t))
	defer cc.Close()
	cc.ReadSnapshot(ctx(t)) // drain the connect snapshot

	chartrtest.WriteMap(t, repo, "watched-map", mapBody)
	chartrtest.WriteTicket(t, repo, "watched-map", "01-first.md", ticket(1, "First", "[]", "task", ""))

	cc.WaitFor(ctx(t), func(m model.Model) bool {
		return hasMap(findSpace(t, m, resp.ID), "watched-map")
	})
}
