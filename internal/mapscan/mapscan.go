// Package mapscan discovers a space's wayfinder maps and derives each into the
// model the harness pushes (ticket 03). It is the harness-side policy layer over
// the ported model layer (internal/wayfinder): where maps live, how a malformed
// one is tolerated, and how derived status crosses onto the wire.
//
// Two rules shape it. Discovery is layout-agnostic — it reads wherever wayfinder
// writes, finding a map by the presence of its map.md rather than by a
// hard-coded path, so both the current `.plan/<slug>/` layout and the eventual
// `.plan/maps/<slug>/` one are found without either being wired in (story 12).
// And a malformed map is rendered as-is with its malformation surfaced, never
// refused (story 17): a ticket that will not parse becomes a surfaced defect and
// the rest of the map still derives.
package mapscan

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"

	"github.com/rengwu/wayfinder-harness/internal/model"
	"github.com/rengwu/wayfinder-harness/internal/wayfinder"
)

// planDir is the one fixed point: wayfinder roots its maps under `.plan/`. What
// sits *below* it — a map directory directly, or nested under `maps/` — is the
// convention the harness follows rather than hard-codes.
const planDir = ".plan"

// Discover finds every wayfinder map under repoRoot's `.plan/` and derives each
// into a model.Map, ordered for the sidebar (finished maps last, then by slug).
// The result is never nil. A repo with no `.plan/` yields an empty slice, not an
// error — a mapless space is normal.
func Discover(repoRoot string) []model.Map {
	dirs := mapDirs(filepath.Join(repoRoot, planDir))
	maps := make([]model.Map, 0, len(dirs))
	for _, dir := range dirs {
		maps = append(maps, deriveMap(dir))
	}
	// Finished maps sort last; among the rest, slug order is stable against
	// everything but the map set itself, so the sidebar holds still under a push.
	sort.SliceStable(maps, func(i, j int) bool {
		if maps[i].Finished != maps[j].Finished {
			return !maps[i].Finished
		}
		return maps[i].Slug < maps[j].Slug
	})
	return maps
}

// mapDirs walks a `.plan/` tree and returns every directory that directly holds
// a map.md, in path order. Finding one stops the descent into it — a map's own
// tickets/ and assets/ never nest another map — which is what lets one walk
// handle both the flat `.plan/<slug>/` layout and the nested `.plan/maps/<slug>/`
// one without naming either.
func mapDirs(planRoot string) []string {
	var dirs []string
	_ = filepath.WalkDir(planRoot, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			// An unreadable subtree (permissions, a vanished dir mid-walk) is
			// skipped, never fatal: discovery surfaces what it can read.
			if d != nil && d.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}
		if !d.IsDir() {
			return nil
		}
		if _, statErr := os.Stat(filepath.Join(path, "map.md")); statErr == nil {
			dirs = append(dirs, path)
			return filepath.SkipDir
		}
		return nil
	})
	sort.Strings(dirs)
	return dirs
}

// deriveMap reads one map directory tolerantly: the map body, then each ticket,
// collecting parse failures as surfaced malformations rather than refusing the
// map. It then runs the ported lint over what parsed and folds its diagnostics
// in, so a dangling edge or a drifted index bites as a malformation on the map
// it belongs to.
func deriveMap(dir string) model.Map {
	slug := filepath.Base(dir)
	m := model.Map{Slug: slug, Name: slug, Dir: dir}

	src, err := os.ReadFile(filepath.Join(dir, "map.md"))
	if err != nil {
		// mapDirs only yields dirs whose map.md stat succeeded, so a read error
		// here is a race (the file vanished) — surface it and render the shell.
		m.Malformations = append(m.Malformations, fmt.Sprintf("map.md: %v", err))
		m.Tickets = []model.Ticket{}
		return m
	}
	wmap := wayfinder.ParseMap(filepath.Join(dir, "map.md"), string(src))
	if wmap.Name != "" {
		m.Name = wmap.Name
	}
	m.Destination = wmap.Destination

	tickets, malformations := loadTickets(dir)
	m.Malformations = append(m.Malformations, malformations...)

	eff := &wayfinder.Effort{Dir: dir, Name: slug, Map: wmap, Tickets: tickets}
	frontier := map[int]bool{}
	for _, t := range eff.Frontier() {
		frontier[t.Num] = true
	}

	// Surface lint diagnostics as malformations — the map is read as-is and its
	// drift is shown where it bites, never a reason to refuse it.
	for _, diag := range wayfinder.Lint(eff, wayfinder.DefaultOptions()) {
		m.Malformations = append(m.Malformations, formatDiag(diag))
	}

	m.Tickets = make([]model.Ticket, 0, len(tickets))
	allClosed := len(tickets) > 0
	for _, t := range tickets {
		if !t.Status.Closed() {
			allClosed = false
		}
		m.Tickets = append(m.Tickets, model.Ticket{
			Num:       t.Num,
			Slug:      t.Slug,
			Title:     t.Title,
			Type:      string(t.Type),
			Status:    string(t.Status),
			BlockedBy: t.BlockedBy,
			Frontier:  frontier[t.Num],
		})
	}
	m.Finished = allClosed
	return m
}

// loadTickets parses every NN-slug.md under dir/tickets, in number order. A
// ticket that will not parse becomes a surfaced malformation and is dropped from
// the derived set; the rest of the map is unaffected. A map with no tickets/
// directory is normal (a freshly charted map) and yields no malformation.
func loadTickets(dir string) ([]*wayfinder.Ticket, []string) {
	ticketDir := filepath.Join(dir, "tickets")
	entries, err := os.ReadDir(ticketDir)
	if err != nil {
		return nil, nil
	}

	var tickets []*wayfinder.Ticket
	var malformations []string
	for _, ent := range entries {
		if ent.IsDir() || filepath.Ext(ent.Name()) != ".md" {
			continue
		}
		p := filepath.Join(ticketDir, ent.Name())
		b, readErr := os.ReadFile(p)
		if readErr != nil {
			malformations = append(malformations, fmt.Sprintf("tickets/%s: %v", ent.Name(), readErr))
			continue
		}
		t, parseErr := wayfinder.ParseTicket(p, ent.Name(), string(b))
		if parseErr != nil {
			malformations = append(malformations, fmt.Sprintf("tickets/%s: %v", ent.Name(), parseErr))
			continue
		}
		tickets = append(tickets, t)
	}
	sort.Slice(tickets, func(i, j int) bool { return tickets[i].Num < tickets[j].Num })
	return tickets, malformations
}

func formatDiag(d wayfinder.Diagnostic) string {
	loc := filepath.Base(d.File)
	if d.Line > 0 {
		return fmt.Sprintf("%s (%s:%d): %s", d.Level, loc, d.Line, d.Msg)
	}
	return fmt.Sprintf("%s (%s): %s", d.Level, loc, d.Msg)
}
