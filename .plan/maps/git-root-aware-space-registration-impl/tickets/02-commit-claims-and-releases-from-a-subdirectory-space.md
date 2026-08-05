---
type: task
blocked_by: [01]
claimed_by: sad179ab3cb27
claimed_at: 2026-08-05T19:29:17Z
---
<!-- triage: ready-for-agent -->

# Commit claims and releases from a subdirectory Space

## Question

Make ticket claiming and release work when the Space path is below the Git root.

When chartr writes a claim or release, it must run Git actions from the resolved
Git root and compute the ticket path from that root. The existing single-ticket
commit behavior must stay in place. A claim or release must not commit unrelated
changes anywhere else in the shared Git root.

## Done when

- A claim from a subdirectory Space writes the claim marker and creates a commit
  for that ticket file only.
- A release from the same Space clears the marker and creates a commit for that
  ticket file only.
- The ticket path works when it is nested below the Git root.
- An unrelated changed file is not staged or committed by either operation.
- Existing claim and release behavior for a Space at the Git root remains green.
- Tests prove the commit contents and Git history through the existing chartr
  process boundary.

## Answer

Implemented root-aware claim and release commits.

- Claim commits now resolve the Git root before they run. They calculate the
  ticket path relative to that root.
- Ticket release and dead-session release use the same Git-root rule.
- The existing path-limited `git add` and `git commit --only` behavior remains.
  An unrelated file in the Git root is not added or committed.
- Added process-boundary tests for a nested claim, a ticket release, and a
  dead-session release. The tests check the root-relative commit path, commit
  history, claim marker, release marker, and unrelated root changes.

Validation:

- `gofmt` passed.
- `go test ./... -run '^$' -count=1` passed with a temporary allowed Go cache.
- `go vet ./...` passed.
- `go test ./internal/registry -count=1` passed.
- The three new process tests could not run. The test harness cannot bind
  `127.0.0.1:0` in this sandbox and reports `operation not permitted`.

No public Git-root field, Space-path change, file-discovery change, or new Git
action was added.
