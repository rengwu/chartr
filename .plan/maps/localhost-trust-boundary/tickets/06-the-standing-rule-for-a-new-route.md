---
type: grilling
blocked_by: [03, 04, 05]
claimed_by: s12b38a391114
claimed_at: 2026-08-02T19:36:15Z
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

## Answer

**The rule is one sentence, it lives in `CLAUDE.md` with its argument in ADR 0017,
and what enforces it is that there is nothing for a route author to do.**

> **Every request is admitted by the one middleware around `s.mux` — exact Origin
> and Host for browser callers, plus the per-process capability for every caller —
> before any handler runs; a handler validates what was sent, never who sent it.**

The unauthenticated route list is **empty**, and stays empty because the two
unauthenticated behaviours are branches inside the gate rather than routes. One new
test ships. The route-enumeration test the question proposed is **rejected**, along
with the route-table refactor it would require — see *The test question* below, which
is the largest subtraction in this answer.

### Why the rule reads that way

The sentence has to survive being followed literally by someone in a hurry, so every
clause is load-bearing:

- **"admitted"**, not "reachable from the bound origin". The example sentence in the
  question was written before 03. A rule naming only origin gets followed exactly and
  leaves the next route open to `curl`, which is the caller 02 explicitly puts
  outside the boundary.
- **"exact Origin and Host … plus the per-process capability"** — both proofs, named,
  in the sentence a session actually reads. The `plus` is not decoration: Origin and
  Host are browser proofs and prove nothing about a local process; the capability
  identifies the admitted client and proves nothing about DNS rebinding. Neither one
  is the boundary.
- **"the one middleware around `s.mux`"** names the place, so the sentence is
  checkable against a diff rather than against a feeling. If a change touches
  admission anywhere else, it is wrong by inspection.
- **"before any handler runs"** is what makes route classification impossible. There
  is no read/write split to get wrong (03), no per-route token (03), no bind-conditional
  branch (04), no `desktopSkipAuth` (05). The gate applies to `/api/health` and
  `/ws/terminal/{id}` identically because it applies before it knows which one it is.
- **"a handler validates what was sent, never who sent it"** is the half that does
  real work for an author, and it is the half that would have caught the reported
  bug. `websocket.Accept(..., InsecureSkipVerify: true)` is a handler deciding about
  its caller. Under this clause it is wrong in form, before anyone reasons about
  whether localhost makes it safe — which is the reasoning that produced it and then
  got copied to a second call site along with the defect.

Ticket 04's bind rule is deliberately **not** in the sentence. It is a startup
decision about a listener, evaluated once before `net.Listen`, and it has no
route-level consequence; folding it in would cost a clause and teach a route author
something they can do nothing with. It is a clause of the ADR, exactly as 04 asked.

### One consequence for both websocket handlers, which the fix map should know about

Once the gate exists, the correct `AcceptOptions` at `control.go` and `terminals.go`
is **none at all** — `websocket.Accept(w, r, nil)`.

This matters because the obvious two repairs are both wrong under the rule. Passing
`InsecureSkipVerify: true` and relying on the gate is the correct *behaviour* spelled
with the exact string that was the bug, which no future reader can be expected to
tell apart. Passing `OriginPatterns` is a handler deciding about its caller — a second
admission mechanism, needing the admitted-origin set plumbed into two handlers, which
is the "gate with modes" shape 04 rejected.

The library's default check is Origin-host against the `Host` header. The gate has
already pinned `Host` to a closed allowlist, so the default check is then exactly
right, costs nothing, and needs no configuration. It also holds under Vite, whose
proxy preserves `Host` by default, so the dev origin satisfies it without a pattern —
the dev origin is admitted at the gate (02), which is the only place it should be
named.

**Sequencing, so this contradicts nobody.** The fix map's ticket 01 ships
`OriginPatterns` and should ship as written — in a world with no gate, the handler is
the only place the check can live. When the gate lands, those options come *out* of
both handlers. That removal is part of the gate's implementation ticket, not a
revision of the fix map.

### Where it lives: `CLAUDE.md` and one ADR, and nowhere else

Both, with a hard split: **`CLAUDE.md` holds the rule, the ADR holds the argument.**
The map already knows why an unargued rule is not enough — the comment at
`control.go:23-25` is a rule-shaped statement with no argument behind it, and what
made it dangerous was that it read as settled. And `CLAUDE.md` alone is where a
reader who disagrees has nothing to disagree with.

`CLAUDE.md` earns it because it is the one document loaded into every session's
context, and because the design-system section is the working precedent in this repo
for hard rules that sessions actually follow. The new section is short on purpose —
`CLAUDE.md` stops being read when it stops being scannable.

```markdown
## The server's trust boundary (`internal/server/`)

**Every request is admitted by the one middleware around `s.mux` — exact Origin and
Host for browser callers, plus the per-process capability for every caller — before
any handler runs; a handler validates what was sent, never who sent it.** The
argument is [ADR 0017](docs/adr/0017-....md); read it before changing admission.

- **A new route needs nothing.** It inherits the gate by being on the mux. Do not add
  an admission check to a handler, do not classify a route as read-only or internal,
  and do not pass `AcceptOptions` to `websocket.Accept`.
- **No route is unauthenticated.** The bootstrap redirect and the denial page are
  branches inside the gate, not routes. Adding an entry to that branch is a change to
  the trust boundary and needs the ADR amended.
- **Handlers still own their inputs.** The gate proves the caller is the operator. It
  proves nothing about what the operator's browser was told to send — a path, a
  command argument, a URL, a rendered string. Guard those in the handler.
- **The capability lives in memory for the process lifetime.** Never in a file,
  `~/.config`, `.chartr/shell.lock`, argv, a child environment, a log, or a snapshot.
  Never present restarting chartr as a way to recover access.
```

**ADR 0017** — next free number; the last is 0016. It is architectural in the same
sense ADR 0010's chrome/island split is: it decides a boundary the whole server is
built either side of. Per 04's instruction it is **one** ADR, carrying 02, 03, 04 and
05 with their amendments as clauses — not four, and not a separate ADR for the bind
rule, which would imply the bind is its own posture.

Its spine, in the repo's ADR idiom (sentence title, argument, `## Consequences`,
`## Considered options`):

1. **The boundary is client admission** (02) — who is in, who is out, and that
   disclosure is protected in its own right.
2. **One reusable per-process capability at one gate** (03 + amendment) — and,
   stated as the thing a future session will try to simplify: **no expiry, no
   rotation, no consumption, and restart is not a recovery path.** 03's amendment
   exists because a consumed nonce made restart the accidental recovery mechanism;
   that is the sentence the ADR is carrying forward.
3. **Loopback by default, `-expose-cleartext` or refuse** (04) — as a clause, with
   the cleartext-disclosure argument that decides it.
4. **One posture, two delivery adapters** (05 + amendment) — the shell's `?k=` first
   navigation and the `SIGUSR1` second launch, with no desktop auth mode.
5. **What the gate does not cover**, with the worked examples below.

**Rejected homes.** A `doc.go` package comment in `internal/server` — appealing
because it is closest to the code, rejected because it makes three copies of one rule
and this repo already has a standing drift problem with duplicated specs; the
middleware's own comment is the code, not a third document. `SECURITY.md` — that is
the companion map's, it faces outward at reporters, and a session adding a route will
never open it. An ADR with no `CLAUDE.md` entry — nobody reads `docs/adr/` before
adding a handler, which is the entire failure this ticket exists to prevent.

**When they land: in the same commit as the gate, not now.** A `CLAUDE.md` rule that
points at a middleware which does not exist tells every session in the interim
something false about the repo, and a stale hard rule is how hard rules stop being
believed. The exact text is written above so the implementing session copies it rather
than re-derives it. **Trigger to land: the commit that introduces the admission
middleware.**

### What the gate covers, and what it cannot

**Covers, for every route, with no author involvement:** Host against the closed
allowlist; Origin against the admitted set; the capability by cookie, `?k=` query
parameter or `Authorization: Bearer`; the bootstrap redirect; the denial response.
Both websockets are covered because a handshake is an ordinary HTTP `GET` until
`Accept` hijacks it, so the middleware sees it and the cookie rides it — that is what
keeps "one gate" true of the code and not only of the prose.

**Cannot cover — and this is the half that needs saying, because "the gate handles
it" is a comfortable thing to believe:** the gate proves the caller is the operator's
admitted browser document. It proves nothing about what that document was *told* to
send. Ticket 02 puts the admitted document inside the boundary while conceding that
agent-authored text — terminal output, ticket bodies, tool results — flows through it.
So handler inputs are attacker-influenced even from a perfectly admitted client, and
per-handler concerns survive the gate intact:

- **A path from the client.** `handleRegister` takes a filesystem path; the config
  surface deliberately does not, resolving *named* layers server-side
  (`internal/server/configsurface.go`) so a local server cannot be talked into
  opening an arbitrary file.
- **A command being constructed.** Spawn and launch build agent command lines; the
  folder picker interpolates into AppleScript and quotes rather than trusts it
  (`internal/server/folderpicker.go:93-95`).
- **A write's blast radius.** Claim commits are pathspec-limited, never amending and
  never pushing (`internal/server/claim.go:67-72`).
- **A string being rendered.** The markdown renderer escapes then transforms
  (`web/src/lib/markdown.ts`).
- **A value crossing into native code.** `checkExternalURL` re-validates in native
  what the page passed (`cmd/webview/external.go`), because either side alone would
  be the only thing between terminal output and a launched application.

**These four are named in the ADR, not in `CLAUDE.md`.** A rule with a precedent
attached is easier to apply — the question is right about that — but file paths rot,
and `CLAUDE.md` is the one document that must never be stale to keep working. An ADR
is a dated record and is allowed to age. The `CLAUDE.md` bullet states the division of
labour abstractly and points at the ADR for the examples.

### The unauthenticated exemption list, enumerated: it is empty

Not "small" — **empty**, at the route level, and that is a stronger claim than the
ticket assumed. The question inherited "inert static assets, the bootstrap redirect,
the denial page" as the exempt set. Working it through, none of the three is a route:

- **Static assets are not exempt and do not need to be.** Bootstrap sets an
  `HttpOnly` cookie at `Path=/` on the operator's own origin, so the browser sends it
  with `/assets/*.js`, the CSS, and the self-hosted fonts exactly as it sends it with
  `/api/*`. The SPA is `go:embed`ed and fetches nothing from a CDN (CLAUDE.md), so
  there is no pre-admission asset request in the product. The first request an
  unadmitted browser makes is the one the gate answers.
- **The bootstrap redirect is a branch, not a route** — a `GET` carrying a valid `k`
  gets `Set-Cookie` and a `302`, inside the middleware, before routing (03 amendment).
  It is reachable at every path because it is not at any path.
- **The denial page is rendered by the gate**, and this is now a **constraint on the
  ticket that builds it**, not an observation: it must be a self-contained response
  written by the middleware — inline styles, no bundle fetch, no `mux` registration.
  If it becomes a route, the exemption list stops being empty and every claim in this
  section weakens. The map already noticed it "cannot use anything the cockpit fetches
  after admission"; the reason is exactly this.

Two consequences follow, both worth writing down:

- **`/api/health` is gated.** 03 said so, and it means the release smoke that curls
  health (`mac-app-bundle-impl` tickets 01-03) needs the capability or needs replacing
  by the screenshot check it already runs beside. Small, real, and better found here
  than in a release.
- **The test rig changes in one file.** `internal/chartrtest/rig.go` uses a bare
  `http.Get`; adding `Authorization: Bearer` there and on the two dials is 03's
  "test-only handoff", and every one of the ~30 existing test files is untouched. The
  conformance cost of the whole boundary is one file.

### The test question: one test, and a refusal

**Rejected: the route-enumeration test.** It was the question's own best idea and it
does not survive the enforcement being structural. Four reasons, any one sufficient:

1. **The wrapper is total, so enumeration is vacuous.** The gate refuses before
   routing, so it refuses `/api/spaces`, `/ws/control` and `/nonsense` identically.
   Asserting the same refusal 25 times proves what one request proves.
2. **There is nothing to enumerate.** Go's `http.ServeMux` does not expose its
   patterns, so the test needs a route table — converting 25 `HandleFunc` calls into
   a declared slice, bought entirely to feed a test. That is the shape of
   overengineering this map's amendments have twice had to remove.
3. **The exempt set is empty**, per above. A test that enumerates an empty list to
   prove nothing was added to it is a comment with a compiler.
4. **"A new route fails until someone thinks about it" is the wrong goal here.** It
   is the property you need when enforcement is per-route discipline. Under a wrapper
   the correct property is the opposite: a new route needs *no* thought and cannot get
   admission wrong. What must be hard is adding an *exemption* — and that is a change
   to the gate function, which is one small file a reviewer reads, not a diff hidden
   among 25 registrations.

**Kept, and it is the only new test: the source guard.** One test walking the tree and
failing if `InsecureSkipVerify` appears anywhere. It is roughly fifteen lines, it has
no maintenance surface, and it is literally the test that would have caught the
reported bug — including the second call site, which is the copy-paste this ticket
exists to prevent. It becomes unambiguous the moment the gate lands, because after
that there is no legitimate use of the string: the correct handler passes `nil`.

**Extended by two assertions, not a suite.** The gate's conformance tests are already
mandated by 05's amendment (protected HTTP, both WebSockets, bootstrap redirect and
cookie issuance, clean-URL denial). Add exactly two cases to them: **`GET /api/health`
without a capability is refused**, and **`GET /` without a capability is refused**.
Those two are chosen because they are the routes most likely to be argued into an
exemption later — "it's only a health check", "it's only static HTML". Naming them in
a test makes that argument a visible diff, which is the whole property the question
wanted from enumeration, at two lines instead of a refactor.

### One doubt on a blocker, worth checking before it becomes a support thread

**03's `SameSite=Strict` may break its own bootstrap redirect, and the failure lands
squarely on the operator.** The flow is `GET /?k=<cap>` → `Set-Cookie` → `302 /` →
cockpit. A `Strict` cookie is *set* on that first response regardless, but whether the
browser *sends* it on the redirect hop depends on how it classifies that navigation.
A URL the operator clicks in their terminal is browser-initiated with no initiator
origin and should be treated as same-site; a URL opened from a page — a link relayed
into a browser tab — may not be, in which case a successful bootstrap lands on the
denial page. That reads as "authentication is broken", and the remedy it suggests to
a frustrated operator is restarting the process, which is the one outcome 03's
amendment exists to make impossible.

Not reopening 03 and not designing around it. **This is a verification item for the
gate's implementation ticket:** confirm the bootstrap hop carries the cookie in the
browsers chartr supports and in the webview. If it does not, the correction is
`SameSite=Lax`, and **Lax is sufficient here** — it exposes only cross-site top-level
`GET` navigations, whose responses an attacker cannot read, and the gate independently
requires exact Origin and Host on anything that matters. Not a weakening; a different
spelling of a control that is not carrying the weight alone.

Nothing else in 03, 04 or 05 came out shaky under this ticket's weight. 04's bind rule
has no route-level consequence and needed none. 05's `SIGUSR1` path touches no server
code, which was its author's point and holds.

### What this ticket deliberately does not build

The failure mode on this map has twice been a good decision arriving with machinery
attached, so the refusals are as much the answer as the decisions:

- **No route table, no enumeration test.** Argued above.
- **No custom linter or `go vet` analyser.** One fifteen-line test does the one job;
  `go test ./...` already runs on every commit (CLAUDE.md).
- **No CI security job.** Same reason.
- **No per-route annotation, comment convention, or "is this route sensitive?"
  checklist.** 03 rejected classification because it is the trap every new handler
  can get wrong. A checklist is per-route discipline in a document, which is the thing
  the wrapper replaces.
- **No second policy document.** Two homes, and the second is an ADR that ages
  honestly.
- **No cockpit indicator of authentication state.** 04 refused the bind indicator for
  the reason that applies here too: it would report back a fact the operator cannot
  act on, and an admitted operator already knows they are admitted — they are looking
  at the cockpit.
- **No rate limiting on the gate.** 04 named the missing rate limiting and accepted
  it; a 32-byte capability is not brute-forced, and adding a limiter would put timing
  state on the hot path of a long-running process for no gain.

One implementation note that is spelling rather than machinery: compare the capability
with `subtle.ConstantTimeCompare`, not `==`. One standard-library call, no mechanism.

### Revisit trigger

- **Anything is proposed for the unauthenticated branch.** A fifth entry beside
  bootstrap and denial is a change to the boundary and needs ADR 0017 amended, not a
  commit. The bootstrap redirect, `Enter`-to-reprint and `SIGUSR1` remain the only
  sanctioned out-of-band paths (03, 05).
- **A route appears that the gate cannot cover** — a second listener, a Unix socket, a
  handler registered outside `s.mux`, an automation interface (03 anticipates one).
  Then "one middleware around `s.mux`" has stopped naming the boundary and the rule's
  sentence is wrong, not merely incomplete.
- **A second `websocket.Accept` or an equivalent per-handler caller check lands
  anyway.** That is evidence the rule is not being read where it is, and the answer is
  to move it, not to add process around it.
- **The source guard fires on a legitimate change**, or the gate's conformance tests
  become expensive to keep. Either means the enforcement has drifted from the code and
  should be re-argued rather than patched.
- **`CLAUDE.md`'s trust-boundary section grows past the bullets above.** Length is the
  measurable failure mode for a document whose only power is being read. Growth means
  the rule has stopped being one sentence, and the excess belongs in the ADR.
