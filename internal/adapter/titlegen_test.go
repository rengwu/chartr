package adapter

import (
	"reflect"
	"testing"
)

func TestGenCommand(t *testing.T) {
	tests := []struct {
		name    string
		adapter string
		model   string
		want    []string
		wantOK  bool
	}{
		{"claude with model", "claude", "claude-haiku-4-5", []string{"-p", "hi", "--model", "claude-haiku-4-5"}, true},
		{"claude default model", "claude", "", []string{"-p", "hi"}, true},
		{"codex with model", "codex", "gpt-5-nano", []string{"exec", "--model", "gpt-5-nano", "hi"}, true},
		{"codex default model", "codex", "", []string{"exec", "hi"}, true},
		{"opencode with model", "opencode", "openai/gpt-5-nano", []string{"run", "-m", "openai/gpt-5-nano", "hi"}, true},
		{"opencode default model", "opencode", "", []string{"run", "hi"}, true},
		{"pi is ephemeral", "pi", "", []string{"--no-session", "-p", "hi"}, true},
		{"kimi default model", "kimi", "", []string{"-p", "hi"}, true},
		{"grok default model", "grok", "", []string{"-p", "hi"}, true},
		{"unknown adapter", "nothing-chartr-knows", "whatever", nil, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := GenCommand(tt.adapter, tt.model, "hi")
			if ok != tt.wantOK {
				t.Fatalf("ok = %v, want %v", ok, tt.wantOK)
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Fatalf("argv = %v, want %v", got, tt.want)
			}
		})
	}
}

// The prompt always lands in the final slot for codex, so an operator arg could
// never push it out (the same invariant Command keeps for the opener).
func TestGenCommandPromptTrailsCodex(t *testing.T) {
	argv, _ := GenCommand("codex", "gpt-5-mini", "summarise this")
	if argv[len(argv)-1] != "summarise this" {
		t.Fatalf("prompt not last: %v", argv)
	}
}

func TestGenLadderCheapestFirst(t *testing.T) {
	// Both adapters registered: the ladder merges and sorts by rank, so the global
	// cheapest (codex nano, rank 10) leads, then claude haiku (20), then codex mini
	// (40), claude sonnet (60), and finally each adapter's default at the bottom.
	got := GenLadder([]string{"claude", "codex"})
	want := []GenCandidate{
		{Adapter: "codex", Model: "gpt-5-nano", Rank: 10},
		{Adapter: "claude", Model: "claude-haiku-4-5", Rank: 20},
		{Adapter: "codex", Model: "gpt-5-mini", Rank: 40},
		{Adapter: "claude", Model: "claude-sonnet-4-5", Rank: 60},
		{Adapter: "claude", Rank: defaultRank},
		{Adapter: "codex", Rank: defaultRank},
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("ladder = %+v\nwant %+v", got, want)
	}
}

func TestGenLadderSkipsUnknownAndDuplicates(t *testing.T) {
	// An adapter chartr has no recipe for is dropped; a repeated adapter
	// contributes once.
	got := GenLadder([]string{"nothing-chartr-knows", "claude", "claude"})
	for _, c := range got {
		if c.Adapter != "claude" {
			t.Fatalf("unexpected adapter %q in ladder %+v", c.Adapter, got)
		}
	}
	// claude yields two named models plus one default = three rungs.
	if len(got) != 3 {
		t.Fatalf("want 3 claude rungs, got %d: %+v", len(got), got)
	}
}

func TestGenLadderTieBreaksByCallerOrder(t *testing.T) {
	// Give two adapters an equal-ranked model by faking the table, so the tie-break
	// is the only thing under test: the caller's order decides, deterministically.
	orig := genModels
	t.Cleanup(func() { genModels = orig })
	genModels = map[string][]genModel{
		"a": {{"a-model", 5}},
		"b": {{"b-model", 5}},
	}
	first := GenLadder([]string{"a", "b"})
	if first[0].Adapter != "a" {
		t.Fatalf("want a first, got %q", first[0].Adapter)
	}
	rotated := GenLadder([]string{"b", "a"})
	if rotated[0].Adapter != "b" {
		t.Fatalf("rotated: want b first, got %q", rotated[0].Adapter)
	}
}

func TestCanGenerate(t *testing.T) {
	// Every provider with a transcript adapter must also be able to spend, or a
	// tab with no native title would have no way to a title at all.
	for _, name := range []string{"claude", "codex", "opencode", "pi", "kimi", "grok"} {
		if !CanGenerate(name) {
			t.Errorf("%s has a transcript adapter but no headless recipe", name)
		}
	}
	if CanGenerate("nothing-chartr-knows") {
		t.Fatal("an unmeasured harness has no recipe")
	}
}

// A provider with no named cheap model still reaches the ladder: its single rung
// is the CLI's own default, which is what makes a measured-but-unpriced provider
// titleable rather than silently unspendable.
func TestGenLadderGivesADefaultOnlyProviderOneRung(t *testing.T) {
	got := GenLadder([]string{"grok"})
	want := []GenCandidate{{Adapter: "grok", Rank: defaultRank}}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("ladder = %+v, want %+v", got, want)
	}
}
