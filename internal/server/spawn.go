package server

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/mapscan"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
	"github.com/rengwu/chartr/internal/terminal"
)

// sessionRunDir is the gitignored directory, inside each space, that holds live
// sessions' composed payloads (ADR 0005 — one inspectable file per session), and
// the ideate on-ramp's starter prompt (ticket 15), which reuses the same
// writeSessionPayload path though it is deliberately not a session. It sits under
// chartr's committed `.chartr/` directory but is itself never
// committed: chartr drops a `.gitignore` of `*` beside it so an agent's
// `git commit -a` can never sweep a payload into the audit trail (ADR 0008).
const sessionRunDir = ".chartr/run"

// handleSpawn is the product's tracer bullet (ticket 09): from a frontier ticket
// it wires the whole chain — resolve the role's binding, hard-block a missing
// agent, compose the payload, write the claim commit, drop the gitignored payload
// and its archived copy, and launch the agent's own TUI with the read-this-file
// opener typed in. The session seats as a tab bound to exactly one ticket.
//
// Every step that can fail does so *before* anything launches, so a refused spawn
// leaves the space exactly as it was — except the claim, which by ADR 0008 stands
// once written (a launch failure after a good claim is a dead session the human
// resolves, never an automatic rollback).
func (s *Server) handleSpawn(w http.ResponseWriter, r *http.Request) {
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
	var body struct {
		Role string `json:"role"`
		// Agent names a registered agent from the operator's library — the explicit
		// "run this session on *that*" the picker sends. Optional here and only
		// here: while it is absent the role's binding still decides, so nothing that
		// does not send it changes behaviour. Ticket 04 makes it required.
		Agent string `json:"agent"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	role := body.Role
	if role == "" {
		httpError(w, http.StatusBadRequest, "role is required")
		return
	}
	// The role must name one of the four. *Which* role is never refused (see
	// below) — but a string that is not a role at all is a malformed request, and
	// saying so here is what keeps it from arriving at `bindingFor` as a binding
	// that could not exist and being reported as a 500. The payload preview
	// answers the same input the same way, through prompt.Compose.
	if !config.IsRole(role) {
		httpError(w, http.StatusBadRequest, "unknown role "+role)
		return
	}

	// Discover fresh (as the preview does) so the spawn acts on the truth on disk,
	// not a cached snapshot — the ticket may have been resolved or claimed since the
	// last push.
	slug := r.PathValue("slug")
	m, found := findMap(mapscan.Discover(e.Path), slug)
	if !found {
		httpError(w, http.StatusNotFound, "no such map")
		return
	}
	tk, found := findTicket(m, num)
	if !found {
		httpError(w, http.StatusNotFound, "no such ticket")
		return
	}

	// Every ticket on every discovered map offers all four roles, so the role
	// itself is never refused: trust-at-the-gate, where the gate is the operator
	// choosing in the spawn preview.
	res := s.resolve(e)

	// Every role is a fresh spawn onto the frontier (open, unclaimed, every blocker
	// resolved): anything already claimed, closed, or held behind an unresolved
	// blocker is not its to take.
	if !tk.Frontier {
		httpError(w, http.StatusConflict, "ticket is not on the frontier — it is not a takeable ticket")
		return
	}

	// Settle what will actually run, and hard-block an absent binary *here*, before
	// any write: an unregistered name and a missing CLI are both doorstep
	// diagnoses, refused in the same place and the same order whether the agent was
	// named explicitly or inherited from the role's binding (story 40), and neither
	// blocks anything else.
	spec, status, err := launchSpecFor(res, role, body.Agent)
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	// One session per space at a time (spec, State model). Check before writing the
	// claim so we never claim a ticket we cannot then seat; OpenSession re-checks
	// under its own lock to close the race.
	if s.terms.HasLiveSession(e.ID) {
		httpError(w, http.StatusConflict, "this space already has a live session — end it before spawning another")
		return
	}

	// From here the mechanics are shared with respawn (ticket 10): compose the
	// payload, write the claim commit, drop the gitignored and archived copies, and
	// launch the TUI. A fresh spawn mints a new session id.
	result, status, err := s.launchSession(sessionLaunch{
		entry:     e,
		slug:      slug,
		m:         m,
		tk:        tk,
		role:      role,
		spec:      spec,
		sessionID: newSessionID(),
	})
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	// The space remembers what it just spawned with, so the next spawn here is one
	// click. Written only now, past every refusal and past the launch itself: a
	// blocked spawn must leave the memory exactly as it was. An override is
	// therefore self-persisting and needs no confirming action. Failing to persist
	// costs the operator one re-pick, never the running session, so it does not
	// fail the request.
	if spec.Name != "" {
		if err := s.reg.SetLastAgent(e.ID, spec.Name); err == nil {
			s.rebuild()
		}
	}
	writeJSON(w, http.StatusOK, result)
}

// sessionLaunch carries the resolved inputs the launch mechanics need once every
// gate has passed. It is the seam a fresh spawn and a respawn share (ticket 10):
// both compose the same payload, write the same claim, and open the same kind of
// session tab — they differ only in the gating that precedes this and in whether
// the session id is new.
type sessionLaunch struct {
	entry     registry.Entry
	slug      string
	m         model.Map
	tk        model.Ticket
	role      string
	spec      launchSpec
	sessionID string
}

// launchSpec is the one thing the launch mechanics need to know about execution:
// the binary, its flags, how the opener reaches it, and — when the operator chose
// from the library rather than inheriting a role's binding — the registered name
// they chose. Both paths converge on it *before* launchSession, so the mechanics
// below never learn that role bindings exist, and deleting them (ticket 05) is a
// subtraction rather than a rewrite.
type launchSpec struct {
	// Name is the registered agent's name, empty on the binding path — a local
	// name, meaningful only on this machine, which is why the audit trail records
	// the adapter and args beside it rather than instead of them.
	Name    string
	Adapter string
	Args    []string
	// Prompt is how the opener reaches this harness — `argv`, `type`, or a flag
	// name. Empty leaves the adapter's own default in force.
	Prompt string
	// AdapterFrom and ArgsFrom are the config layers the binding path resolved
	// those fields from, and are empty on the explicit-agent path, which consults
	// no layers at all. They travel only as far as the claim trailers, and go with
	// the layers they name in ticket 05.
	AdapterFrom config.Layer
	ArgsFrom    config.Layer
}

// launchSpecFor settles what a spawn will run. A named agent is resolved against
// the operator's library — unregistered is a malformed request (400), registered
// but absent from PATH is a conflict carrying the library's own diagnosis (409),
// and a stale picker therefore can never launch nothing (story 18). With no name
// the role's binding decides exactly as it always has, refused the same two ways.
// Either way it settles before the claim and before any write, so a refusal
// leaves the space untouched (story 33). The int is the HTTP status to surface
// alongside a non-nil error.
func launchSpecFor(res config.Resolution, role, agent string) (launchSpec, int, error) {
	if agent != "" {
		return agentSpec(res, agent)
	}

	binding, ok := bindingFor(res, role)
	if !ok {
		return launchSpec{}, http.StatusInternalServerError, errors.New("no binding for role " + role)
	}
	if !binding.Present {
		return launchSpec{}, http.StatusConflict, errors.New(binding.Missing)
	}
	return specOf(binding), 0, nil
}

// agentSpec resolves one registered agent's name into what it launches. It is the
// whole of the explicit-selection path, shared by every surface that names an
// agent — spawn, ideate, and the halt actions that relaunch what a dead session
// ran — so an unregistered name is a malformed request (400) and a registered
// agent absent from PATH is a conflict carrying the library's own diagnosis (409)
// in exactly one place, whichever surface asked. An empty name is refused here
// too: it is the request that named nothing, and no path reaching this function
// has anything to fall back to (ticket 04 makes that the rule everywhere).
func agentSpec(res config.Resolution, agent string) (launchSpec, int, error) {
	if agent == "" {
		return launchSpec{}, http.StatusBadRequest, errors.New("an agent is required — pick one from your library")
	}
	for _, a := range res.Agents {
		if a.Name != agent {
			continue
		}
		if !a.Present {
			return launchSpec{}, http.StatusConflict, errors.New(a.Missing)
		}
		return launchSpec{Name: a.Name, Adapter: a.Adapter, Args: a.Args, Prompt: a.Prompt}, 0, nil
	}
	return launchSpec{}, http.StatusBadRequest, fmt.Errorf("no agent named %q is registered", agent)
}

// specOf reads a resolved role binding as a launch spec. A binding assigned to a
// registered agent keeps that name — it is still a name out of the library, and
// dropping it would make the trailer poorer for no reason. A binding bound field
// by field has no name to keep, and one naming an agent the library does not hold
// is not a registered agent at all, so both carry none.
func specOf(b config.Resolved) launchSpec {
	name := b.Agent
	if b.AgentMissing != "" {
		name = ""
	}
	return launchSpec{
		Name:        name,
		Adapter:     b.Adapter,
		Args:        b.Args,
		Prompt:      b.Prompt,
		AdapterFrom: b.AdapterFrom,
		ArgsFrom:    b.ArgsFrom,
	}
}

// launchSession runs the spawn mechanics: it composes the payload fresh
// off disk (the same assembly the preview shows — ADR 0005), writes the claim
// commit (ADR 0008), drops the payload gitignored inside the space and archived in
// chartr state (story 49), and launches the agent's TUI with the read-this-file
// opener typed in. It returns the action's result and, on failure, the HTTP status
// the caller should surface. Every write that can fail happens here in order, so a
// failure after the claim leaves the claim standing (never rolled back) for the
// death halt to resolve.
func (s *Server) launchSession(in sessionLaunch) (map[string]any, int, error) {
	payload, err := prompt.Compose(prompt.ComposeInput{
		Role:  in.role,
		Roots: s.skillRoots(in.entry.Path),
		Bundle: prompt.Bundle{
			MapName:     in.m.Name,
			MapBody:     in.m.Body,
			TicketNum:   in.tk.Num,
			TicketTitle: in.tk.Title,
			TicketBody:  in.tk.Body,
			Blockers:    blockersOf(in.m, in.tk),
		},
	})
	if err != nil {
		return nil, http.StatusBadRequest, err
	}

	sum := sha256.Sum256([]byte(payload.Markdown))
	payloadSHA := hex.EncodeToString(sum[:])
	claimedAt := time.Now().UTC().Format(time.RFC3339)

	ticketPath, err := ticketFilePath(in.m.Dir, in.tk.Num)
	if err != nil {
		return nil, http.StatusNotFound, err
	}

	// The claim commit — chartr's one write here (ADR 0008), pathspec-limited
	// to the ticket file and carrying the binding and payload-hash trailers. On a
	// respawn the ticket already carries a stale claim; stampClaim replaces it, so
	// the new session id supersedes the dead one.
	if err := writeClaimCommit(in.entry.Path, ticketPath, in.sessionID, claimedAt, claim{
		SessionID:   in.sessionID,
		Role:        in.role,
		Agent:       in.spec.Name,
		Adapter:     in.spec.Adapter,
		Args:        in.spec.Args,
		PayloadSHA:  payloadSHA,
		Skills:      payload.Skills,
		AdapterFrom: in.spec.AdapterFrom,
		ArgsFrom:    in.spec.ArgsFrom,
	}); err != nil {
		return nil, http.StatusInternalServerError, fmt.Errorf("writing the claim commit: %w", err)
	}

	payloadPath, err := s.writeSessionPayload(in.entry.Path, in.sessionID, payload.Markdown)
	if err != nil {
		return nil, http.StatusInternalServerError, fmt.Errorf("writing the session payload: %w", err)
	}
	if err := s.archivePayload(in.sessionID, payload.Markdown); err != nil {
		return nil, http.StatusInternalServerError, fmt.Errorf("archiving the session payload: %w", err)
	}

	// Resolve the binding — and the read-this-file opener — onto this agent's
	// command line (ADR 0002) and launch its TUI in a PTY. Whether the opener rides
	// the argv or gets typed in is the adapter's call, so the spawn path hands over
	// the same line either way.
	launch := adapter.Command(adapter.Spawn{
		Adapter: in.spec.Adapter,
		Args:    in.spec.Args,
		Prompt:  adapter.Opener(payloadPath),
		Deliver: in.spec.Prompt,
	})
	if _, err := s.terms.OpenSession(in.entry.ID, in.entry.Path, in.sessionID, launch.Name, launch.Args,
		launch.TypeIn, terminal.Session{
			MapSlug:   in.slug,
			TicketNum: in.tk.Num,
			Role:      in.role,
			Agent:     in.spec.Adapter,
			// The registered name travels with the session so a resume or a respawn
			// relaunches the agent this session actually ran, rather than re-deciding
			// it (ticket 03).
			AgentName: in.spec.Name,
		}); err != nil {
		// The claim already stands (ADR 0008: it is not rolled back). A live-session
		// race is a conflict; a launch failure after a present-on-PATH check is an
		// environment fault the death halt (ticket 10) picks up.
		if errors.Is(err, terminal.ErrSessionExists) {
			return nil, http.StatusConflict, errors.New("this space already has a live session")
		}
		return nil, http.StatusInternalServerError, fmt.Errorf("launching the session: %w", err)
	}

	return map[string]any{
		"sessionId": in.sessionID,
		"ticketNum": in.tk.Num,
		"role":      in.role,
		// `agent` stays the adapter — what launched — and `agentName` carries the
		// registered name when the operator picked one, so the response says both
		// what ran and what they chose without either changing meaning.
		"agent":      in.spec.Adapter,
		"agentName":  in.spec.Name,
		"args":       in.spec.Args,
		"payloadSha": payloadSHA,
	}, http.StatusOK, nil
}

// resolve resolves a space's whole config — the role bindings — across the three
// layers, exactly as the pushed model renders it, so the spawn path reads a
// binding and its PATH presence from what the operator sees.
func (s *Server) resolve(e registry.Entry) config.Resolution {
	userTOML, _ := os.ReadFile(filepath.Join(s.opts.DataDir, userConfigName))
	workspaceTOML, _ := os.ReadFile(filepath.Join(e.Path, config.WorkspaceConfigName))
	return config.Resolve(config.Input{
		WorkspaceTOML: workspaceTOML,
		UserTOML:      userTOML,
		SpacePath:     e.Path,
	})
}

// bindingFor picks one role's resolved binding out of a resolution.
func bindingFor(res config.Resolution, role string) (config.Resolved, bool) {
	for _, b := range res.Bindings {
		if string(b.Role) == role {
			return b, true
		}
	}
	return config.Resolved{}, false
}

// writeSessionPayload writes the composed payload to the gitignored run directory
// inside the space and returns its absolute path (what the opener points the agent
// at). It (re)writes a `.gitignore` of `*` beside the payloads on every spawn, so
// the ignore holds even on a fresh clone that never had the run directory.
func (s *Server) writeSessionPayload(repo, sessionID, markdown string) (string, error) {
	runDir := filepath.Join(repo, sessionRunDir)
	if err := os.MkdirAll(filepath.Join(runDir, sessionID), 0o755); err != nil {
		return "", err
	}
	if err := os.WriteFile(filepath.Join(runDir, ".gitignore"), []byte("*\n"), 0o644); err != nil {
		return "", err
	}
	path := filepath.Join(runDir, sessionID, "payload.md")
	if err := os.WriteFile(path, []byte(markdown), 0o644); err != nil {
		return "", err
	}
	return path, nil
}

// archivePayload keeps the exact payload a session received in chartr-owned state
// outside the repository (under the data root), so the record survives the space's
// gitignored copy being cleaned and answers "what was this session told" word for
// word (story 49).
func (s *Server) archivePayload(sessionID, markdown string) error {
	dir := filepath.Join(s.opts.DataDir, "sessions", sessionID)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	return os.WriteFile(filepath.Join(dir, "payload.md"), []byte(markdown), 0o644)
}

// newSessionID mints a session's stable identity — the id its tab, its claim
// trailer (claimed_by), its gitignored payload path, and its archive are all keyed
// by, so the whole spawn refers to one session everywhere. Random rather than
// sequential so two spaces' ids never collide and an id leaks no ordering.
func newSessionID() string {
	var b [6]byte
	_, _ = rand.Read(b[:])
	return "s" + hex.EncodeToString(b[:])
}
