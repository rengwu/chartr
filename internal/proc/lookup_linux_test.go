//go:build linux

package proc

import (
	"errors"
	"strconv"
	"strings"
	"testing"
)

// statLine builds a /proc/<pid>/stat line: pid, the executable name in
// parentheses, then the remaining fields from 3 onward. Only two of them are
// read — pgrp (5) and starttime (22) — but the offsets are what the parser gets
// right or wrong, so the fixtures carry a full-length tail.
func statLine(pid int, comm string, pgrp int, starttime uint64) string {
	fields := []string{
		strconv.Itoa(pid),  // 1 pid
		"(" + comm + ")",   // 2 comm
		"S",                // 3 state
		"1",                // 4 ppid
		strconv.Itoa(pgrp), // 5 pgrp
	}
	for f := 6; f <= 21; f++ {
		fields = append(fields, "0")
	}
	fields = append(fields, strconv.FormatUint(starttime, 10)) // 22 starttime
	fields = append(fields, "0", "0", "0")                     // and the tail after it
	return strings.Join(fields, " ") + "\n"
}

func TestParseStat(t *testing.T) {
	for _, tc := range []struct {
		name          string
		line          string
		wantPGID      int
		wantStartTime uint64
	}{{
		name:          "an ordinary process",
		line:          statLine(4242, "node", 4200, 987654),
		wantPGID:      4200,
		wantStartTime: 987654,
	}, {
		// The one documented hazard of the format, and the reason fields are
		// counted from the last `)` rather than from the start of the line.
		name:          "an executable whose name contains spaces and parentheses",
		line:          statLine(4242, "my agent) (v2", 4200, 987654),
		wantPGID:      4200,
		wantStartTime: 987654,
	}} {
		t.Run(tc.name, func(t *testing.T) {
			pgid, ticks, err := parseStat(tc.line)
			if err != nil {
				t.Fatalf("parseStat: %v", err)
			}
			if pgid != tc.wantPGID {
				t.Errorf("pgid = %d, want %d", pgid, tc.wantPGID)
			}
			if ticks != tc.wantStartTime {
				t.Errorf("starttime = %d, want %d", ticks, tc.wantStartTime)
			}
		})
	}
}

// A kernel whose stat format is not the one above is unavailable rather than
// parsed on a guess — the same rule the transcript adapters hold for a provider
// that changed its schema.
func TestParseStatRejectsAnUnknownShape(t *testing.T) {
	for _, tc := range []struct{ name, line string }{
		{"no closing parenthesis", "4242 node S 1 4200"},
		{"too few fields", "4242 (node) S 1 4200"},
		{"a non-numeric process group", "4242 (node) S 1 nope 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 987654 0"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if pgid, ticks, err := parseStat(tc.line); !errors.Is(err, errShape) {
				t.Fatalf("parseStat = %d, %d, %v; want errShape", pgid, ticks, err)
			}
		})
	}
}

// The environment arrives NUL-separated with a trailing separator, which must
// not become an empty variable.
func TestSplitNUL(t *testing.T) {
	got := splitNUL([]byte("CLAUDE_CONFIG_DIR=/home/op/.claude2\x00PATH=/usr/bin\x00"))
	want := []string{"CLAUDE_CONFIG_DIR=/home/op/.claude2", "PATH=/usr/bin"}
	if len(got) != len(want) {
		t.Fatalf("entries = %q, want %q", got, want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("entries = %q, want %q", got, want)
		}
	}
	if entries := splitNUL(nil); entries != nil {
		t.Errorf("splitNUL(nil) = %q, want nothing", entries)
	}
}
