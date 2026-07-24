---
type: task
---

# The OSC sniffer misreads a UTF-8 byte as a C1 control

## Question

`internal/terminal/osc.go` recognises the C1 OSC introducer as a raw `0x9D`
byte in ground state:

```go
case oscGround:
    switch b {
    case 0x1b: s.state = oscEsc
    case 0x9d: s.beginOSC()   // C1 OSC, matched on a bare byte
    }
```

The PTY stream is **UTF-8**, where `0x80`–`0xBF` are continuation bytes — and
`0x9D` is an ordinary one. Any non-ASCII character whose encoding contains `0x9D`
in normal (non-OSC) output trips the sniffer into a phantom OSC sequence:

- `Ý` (U+00DD) = `0xC3 0x9D`
- a range of CJK ideographs and emoji carry a `0x9D` byte

When it fires, the scanner enters `oscCode`/`oscSkip` mid-stream and swallows the
following bytes until the next `BEL` or `ESC \`. Two ways that hurts the one
thing this map is about — detection:

- a **real** OSC title arriving inside that window is lost, so the tab's state
  stops updating until the scanner happens to resync at the next escape; and
- if the swallowed bytes happen to shape up as `<code>;<text>\x07`, a
  **fabricated** title is handed to the rule engine as evidence.

It is confined to the detection path — the browser renders from raw scrollback
(`broadcast`), independent of the sniffer — and it partially self-heals at the
next escape sequence, so nothing visible corrupts. But detection *is* the point,
and the trigger is ordinary text an agent prints (a diff, a filename, CJK/emoji),
so a tab can silently read the wrong state on a normal turn.

`ESC` (`0x1B`) is safe by contrast: it is ASCII and never appears inside a UTF-8
multibyte sequence. Only the raw C1 path is unsafe — and note it is already
half-in/half-out, because C1 ST (`0x9C`) is *not* honoured as a terminator, so
nothing the sniffer starts on a C1 introducer could even close on a C1 ST.

**The fix.** Stop recognising the bare C1 introducer. Every agent measured for
this map (`rec-claude`, `rec-kimi`) opens its OSCs with the 7-bit `ESC ]` form,
which is unaffected, so dropping raw-C1 recognition costs no real coverage. If C1
support is wanted for completeness, it must be UTF-8-aware — a `0x9D`/`0x9C` that
is a continuation byte of a multibyte rune must not be read as a control — but
the simpler correct move is to drop it, matching the ST side that was never there.

While here, close the adjacent latent gap: `haveCode` is set but never checked,
so a no-code OSC (`ESC ] ; text BEL`) parses as `code == 0` and is delivered as a
title. Gate `enterPayload` on `haveCode` so a malformed sequence with no numeric
code is skipped rather than fabricating a title(0).

Tests lead: a scanner test feeding `0xC3 0x9D` ("Ý") in ground state and
asserting no title/progress callback fires and the state machine stays in ground;
a companion asserting a genuine `ESC ] 0 ; … BEL` immediately after such a
character is still read (proving the fix doesn't just swallow the next OSC too);
and a case pinning that `ESC ] ; x BEL` yields no title once `haveCode` gates it.
Extend against the real captures if a `0x9D`-bearing line exists in either
recording.

Done when: a `0x9D` byte in ground-state output no longer opens an OSC sequence;
the 7-bit `ESC ]` path and all existing OSC tests are unchanged and passing; a
malformed no-code OSC no longer delivers a title; and `go vet ./...` /
`go test ./...` pass.
