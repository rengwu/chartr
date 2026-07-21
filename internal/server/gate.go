package server

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/config"
	"github.com/rengwu/wayfinder-harness/internal/mapscan"
	"github.com/rengwu/wayfinder-harness/internal/model"
	"github.com/rengwu/wayfinder-harness/internal/prompt"
	"github.com/rengwu/wayfinder-harness/internal/registry"
)

// The human review hub (ticket 12) — the gate, whole. Ticket 11 left the brief on
// disk; this is the four exits a human takes from it, each a plain HTTP action so
// nothing about the gate happens without an operator's call:
//
//   - approve   — promote `## Proposed Answer` → `## Answer` as its own
//     pathspec-limited commit (never an amend, ADR 0008), which is what unblocks
//     the ticket's dependents on the stricter frontier. A rejecting verdict
//     requires exactly one acknowledgement (story 56) and the override is recorded
//     in the commit.
//   - send back — a fix-up session on the *still-proposed* ticket, briefed with
//     the blocking finding always, advisories opt-in, and the human's optional
//     note — all riding the injected payload and its archive, never the ticket
//     file (story 59): live steering and the ticket's permanent record stay apart.
//   - take it further — the same stacking mechanism without the fix-up framing:
//     more sessions on the same proposal, whose commits accumulate and whose
//     `## Proposed Answer` is rewritten in place (story 58).
//   - abandon   — demote the proposal to dated `### Rejected` prose carrying a
//     reason addressed to the next attempt, returning the ticket to the frontier
//     (story 60). It destroys nothing: revert (and reset, when the work commits
//     are verifiably the tip) are levers a human pulls, off by default.
//
// The GUI adds buttons and nothing else (story 62): the brief these act on is the
// markdown ticket 11 wrote to disk, served back verbatim by handleReviewRead.

// gateTarget resolves the {space, map, ticket} a gate action names, fresh off
// disk (never a cached snapshot — the ticket may have moved since the last push)
// and requires the ticket to be `proposed`: the gate acts on a proposal, and only
// a proposal. It writes the error response and returns ok=false otherwise.
func (s *Server) gateTarget(w http.ResponseWriter, r *http.Request) (registry.Entry, model.Map, model.Ticket, bool) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return registry.Entry{}, model.Map{}, model.Ticket{}, false
	}
	num, err := strconv.Atoi(r.PathValue("num"))
	if err != nil {
		httpError(w, http.StatusBadRequest, "ticket number must be an integer")
		return registry.Entry{}, model.Map{}, model.Ticket{}, false
	}
	slug := r.PathValue("slug")
	m, found := findMap(mapscan.Discover(e.Path), slug)
	if !found {
		httpError(w, http.StatusNotFound, "no such map")
		return registry.Entry{}, model.Map{}, model.Ticket{}, false
	}
	if kind, ok := s.resolve(e).Kinds[slug]; ok {
		m.Kind = kind
	}
	tk, found := findTicket(m, num)
	if !found {
		httpError(w, http.StatusNotFound, "no such ticket")
		return registry.Entry{}, model.Map{}, model.Ticket{}, false
	}
	if tk.Status != "proposed" {
		httpError(w, http.StatusConflict,
			"the gate acts on a proposed ticket — this ticket is "+tk.Status+", not proposed")
		return registry.Entry{}, model.Map{}, model.Ticket{}, false
	}
	// A verdict written but not yet assembled is built here, before any exit reads
	// it. Every gate action comes through this function, so the verdict a reviewer
	// wrote can never be *missed* by the gate — which matters most for approve,
	// where an unread blocking finding would otherwise mean one fewer tick rather
	// than one more. The assembly is a pure function of the verdict and the ticket,
	// so doing it here changes when the file appears, never what it says.
	s.ensureBrief(e, slug, num)
	return e, m, tk, true
}

// handleReviewRead serves the brief a human reads at the gate, straight off disk
// (story 62): the exact markdown ticket 11 assembled, plus the mechanical shape
// the hub's buttons key on — the recommendation, the blocking findings (which
// gate approval behind one acknowledgement) and the advisories (which the
// send-back dialog offers as opt-in ticks). Nothing here is the agent's prose
// about what to do; the recommendation is derived from the findings' anchoring.
//
// A 409 is the ordinary "no brief yet" case — no review has run, or its verdict
// is unwritten — not an error in the harness.
func (s *Server) handleReviewRead(w http.ResponseWriter, r *http.Request) {
	e, m, tk, ok := s.gateTarget(w, r)
	if !ok {
		return
	}
	rv, sid, found := s.reviewFor(e, m.Slug, tk.Num)
	if !found {
		httpError(w, http.StatusConflict,
			"no review brief for this ticket yet — the reviewer has not written its verdict")
		return
	}
	brief, err := os.ReadFile(filepath.Join(e.Path, sessionRunDir, sid, reviewBriefName))
	if err != nil {
		httpError(w, http.StatusConflict, "the review brief is no longer on disk — re-assemble it from the review session")
		return
	}

	// resetAvailable mirrors abandon's own tip check (a clean tree is not asserted
	// here — the actual abandon call re-checks it, since the tree can go dirty
	// between this read and that write) so the abandon dialog can offer the reset
	// lever only where it would actually be accepted, rather than always showing it
	// and letting the operator discover the refusal after already writing a reason
	// (ticket 17: the dialog only ever offered revert).
	resetAvailable := false
	if ticketPath, err := ticketFilePath(m.Dir, tk.Num); err == nil {
		resetAvailable = tipOf(e.Path, workCommits(e.Path, repoRel(e.Path, ticketPath)))
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"sessionId":      sid,
		"ticketNum":      tk.Num,
		"brief":          string(brief),
		"recommendation": recommendation(rv),
		"verdictLine":    rv.Line,
		"blocking":       findingsJSON(rv.blocking()),
		"advisories":     findingsJSON(rv.advisories()),
		"proposedAnswer": prompt.ProposedAnswerSection(tk.Body),
		"resetAvailable": resetAvailable,
	})
}

func findingsJSON(fs []finding) []map[string]any {
	out := make([]map[string]any, 0, len(fs))
	for _, f := range fs {
		out = append(out, map[string]any{"text": f.Text, "clause": f.Clause})
	}
	return out
}

// handleApprove is the gate's one-click exit (story 56). It promotes the ticket's
// `## Proposed Answer` to `## Answer` as its own pathspec-limited commit — never
// an amend (story 65), and never waiting on a live session (ADR 0008) — which is
// the act that unblocks the ticket's dependents on the stricter frontier.
//
// A rejecting verdict costs exactly one tick: `acknowledged` must be true, and
// the override is recorded in the commit's trailers so the audit trail says a
// human approved over a rejection rather than that no one noticed. A passing
// verdict needs no tick at all — the gate is neither accidental nor exhausting.
func (s *Server) handleApprove(w http.ResponseWriter, r *http.Request) {
	e, m, tk, ok := s.gateTarget(w, r)
	if !ok {
		return
	}
	var body struct {
		Acknowledged bool `json:"acknowledged"`
	}
	if r.ContentLength > 0 {
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			httpError(w, http.StatusBadRequest, "invalid request body")
			return
		}
	}

	// The verdict, when there is one. Approving with no review at all is allowed —
	// the human is the gate, not the reviewer — but a *rejecting* verdict costs the
	// one acknowledgement.
	rv, sid, hasReview := s.reviewFor(e, m.Slug, tk.Num)
	blocking := rv.blocking()
	if hasReview && len(blocking) > 0 && !body.Acknowledged {
		httpError(w, http.StatusConflict,
			"the reviewer's finding blocks against a Done-when clause — tick “I've read the blocking finding” to approve over it")
		return
	}

	ticketPath, err := ticketFilePath(m.Dir, tk.Num)
	if err != nil {
		httpError(w, http.StatusNotFound, err.Error())
		return
	}
	src, err := os.ReadFile(ticketPath)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "reading the ticket: "+err.Error())
		return
	}
	next, promoted := promoteAnswer(string(src))
	if !promoted {
		httpError(w, http.StatusConflict, "the ticket carries no `## Proposed Answer` to promote")
		return
	}

	rel := repoRel(e.Path, ticketPath)
	gc := gateCommit{
		Subject: fmt.Sprintf("Resolve %s — promote the proposed answer", rel),
		Trailers: []trailer{
			{"Ticket", fmt.Sprintf("%02d", tk.Num)},
			{"Role", "gate"},
			{"Gate", "human"},
			{"Agent", observedImplement(e.Path, rel).agent},
			{"Model", observedImplement(e.Path, rel).model},
		},
	}
	if hasReview {
		gc.Trailers = append(gc.Trailers,
			trailer{"Review-Session", sid},
			trailer{"Verdict", recommendation(rv)})
		if len(blocking) > 0 {
			// Approving over a rejection is recorded, never silent.
			gc.Trailers = append(gc.Trailers,
				trailer{"Approved-Over-Rejection", "true"},
				trailer{"Acknowledged-Blocking", strings.TrimSpace(firstLine(blocking[0].Text))})
		}
	} else {
		gc.Trailers = append(gc.Trailers, trailer{"Verdict", "none — approved without an agent review"})
	}

	sha, smearedInto, err := writeGateCommit(e.Path, ticketPath, next, gc)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "writing the promotion commit: "+err.Error())
		return
	}
	s.rebuild()

	// What the approval bought, read back off disk: the dependents it unblocked and
	// the next best frontier ticket the post-approve strip suggests.
	unblocked, next2 := s.frontierAfter(e, m.Slug, tk.Num)
	resp := map[string]any{
		"ticketNum":             tk.Num,
		"commit":                sha,
		"unblocked":             unblocked,
		"approvedOverRejection": hasReview && len(blocking) > 0,
	}
	if next2 != nil {
		resp["next"] = map[string]any{"num": next2.Num, "title": next2.Title}
	}
	if smearedInto != "" {
		// ADR 0008's residual race: the promotion landed, but under someone else's
		// commit. Reported, never hidden and never retried.
		resp["smearedInto"] = smearedInto
		resp["warning"] = "the promotion was swept into commit " + shortSHA(smearedInto) +
			" by another writer before the harness could commit it — the answer is promoted, but the attribution is theirs"
	}
	writeJSON(w, http.StatusOK, resp)
}

// handleAbandon rejects the proposal, not the ticket (story 60). It demands one
// thing — a reason addressed to the next attempt — demotes `## Proposed Answer`
// to dated `### Rejected` prose carrying that reason, and commits it as the
// harness's own pathspec-limited write. The ticket then derives `open` and
// returns to the frontier, armed with the record.
//
// It destroys nothing by default. Reverting the work commits is an unticked lever
// (`revert`), and resetting to before them is offered only when they are
// verifiably the tip of a clean tree (`reset`) — undoing history is the human's
// act, taken here as a convenience or later in their own terminal.
func (s *Server) handleAbandon(w http.ResponseWriter, r *http.Request) {
	e, m, tk, ok := s.gateTarget(w, r)
	if !ok {
		return
	}
	var body struct {
		Reason string `json:"reason"`
		Revert bool   `json:"revert"`
		Reset  bool   `json:"reset"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if strings.TrimSpace(body.Reason) == "" {
		httpError(w, http.StatusBadRequest,
			"abandon asks for one thing: a reason addressed to the next attempt")
		return
	}
	if body.Revert && body.Reset {
		httpError(w, http.StatusBadRequest, "revert and reset are alternatives — pick one lever, not both")
		return
	}

	ticketPath, err := ticketFilePath(m.Dir, tk.Num)
	if err != nil {
		httpError(w, http.StatusNotFound, err.Error())
		return
	}
	rel := repoRel(e.Path, ticketPath)

	// The work under the proposal, computed *before* any write, so "is it still the
	// tip?" is answered about the work rather than about our own demotion commit.
	work := workCommits(e.Path, rel)

	if body.Reset {
		if !tipOf(e.Path, work) {
			httpError(w, http.StatusConflict,
				"reset is offered only while the work commits are verifiably the tip — they are not, so revert instead")
			return
		}
		if dirty, _ := git(e.Path, "--no-optional-locks", "status", "--porcelain"); strings.TrimSpace(dirty) != "" {
			httpError(w, http.StatusConflict,
				"the working tree carries uncommitted changes — reset would destroy them; commit or stash first")
			return
		}
		// Reset first, then demote onto the pre-work tip: the demotion is the record
		// that survives, and history reads as if the attempt had only ever been tried.
		if out, err := git(e.Path, "reset", "--hard", work[len(work)-1]+"^"); err != nil {
			httpError(w, http.StatusInternalServerError, "resetting to before the work: "+err.Error()+"\n"+out)
			return
		}
		// The reset took the proposal with it; re-read the ticket from the tree.
	}

	src, err := os.ReadFile(ticketPath)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "reading the ticket: "+err.Error())
		return
	}
	date := time.Now().UTC().Format("2006-01-02")
	next, demoted := demoteProposal(stripClaim(string(src)), body.Reason, date)
	if !demoted && !body.Reset {
		httpError(w, http.StatusConflict, "the ticket carries no `## Proposed Answer` to demote")
		return
	}

	rv, sid, hasReview := s.reviewFor(e, m.Slug, tk.Num)
	gc := gateCommit{
		Subject: fmt.Sprintf("Abandon the proposal on %s — back to the frontier", rel),
		Trailers: []trailer{
			{"Ticket", fmt.Sprintf("%02d", tk.Num)},
			{"Role", "gate"},
			{"Gate", "human"},
			{"Verdict", "abandoned"},
		},
	}
	if hasReview {
		gc.Trailers = append(gc.Trailers,
			trailer{"Review-Session", sid},
			trailer{"Review-Recommendation", recommendation(rv)})
	}

	sha, smearedInto, err := writeGateCommit(e.Path, ticketPath, next, gc)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "writing the demotion commit: "+err.Error())
		return
	}

	resp := map[string]any{
		"ticketNum":   tk.Num,
		"commit":      sha,
		"workCommits": work,
		"reverted":    false,
		"reset":       body.Reset,
	}
	if body.Revert && len(work) > 0 {
		// Newest first, so each revert applies cleanly onto the one before it. A
		// conflict aborts and is reported: the demotion stands regardless, and undoing
		// the work stays the human's business.
		args := append([]string{"revert", "--no-edit"}, work...)
		if out, err := git(e.Path, args...); err != nil {
			_, _ = git(e.Path, "revert", "--abort")
			resp["revertError"] = "the revert did not apply cleanly and was aborted — the demotion stands; undo the work in your own terminal: " + out
		} else {
			resp["reverted"] = true
		}
	}
	if smearedInto != "" {
		resp["smearedInto"] = smearedInto
		resp["warning"] = "the demotion was swept into commit " + shortSHA(smearedInto) + " by another writer before the harness could commit it"
	}
	s.rebuild()
	writeJSON(w, http.StatusOK, resp)
}

// handleFollowUp stacks another session on a still-proposed ticket — the shared
// mechanism behind two doors in the hub. **Send back** is this with the blocking
// finding attached (always) plus any advisories the human ticked; **take it
// further** is this with whatever the human chose to attach, or nothing.
//
// Nothing new is invented for it: it is ticket 09's launch with steering added to
// the payload. The claim is re-stamped onto the same ticket, so the follow-up's
// commits accumulate on the proposal, and because `proposed` outranks `claimed`
// in the derived-status table the ticket stays proposed throughout and comes back
// to this hub (story 58).
//
// The steering — findings and the human's note — rides the injected payload and
// its archive and *nowhere else* (story 59). It is never written to the ticket
// file: only abandonment writes there, because only abandonment needs the next
// fresh attempt to read it.
func (s *Server) handleFollowUp(w http.ResponseWriter, r *http.Request) {
	e, m, tk, ok := s.gateTarget(w, r)
	if !ok {
		return
	}
	var body struct {
		Role string `json:"role"`
		Note string `json:"note"`
		// Advisories are indices into the brief's advisory list the human ticked in.
		// The blocking findings ride unconditionally — the fix-up is briefed with what
		// blocks, never optionally.
		Advisories      []int `json:"advisories"`
		IncludeFindings *bool `json:"includeFindings"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	role := body.Role
	if role == "" {
		role = string(config.RoleImplement)
	}
	if m.Kind == model.KindUnclassified {
		httpError(w, http.StatusConflict, "this map is unclassified and offers no sessions — classify it first")
		return
	}
	if !config.KindOffersRole(m.Kind, role) {
		httpError(w, http.StatusBadRequest, "role "+role+" is not offered by a "+m.Kind+" map")
		return
	}
	binding, ok := bindingFor(s.resolve(e), role)
	if !ok {
		httpError(w, http.StatusInternalServerError, "no binding for role "+role)
		return
	}
	if !binding.Present {
		httpError(w, http.StatusConflict, binding.Missing)
		return
	}
	if s.terms.HasLiveSession(e.ID) {
		httpError(w, http.StatusConflict, "this space already has a live session — end it before spawning a follow-up")
		return
	}

	steering := s.steeringFor(e, m.Slug, tk.Num, body.IncludeFindings == nil || *body.IncludeFindings, body.Advisories, body.Note)

	result, status, err := s.launchSession(sessionLaunch{
		entry:     e,
		slug:      m.Slug,
		m:         m,
		tk:        tk,
		role:      role,
		binding:   binding,
		sessionID: newSessionID(),
		steering:  steering,
	})
	if err != nil {
		httpError(w, status, err.Error())
		return
	}
	result["followUp"] = true
	writeJSON(w, http.StatusOK, result)
}

// steeringFor assembles the live-steering blocks a follow-up session's payload
// carries: the reviewer's blocking findings (always, when there is a verdict and
// the caller wants them), the advisories the human ticked by index, and the
// human's own note. Each is a labelled context part, so the operator sees exactly
// what the session was told in the payload preview and its archive.
func (s *Server) steeringFor(e registry.Entry, slug string, num int, includeFindings bool, advisories []int, note string) []prompt.Steer {
	var out []prompt.Steer
	if includeFindings {
		if rv, _, ok := s.reviewFor(e, slug, num); ok {
			if blocking := rv.blocking(); len(blocking) > 0 {
				var b strings.Builder
				b.WriteString("The review that sent this back blocks on the following. Each cites the Done-when clause it breaks; clearing it is the point of this session.\n")
				for _, f := range blocking {
					b.WriteString("\n- " + strings.TrimSpace(f.Text) + "\n")
				}
				out = append(out, prompt.Steer{Label: "The blocking finding", Text: b.String()})
			}
			adv := rv.advisories()
			var picked []string
			for _, i := range advisories {
				if i >= 0 && i < len(adv) {
					picked = append(picked, "- "+strings.TrimSpace(adv[i].Text))
				}
			}
			if len(picked) > 0 {
				out = append(out, prompt.Steer{
					Label: "Advisories the operator chose to include",
					Text:  "Not blocking — address them if they are cheap and right:\n\n" + strings.Join(picked, "\n"),
				})
			}
		}
	}
	if n := strings.TrimSpace(note); n != "" {
		out = append(out, prompt.Steer{
			Label: "A note from the operator",
			Text:  n,
		})
	}
	return out
}

// handleTicketDiff serves the work under the proposal at one of three scopes
// (story 58): **all** the commits since the ticket was first claimed, everything
// **since the verdict** the human is reading, or everything **since their last
// read** (the client passes the sha it last saw). A follow-up session's commits
// stack onto the same proposal, so the scopes are what keep a second read from
// re-reading the first.
func (s *Server) handleTicketDiff(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	num, err := strconv.Atoi(r.PathValue("num"))
	if err != nil {
		httpError(w, http.StatusBadRequest, "ticket number must be an integer")
		return
	}
	m, found := findMap(mapscan.Discover(e.Path), r.PathValue("slug"))
	if !found {
		httpError(w, http.StatusNotFound, "no such map")
		return
	}
	ticketPath, err := ticketFilePath(m.Dir, num)
	if err != nil {
		httpError(w, http.StatusNotFound, err.Error())
		return
	}
	rel := repoRel(e.Path, ticketPath)

	scope := r.URL.Query().Get("scope")
	if scope == "" {
		scope = "all"
	}
	var base string
	switch scope {
	case "all":
		// The first claim on this ticket: everything the attempt has produced. A
		// proposal that arrived without an implement claim — hand-written, or pulled
		// from a teammate — falls back to the oldest claim of any role, so the scope
		// still anchors somewhere honest rather than silently showing nothing.
		base = oldestClaimSHA(e.Path, rel, string(config.RoleImplement))
		if base == "" {
			base = oldestClaimSHA(e.Path, rel, "")
		}
	case "verdict":
		// The review's claim commit is the moment the verdict was formed.
		base = newestClaimSHA(e.Path, rel, string(config.RoleReview))
	case "read":
		base = strings.TrimSpace(r.URL.Query().Get("since"))
	default:
		httpError(w, http.StatusBadRequest, "scope must be all, verdict, or read")
		return
	}
	if base == "" {
		// No anchor for this scope (no claim in history, no last-read sha) — say so
		// rather than silently widening to the whole repository.
		writeJSON(w, http.StatusOK, map[string]any{
			"scope": scope, "base": "", "head": "", "patch": "", "stat": "",
			"note": "no commit anchors this scope yet",
		})
		return
	}

	head, err := git(e.Path, "rev-parse", "HEAD")
	if err != nil {
		httpError(w, http.StatusConflict, "this space has no commits yet")
		return
	}
	head = strings.TrimSpace(head)
	// The ticket file itself is excluded: the hub already shows the proposal and the
	// verdict, and the claim churn on the ticket is noise against the work. A stale
	// or unknown `since` sha (scope=read, an sha the client remembered from a since-
	// discarded branch or a garbage-collected commit) makes this call fail — that
	// must surface as the anchored error it is, never read as "nothing changed"
	// (ticket 17: the discarded error used to render an honest-looking empty diff).
	patch, err := git(e.Path, "--no-optional-locks", "diff", base+".."+head, "--", ".", ":(exclude)"+rel)
	if err != nil {
		httpError(w, http.StatusConflict, "the diff's anchor ("+shortSHA(base)+") does not resolve in this repository — it may be stale or unknown: "+patch)
		return
	}
	stat, _ := git(e.Path, "--no-optional-locks", "diff", "--stat", base+".."+head, "--", ".", ":(exclude)"+rel)

	writeJSON(w, http.StatusOK, map[string]any{
		"scope": scope,
		"base":  base,
		"head":  head,
		"patch": patch,
		"stat":  stat,
	})
}

// reviewFor finds the assembled review brief for a ticket and parses the verdict
// behind it. The per-ticket pointer (ticket 17) is the index — never a scan of
// live session tabs — so the sid it returns is always whichever review was
// assembled *last*, and reading it never depends on that session's tab still
// being open or the process that ran it still being up. It reads the verdict
// rather than the brief so the mechanical rules (anchoring, the recommendation)
// are applied from the source, exactly as ticket 11 applied them when it wrote
// the brief.
func (s *Server) reviewFor(e registry.Entry, slug string, num int) (verdict, string, bool) {
	sid := readReviewPointer(e.Path, slug, num)
	if sid == "" {
		return verdict{}, "", false
	}
	dir := filepath.Join(e.Path, sessionRunDir, sid)
	raw, err := os.ReadFile(filepath.Join(dir, reviewVerdictName))
	if err != nil {
		return verdict{}, "", false
	}
	return parseVerdict(string(raw)), sid, true
}

// ensureBrief builds the review brief for a ticket whose reviewer has written a
// verdict but whose brief has not been assembled yet. It is idempotent (the
// assembly is a pure function of the verdict and the ticket) and silent on
// failure: a reviewer that has written nothing simply leaves the hub reporting
// that no brief is waiting.
func (s *Server) ensureBrief(e registry.Entry, slug string, num int) {
	for _, info := range s.terms.ForSpace(e.ID) {
		if info.Session == nil || info.Session.Role != string(config.RoleReview) {
			continue
		}
		if info.Session.MapSlug != slug || info.Session.TicketNum != num {
			continue
		}
		dir := filepath.Join(e.Path, sessionRunDir, info.ID)
		if _, err := os.Stat(filepath.Join(dir, reviewBriefName)); err == nil {
			continue
		}
		if _, _, _, err := s.assembleReviewBrief(e, info); err == nil {
			s.rebuild()
		}
	}
}

// reviewState is the snapshot's gate-level signal for a ticket: a brief is
// assembled and a human is being waited on (ticket 12). It carries no prose —
// the hub fetches the brief when it opens — only what the star-map and the
// "Needs you" queue need to know that this ticket is at the gate.
func (s *Server) reviewState(e registry.Entry, slug string, num int) *model.Review {
	rv, sid, ok := s.reviewFor(e, slug, num)
	if !ok {
		return nil
	}
	return &model.Review{
		SessionID:      sid,
		Recommendation: recommendation(rv),
		Blocking:       len(rv.blocking()),
		Advisories:     len(rv.advisories()),
	}
}

// frontierAfter reads back, off disk, what an approval bought: the dependents it
// unblocked onto the stricter frontier, and the next best frontier ticket to
// suggest — ranked by what it unblocks, then by number. The suggestion is offered
// by the post-approve strip and never acted on (story 61).
func (s *Server) frontierAfter(e registry.Entry, slug string, num int) ([]int, *model.Ticket) {
	m, ok := findMap(mapscan.Discover(e.Path), slug)
	if !ok {
		return nil, nil
	}
	var unblocked []int
	for _, t := range m.Tickets {
		if !t.Frontier {
			continue
		}
		for _, b := range t.BlockedBy {
			if b == num {
				unblocked = append(unblocked, t.Num)
				break
			}
		}
	}
	sort.Ints(unblocked)

	dependents := func(n int) int {
		c := 0
		for _, t := range m.Tickets {
			for _, b := range t.BlockedBy {
				if b == n {
					c++
				}
			}
		}
		return c
	}
	var frontier []model.Ticket
	for _, t := range m.Tickets {
		if t.Frontier {
			frontier = append(frontier, t)
		}
	}
	if len(frontier) == 0 {
		return unblocked, nil
	}
	// Prefer one this approval just unblocked, then the one that unblocks the most.
	sort.SliceStable(frontier, func(i, j int) bool {
		ui, uj := contains(unblocked, frontier[i].Num), contains(unblocked, frontier[j].Num)
		if ui != uj {
			return ui
		}
		di, dj := dependents(frontier[i].Num), dependents(frontier[j].Num)
		if di != dj {
			return di > dj
		}
		return frontier[i].Num < frontier[j].Num
	})
	return unblocked, &frontier[0]
}

func contains(xs []int, x int) bool {
	for _, v := range xs {
		if v == x {
			return true
		}
	}
	return false
}

// workCommits lists the commits an attempt produced, newest first: everything
// since the ticket was first claimed, minus the harness's own lifecycle commits
// (claim, release, gate), which are the record of the attempt rather than the
// attempt. These are what abandon's revert and reset levers act on. A commit is
// read whole (%B, not just its subject) so isHarnessCommit can match the
// Harness-Write trailer every lifecycle write carries — an agent commit whose
// subject happens to start with "Claim " or "Resolve " is work, not the harness's,
// and trailer-matching is what tells the two apart (ticket 17).
func workCommits(repo, ticketRel string) []string {
	base := oldestClaimSHA(repo, ticketRel, string(config.RoleImplement))
	if base == "" {
		return nil
	}
	out, err := git(repo, "log", "--format=%H%x1f%B%x00", base+"..HEAD")
	if err != nil {
		return nil
	}
	var shas []string
	for _, rec := range strings.Split(out, "\x00") {
		sha, msg, found := strings.Cut(strings.TrimSpace(rec), "\x1f")
		if !found || sha == "" {
			continue
		}
		if isHarnessCommit(msg) {
			continue
		}
		shas = append(shas, sha)
	}
	return shas
}

// isHarnessCommit reports whether a commit is one of the harness's own
// enumerated lifecycle writes (ADR 0008) rather than an agent's work — matched by
// the `Harness-Write: true` trailer every claim, release, and gate commit carries,
// never by guessing from the subject line (ticket 17: a subject-prefix match
// silently excluded any agent commit whose own message happened to start with
// "Claim ", "Release ", "Resolve ", or "Abandon the proposal on ").
func isHarnessCommit(msg string) bool {
	return reHarnessWriteTrailer.MatchString(msg)
}

var reHarnessWriteTrailer = regexp.MustCompile(`(?m)^Harness-Write:\s*true\s*$`)

// tipOf reports whether the given commits (newest first) are verifiably the tip
// of the current branch — an unbroken run ending at HEAD. Only then is reset
// offered; anything else and a rejected attempt left in history is a truthful
// history.
func tipOf(repo string, commits []string) bool {
	if len(commits) == 0 {
		return false
	}
	out, err := git(repo, "log", "--format=%H", "-n", strconv.Itoa(len(commits)))
	if err != nil {
		return false
	}
	tip := strings.Fields(out)
	if len(tip) != len(commits) {
		return false
	}
	for i := range commits {
		if tip[i] != commits[i] {
			return false
		}
	}
	return true
}

// oldestClaimSHA and newestClaimSHA find the commit that stamped a claim of a
// given role onto a ticket — the anchors the diff scopes and the work-commit
// range are measured from. They read the claim trailers ADR 0008 writes, so git
// alone answers "when did this attempt start" and "when was it reviewed". An
// empty role matches a claim of any role.
func oldestClaimSHA(repo, ticketRel, role string) string {
	return claimSHA(repo, ticketRel, role, true)
}
func newestClaimSHA(repo, ticketRel, role string) string {
	return claimSHA(repo, ticketRel, role, false)
}

func claimSHA(repo, ticketRel, role string, oldest bool) string {
	out, err := git(repo, "log", "--format=%H%x1f%B%x00", "--", ticketRel)
	if err != nil {
		return ""
	}
	var found string
	for _, rec := range strings.Split(out, "\x00") {
		sha, msg, ok := strings.Cut(strings.TrimSpace(rec), "\x1f")
		if !ok {
			continue
		}
		rm := reRoleTrailer.FindStringSubmatch(msg)
		if rm == nil || (role != "" && strings.TrimSpace(rm[1]) != role) {
			continue
		}
		found = sha
		if !oldest {
			// The log is newest-first, so the first match is the newest.
			return found
		}
	}
	return found
}

func firstLine(s string) string {
	if i := strings.IndexByte(s, '\n'); i >= 0 {
		return s[:i]
	}
	return s
}

func shortSHA(s string) string {
	if len(s) > 8 {
		return s[:8]
	}
	return s
}
