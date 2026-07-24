// Package tracker installs and recognizes chartr's tracker adapter — the
// docs/agents/issue-tracker.md a watched repo carries so a vanilla
// wayfinder-family skill writes maps in chartr's convention (local markdown under
// .plan/maps/) rather than its own .scratch/ default.
//
// Its job is to classify what already sits at that path so a caller can decide,
// with the operator's consent, whether to install, refresh, or leave it. Two
// rules shape it. Recognition is by an exact marker, never fuzzy prose-matching:
// a file chartr wrote carries Marker, and nothing else claims to. And a file
// without that marker is Foreign and is never overwritten without explicit
// consent — which is what makes "refuse to clobber a repo's existing tracker"
// fall out for free, with no brittle remote-vs-local detection gating behavior.
package tracker

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
)

// RelPath is where the adapter lives in a repo — the seam Matt-Pocock-family
// skills read to discover this repo's tracker.
const RelPath = "docs/agents/issue-tracker.md"

// Marker is the sentinel every chartr-authored adapter carries. Its presence
// means "chartr wrote this file"; the version string after it is informational,
// so a bumped template still reads as chartr's (Stale), never Foreign.
const Marker = "<!-- chartr-tracker-adapter:"

// State is what sits at RelPath in a repo.
type State int

const (
	// Absent — nothing at the path; a clean install.
	Absent State = iota
	// UpToDate — chartr's adapter, byte-identical to the current template.
	UpToDate
	// Stale — chartr's adapter, but a different version or operator-edited. A
	// refresh is offered, never applied silently.
	Stale
	// Foreign — a file with no chartr marker: another tracker (remote or local)
	// or a hand-authored one. Never overwritten without explicit consent.
	Foreign
)

func (s State) String() string {
	switch s {
	case Absent:
		return "absent"
	case UpToDate:
		return "up-to-date"
	case Stale:
		return "stale"
	case Foreign:
		return "foreign"
	default:
		return "unknown"
	}
}

// Result is a classification of a repo's adapter file.
type Result struct {
	State State
	// Path is the absolute path classified, whether or not a file exists there.
	Path string
	// RemoteHint is a best-effort guess at which remote tracker a Foreign file
	// configures ("gh", "glab", "linear"), or "". It only phrases a message —
	// behavior never depends on it. Empty for every non-Foreign state.
	RemoteHint string
}

// Classify reads the adapter file under repoRoot and says what it is, comparing
// it against template (the adapter chartr would write now). It never modifies the
// repo. A missing file is Absent, not an error; any other read error is returned.
func Classify(repoRoot, template string) (Result, error) {
	p := filepath.Join(repoRoot, filepath.FromSlash(RelPath))
	res := Result{Path: p}

	b, err := os.ReadFile(p)
	if errors.Is(err, os.ErrNotExist) {
		res.State = Absent
		return res, nil
	}
	if err != nil {
		return res, err
	}
	content := string(b)

	switch {
	case !strings.Contains(content, Marker):
		res.State = Foreign
		res.RemoteHint = remoteHint(content)
	case content == template:
		res.State = UpToDate
	default:
		res.State = Stale
	}
	return res, nil
}

// Install writes template to the adapter path under repoRoot, creating
// docs/agents/ as needed, and returns the absolute path written. It overwrites
// whatever is there, so the caller must have obtained consent for a Stale refresh
// or a Foreign replace (Classify says which is which). chartr writes the file
// only; committing it is the operator's — the path is theirs to review.
func Install(repoRoot, template string) (string, error) {
	p := filepath.Join(repoRoot, filepath.FromSlash(RelPath))
	if err := os.MkdirAll(filepath.Dir(p), 0o755); err != nil {
		return "", err
	}
	if err := writeAtomic(p, []byte(template)); err != nil {
		return "", err
	}
	return p, nil
}

// remoteHint best-effort-guesses which remote tracker a Foreign adapter names,
// purely to phrase a message. It is deliberately conservative and is never used
// to decide whether to write.
func remoteHint(content string) string {
	c := strings.ToLower(content)
	switch {
	case strings.Contains(c, "linear.app"), strings.Contains(c, "linear issue"):
		return "linear"
	case strings.Contains(c, "glab"), strings.Contains(c, "gitlab"):
		return "glab"
	case strings.Contains(c, "gh issue"), strings.Contains(c, "github.com"), strings.Contains(c, "gh cli"):
		return "gh"
	default:
		return ""
	}
}

// writeAtomic writes data to path via a temp file in the same directory and a
// rename, so a reader (or the fs watcher) never sees a half-written adapter.
func writeAtomic(path string, data []byte) error {
	tmp, err := os.CreateTemp(filepath.Dir(path), ".issue-tracker-*.tmp")
	if err != nil {
		return err
	}
	name := tmp.Name()
	defer os.Remove(name)
	if _, err := tmp.Write(data); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	return os.Rename(name, path)
}
