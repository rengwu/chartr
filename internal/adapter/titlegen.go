package adapter

import "sort"

// This file is the adapter package's third kind of per-agent knowledge, beside
// prompt delivery (adapter.go) and launch preflight (preflight.go): how to run an
// agent's own CLI *headlessly* for a cheap one-shot completion — the primitive the
// auto-title feature spends on to summarise a tab's recent session into a label.
//
// It exists so the title generator never has a single point of failure. Rather
// than always reaching for one hard-coded model, chartr fans a request across
// every adapter the operator has registered, cheapest model first, and falls
// through to the next the moment one declines (a rate limit, an exhausted
// subscription, a missing binary). The cost data here is what orders that ladder.
//
// It is pure data and one small builder, exactly like the delivery table: teaching
// chartr to generate on a new harness is a row, never a change to the caller.

// genModel is one model an adapter can run for a throwaway title, with a coarse
// cross-adapter cost rank — lower is cheaper. The rank is an *ordinal that orders a
// fallback ladder*, never a price: comparing a rank across two vendors is only ever
// "try this one before that one", and the numbers are deliberately spread so a new
// model slots between two existing ones without renumbering.
type genModel struct {
	id   string
	rank int
}

// defaultRank sits far above every named model so an adapter's own default model —
// whatever the operator's CLI is configured to use — is only ever tried once every
// cheaper named candidate has already declined. It is the last resort on each
// adapter's rung, not a cheap option.
const defaultRank = 1000

// genModels lists, per adapter, the models cheap enough to spend on a title,
// cheapest first, with a cross-adapter rank. Ranks are approximate by nature (no
// two vendors price the same way); they order the ladder, they are not billing.
// Only adapters with an entry here can generate — the rest contribute nothing to
// the ladder, so a wrong guess for a harness chartr has never driven is impossible
// rather than merely unlikely.
//
// The cheapest known model per vendor leads. A model id that turns out wrong on
// this machine simply fails its one attempt and the ladder moves on, so listing an
// aggressively cheap id costs at most one skipped candidate, never a wrong charge.
var genModels = map[string][]genModel{
	// Claude Code: `claude -p <prompt> --model <id>`. Haiku is the cheapest active
	// Claude model ($1/$5 per Mtok); Sonnet is the next rung before the account's
	// own default is tried.
	"claude": {
		{"claude-haiku-4-5", 20},
		{"claude-sonnet-4-5", 60},
	},
	// Codex CLI: `codex exec --model <id> <prompt>`. The nano/mini OpenAI tier is
	// the cheapest; nano leads even though it may not be selectable on every
	// account, because a wrong id just skips a rung.
	"codex": {
		{"gpt-5-nano", 10},
		{"gpt-5-mini", 40},
	},
}

// GenCandidate is one (adapter, model) the title generator may try, carrying the
// rank that placed it on the ladder. An empty Model means "the adapter's own
// default model" — the flag-free invocation that works whatever the account is
// configured for, and the reason a wrong cheap-model id is never fatal.
type GenCandidate struct {
	Adapter string
	Model   string
	Rank    int
}

// CanGenerate reports whether chartr has a headless recipe for adapter — whether it
// can contribute to the title-generation ladder at all.
func CanGenerate(adapter string) bool {
	_, ok := genModels[adapter]
	return ok
}

// GenLadder builds the cheap-generation ladder for a set of adapter names: every
// adapter chartr can generate on, each contributing its named cheap models plus a
// default-model last resort, all merged and sorted cheapest-first. Auto-titling
// calls it with a single adapter (the tab's own — titling is same-vendor), so the
// ladder is that one harness's fallback rungs; it stays list-shaped so the cost
// data has one home whether one adapter or several are asked about.
//
// The sort is stable, and ties break by the order the caller passes adapters in, so
// the result is deterministic. Adapters chartr has no recipe for, and duplicate
// names, are dropped.
func GenLadder(adapters []string) []GenCandidate {
	seen := make(map[string]bool, len(adapters))
	// order records each adapter's position in the input so ties break by caller
	// order rather than by map iteration, which is what makes rotation meaningful.
	order := make(map[string]int, len(adapters))
	var out []GenCandidate
	for i, name := range adapters {
		if seen[name] {
			continue
		}
		models, ok := genModels[name]
		if !ok {
			continue
		}
		seen[name] = true
		order[name] = i
		for _, m := range models {
			out = append(out, GenCandidate{Adapter: name, Model: m.id, Rank: m.rank})
		}
		out = append(out, GenCandidate{Adapter: name, Rank: defaultRank})
	}
	sort.SliceStable(out, func(i, j int) bool {
		if out[i].Rank != out[j].Rank {
			return out[i].Rank < out[j].Rank
		}
		return order[out[i].Adapter] < order[out[j].Adapter]
	})
	return out
}

// GenCommand builds the argv (after the binary, which is the adapter name itself)
// that runs a one-shot headless completion on adapter for prompt, printing the
// answer to stdout. model selects a cheap model; an empty model runs the CLI's own
// default. It reports false for an adapter with no headless recipe — the same set
// GenLadder draws from, so a candidate off the ladder always resolves here.
func GenCommand(adapterName, model, prompt string) ([]string, bool) {
	switch adapterName {
	case "claude":
		// `claude -p <prompt>` is print mode: one prompt in, the completion on
		// stdout, no TUI. --model overrides the account default with a cheap one.
		argv := []string{"-p", prompt}
		if model != "" {
			argv = append(argv, "--model", model)
		}
		return argv, true
	case "codex":
		// `codex exec <prompt>` is codex's non-interactive path; --model picks a
		// cheap OpenAI model. The prompt trails so a flag never displaces it.
		argv := []string{"exec"}
		if model != "" {
			argv = append(argv, "--model", model)
		}
		return append(argv, prompt), true
	}
	return nil, false
}
