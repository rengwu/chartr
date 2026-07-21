package server

import (
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/rengwu/wayfinder-harness/internal/config"
	"github.com/rengwu/wayfinder-harness/internal/mapscan"
	"github.com/rengwu/wayfinder-harness/internal/prompt"
	"github.com/rengwu/wayfinder-harness/internal/registry"
	"github.com/rengwu/wayfinder-harness/internal/terminal"
)

// Ticket 11: the pipeline from work landing to a readable verdict. An implementing
// session's committed `## Proposed Answer` derives the ticket `proposed` (the
// model layer already does this; spawn.go widened its gate so a review can seat on
// it). A review session composes the payload that carries the Done-when and the
// spec (prompt.Compose, story 53) and writes its verdict — in the clause-anchored
// format the review prompt defines — to `verdict.md` beside its payload.
//
// This file turns that verdict into the review brief: plain markdown on disk the
// GUI merely renders (story 62). The brief's recommendation is derived
// *mechanically* from the verdict, never lifted from the agent's prose (story 54):
// a finding gates only by citing the Done-when clause it breaks, so an unanchored
// finding — however strongly worded — lands advisory and cannot change the
// recommendation (story 55). The observed models (implement vs review, read from
// git history and the live session) are surfaced so heterogeneity is judged at the
// gate rather than falsely enforced in config (story 52).

// reviewVerdictName and reviewBriefName are the review session's two artifacts,
// siblings of its payload in the gitignored run directory. The reviewer writes the
// verdict; the harness assembles the brief.
const (
	reviewVerdictName = "verdict.md"
	reviewBriefName   = "brief.md"
)

// handleReviewBrief assembles the review brief for a review session from the
// verdict the reviewer wrote, and writes it to disk (story 62). It is a plain
// operator action — the GUI's "assemble brief" button — so a missing verdict or a
// ticket that is no longer proposed surfaces as a response rather than a silent
// gap. The brief is written to the session's gitignored run directory (a sibling
// of the payload and the verdict) and returned in the response.
func (s *Server) handleReviewBrief(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	info, ok := s.terms.Lookup(r.PathValue("sid"))
	if !ok || info.SpaceID != e.ID {
		httpError(w, http.StatusNotFound, "no such session")
		return
	}
	if info.Session == nil {
		httpError(w, http.StatusBadRequest, "that tab is an ad-hoc shell, not a session")
		return
	}
	if info.Session.Role != string(config.RoleReview) {
		httpError(w, http.StatusBadRequest, "that session is not a review — only a review session produces a verdict")
		return
	}

	brief, v, status, err := s.assembleReviewBrief(e, info)
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	// The brief's arrival is the moment the ticket reaches the human gate (ticket
	// 12), and nothing on the `.plan/` watch fires for a write into the gitignored
	// run directory — so push the new state explicitly. The star-map's human-review
	// state and the hub's entry point both read it.
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{
		"sessionId":      info.ID,
		"ticketNum":      info.Session.TicketNum,
		"brief":          brief,
		"recommendation": recommendation(v),
	})
}

// assembleReviewBrief is the assembly itself, shared by the explicit action above
// and by the hub opening on a ticket whose verdict is written but whose brief has
// not been built yet (ticket 12). It is a pure function of the verdict and the
// ticket on disk, so building it twice builds the same file — which is why the
// hub may build it on demand without making the pipeline any less deterministic.
// It returns the brief, the parsed verdict, and the HTTP status a caller should
// surface on failure.
func (s *Server) assembleReviewBrief(e registry.Entry, info terminal.Info) (string, verdict, int, error) {
	// The verdict the reviewer wrote, beside its payload in the run directory. Its
	// absence is the ordinary "the reviewer hasn't finished" case, not an error in
	// the harness.
	raw, err := os.ReadFile(filepath.Join(e.Path, sessionRunDir, info.ID, reviewVerdictName))
	if err != nil {
		return "", verdict{}, http.StatusConflict, errors.New("no verdict yet — the reviewer has not written verdict.md")
	}

	// The proposed answer the verdict judges, read verbatim off the ticket on disk
	// (never the snapshot), so the brief carries the exact prose a human reads.
	m, found := findMap(mapscan.Discover(e.Path), info.Session.MapSlug)
	if !found {
		return "", verdict{}, http.StatusNotFound, errors.New("no such map")
	}
	tk, found := findTicket(m, info.Session.TicketNum)
	if !found {
		return "", verdict{}, http.StatusNotFound, errors.New("no such ticket")
	}
	proposed := prompt.ProposedAnswerSection(tk.Body)
	if strings.TrimSpace(proposed) == "" {
		return "", verdict{}, http.StatusConflict, errors.New("the ticket carries no `## Proposed Answer` to review")
	}

	ticketPath, err := ticketFilePath(m.Dir, tk.Num)
	if err != nil {
		return "", verdict{}, http.StatusNotFound, err
	}

	v := parseVerdict(string(raw))
	brief := assembleBrief(briefInput{
		mapName:     m.Name,
		ticketNum:   tk.Num,
		ticketTitle: tk.Title,
		proposed:    proposed,
		verdict:     v,
		observed: observedModels{
			implement: observedImplement(e.Path, repoRel(e.Path, ticketPath)),
			review:    model2{agent: info.Session.Agent, model: info.Session.Model},
		},
	})

	briefPath := filepath.Join(e.Path, sessionRunDir, info.ID, reviewBriefName)
	if err := os.WriteFile(briefPath, []byte(brief), 0o644); err != nil {
		return "", verdict{}, http.StatusInternalServerError, errors.New("writing the review brief: " + err.Error())
	}
	// Re-assert the run directory's ignore, so the brief can never be swept into a
	// commit (ADR 0008) — the same guard the payload writer holds.
	_ = os.WriteFile(filepath.Join(e.Path, sessionRunDir, ".gitignore"), []byte("*\n"), 0o644)
	return brief, v, http.StatusOK, nil
}

// finding is one line of a verdict's Findings section: its prose and the Done-when
// clause it cites, if any. Clause is empty when the finding anchors to no clause —
// the structural mark that keeps it advisory (story 55).
type finding struct {
	Text   string
	Clause string
}

// Anchored reports whether the finding cites a Done-when clause — the only thing
// that lets it gate approval.
func (f finding) Anchored() bool { return strings.TrimSpace(f.Clause) != "" }

// verdict is the reviewer's assessment parsed from verdict.md. Line is the raw
// pass/fail line surfaced verbatim; Pass is its parsed sense (informational only —
// the recommendation is derived from the findings, not this word). Findings are
// split into those that cite a clause (able to block) and those that do not, but
// the split is re-applied at brief time so a "blocking"-marked finding with no
// clause is demoted to advisory.
type verdict struct {
	Line     string
	Pass     bool
	DoneWhen string
	Findings []markedFinding
}

// markedFinding pairs a parsed finding with whether the reviewer marked it
// blocking. The mechanical rule (recommendation, brief) is: a finding is a real
// blocker iff it is marked blocking *and* anchored; everything else is advisory.
type markedFinding struct {
	finding
	MarkedBlocking bool
}

// blocking returns the findings that actually gate — marked blocking and anchored
// to a Done-when clause. An unanchored "blocking" finding is not here.
func (v verdict) blocking() []finding {
	var out []finding
	for _, f := range v.Findings {
		if f.MarkedBlocking && f.Anchored() {
			out = append(out, f.finding)
		}
	}
	return out
}

// advisories returns every finding that does not gate: the reviewer's advisories
// plus any "blocking"-marked finding demoted for citing no clause (story 55). The
// demoted ones carry a note so the human sees why a finding the reviewer called
// blocking is filed advisory.
func (v verdict) advisories() []finding {
	var out []finding
	for _, f := range v.Findings {
		if f.MarkedBlocking && f.Anchored() {
			continue
		}
		note := f.finding
		if f.MarkedBlocking && !f.Anchored() {
			note.Text = note.Text + " _(the reviewer marked this blocking, but it cites no Done-when clause — advisory by rule)_"
		}
		out = append(out, note)
	}
	return out
}

var (
	reFindingItem   = regexp.MustCompile(`(?m)^\s*[-*]\s+(.*)$`)
	reBlockingMark  = regexp.MustCompile(`(?i)^\s*\(?blocking\b`)
	reAdvisoryMark  = regexp.MustCompile(`(?i)^\s*\(?advisory\b`)
	reClauseCite    = regexp.MustCompile(`(?i)Done-?when:\s*"([^"]+)"`)
	reClauseNoQuote = regexp.MustCompile(`(?i)Done-?when:\s*([^—\n]+?)\s*(?:—|-\s|\)|$)`)
)

// parseVerdict reads the reviewer's verdict.md into a verdict. It is tolerant: it
// reads the sections the review prompt defines but never refuses a malformed
// verdict — a verdict it cannot parse yields no findings, which the mechanical rule
// then reads as "nothing blocks", surfaced honestly in the brief rather than
// treated as a hidden pass.
func parseVerdict(raw string) verdict {
	v := verdict{}
	if line := firstNonEmptyLine(sectionBody(raw, "Verdict")); line != "" {
		v.Line = line
		low := strings.ToLower(line)
		v.Pass = strings.Contains(low, "pass") && !strings.Contains(low, "fail")
	}
	v.DoneWhen = sectionBody(raw, "Done-when")
	if v.DoneWhen == "" {
		v.DoneWhen = sectionBody(raw, "Done when")
	}
	for _, m := range reFindingItem.FindAllStringSubmatch(sectionBody(raw, "Findings"), -1) {
		item := strings.TrimSpace(m[1])
		if item == "" {
			continue
		}
		blocking := reBlockingMark.MatchString(item)
		advisory := reAdvisoryMark.MatchString(item)
		// An item marked neither is treated as blocking-candidate only if it cites a
		// clause; otherwise advisory. This keeps a plainly-written finding from
		// silently gating without a citation.
		v.Findings = append(v.Findings, markedFinding{
			finding:        finding{Text: item, Clause: clauseOf(item)},
			MarkedBlocking: blocking || (!advisory && clauseOf(item) != ""),
		})
	}
	return v
}

// clauseOf extracts the Done-when clause a finding cites, quoted form preferred.
// Empty when the finding anchors to no clause.
func clauseOf(item string) string {
	if m := reClauseCite.FindStringSubmatch(item); m != nil {
		return strings.TrimSpace(m[1])
	}
	if m := reClauseNoQuote.FindStringSubmatch(item); m != nil {
		return strings.TrimSpace(m[1])
	}
	return ""
}

// recommendation is the brief's mechanical verdict-to-action mapping (story 54):
// any anchored blocking finding means send back; none means approve. It is a pure
// function of the findings' anchoring — never the agent's pass/fail prose — so an
// unanchored finding can never flip it.
func recommendation(v verdict) string {
	if len(v.blocking()) > 0 {
		return "Send back"
	}
	return "Approve"
}

type model2 struct {
	agent string
	model string
}

func (o model2) String() string {
	switch {
	case o.model == "" && o.agent == "":
		return "unknown"
	case o.agent == "":
		return o.model
	case o.model == "":
		return o.agent
	default:
		return o.model + " (" + o.agent + ")"
	}
}

// observedModels is the heterogeneity line's data: the model each side actually
// ran, not what config declared (story 52). implement is read from the ticket's
// claim history; review from the live review session's binding.
type observedModels struct {
	implement model2
	review    model2
}

type briefInput struct {
	mapName     string
	ticketNum   int
	ticketTitle string
	proposed    string
	verdict     verdict
	observed    observedModels
}

// assembleBrief renders the review brief a human reads at the gate (story 54): the
// proposed answer verbatim, the one-line verdict with the blocking finding, the
// mechanically derived recommendation, the observed-model heterogeneity line, and
// the advisories. It is plain markdown — the GUI adds buttons and nothing else
// (story 62), so what a TUI-only operator reads on disk is exactly what the hub
// renders.
func assembleBrief(in briefInput) string {
	var b strings.Builder
	fmt.Fprintf(&b, "# Review brief — %s #%02d %s\n\n", orDashS(in.mapName), in.ticketNum, orDashS(in.ticketTitle))

	// Proposed answer, verbatim — the work under review, first.
	b.WriteString("## Proposed answer\n\n")
	b.WriteString(strings.TrimRight(in.proposed, "\n"))
	b.WriteString("\n\n")

	// The one-line verdict plus the blocking finding.
	b.WriteString("## Verdict\n\n")
	if in.verdict.Line != "" {
		b.WriteString(in.verdict.Line)
	} else {
		b.WriteString("_(the reviewer wrote no parseable verdict line)_")
	}
	b.WriteString("\n\n")

	blocking := in.verdict.blocking()
	b.WriteString("### Blocking finding\n\n")
	if len(blocking) == 0 {
		b.WriteString("_None — no finding cites a Done-when clause it breaks._\n\n")
	} else {
		b.WriteString(strings.TrimRight(blocking[0].Text, "\n"))
		b.WriteString("\n\n")
		for _, f := range blocking[1:] {
			b.WriteString("Also blocking: ")
			b.WriteString(strings.TrimRight(f.Text, "\n"))
			b.WriteString("\n\n")
		}
	}

	// The recommendation, derived mechanically from the verdict above.
	b.WriteString("## Recommendation\n\n")
	if len(blocking) > 0 {
		b.WriteString("**Send back.** A finding blocks by citing a Done-when clause it breaks; do not approve until it is addressed.\n\n")
	} else {
		b.WriteString("**Approve.** No finding blocks against a Done-when clause.\n\n")
	}

	// The reviewer's per-clause Done-when assessment, passed through verbatim — the
	// gate's clause-by-clause check (story 54). Informational: the recommendation
	// above is mechanical and does not read this prose.
	if dw := strings.TrimSpace(in.verdict.DoneWhen); dw != "" {
		b.WriteString("## Done-when assessment\n\n")
		b.WriteString(dw)
		b.WriteString("\n\n")
	}

	// Observed models — heterogeneity surfaced, not enforced.
	b.WriteString("## Observed models\n\n")
	fmt.Fprintf(&b, "Implement: %s · Review: %s", in.observed.implement, in.observed.review)
	if same := observedSame(in.observed); same != "" {
		b.WriteString(" — " + same)
	}
	b.WriteString("\n\n")

	// Advisories, including any finding demoted for citing no clause.
	if adv := in.verdict.advisories(); len(adv) > 0 {
		b.WriteString("## Advisories\n\n")
		for _, f := range adv {
			b.WriteString("- ")
			b.WriteString(strings.TrimRight(f.Text, "\n"))
			b.WriteString("\n")
		}
		b.WriteString("\n")
	}

	return strings.TrimRight(b.String(), "\n") + "\n"
}

// observedSame notes whether the two sides ran the same model — the marking-your-
// own-homework signal (story 52). Surfaced for the human to weigh, never a gate.
func observedSame(o observedModels) string {
	if o.implement.model == "" || o.review.model == "" {
		return ""
	}
	if strings.EqualFold(o.implement.model, o.review.model) {
		return "the same model reviewed its own work — weigh this verdict accordingly"
	}
	return "heterogeneous"
}

var reModelTrailer = regexp.MustCompile(`(?m)^Model:\s*(.+)\s*$`)
var reAgentTrailer = regexp.MustCompile(`(?m)^Agent:\s*(.+)\s*$`)
var reRoleTrailer = regexp.MustCompile(`(?m)^Role:\s*(.+)\s*$`)

// observedImplement reads the model an implement session actually ran from the
// ticket's claim history — the claim commit carries Agent/Model/Role trailers
// (ADR 0008), so "what reviewed which work on which model" is answered from git
// alone, no second store (story 69). It walks the ticket file's commits newest
// first and takes the first `Role: implement` claim's Agent/Model. Empty when no
// such claim is in history — the brief renders that honestly as "unknown".
func observedImplement(repo, ticketRel string) model2 {
	out, err := git(repo, "log", "--format=%B%x00", "--", ticketRel)
	if err != nil {
		return model2{}
	}
	for _, rec := range strings.Split(out, "\x00") {
		rm := reRoleTrailer.FindStringSubmatch(rec)
		if rm == nil || strings.TrimSpace(rm[1]) != string(config.RoleImplement) {
			continue
		}
		o := model2{}
		if m := reModelTrailer.FindStringSubmatch(rec); m != nil {
			o.model = strings.TrimSpace(m[1])
		}
		if m := reAgentTrailer.FindStringSubmatch(rec); m != nil {
			o.agent = strings.TrimSpace(m[1])
		}
		return o
	}
	return model2{}
}

// repoRel returns path relative to repo, or path unchanged when it cannot (which
// git then resolves against the repo root the same way).
func repoRel(repo, path string) string {
	if rel, err := filepath.Rel(repo, path); err == nil {
		return rel
	}
	return path
}

// sectionBody returns the markdown under the first `## <name>` heading up to the
// next heading of the same or higher level, case-insensitive on the heading text.
func sectionBody(body, name string) string {
	lines := strings.Split(body, "\n")
	want := "## " + strings.ToLower(name)
	for i, l := range lines {
		if strings.ToLower(strings.TrimSpace(l)) == want {
			var out []string
			for j := i + 1; j < len(lines); j++ {
				t := strings.TrimSpace(lines[j])
				if strings.HasPrefix(t, "## ") || strings.HasPrefix(t, "# ") {
					break
				}
				out = append(out, lines[j])
			}
			return strings.TrimSpace(strings.Join(out, "\n"))
		}
	}
	return ""
}

func firstNonEmptyLine(s string) string {
	for _, l := range strings.Split(s, "\n") {
		if t := strings.TrimSpace(l); t != "" {
			return t
		}
	}
	return ""
}

func orDashS(s string) string {
	if strings.TrimSpace(s) == "" {
		return "—"
	}
	return s
}
