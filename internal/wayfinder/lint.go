package wayfinder

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

type Level int

const (
	Warn Level = iota
	Error
)

func (l Level) String() string {
	if l == Error {
		return "error"
	}
	return "warn"
}

type Diagnostic struct {
	Level Level
	File  string
	Line  int
	Msg   string
}

type Options struct {
	StaleClaimAfter time.Duration
	Now             time.Time
}

func DefaultOptions() Options {
	return Options{StaleClaimAfter: 72 * time.Hour, Now: time.Now()}
}

var validTypes = map[Type]bool{TypeResearch: true, TypePrototype: true, TypeGrilling: true, TypeTask: true}
var validStatuses = map[Status]bool{StatusOpen: true, StatusClaimed: true, StatusResolved: true, StatusOutOfScope: true}

// Lint checks the invariants the wayfinder skill states as its format contract.
func Lint(e *Effort, opt Options) []Diagnostic {
	var d []Diagnostic
	add := func(l Level, file string, line int, format string, a ...any) {
		d = append(d, Diagnostic{l, file, line, fmt.Sprintf(format, a...)})
	}

	seen := map[int]*Ticket{}
	for _, t := range e.Tickets {
		if prev, dup := seen[t.Num]; dup {
			add(Error, t.Path, 0, "duplicate ticket number %02d (also %s) — two sessions likely created it in parallel", t.Num, prev.Path)
		}
		seen[t.Num] = t

		if t.Legacy {
			add(Warn, t.Path, 0, "loose header block; migrate Type/Blocked by into YAML frontmatter")
		}
		if t.Title == "" {
			add(Error, t.Path, 0, "no H1 title — the skill refers to tickets by name")
		}
		if !validTypes[t.Type] {
			add(Error, t.Path, 0, "type %q is not research|prototype|grilling|task", t.Type)
		}

		// A stored status is a second copy of a fact the body already carries.
		// Report the disagreement where there is one; otherwise say to drop it.
		if t.StoredStatus != "" {
			switch {
			case !validStatuses[t.StoredStatus]:
				add(Error, t.Path, 0, "stored status %q is not open|claimed|resolved|out_of_scope", t.StoredStatus)
			case t.StoredStatus != t.Status:
				add(Error, t.Path, 0, "stored status %q disagrees with the derived status %q — delete the field and let the body speak", t.StoredStatus, t.Status)
			default:
				add(Warn, t.Path, 0, "`status:` is derived from the body, not stored — delete the field")
			}
		}

		if t.AnswerHeading && t.RuledOutHeading {
			add(Error, t.Path, 0, "has both `## Answer` and `## Ruled out` — a ticket is a step on the route or a boundary of it, never both")
		}
		if t.EmptyClosing() {
			add(Error, t.Path, 0, "a closing heading with nothing under it — the answer *is* the resolution, so a bare heading resolves nothing; a session likely died mid-write")
		}

		switch {
		case t.Status.Closed():
			if t.ClaimedBy != "" || t.ClaimedAt != "" {
				add(Warn, t.Path, 0, "closed but still carries a claim — inert, but clear claimed_by and claimed_at")
			}
		case t.ClaimedBy != "" || t.ClaimedAt != "":
			if t.ClaimedBy == "" {
				add(Warn, t.Path, 0, "claimed_at with no claimed_by — a dead session is indistinguishable from live work")
			}
			if t.ClaimedAt == "" {
				add(Warn, t.Path, 0, "claimed with no claimed_at — cannot detect a stale claim")
			} else if at, err := time.Parse(time.RFC3339, t.ClaimedAt); err != nil {
				add(Error, t.Path, 0, "claimed_at %q is not RFC 3339", t.ClaimedAt)
			} else if age := opt.Now.Sub(at); age > opt.StaleClaimAfter {
				add(Warn, t.Path, 0, "claimed %s ago by %q — likely a dead session; the frontier skips it forever", age.Round(time.Hour), t.ClaimedBy)
			}
		}

		for _, b := range t.BlockedBy {
			dep := e.ByNum(b)
			if dep == nil {
				add(Error, t.Path, 0, "blocked_by %02d, which does not exist", b)
				continue
			}
			if b == t.Num {
				add(Error, t.Path, 0, "blocked_by itself")
			}
			if dep.Status == StatusOutOfScope {
				add(Warn, t.Path, 0, "blocked_by %02d, which was ruled out of scope — this ticket can never unblock", b)
			}
		}
		for _, u := range t.UnderminedBy {
			if e.ByNum(u) == nil {
				add(Error, t.Path, 0, "undermined_by %02d, which does not exist", u)
			}
		}
	}

	for _, c := range findCycles(e) {
		add(Error, e.Map.Path, 0, "blocked_by cycle: %s", c)
	}

	lintMap(e, add)
	return d
}

func lintMap(e *Effort, add func(Level, string, int, string, ...any)) {
	mp := e.Map.Path

	if strings.TrimSpace(e.Map.Destination) == "" {
		add(Error, mp, 0, "no Destination — every session orients to it before choosing a ticket")
	}

	listed := map[int]int{}
	for _, dec := range e.Map.Decisions {
		listed[dec.TicketNum]++
		t := e.ByNum(dec.TicketNum)
		switch {
		case t == nil:
			add(Error, mp, dec.Line, "Decisions-so-far links ticket %02d, which does not exist", dec.TicketNum)
		case t.Status == StatusOutOfScope:
			add(Error, mp, dec.Line, "Decisions-so-far lists %02d, which is out of scope — a scope boundary is not a step on the route", dec.TicketNum)
		case t.Status != StatusResolved:
			add(Error, mp, dec.Line, "Decisions-so-far lists %02d, which is %s, not resolved", dec.TicketNum, t.Status)
		}
	}
	for n, count := range listed {
		if count > 1 {
			add(Warn, mp, 0, "Decisions-so-far lists ticket %02d %d times", n, count)
		}
	}

	scoped := map[int]bool{}
	for _, dec := range e.Map.OutOfScope {
		scoped[dec.TicketNum] = true
	}

	for _, t := range e.Tickets {
		switch t.Status {
		case StatusResolved:
			if listed[t.Num] == 0 {
				add(Error, t.Path, 0, "resolved but absent from the map's Decisions-so-far — the map is the index and it now lies")
			}
		case StatusOutOfScope:
			if !scoped[t.Num] {
				add(Error, t.Path, 0, "out of scope but absent from the map's Out-of-scope section")
			}
		}
	}

	live := map[string]*Ticket{}
	for _, t := range e.Tickets {
		if !t.Status.Closed() {
			live[strings.ToLower(t.Title)] = t
		}
	}
	for _, f := range e.Map.Fog {
		if f.Title == "" {
			add(Warn, mp, f.Line, "fog patch has no bolded lead title — it has no identity to render or reference")
			continue
		}
		if t, dup := live[strings.ToLower(f.Title)]; dup {
			add(Error, mp, f.Line, "fog patch %q duplicates live ticket %02d — the same question is tracked twice", f.Title, t.Num)
		}
		if f.ClearsWith == 0 {
			continue
		}
		dep := e.ByNum(f.ClearsWith)
		if dep == nil {
			add(Error, mp, f.Line, "fog patch %q clears-with %02d, which does not exist", f.Title, f.ClearsWith)
		} else if dep.Status == StatusResolved {
			add(Error, mp, f.Line, "fog patch %q clears-with %02d, already resolved — it should have graduated into a ticket or been struck", f.Title, f.ClearsWith)
		}
	}
}

// findCycles reports each blocked_by cycle once, as a "01 → 02 → 01" path.
func findCycles(e *Effort) []string {
	const (
		white = 0
		grey  = 1
		black = 2
	)
	color := map[int]int{}
	var stack []int
	var found []string

	var visit func(n int)
	visit = func(n int) {
		color[n] = grey
		stack = append(stack, n)
		if t := e.ByNum(n); t != nil {
			for _, b := range t.BlockedBy {
				if e.ByNum(b) == nil {
					continue
				}
				switch color[b] {
				case white:
					visit(b)
				case grey:
					var path []string
					for i, s := range stack {
						if s == b {
							for _, p := range stack[i:] {
								path = append(path, fmt.Sprintf("%02d", p))
							}
							break
						}
					}
					path = append(path, fmt.Sprintf("%02d", b))
					found = append(found, strings.Join(path, " → "))
				}
			}
		}
		stack = stack[:len(stack)-1]
		color[n] = black
	}

	nums := make([]int, 0, len(e.Tickets))
	for _, t := range e.Tickets {
		nums = append(nums, t.Num)
	}
	sort.Ints(nums)
	for _, n := range nums {
		if color[n] == white {
			visit(n)
		}
	}
	return found
}
