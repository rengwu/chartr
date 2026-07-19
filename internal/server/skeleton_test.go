package server_test

import (
	"context"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
)

// The walking skeleton's transport contract (ticket 01 Done-when): a snapshot
// arrives on connect, and a dropped connection gets the whole snapshot again on
// reconnect. These are the first tests of the process-boundary rig every later
// ticket extends.

func TestControlSocketSendsSnapshotOnConnect(t *testing.T) {
	h := harnesstest.Start(t)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	conn := h.DialControl(ctx)
	defer conn.Close()

	snap := conn.ReadSnapshot(ctx)

	// Near-empty, but well-formed: a non-nil, empty spaces array.
	if snap.Spaces == nil {
		t.Fatal("snapshot.spaces was null, want an empty array")
	}
	if len(snap.Spaces) != 0 {
		t.Fatalf("snapshot.spaces = %v, want empty", snap.Spaces)
	}
}

func TestControlSocketResendsSnapshotOnReconnect(t *testing.T) {
	h := harnesstest.Start(t)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	first := h.DialControl(ctx)
	snap1 := first.ReadSnapshot(ctx)

	// Drop the connection, then reconnect: the server must resend the whole
	// snapshot to the new connection with no action taken by the client.
	first.Close()

	second := h.DialControl(ctx)
	defer second.Close()
	snap2 := second.ReadSnapshot(ctx)

	if !reflect.DeepEqual(snap1, snap2) {
		t.Fatalf("reconnect snapshot = %#v, want the same whole snapshot %#v", snap2, snap1)
	}
}

func TestTwoBrowsersEachGetSnapshot(t *testing.T) {
	h := harnesstest.Start(t)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	a := h.DialControl(ctx)
	defer a.Close()
	b := h.DialControl(ctx)
	defer b.Close()

	// Each browser's control socket is independent and self-seeding.
	_ = a.ReadSnapshot(ctx)
	_ = b.ReadSnapshot(ctx)
}

func TestHealthActionOverHTTP(t *testing.T) {
	h := harnesstest.Start(t)

	code, body := h.Get("/api/health")
	if code != 200 {
		t.Fatalf("GET /api/health = %d, want 200", code)
	}
	if !strings.Contains(body, `"status":"ok"`) {
		t.Fatalf("GET /api/health body = %q, want it to report ok", body)
	}
}
