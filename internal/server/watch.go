package server

import (
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/fsnotify/fsnotify"
)

// discoveryDebounce coalesces a burst of filesystem events — a `git pull`
// rewriting many files, an editor's write-then-rename — into a single rebuild.
const discoveryDebounce = 120 * time.Millisecond

// planDirName is the one fixed point of the layout wayfinder writes to: maps
// live under `.plan/`. What sits below it is the convention discovery follows
// rather than hard-codes (see internal/mapscan).
const planDirName = ".plan"

// watcher gives the harness discovery-by-notice (story 11): it watches each
// registered space's repo root and its whole `.plan/` subtree, and on any change
// fires a debounced rebuild so a map created by a hosted shell, an external
// terminal, or a `git pull` enters the snapshot with no operator refresh action.
//
// It watches the repo root as well as `.plan/` so a `.plan/` that does not exist
// yet is still caught when it appears, and it adds a watch to every new
// directory the moment it is created so the map.md written into a fresh slug
// directory a beat later is noticed too. A watch that cannot be established
// degrades to no live discovery for that path — operator actions still rebuild —
// rather than failing the harness.
type watcher struct {
	fsw      *fsnotify.Watcher
	onChange func()

	mu      sync.Mutex
	watched map[string]bool // directories currently under an fsnotify watch
	timer   *time.Timer
}

// newWatcher starts a watcher whose events fire onChange (debounced). If the OS
// watcher cannot be created, it returns a watcher that watches nothing: the
// cockpit stays fully usable and maps still appear on every operator action,
// only unattended notice is lost.
func newWatcher(onChange func()) *watcher {
	w := &watcher{onChange: onChange, watched: map[string]bool{}}
	fsw, err := fsnotify.NewWatcher()
	if err != nil {
		return w
	}
	w.fsw = fsw
	go w.run()
	return w
}

// setRoots reconciles the watch set to cover each root's repo directory and
// every directory under its `.plan/` tree, dropping watches on paths no root
// still claims. It is called on every rebuild, so registering and forgetting
// spaces move their watches with them.
func (w *watcher) setRoots(roots []string) {
	if w.fsw == nil {
		return
	}
	want := map[string]bool{}
	for _, root := range roots {
		want[root] = true
		collectDirs(want, filepath.Join(root, planDirName))
	}

	w.mu.Lock()
	defer w.mu.Unlock()
	for p := range w.watched {
		if !want[p] {
			_ = w.fsw.Remove(p)
			delete(w.watched, p)
		}
	}
	for p := range want {
		if !w.watched[p] {
			if err := w.fsw.Add(p); err == nil {
				w.watched[p] = true
			}
		}
	}
}

func (w *watcher) run() {
	for {
		select {
		case ev, ok := <-w.fsw.Events:
			if !ok {
				return
			}
			// A newly created directory under a watched tree (a new map slug, or
			// `.plan/` itself) must be watched at once, or the map.md written into
			// it a moment later goes unnoticed until the next operator action.
			if ev.Op&fsnotify.Create != 0 {
				if info, err := os.Stat(ev.Name); err == nil && info.IsDir() {
					w.mu.Lock()
					if !w.watched[ev.Name] {
						if err := w.fsw.Add(ev.Name); err == nil {
							w.watched[ev.Name] = true
						}
					}
					w.mu.Unlock()
				}
			}
			w.schedule()
		case _, ok := <-w.fsw.Errors:
			if !ok {
				return
			}
		}
	}
}

func (w *watcher) schedule() {
	w.mu.Lock()
	defer w.mu.Unlock()
	if w.timer != nil {
		w.timer.Stop()
	}
	w.timer = time.AfterFunc(discoveryDebounce, w.onChange)
}

func (w *watcher) close() {
	if w.fsw != nil {
		_ = w.fsw.Close()
	}
}

// collectDirs adds every directory under root (inclusive) to set. A subtree it
// cannot read is skipped rather than fatal — the watch covers what it can.
func collectDirs(set map[string]bool, root string) {
	_ = filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			if d != nil && d.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}
		if d.IsDir() {
			set[path] = true
		}
		return nil
	})
}
