package server

import "testing"

func TestCleanTitle(t *testing.T) {
	tests := []struct {
		name   string
		stdout string
		want   string
		wantOK bool
	}{
		{"plain", "Refactor auth flow", "Refactor auth flow", true},
		{"strips wrapping quotes", "\"Fix login bug\"", "Fix login bug", true},
		{"strips trailing period", "Wire up the sidebar.", "Wire up the sidebar", true},
		{"takes last non-blank line", "thinking...\n\nParse OSC titles\n", "Parse OSC titles", true},
		{"empty is not a title", "\n  \n", "", false},
		{"backticks stripped", "`grid reconstruction`", "grid reconstruction", true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := cleanTitle(tt.stdout)
			if ok != tt.wantOK {
				t.Fatalf("ok = %v, want %v", ok, tt.wantOK)
			}
			if got != tt.want {
				t.Fatalf("title = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestCleanTitleClampsLength(t *testing.T) {
	long := ""
	for range 100 {
		long += "x"
	}
	got, ok := cleanTitle(long)
	if !ok {
		t.Fatal("a long line is still a title")
	}
	if len([]rune(got)) > titleMaxRunes {
		t.Fatalf("title not clamped: %d runes", len([]rune(got)))
	}
}
