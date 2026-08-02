---
type: grilling
blocked_by: [03, 04, 05]
---

# What must a new route satisfy before it ships?

## Question

The decisions from tickets 02 through 05 are worth nothing if the next session
adding a handler does not know about them. This ticket turns them into something
enforceable.

The failure being designed against is documented in this repository. Two
`websocket.Accept` calls carry the same wrong option with near-identical comments
(`control.go:22-27`, `terminals.go:238-243`) — the second copied the first's
reasoning along with its defect. A rule that lives only in prose gets copied around
exactly this way.

Settle:

- **What is the rule, in one sentence a session can check itself against?** Not a
  policy document — a sentence. Something on the order of "every route is reachable
  only from the bound origin, and any handler that executes, writes outside the data
  directory, or discloses a path must additionally X." Get X from tickets 03 and 04.
  If it takes three sentences, it will not be followed.
- **Where does it live so it is found?** `CLAUDE.md` is read by every session and
  already carries hard rules in exactly this form for the design system — that is
  the working precedent for a rule sessions actually follow. An ADR carries the
  argument and the numbered decision. Probably both, with `CLAUDE.md` holding the
  checkable rule and the ADR holding the reasoning. Decide, and decide whether the
  trust boundary is architectural enough to earn an ADR number.
- **What enforces it rather than reminding of it?** The strongest available answer
  is structural: if the Host and origin checks live in one wrapper around the mux
  (fix map, ticket 02), a new route inherits them without knowing they exist, and
  the rule becomes a property of the server rather than a discipline. Say what the
  wrapper covers and, crucially, **what it cannot** — per-handler concerns like
  path validation or command construction fall outside it and need their own answer.
- **Is there a test that would have caught the original bug?** A test asserting that
  no `websocket.Accept` call sets `InsecureSkipVerify` would have. So would one
  enumerating the mux's routes and asserting each is refused from a foreign origin —
  which has the useful property that a *newly added* route fails it by default until
  someone thinks about it. Decide whether such a test is worth its maintenance, and
  if so, specify it precisely enough for the fix map to implement.
- **What about the reviewer's other observation?** They noted the codebase read as
  careful and named the parts that stood out — the open-URL allowlist in
  `cmd/webview/external.go`, the quoted AppleScript picker, the escape-then-transform
  markdown renderer, the pathspec-limited claim commits — calling this "one tradeoff
  that didn't hold rather than a pattern." Those are places where someone already
  did this right. Worth asking whether the rule should point at them as worked
  examples, since a rule with a precedent attached is easier to apply than one
  stated abstractly.

Done when: the rule exists as a checkable sentence; its home is decided and the
argument is recorded where a future session will collide with it; the structural
enforcement is specified along with what it does not cover; and the regression-test
question is answered either way, with a specification if the answer is yes.
