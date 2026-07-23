package terminal

// oscScanner pulls OSC (Operating System Command) sequences out of a raw PTY byte
// stream as they arrive, retaining the latest title and progress an agent
// broadcasts about itself. It is fed every chunk the read loop reads, so a
// sequence that splits across two Reads — a title genuinely does — is stitched
// back together by carrying parser state between feeds.
//
// It keeps only what the rule engine reads: OSC 0/2 (window/icon title) and OSC 9
// (progress). Every other OSC — Kimi emits ~1000 OSC 8 hyperlink sequences per
// turn — is skipped byte-by-byte without buffering, so the flood costs a state
// transition each and nothing more. Both terminators are honoured: BEL (0x07) and
// ST (ESC \).
type oscScanner struct {
	state    oscState
	code     int    // the numeric OSC code being read (Ps)
	haveCode bool   // at least one code digit seen
	kind     oscKind
	buf      []byte // payload of a kept OSC, capped
	escSeen  bool   // an ESC was seen mid-sequence; a following '\' is ST
}

type oscState uint8

const (
	oscGround oscState = iota // outside any escape
	oscEsc                    // saw ESC, waiting to see if it opens an OSC
	oscCode                   // inside an OSC, reading its numeric code
	oscKeep                   // inside a title/progress OSC, buffering its payload
	oscSkip                   // inside some other OSC, discarding to the terminator
)

type oscKind uint8

const (
	oscTitle oscKind = iota
	oscProgress
)

// oscPayloadCap bounds the buffer for one kept OSC. Titles are short; the cap only
// guards against a pathological unterminated sequence growing without bound. A
// payload past the cap is truncated but still read to its terminator.
const oscPayloadCap = 8 << 10

// scan feeds a chunk through the state machine, invoking onTitle / onProgress with
// the payload of each complete OSC 0/2 / OSC 9 it finds. The callbacks run on the
// read-loop goroutine; they must not block.
func (s *oscScanner) scan(chunk []byte, onTitle, onProgress func(string)) {
	for i := 0; i < len(chunk); {
		b := chunk[i]

		// Inside a sequence, ESC begins a possible ST terminator (ESC \). Any other
		// byte after that lone ESC aborts the sequence, and we reprocess it from
		// ground so a following CSI/OSC is not swallowed.
		if s.state != oscGround && s.state != oscEsc {
			if s.escSeen {
				s.escSeen = false
				if b == '\\' { // ST: finalize and consume
					s.finish(onTitle, onProgress)
					i++
					continue
				}
				// Stray ESC: abort this sequence, reprocess b from ground.
				s.abort()
				continue
			}
			if b == 0x1b {
				s.escSeen = true
				i++
				continue
			}
			if b == 0x07 { // BEL: finalize
				s.finish(onTitle, onProgress)
				i++
				continue
			}
		}

		switch s.state {
		case oscGround:
			switch b {
			case 0x1b:
				s.state = oscEsc
			case 0x9d: // C1 OSC
				s.beginOSC()
			}
		case oscEsc:
			if b == ']' {
				s.beginOSC()
			} else {
				// Not an OSC; drop back to ground. A second ESC keeps us waiting.
				if b == 0x1b {
					s.state = oscEsc
				} else {
					s.state = oscGround
				}
			}
		case oscCode:
			switch {
			case b >= '0' && b <= '9':
				s.code = s.code*10 + int(b-'0')
				s.haveCode = true
			case b == ';':
				s.enterPayload()
			default:
				// Malformed code; discard the rest of the sequence.
				s.state = oscSkip
			}
		case oscKeep:
			if len(s.buf) < oscPayloadCap {
				s.buf = append(s.buf, b)
			}
		case oscSkip:
			// discard
		}
		i++
	}
}

// beginOSC resets per-sequence state and starts reading the numeric code.
func (s *oscScanner) beginOSC() {
	s.state = oscCode
	s.code = 0
	s.haveCode = false
	s.buf = s.buf[:0]
	s.escSeen = false
}

// enterPayload is reached at the ';' after the code: decide whether this OSC is
// one we retain (0/2 title, 9 progress) or one we skip cheaply.
func (s *oscScanner) enterPayload() {
	switch {
	case s.code == 0 || s.code == 2:
		s.kind, s.state = oscTitle, oscKeep
	case s.code == 9:
		s.kind, s.state = oscProgress, oscKeep
	default:
		s.state = oscSkip
	}
}

// finish delivers a completed kept OSC and returns to ground.
func (s *oscScanner) finish(onTitle, onProgress func(string)) {
	if s.state == oscKeep {
		payload := string(s.buf)
		switch s.kind {
		case oscTitle:
			onTitle(payload)
		case oscProgress:
			onProgress(payload)
		}
	}
	s.reset()
}

// abort ends the current sequence without delivering anything (a malformed or
// interrupted OSC) and returns to ground.
func (s *oscScanner) abort() { s.reset() }

func (s *oscScanner) reset() {
	s.state = oscGround
	s.buf = s.buf[:0]
	s.escSeen = false
	s.haveCode = false
	s.code = 0
}
