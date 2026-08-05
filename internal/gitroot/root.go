// Package gitroot resolves the Git root for a Space path.
package gitroot

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// ErrNotRepository means that the Space path is not inside a Git repository.
// Registration may use this result to allow git init. Other errors must stop
// registration.
var ErrNotRepository = errors.New("git root: path is not inside a repository")

// Resolve returns the nearest Git repository root for spacePath.
func Resolve(spacePath string) (string, error) {
	abs, err := filepath.Abs(spacePath)
	if err != nil {
		return "", fmt.Errorf("normalizing Space path %q: %w", spacePath, err)
	}
	spacePath = filepath.Clean(abs)

	cmd := exec.Command("git", "-C", spacePath, "rev-parse", "--show-toplevel")
	out, err := cmd.CombinedOutput()
	if err != nil {
		output := strings.TrimSpace(string(out))
		if isNoRepository(output) && !hasGitMarker(spacePath) {
			return "", ErrNotRepository
		}
		if output == "" {
			return "", fmt.Errorf("git root discovery in %s failed: %w", spacePath, err)
		}
		return "", fmt.Errorf("git root discovery in %s failed: %w\n%s", spacePath, err, output)
	}

	root := strings.TrimSpace(string(out))
	if root == "" {
		return "", fmt.Errorf("git root discovery in %s returned an empty root", spacePath)
	}
	abs, err = filepath.Abs(root)
	if err != nil {
		return "", fmt.Errorf("normalizing Git root %q: %w", root, err)
	}
	return filepath.Clean(abs), nil
}

// Init creates a Git repository in spacePath.
func Init(spacePath string) error {
	cmd := exec.Command("git", "init")
	cmd.Dir = spacePath
	if out, err := cmd.CombinedOutput(); err != nil {
		output := strings.TrimSpace(string(out))
		if output == "" {
			return fmt.Errorf("git init in %s failed: %w", spacePath, err)
		}
		return fmt.Errorf("git init in %s failed: %w\n%s", spacePath, err, output)
	}
	return nil
}

func isNoRepository(output string) bool {
	return strings.Contains(output, "not a git repository (or any of the parent directories): .git")
}

// hasGitMarker only classifies a failed Git discovery. It never selects a
// repository root; Git remains the source of truth for successful discovery.
func hasGitMarker(spacePath string) bool {
	current := filepath.Clean(spacePath)
	for {
		if _, err := os.Lstat(filepath.Join(current, ".git")); err == nil {
			return true
		}
		parent := filepath.Dir(current)
		if parent == current {
			return false
		}
		current = parent
	}
}
