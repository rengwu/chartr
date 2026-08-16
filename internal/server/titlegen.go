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
	"github.com/rengwu/chartr/internal/terminal"
)

// This file is the server half of auto-titling: the generator the terminal Manager
// spends its one attempt per session on. The Manager owns *when* — the first
// completed turn of a session with no native title, once ever (titler.go); this
// owns *how*.
//
// The how is same-vendor and same-account by design: a turn is summarised only by
// the harness already running it — a claude turn goes to claude, a codex turn to
// codex — and under the state root that session's own process resolved, so
// content the operator chose to run on one vendor or one login is never sent
// through another. Within that one adapter it still fails over cheapest model
// first, so a haiku rate limit falls through to the account's default rather than
// giving up; the whole fall-through is one attempt, not several.

const (
	// titleGenPerTry bounds one candidate's run. A cheap model answers in a couple
	// of seconds; the cap is only there so a wedged CLI can't hold the goroutine.
	titleGenPerTry = 45 * time.Second
	// titleGenMaxTries caps how far down the ladder a single generation walks, so a
	// run of declining models can't fan one title out into a dozen paid attempts.
	titleGenMaxTries = 4
	// titleMaxRunes clamps the final label — a marquee is short, and a model that
	// ignored the "few words" instruction is trimmed rather than trusted. It is the
	// cockpit's own display contract, shared with the native titles that never come
	// through this file at all.
	titleMaxRunes = terminal.MaxTitleRunes
)

// generateCheapTitle summarises one completed turn into a short label using the
// tab's own agent and profile. It walks that one adapter's cheapest-first model
// ladder until a model answers, and clamps how far it walks so a run of declines
// can't fan one title into many paid attempts. The whole walk is the session's one
// attempt: falling through candidates inside it is not a second one.
//
// Failing silently is the contract: an adapter chartr has no recipe for, a missing
// binary, a non-zero exit, a rate limit, empty output — every one just moves to the
// next rung, and an exhausted ladder returns ok=false so the caller shows no title
// and raises nothing. It runs off the transcript beat on its own goroutine and may
// block up to titleGenPerTry per candidate.
func (s *Server) generateCheapTitle(req terminal.TitleRequest) (string, bool) {
	prompt := buildTitlePrompt(req.Context)
	tries := 0
	// A single-adapter ladder: this tab's harness, its cheap models cheapest-first,
	// its own default model last.
	for _, c := range adapter.GenLadder([]string{req.Adapter}) {
		if tries >= titleGenMaxTries {
			break
		}
		argv, ok := adapter.GenCommand(c.Adapter, c.Model, prompt)
		if !ok {
			continue
		}
		tries++
		if title, ok := runGen(c.Adapter, argv, req.Env); ok {
			return title, true
		}
	}
	return "", false
}

// buildTitlePrompt wraps one turn in the instruction that turns it into a label.
// It asks for the title alone so a well-behaved model prints one clean line;
// cleanTitle defends against the rest.
func buildTitlePrompt(contextText string) string {
	return "You are labelling a terminal tab in a developer cockpit. Below is one exchange from an AI coding-agent session: the user's prompt, and the final response to it. Reply with a SHORT title of at most five words describing what the session is about. Reply with the title only — no quotes, no punctuation, no preamble, no explanation.\n\n--- TURN ---\n" + contextText
}

// runGen executes one ladder candidate and extracts a title from its stdout. The
// binary is the adapter name resolved on PATH, and the process runs under the host
// environment — the same auth basis a launched session runs under, which is what
// lets a cheap generation reuse the operator's existing login.
//
// profile is the tab's own state-root environment, appended last so it wins: a
// conversation held under a custom account or configuration directory is
// summarised under that same one, rather than under whichever default chartr
// itself happened to be started with. It is the only thing the live agent's
// environment contributes here. Anything but a clean run with a usable line is a
// silent miss.
func runGen(adapterName string, argv, profile []string) (string, bool) {
	ctx, cancel := context.WithTimeout(context.Background(), titleGenPerTry)
	defer cancel()
	cmd := exec.CommandContext(ctx, adapterName, argv...)
	// The generator is the operator's own agent binary; it runs under the
	// host environment, not the AppImage bundle's loader paths.
	cmd.Env = append(env.HostEnviron(), profile...)
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
