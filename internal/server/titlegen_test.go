package server

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/terminal"
)

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

// A generation runs under the live agent's own profile: the state-root variable
// the tab resolved reaches the subprocess, so a conversation held under a custom
// account or configuration directory is summarised under that same one instead of
// whichever default chartr itself was started with. The stub is a real executable
// named `claude` on PATH that prints the variable it was given.
func TestGenerationRunsUnderTheTabsProfile(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("the stub agent is a POSIX shell script")
	}
	bin := t.TempDir()
	script := "#!/bin/sh\nprintf '%s\\n' \"$CLAUDE_CONFIG_DIR\"\n"
	if err := os.WriteFile(filepath.Join(bin, "claude"), []byte(script), 0o755); err != nil {
		t.Fatalf("writing stub agent: %v", err)
	}
	t.Setenv("PATH", bin+string(os.PathListSeparator)+os.Getenv("PATH"))
	t.Setenv("CLAUDE_CONFIG_DIR", "/home/op/.claude-default")

	// A short literal root: nothing is read from it — the stub only echoes what it
	// was given — and a long temporary path would be clamped by the title contract.
	const root = "/opt/op/.claude-work"
	s := &Server{}
	got, ok := s.generateCheapTitle(terminal.TitleRequest{
		Adapter: "claude",
		Env:     []string{"CLAUDE_CONFIG_DIR=" + root},
		Context: "User prompt:\nwhat root am I under\n\nFinal response:\nthis one",
	})
	if !ok {
		t.Fatal("the stub adapter produced no title")
	}
	if got != strings.TrimSpace(root) {
		t.Fatalf("generation ran under %q, want the tab's own state root %q", got, root)
	}
}
