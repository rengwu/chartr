// Package detect turns the evidence an agent produces about itself — the OSC
// title it broadcasts, the OSC progress it pulses, and (from ticket 02) the
// screen it draws — into one of chartr's activity states. It is the real signal
// behind a tab's indicator, replacing the process-liveness proxy that could not
// tell a thinking agent from one waiting on its human.
//
// The work splits in two. Identify resolves a foreground process group to a known
// agent (or to nothing) by scoring the command names against per-agent manifests,
// so a node-launched claude reads as claude and a bare shell reads as no agent.
// Evaluate is then a pure function from (agent, evidence) to a state: it runs the
// agent's manifest rules — highest priority first — against the evidence regions
// and returns the first match, or a veto, or nothing.
//
// The rules are data, not code (per-agent TOML, go:embed'ed), so fixing a TUI
// that changed its title is a data edit rather than a recompile. The region seam
// (region) is the one place ticket 02 widens to add screen regions without
// touching rule evaluation.
package detect

import (
	"embed"
	"fmt"
	"io/fs"
	"path"
	"regexp"
	"sort"
	"strconv"
	"strings"

	"github.com/BurntSushi/toml"
)

//go:embed manifests/*.toml
var manifestFS embed.FS

// Evidence is everything the engine reads to decide a state. Title and Progress
// are the retained OSC values (this ticket); Screen is the reconstructed grid
// ticket 02 fills in. A region name maps to exactly one of these fields (region),
// which is why a new region is a new field plus one case, never a change to rule
// evaluation.
type Evidence struct {
	Title    string
	Progress string
	Screen   string
}

// Result is the engine's verdict for one sample. State is the matched activity
// (model.Terminal*), or "" when no rule matched — an *absence* the sampler treats
// as a candidate idle it must confirm, distinct from a rule that positively named
// idle. Veto is set when a skip_state_update rule matched: the sample is
// meaningless (a transcript viewer or model picker showing stale prompt text) and
// must not move the published state at all.
type Result struct {
	State string
	Veto  bool
}

// genericRuntime names the interpreters and multiplexers that launch an agent but
// are never themselves the agent. Identify skips them so a real match always wins
// — the "score candidates so a generic runtime never wins" rule — which is what
// makes a `node`-launched or `python`-launched agent resolve to the agent rather
// than to its runtime.
var genericRuntime = map[string]bool{
	"node": true, "nodejs": true, "bun": true, "deno": true,
	"python": true, "python3": true, "python2": true, "ruby": true,
	"npx": true, "pnpm": true, "yarn": true, "npm": true,
	"sh": true, "bash": true, "zsh": true, "fish": true, "dash": true,
	"tmux": true, "screen": true, "env": true, "sudo": true, "time": true,
}

// Rule is one entry in a manifest: a state to publish when its matchers all pass
// against its region. Priority orders rules (highest first); SkipStateUpdate marks
// a veto rule. The matcher fields are ANDed — every one that is set must pass — so
// a rule narrows rather than widens as fields are added.
type Rule struct {
	ID              string   `toml:"id"`
	State           string   `toml:"state"`
	Priority        int      `toml:"priority"`
	Region          string   `toml:"region"`
	Contains        []string `toml:"contains"`
	Any             []string `toml:"any"`
	All             []string `toml:"all"`
	Not             []string `toml:"not"`
	Regex           []string `toml:"regex"`
	LineRegex       []string `toml:"line_regex"`
	SkipStateUpdate bool     `toml:"skip_state_update"`

	regex     []*regexp.Regexp
	lineRegex []*regexp.Regexp
}

// Manifest is one agent's data: the process names that identify it and its ordered
// rules. Match is the argv basenames that name this agent; Aliases fold in the
// operator-facing names (claude-code → claude). Both, plus the agent's own name,
// become identifying tokens.
type Manifest struct {
	Agent   string   `toml:"agent"`
	Match   []string `toml:"match"`
	Aliases []string `toml:"aliases"`
	Rules   []Rule   `toml:"rule"`
}

// Engine is the parsed, ready-to-run set of manifests: each agent's rules
// (pre-sorted and pre-compiled) and the token→agent index Identify walks.
type Engine struct {
	manifests map[string]*Manifest
	tokens    map[string]string
}

// Builtin parses the manifests shipped with chartr. They are embedded and static,
// so a parse failure is a build-time defect, not a runtime condition — it panics,
// the template.Must convention.
func Builtin() *Engine {
	e, err := New(manifestFS, "manifests")
	if err != nil {
		panic(fmt.Sprintf("detect: parsing built-in manifests: %v", err))
	}
	return e
}

// New parses every *.toml under dir in fsys into an Engine. It is the seam a test
// uses to feed inline manifests (a synthetic veto rule, a matcher exercise)
// without touching the shipped data.
func New(fsys fs.FS, dir string) (*Engine, error) {
	entries, err := fs.ReadDir(fsys, dir)
	if err != nil {
		return nil, fmt.Errorf("detect: reading %s: %w", dir, err)
	}
	e := &Engine{manifests: map[string]*Manifest{}, tokens: map[string]string{}}
	for _, ent := range entries {
		if ent.IsDir() || !strings.HasSuffix(ent.Name(), ".toml") {
			continue
		}
		raw, err := fs.ReadFile(fsys, path.Join(dir, ent.Name()))
		if err != nil {
			return nil, fmt.Errorf("detect: reading %s: %w", ent.Name(), err)
		}
		m := &Manifest{}
		if err := toml.Unmarshal(raw, m); err != nil {
			return nil, fmt.Errorf("detect: parsing %s: %w", ent.Name(), err)
		}
		if m.Agent == "" {
			return nil, fmt.Errorf("detect: %s: manifest has no agent", ent.Name())
		}
		if err := e.add(m); err != nil {
			return nil, fmt.Errorf("detect: %s: %w", ent.Name(), err)
		}
	}
	return e, nil
}

// add compiles a manifest's rules, sorts them by descending priority, indexes its
// identifying tokens, and files it under its agent name.
func (e *Engine) add(m *Manifest) error {
	for i := range m.Rules {
		r := &m.Rules[i]
		for _, pat := range r.Regex {
			re, err := regexp.Compile(pat)
			if err != nil {
				return fmt.Errorf("rule %q: regex %q: %w", r.ID, pat, err)
			}
			r.regex = append(r.regex, re)
		}
		for _, pat := range r.LineRegex {
			re, err := regexp.Compile(pat)
			if err != nil {
				return fmt.Errorf("rule %q: line_regex %q: %w", r.ID, pat, err)
			}
			r.lineRegex = append(r.lineRegex, re)
		}
	}
	// Highest priority first; a stable sort keeps ties in declaration order.
	sort.SliceStable(m.Rules, func(i, j int) bool { return m.Rules[i].Priority > m.Rules[j].Priority })

	e.manifests[m.Agent] = m
	for _, tok := range append(append([]string{m.Agent}, m.Match...), m.Aliases...) {
		tok = strings.ToLower(tok)
		if tok != "" && !genericRuntime[tok] {
			e.tokens[tok] = m.Agent
		}
	}
	return nil
}

// Identify resolves a foreground process group's command names to a known agent,
// or "" for none. It scores every argv token across the group's processes — not
// just the leader's — skipping generic runtimes, so the real agent wins over the
// `node`/`python`/`sh` that launched it. names is the flat list of argv tokens
// (paths or bare words) gathered from the group.
func (e *Engine) Identify(names []string) string {
	for _, n := range names {
		base := strings.ToLower(path.Base(strings.TrimSpace(n)))
		if base == "" || base == "." || genericRuntime[base] {
			continue
		}
		if agent, ok := e.tokens[base]; ok {
			return agent
		}
	}
	return ""
}

// Evaluate runs agent's rules against ev and returns the first (highest-priority)
// match: its state, or a veto, or an empty Result when nothing matched. An unknown
// agent yields an empty Result — the caller falls back to the shell grammar.
func (e *Engine) Evaluate(agent string, ev Evidence) Result {
	m := e.manifests[agent]
	if m == nil {
		return Result{}
	}
	for i := range m.Rules {
		r := &m.Rules[i]
		if r.matches(region(r.Region, ev)) {
			if r.SkipStateUpdate {
				return Result{Veto: true}
			}
			return Result{State: r.State}
		}
	}
	return Result{}
}

// Known reports whether the engine ships a manifest for agent.
func (e *Engine) Known(agent string) bool { return e.manifests[agent] != nil }

// region is the single seam between a region name and the evidence it reads. The
// OSC regions serve the retained title/progress values; the screen regions slice
// the reconstructed grid (ev.Screen) structurally, which is what keeps a keyword in
// transcript prose from being read as live chrome. A region may carry an integer
// argument in parentheses, e.g. bottom_non_empty_lines(6). An unknown name reads
// empty rather than throwing, so a manifest can name a region the engine does not
// know without breaking evaluation.
func region(name string, ev Evidence) string {
	fn, arg := parseRegion(name)
	switch fn {
	case "osc_title":
		return ev.Title
	case "osc_progress":
		return ev.Progress
	case "screen", "whole_recent":
		return ev.Screen
	case "bottom_non_empty_lines":
		return bottomNonEmptyLines(ev.Screen, arg)
	case "after_last_horizontal_rule":
		return afterLastHorizontalRule(ev.Screen)
	case "prompt_box_body":
		return promptBoxBody(ev.Screen)
	default:
		return ""
	}
}

// parseRegion splits a region name into its function and an optional integer
// argument: "bottom_non_empty_lines(6)" → ("bottom_non_empty_lines", 6), and a
// bare "screen" → ("screen", 0). A malformed argument reads as 0.
func parseRegion(name string) (string, int) {
	open := strings.IndexByte(name, '(')
	if open < 0 || !strings.HasSuffix(name, ")") {
		return name, 0
	}
	arg, _ := strconv.Atoi(strings.TrimSpace(name[open+1 : len(name)-1]))
	return name[:open], arg
}

// hRuleGlyph is the light horizontal box-drawing character (U+2500) that claude and
// kimi rule a full-width line with to frame a region. hRuleMin is how many of them
// a line needs to count as a rule — enough to exclude a stray "──" inside prose.
const (
	hRuleGlyph = '─'
	hRuleMin   = 10
)

// isHorizontalRule reports whether a rendered line is one of the flat horizontal
// rules a TUI draws to frame a region: a run of U+2500 padded only by spaces. It
// deliberately rejects rounded, cornered box borders (╭ ╮ ╰ ╯ with │ sides), so
// kimi's bordered input box is not read as a rule while the flat rules both agents
// frame their prompt and permission/approval panels with are.
func isHorizontalRule(line string) bool {
	n := 0
	for _, r := range line {
		switch {
		case r == hRuleGlyph:
			n++
		case r == ' ' || r == '\t':
		default:
			return false
		}
	}
	return n >= hRuleMin
}

// bottomNonEmptyLines returns the last n non-blank lines of the screen, in
// top-to-bottom order — the coarse "recent activity" region, the live chrome an
// agent keeps at the foot of the screen without the scrolled-off transcript above.
func bottomNonEmptyLines(screen string, n int) string {
	if n <= 0 {
		return ""
	}
	lines := strings.Split(screen, "\n")
	var kept []string
	for i := len(lines) - 1; i >= 0 && len(kept) < n; i-- {
		if strings.TrimSpace(lines[i]) != "" {
			kept = append(kept, lines[i])
		}
	}
	for i, j := 0, len(kept)-1; i < j; i, j = i+1, j-1 {
		kept[i], kept[j] = kept[j], kept[i]
	}
	return strings.Join(kept, "\n")
}

// afterLastHorizontalRule returns the lines below the last flat horizontal rule on
// the screen. Claude frames a permission dialog with a rule above its body and
// nothing below, so this isolates the dialog's numbered options and footer hints
// from the transcript above the rule. Empty when the screen carries no rule.
func afterLastHorizontalRule(screen string) string {
	lines := strings.Split(screen, "\n")
	last := -1
	for i, l := range lines {
		if isHorizontalRule(l) {
			last = i
		}
	}
	if last < 0 {
		return ""
	}
	return strings.Join(lines[last+1:], "\n")
}

// promptBoxBody returns the lines between the last two flat horizontal rules — the
// body a TUI frames with a rule above and below. It is claude's input box at the
// prompt (a `❯` line between two rules) and kimi's approval panel (the whole menu
// between two rules); a rounded, cornered box is not framed by flat rules, so an
// idle kimi input box yields nothing here. Empty when fewer than two rules are
// present, which is the case on claude's one-rule permission dialog.
func promptBoxBody(screen string) string {
	lines := strings.Split(screen, "\n")
	var rules []int
	for i, l := range lines {
		if isHorizontalRule(l) {
			rules = append(rules, i)
		}
	}
	if len(rules) < 2 {
		return ""
	}
	top, bottom := rules[len(rules)-2], rules[len(rules)-1]
	return strings.Join(lines[top+1:bottom], "\n")
}

// matches reports whether every set matcher on the rule passes against text. An
// unset field imposes nothing; a rule with no matchers at all never matches, so a
// stray empty rule cannot swallow every sample.
func (r *Rule) matches(text string) bool {
	if r.empty() {
		return false
	}
	for _, s := range r.Contains {
		if !strings.Contains(text, s) {
			return false
		}
	}
	for _, s := range r.All {
		if !strings.Contains(text, s) {
			return false
		}
	}
	for _, s := range r.Not {
		if strings.Contains(text, s) {
			return false
		}
	}
	if len(r.Any) > 0 {
		found := false
		for _, s := range r.Any {
			if strings.Contains(text, s) {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}
	for _, re := range r.regex {
		if !re.MatchString(text) {
			return false
		}
	}
	for _, re := range r.lineRegex {
		if !matchesLine(text, re) {
			return false
		}
	}
	return true
}

// empty reports whether a rule specifies no matcher at all.
func (r *Rule) empty() bool {
	return len(r.Contains) == 0 && len(r.All) == 0 && len(r.Not) == 0 &&
		len(r.Any) == 0 && len(r.regex) == 0 && len(r.lineRegex) == 0
}

// matchesLine reports whether re matches at least one line of text. It is the
// line-anchored cousin of regex: a screen region is many lines (ticket 02), and a
// pattern meant for one row must not straddle a line break. For a single-line
// region (an OSC title) it behaves exactly like regex.
func matchesLine(text string, re *regexp.Regexp) bool {
	for _, line := range strings.Split(text, "\n") {
		if re.MatchString(line) {
			return true
		}
	}
	return false
}

