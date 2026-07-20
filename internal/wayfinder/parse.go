package wayfinder

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"
)

// Status is derived from what a ticket already says — never read from a
// `status:` field, which would be a second copy free to go stale. See Derive.
type Status string

const (
	StatusOpen       Status = "open"
	StatusClaimed    Status = "claimed"
	StatusResolved   Status = "resolved"
	StatusOutOfScope Status = "out_of_scope"
	// StatusProposed is the harness's one addition to the derived-status table
	// (ADR 0004): an implementation ticket carrying `## Proposed Answer` but no
	// `## Answer` — work committed but no gate has blessed it. It is deliberately
	// *not* resolved, so the frontier scan (`^## (Answer|Ruled out)`) does not
	// match it and its dependents stay blocked until a human approves. `proposed`
	// derives from the file, so it survives a harness crash. A vanilla wayfinder
	// tool, which knows only the four statuses above, reads such a ticket as
	// `claimed` or `open` and still reads the map correctly.
	StatusProposed Status = "proposed"
)

// Closed reports whether the ticket is off the frontier for good. Out-of-scope
// tickets are closed but are not decisions on the route, so they never satisfy
// a Blocked-by edge.
func (s Status) Closed() bool { return s == StatusResolved || s == StatusOutOfScope }

type Type string

const (
	TypeResearch  Type = "research"
	TypePrototype Type = "prototype"
	TypeGrilling  Type = "grilling"
	TypeTask      Type = "task"
)

type Ticket struct {
	Num          int
	Slug         string
	Path         string
	Title        string
	Type         Type
	Status       Status
	BlockedBy    []int
	UnderminedBy []int
	ClaimedBy    string
	ClaimedAt    string
	Assets       []string

	// HasAnswer and HasRuledOut mean a closing section with prose under it. A
	// bare heading is not an answer, so the *Heading fields track it separately:
	// a session that died just after typing one must not read as finished.
	HasAnswer       bool
	HasRuledOut     bool
	AnswerHeading   bool
	RuledOutHeading bool

	// HasProposedAnswer means a `## Proposed Answer` section with prose under it
	// (ADR 0004) — the harness's non-resolving heading. Tracked the same way as a
	// real answer: a bare `## Proposed Answer` from a session that died mid-write
	// proposes nothing, so ProposedHeading records the heading separately.
	HasProposedAnswer bool
	ProposedHeading   bool

	Legacy       bool   // loose "Type:" header rather than YAML frontmatter
	StoredStatus Status // a deprecated `status:` field, if the file still carries one

	// Body is the ticket's markdown below its H1 title — Question and Done-when,
	// and any closing section (Answer / Proposed Answer / Ruled out). The harness
	// inlines it into the pushed model so the detail pane reads the full ticket
	// from one snapshot with no second fetch (ticket 07; starmap-design.md).
	Body string
}

// EmptyClosing reports a closing heading with nothing written under it.
func (t *Ticket) EmptyClosing() bool {
	return (t.AnswerHeading && !t.HasAnswer) || (t.RuledOutHeading && !t.HasRuledOut)
}

// Derive sets Status from the ticket's own contents. Closure is read first, so
// a claim left behind on a closed ticket is inert litter rather than a broken
// invariant: it can never hold the frontier. `proposed` is read after closure
// and before the claim, so a blessed answer always wins over a proposal it
// supersedes, and a ticket carrying a proposal reads `proposed` rather than the
// `claimed` its still-set claim would otherwise imply.
func (t *Ticket) Derive() {
	switch {
	case t.HasAnswer:
		t.Status = StatusResolved
	case t.HasRuledOut:
		t.Status = StatusOutOfScope
	case t.HasProposedAnswer:
		t.Status = StatusProposed
	case t.ClaimedBy != "":
		t.Status = StatusClaimed
	default:
		t.Status = StatusOpen
	}
}

type FogPatch struct {
	Title      string
	ClearsWith int // 0 when unanchored
	Line       int
}

type Decision struct {
	TicketNum int
	Line      int
}

type Map struct {
	Path        string
	Name        string
	Destination string
	Fog         []FogPatch
	Decisions   []Decision
	OutOfScope  []Decision

	// Body is the map's markdown below its H1 title — its Destination, Notes,
	// Decisions, Out of scope, and Not-yet-specified material. The harness inlines
	// it into the pushed model so the map-material pane (ticket 07) reads it whole.
	Body string
}

type Effort struct {
	Dir     string
	Name    string
	Map     *Map
	Tickets []*Ticket
}

func (e *Effort) ByNum(n int) *Ticket {
	for _, t := range e.Tickets {
		if t.Num == n {
			return t
		}
	}
	return nil
}

// Frontier returns the open, unclaimed tickets whose every blocker is resolved,
// in ticket-number order — the edge of the known.
func (e *Effort) Frontier() []*Ticket {
	var out []*Ticket
	for _, t := range e.Tickets {
		if t.Status != StatusOpen {
			continue
		}
		ready := true
		for _, b := range t.BlockedBy {
			dep := e.ByNum(b)
			if dep == nil || dep.Status != StatusResolved {
				ready = false
				break
			}
		}
		if ready {
			out = append(out, t)
		}
	}
	return out
}

func (e *Effort) Count(s Status) int {
	n := 0
	for _, t := range e.Tickets {
		if t.Status == s {
			n++
		}
	}
	return n
}

var (
	reLegacyKey = regexp.MustCompile(`(?m)^(Type|Status|Blocked by):[ \t]*(.*)$`)
	reFilename  = regexp.MustCompile(`^(\d+)-(.+)\.md$`)
	reDecision  = regexp.MustCompile(`\(\./tickets/(\d+)-[^)]*\)`)
	reFogTitle  = regexp.MustCompile(`^-\s+\*\*(.+?)\*\*`)
	reClearsRaw = regexp.MustCompile(`clears-with:\s*(\d+)`)
)

func blank(s string) bool { return strings.TrimSpace(s) == "" }

// fenceRun returns the delimiter a line opens or closes a code fence with —
// a run of three or more backticks or tildes — or "" if it is not a fence.
func fenceRun(line string) string {
	t := strings.TrimLeft(line, " \t")
	for _, c := range []byte{'`', '~'} {
		n := 0
		for n < len(t) && t[n] == c {
			n++
		}
		if n >= 3 {
			return t[:n]
		}
	}
	return ""
}

// splitScan returns the source's lines, plus a parallel slice in which every
// line inside a fenced code block — and the fence delimiters themselves — is
// blank.
//
// Structure is detected on the blanked copy and content is read from the
// original at the same index. A ticket whose Question quotes the ticket format
// must not resolve itself by containing the string "## Answer".
func splitScan(src string) (raw, scan []string) {
	raw = strings.Split(src, "\n")
	scan = make([]string, len(raw))
	open := ""
	for i, l := range raw {
		if open == "" {
			if d := fenceRun(l); d != "" {
				open = d
				continue
			}
			scan[i] = l
			continue
		}
		if d := fenceRun(l); d != "" && d[0] == open[0] && len(d) >= len(open) {
			open = ""
		}
	}
	return raw, scan
}

// sectionRange locates a `## <name>` heading in scan and returns the half-open
// line range of its body, up to the next `## ` heading. ok is false when absent.
func sectionRange(scan []string, name string) (start, end int, ok bool) {
	head := -1
	for i, l := range scan {
		if strings.TrimSpace(l) == "## "+name {
			head = i
			break
		}
	}
	if head < 0 {
		return 0, 0, false
	}
	start = head + 1
	for i := start; i < len(scan); i++ {
		if strings.HasPrefix(scan[i], "## ") {
			return start, i, true
		}
	}
	return start, len(scan), true
}

// sectionOf returns the raw body under `## name`. The body is taken from the
// original lines, so a section whose entire content is a code fence still
// counts as written.
func sectionOf(raw, scan []string, name string) string {
	start, end, ok := sectionRange(scan, name)
	if !ok {
		return ""
	}
	return strings.Join(raw[start:end], "\n")
}

func hasHeading(scan []string, name string) bool {
	_, _, ok := sectionRange(scan, name)
	return ok
}

// firstH1 returns the text of the first `# ` heading outside any code fence.
func firstH1(raw, scan []string, from int) string {
	for i := from; i < len(scan); i++ {
		if strings.HasPrefix(scan[i], "# ") {
			return strings.TrimSpace(strings.TrimPrefix(raw[i], "# "))
		}
	}
	return ""
}

// bodyAfterH1 returns the raw markdown after the first `# ` heading from `from`,
// trimmed — the material the detail pane renders. Fences are respected via scan,
// so a `# ` inside a code block is not mistaken for the title. When there is no
// H1, the whole region from `from` is the body.
func bodyAfterH1(raw, scan []string, from int) string {
	for i := from; i < len(scan); i++ {
		if strings.HasPrefix(scan[i], "# ") {
			return strings.TrimSpace(strings.Join(raw[i+1:], "\n"))
		}
	}
	return strings.TrimSpace(strings.Join(raw[from:], "\n"))
}

// parseFrontmatter splits a leading `---` delimited block into key/value pairs.
// Values are raw strings; list values keep their brackets for splitList. The
// second return is the line index at which the body begins.
func parseFrontmatter(raw []string) (map[string]string, int, bool) {
	if len(raw) == 0 || strings.TrimSpace(raw[0]) != "---" {
		return nil, 0, false
	}
	end := -1
	for i := 1; i < len(raw); i++ {
		if strings.TrimSpace(raw[i]) == "---" {
			end = i
			break
		}
	}
	if end < 0 {
		return nil, 0, false
	}

	kv := map[string]string{}
	for _, line := range raw[1:end] {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		i := strings.Index(line, ":")
		if i < 0 {
			continue
		}
		key := strings.TrimSpace(line[:i])
		val := strings.TrimSpace(line[i+1:])
		if c := strings.Index(val, " #"); c >= 0 {
			val = strings.TrimSpace(val[:c])
		}
		kv[key] = val
	}
	return kv, end + 1, true
}

func splitList(v string) []string {
	v = strings.TrimSpace(v)
	v = strings.TrimPrefix(v, "[")
	v = strings.TrimSuffix(v, "]")
	v = strings.TrimSpace(v)
	if v == "" || strings.EqualFold(v, "none") {
		return nil
	}
	var out []string
	for _, p := range strings.Split(v, ",") {
		if p = strings.TrimSpace(p); p != "" {
			out = append(out, p)
		}
	}
	return out
}

func splitNums(v string) ([]int, error) {
	var out []int
	for _, p := range splitList(v) {
		n, err := strconv.Atoi(strings.TrimLeft(p, "0"))
		if err != nil {
			return nil, fmt.Errorf("%q is not a ticket number", p)
		}
		out = append(out, n)
	}
	return out, nil
}

func ParseTicket(path, filename, src string) (*Ticket, error) {
	m := reFilename.FindStringSubmatch(filename)
	if m == nil {
		return nil, fmt.Errorf("filename %q is not NN-slug.md", filename)
	}
	num, _ := strconv.Atoi(strings.TrimLeft(m[1], "0"))

	t := &Ticket{Num: num, Slug: m[2], Path: path}
	raw, scan := splitScan(src)

	kv, bodyAt, ok := parseFrontmatter(raw)
	if !ok {
		kv = map[string]string{}
		bodyAt = 0
		t.Legacy = true
		for _, km := range reLegacyKey.FindAllStringSubmatch(strings.Join(scan, "\n"), -1) {
			key := strings.ToLower(km[1])
			if key == "blocked by" {
				key = "blocked_by"
			}
			kv[key] = strings.TrimSpace(km[2])
		}
	}

	t.Title = firstH1(raw, scan, bodyAt)
	t.Body = bodyAfterH1(raw, scan, bodyAt)

	t.Type = Type(strings.ToLower(kv["type"]))
	t.ClaimedBy = kv["claimed_by"]
	t.ClaimedAt = kv["claimed_at"]
	t.Assets = splitList(kv["assets"])

	t.AnswerHeading = hasHeading(scan, "Answer")
	t.RuledOutHeading = hasHeading(scan, "Ruled out")
	t.HasAnswer = t.AnswerHeading && !blank(sectionOf(raw, scan, "Answer"))
	t.HasRuledOut = t.RuledOutHeading && !blank(sectionOf(raw, scan, "Ruled out"))
	// `## Proposed Answer` is a distinct heading — the exact-match section scan
	// never confuses it with `## Answer`, so the frontier stays blind to it and
	// only the harness reads it as `proposed` (ADR 0004).
	t.ProposedHeading = hasHeading(scan, "Proposed Answer")
	t.HasProposedAnswer = t.ProposedHeading && !blank(sectionOf(raw, scan, "Proposed Answer"))
	if s := kv["status"]; s != "" {
		t.StoredStatus = Status(strings.ToLower(strings.ReplaceAll(s, " ", "_")))
	}
	t.Derive()

	var err error
	if t.BlockedBy, err = splitNums(kv["blocked_by"]); err != nil {
		return nil, fmt.Errorf("blocked_by: %w", err)
	}
	if t.UnderminedBy, err = splitNums(kv["undermined_by"]); err != nil {
		return nil, fmt.Errorf("undermined_by: %w", err)
	}
	return t, nil
}

type bullet struct {
	Text string
	Line int
}

// bullets splits a section into top-level `- ` bullets with their 1-based line
// numbers, folding indented continuation lines into the bullet they belong to.
// A bullet inside a code fence is not a bullet.
func bullets(raw, scan []string, start, end int) []bullet {
	var out []bullet
	for i := start; i < end; i++ {
		if strings.HasPrefix(scan[i], "- ") {
			out = append(out, bullet{raw[i], i + 1})
			continue
		}
		if len(out) > 0 && !blank(scan[i]) && strings.HasPrefix(scan[i], " ") {
			out[len(out)-1].Text += "\n" + raw[i]
		}
	}
	return out
}

func decisionsIn(raw, scan []string, name string) []Decision {
	start, end, ok := sectionRange(scan, name)
	if !ok {
		return nil
	}
	var out []Decision
	for _, b := range bullets(raw, scan, start, end) {
		for _, m := range reDecision.FindAllStringSubmatch(b.Text, -1) {
			n, _ := strconv.Atoi(strings.TrimLeft(m[1], "0"))
			out = append(out, Decision{TicketNum: n, Line: b.Line})
		}
	}
	return out
}

func ParseMap(path, src string) *Map {
	raw, scan := splitScan(src)
	m := &Map{Path: path}
	m.Name = firstH1(raw, scan, 0)
	m.Body = bodyAfterH1(raw, scan, 0)
	m.Destination = strings.TrimSpace(sectionOf(raw, scan, "Destination"))

	m.Decisions = decisionsIn(raw, scan, "Decisions so far")
	m.OutOfScope = decisionsIn(raw, scan, "Out of scope")

	if start, end, ok := sectionRange(scan, "Not yet specified"); ok {
		for _, b := range bullets(raw, scan, start, end) {
			p := FogPatch{Line: b.Line}
			if tm := reFogTitle.FindStringSubmatch(b.Text); tm != nil {
				p.Title = strings.TrimRight(tm[1], ".")
			}
			if cm := reClearsRaw.FindStringSubmatch(b.Text); cm != nil {
				p.ClearsWith, _ = strconv.Atoi(strings.TrimLeft(cm[1], "0"))
			}
			m.Fog = append(m.Fog, p)
		}
	}
	return m
}
