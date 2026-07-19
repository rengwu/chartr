package wayfinder

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
)

// Load reads an effort directory: a map.md plus a tickets/ directory.
func Load(dir string) (*Effort, error) {
	mapPath := filepath.Join(dir, "map.md")
	src, err := os.ReadFile(mapPath)
	if err != nil {
		return nil, fmt.Errorf("no map.md in %s: %w", dir, err)
	}

	e := &Effort{Dir: dir, Name: filepath.Base(dir), Map: ParseMap(mapPath, string(src))}

	ticketDir := filepath.Join(dir, "tickets")
	entries, err := os.ReadDir(ticketDir)
	if err != nil {
		return nil, fmt.Errorf("no tickets/ in %s: %w", dir, err)
	}
	for _, ent := range entries {
		if ent.IsDir() || filepath.Ext(ent.Name()) != ".md" {
			continue
		}
		p := filepath.Join(ticketDir, ent.Name())
		b, err := os.ReadFile(p)
		if err != nil {
			return nil, err
		}
		t, err := ParseTicket(p, ent.Name(), string(b))
		if err != nil {
			return nil, fmt.Errorf("%s: %w", ent.Name(), err)
		}
		e.Tickets = append(e.Tickets, t)
	}
	sort.Slice(e.Tickets, func(i, j int) bool { return e.Tickets[i].Num < e.Tickets[j].Num })
	return e, nil
}
