package server

import "os"

// chartr's own artefacts are the operator's alone. A session payload is the whole
// prompt a session was handed — ticket text, map notes, whatever skill content was
// composed into it — and the config beside it records what runs against their
// repositories. None of that is another login's business, so chartr writes it 0600
// under 0700 rather than leaving a 0644 for the machine's umask to narrow if it
// happens to be set tightly (websocket-origin-fix, ticket 05).
//
// This is deliberately not every mode in the tree. What chartr writes into the
// operator's *repository* — the `*` ignore marker beside the run directory, the
// tickets a claim rewrites under `.plan/`, the adapter installed under `docs/` —
// stays an ordinary repository file at 0644/0755. Those belong to git and to the
// human, and tightening them would be tightening someone else's checkout.
const (
	ownerFileMode os.FileMode = 0o600
	ownerDirMode  os.FileMode = 0o700
)

// writeOwnerFile writes data to path as an owner-only file, chmod included.
// os.WriteFile applies its mode only when it *creates* the file, so a path that
// already exists — a temp file left behind by a crashed atomic write, a payload
// written over an earlier one — would otherwise keep whatever mode it was born
// with, which is exactly the 0644 this closes.
func writeOwnerFile(path string, data []byte) error {
	if err := os.WriteFile(path, data, ownerFileMode); err != nil {
		return err
	}
	return os.Chmod(path, ownerFileMode)
}
