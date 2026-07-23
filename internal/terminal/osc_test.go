package terminal

import (
	"strings"
	"testing"
)

// feed runs chunks through one scanner in order, returning the retained title and
// progress — the latest value of each, which is all the sampler ever reads.
func feed(chunks ...string) (title, progress string) {
	var s oscScanner
	for _, c := range chunks {
		s.scan([]byte(c), func(v string) { title = v }, func(v string) { progress = v })
	}
	return title, progress
}

func TestOSCScanner(t *testing.T) {
	for _, tc := range []struct {
		name         string
		chunks       []string
		title, progr string
	}{
		{
			name:   "OSC 0 title terminated by BEL",
			chunks: []string{"\x1b]0;✳ Claude Code\x07"},
			title:  "✳ Claude Code",
		},
		{
			name:   "OSC 2 is a title too",
			chunks: []string{"\x1b]2;a window title\x07"},
			title:  "a window title",
		},
		{
			name:   "OSC 0 terminated by ST rather than BEL",
			chunks: []string{"\x1b]0;⠂ working\x1b\\"},
			title:  "⠂ working",
		},
		{
			// The case the ticket calls out: a title genuinely splits across two
			// Reads, so the scanner has to carry its state between feeds.
			name:   "a title split across chunk boundaries",
			chunks: []string{"\x1b]0;⠂ Count ", "to 10 ", "slowly\x07"},
			title:  "⠂ Count to 10 slowly",
		},
		{
			name:   "split even mid-escape and mid-terminator",
			chunks: []string{"\x1b", "]", "0", ";", "hi", "\x1b", "\\"},
			title:  "hi",
		},
		{
			name:   "only the latest title is retained",
			chunks: []string{"\x1b]0;first\x07some output\x1b]0;second\x07"},
			title:  "second",
		},
		{
			name:   "OSC 9 is retained as progress",
			chunks: []string{"\x1b]9;4;1;40\x07"},
			progr:  "4;1;40",
		},
		{
			name:   "title and progress are retained independently",
			chunks: []string{"\x1b]0;a title\x07\x1b]9;4;1;10\x07"},
			title:  "a title", progr: "4;1;10",
		},
		{
			// Kimi emits ~1000 OSC 8 hyperlinks a turn. They must not land anywhere.
			name:   "OSC 8 hyperlinks are discarded, not retained",
			chunks: []string{"\x1b]0;kept\x07" + strings.Repeat("\x1b]8;;https://example.com/x\x1b\\link\x1b]8;;\x1b\\", 200)},
			title:  "kept",
		},
		{
			name:   "an unrelated OSC never becomes the title",
			chunks: []string{"\x1b]11;?\x07"},
			title:  "",
		},
		{
			name:   "an empty title clears to empty rather than sticking",
			chunks: []string{"\x1b]0;something\x07", "\x1b]0;\x07"},
			title:  "",
		},
		{
			name:   "plain output is not mistaken for a sequence",
			chunks: []string{"just some ordinary output ]0; not an osc\n"},
			title:  "",
		},
		{
			name:   "a CSI sequence between titles is ignored",
			chunks: []string{"\x1b]0;one\x07\x1b[2J\x1b[H\x1b]0;two\x07"},
			title:  "two",
		},
		{
			// A sequence interrupted by a stray ESC is abandoned rather than
			// swallowing everything after it.
			name:   "an aborted OSC does not swallow the next one",
			chunks: []string{"\x1b]0;abandoned\x1b[0m\x1b]0;kept\x07"},
			title:  "kept",
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			title, progr := feed(tc.chunks...)
			if title != tc.title {
				t.Errorf("title = %q, want %q", title, tc.title)
			}
			if progr != tc.progr {
				t.Errorf("progress = %q, want %q", progr, tc.progr)
			}
		})
	}
}

// A byte-at-a-time feed must reach exactly the same answer as one whole chunk —
// the property that makes chunk boundaries a non-event.
func TestOSCScannerIsChunkingInvariant(t *testing.T) {
	stream := "\x1b]0;⠂ Count to 10 slowly\x07output\x1b]9;4;1;40\x07more\x1b]0;✳ done\x1b\\"

	wholeTitle, wholeProgr := feed(stream)

	var byteChunks []string
	for i := 0; i < len(stream); i++ {
		byteChunks = append(byteChunks, stream[i:i+1])
	}
	splitTitle, splitProgr := feed(byteChunks...)

	if splitTitle != wholeTitle || splitProgr != wholeProgr {
		t.Errorf("byte-at-a-time = (%q, %q), whole-chunk = (%q, %q); they must agree",
			splitTitle, splitProgr, wholeTitle, wholeProgr)
	}
	if wholeTitle != "✳ done" || wholeProgr != "4;1;40" {
		t.Errorf("retained (%q, %q), want (%q, %q)", wholeTitle, wholeProgr, "✳ done", "4;1;40")
	}
}
