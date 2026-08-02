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

**Constraints inherited from 03 and 05 — read these before answering.** The X in the
rule sentence is now known, and the shape of the enforcement with it. Four things
would be defects to reintroduce:

- **The rule names two proofs, not one.** The example sentence above mentions only
  "reachable from the bound origin", which is the pre-03 world. A route is reached by
  an *admitted* client: exact Origin and Host for browser-shaped callers **and** the
  per-process capability, carried by cookie or `Authorization` header. A rule that
  says only "origin" will be followed literally and will leave the next route open to
  `curl`.
- **Nothing is classified per route.** Ticket 03 rejected read/write splits,
  route-specific tokens and per-handler admission checks, explicitly because
  classification is a trap every new handler can get wrong. The rule must therefore
  not ask an author to categorise their route — the gate applies to all of it, and
  the only thing an author can get wrong is landing in the exemption list.
- **The exemption list is the thing worth testing.** The structural enforcement is one
  middleware wrapping `s.mux` that owns both admission and the `?k=` bootstrap branch
  (03 amendment, 05 amendment). Its unauthenticated surface is closed and small —
  inert static assets, the bootstrap redirect, the denial page — so the regression
  test that has teeth is one that enumerates the mux's routes and asserts each is
  refused **both** from a foreign origin and without a capability, with the
  unauthenticated set named explicitly so adding to it is a visible diff rather than
  a quiet default. A newly added route should fail that test until someone thinks
  about it, which is the property the question above already identified.
- **Recovery is part of the rule's world, not an exception to it.** The bootstrap
  redirect, the CLI's `Enter`-to-reprint, and the shell's `SIGUSR1` activation are
  the *only* sanctioned unauthenticated or out-of-band paths. A new route must not
  add a fifth, and no route, message, or document may present restarting the process
  as a way to recover access.

If the trust boundary earns an ADR — it should — the ADR carries tickets 02, 03 and
05 together with their amendments, since the capability's process lifetime and the
absence of any restart-based recovery are the parts a future session is most likely
to "simplify" back into a defect.

Done when: the rule exists as a checkable sentence naming both proofs; its home is
decided and the argument is recorded where a future session will collide with it; the
structural enforcement is specified along with what it does not cover; the
unauthenticated exemption list is enumerated rather than implied; and the
regression-test question is answered either way, with a specification if the answer
is yes.
