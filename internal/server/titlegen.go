package server

import (
	"bufio"
	"bytes"
	"context"
	"os/exec"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/env"
)

// This file is the server half of auto-titling: the generator the terminal Manager
// spends on when a tab is due for a label. The Manager owns *when* (idle,
// unchanged-guarded, debounced — titler.go); this owns *how*.
//
// The how is same-vendor by design: a tab's screen is summarised only by the
// harness already running it — a claude session's screen goes to claude, a codex
// session's to codex — so content the operator chose to run on one vendor is never
// sent to another. Within that one adapter it still fails over cheapest model
// first, so a haiku rate limit falls through to the account's default rather than
// giving up.

const (
	// titleGenPerTry bounds one candidate's run. A cheap model answers in a couple
	// of seconds; the cap is only there so a wedged CLI can't hold the goroutine.
	titleGenPerTry = 45 * time.Second
	// titleGenMaxTries caps how far down the ladder a single generation walks, so a
	// run of declining models can't fan one title out into a dozen paid attempts.
	titleGenMaxTries = 4
	// titleMaxRunes clamps the final label — a marquee is short, and a model that
	// ignored the "few words" instruction is trimmed rather than trusted.
	titleMaxRunes = 60
)

// generateCheapTitle summarises a tab's recent screen into a short label using the
// tab's own agent — same-vendor. It walks that one adapter's cheapest-first model
// ladder until a model answers, and clamps how far it walks so a run of declines
// can't fan one title into many paid attempts.
//
// Failing silently is the contract: an adapter chartr has no recipe for, a missing
// binary, a non-zero exit, a rate limit, empty output — every one just moves to the
// next rung, and an exhausted ladder returns ok=false so the caller shows no title
// and raises nothing. It runs off the sampler on its own goroutine and may block up
// to titleGenPerTry per attempt.
func (s *Server) generateCheapTitle(agent, contextText string) (string, bool) {
	prompt := buildTitlePrompt(contextText)
	tries := 0
	// A single-adapter ladder: this tab's harness, its cheap models cheapest-first,
	// its own default model last.
	for _, c := range adapter.GenLadder([]string{agent}) {
		if tries >= titleGenMaxTries {
			break
		}
		argv, ok := adapter.GenCommand(c.Adapter, c.Model, prompt)
		if !ok {
			continue
		}
		tries++
		if title, ok := runGen(c.Adapter, argv); ok {
			return title, true
		}
	}
	return "", false
}

// buildTitlePrompt wraps a tab's recent screen in the instruction that turns it
// into a label. It asks for the title alone so a well-behaved model prints one
// clean line; cleanTitle defends against the rest.
func buildTitlePrompt(contextText string) string {
	return "You are labelling a terminal tab in a developer cockpit. Below is the recent on-screen output of an AI coding-agent session. Reply with a SHORT title of at most five words describing what the session is about. Reply with the title only — no quotes, no punctuation, no preamble, no explanation.\n\n--- SESSION ---\n" + contextText
}

// runGen executes one ladder candidate and extracts a title from its stdout. The
// binary is the adapter name resolved on PATH, and the process inherits chartr's
// environment — the same auth basis a launched session runs under, which is what
// lets a cheap generation reuse the operator's existing login. Anything but a clean
// run with a usable line is a silent miss.
func runGen(adapterName string, argv []string) (string, bool) {
	ctx, cancel := context.WithTimeout(context.Background(), titleGenPerTry)
	defer cancel()
	cmd := exec.CommandContext(ctx, adapterName, argv...)
	// The generator is the operator's own agent binary; it runs under the
	// host environment, not the AppImage bundle's loader paths.
	cmd.Env = env.HostEnviron()
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return "", false
	}
	return cleanTitle(out.String())
}

// cleanTitle turns a model's raw stdout into a tab label: the last non-blank line
// (an agent that logs progress before its answer leaves the answer last), stripped
// of wrapping quotes and trailing sentence punctuation and clamped to
// titleMaxRunes. Output that cleans to nothing is not a title.
func cleanTitle(stdout string) (string, bool) {
	line := ""
	sc := bufio.NewScanner(strings.NewReader(stdout))
	sc.Buffer(make([]byte, 0, 64*1024), 1<<20)
	for sc.Scan() {
		if t := strings.TrimSpace(sc.Text()); t != "" {
			line = t
		}
	}
	line = strings.TrimSpace(strings.Trim(line, "\"'`"))
	line = strings.TrimSpace(strings.TrimRight(line, ".!,;:"))
	if line == "" {
		return "", false
	}
	if r := []rune(line); len(r) > titleMaxRunes {
		line = strings.TrimSpace(string(r[:titleMaxRunes]))
	}
	return line, true
}
