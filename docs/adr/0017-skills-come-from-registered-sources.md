# Skills come from registered sources; chartr ships none

Supersedes the content half of [0009](0009-config-layers-execution-vs-content.md).

chartr no longer ships a skill library, and no longer resolves one. The three
layers — built-in, user, workspace — the closed name set, whole-skill shadowing,
the fork marker and its drift warning, the shipped-hash comparison and the
materializer are all deleted. In their place the operator registers an **ordered
list of skill sources** — local folders and pinned git repos — and chartr resolves
into it.

The deciding finding is that the layer model owned the one thing the operator most
wants to own. Wanting someone else's `prototype` skill meant forking chartr's copy
in place and maintaining a drift warning against it, rather than pointing at the
repo it already lives in; and a skill chartr did not ship could not be reached at
all. A cockpit that claims to be hackable cannot also own what its agents are told.

## The model

- **One ordered list, position is precedence.** A bare skill name takes the first
  hit; a `Source name/skill` reference addresses one source exactly and never falls
  through — naming a source and silently receiving a different source's skill is
  worse than an error. A disabled source is skipped by both forms.
- **Explicit `[roles]` bindings.** Four lines in the operator's own config bind
  each ticket type to one *source-qualified* skill, so reordering sources is a
  discovery change and never a behavioural one. An unresolvable binding refuses the
  spawn — no terminal, no claim commit.
- **chartr ships two payloads and one conventions ruleset, and nothing else.** A
  ticket session gets chartr's embedded core plus the bound role skill's body,
  concatenated whole. A free session is told what chartr is and what skills exist,
  and nothing about how to behave. Both point at a generated `conventions.md` —
  chartr's whole file-format contract, embedded and materialized under the config
  root — and both append the operator's `preferences.md` verbatim and unranked.
  The conventions file is the one document an operator cannot shadow, disable or
  reorder away, because a source whose skills write maps chartr cannot read is a
  source whose work is invisible. That is the narrow exception to hackability a
  parser contract requires.
- **A default source, seeded from the binary.** `chartr-skills` is a separate MIT
  repo of seven skills, vendored in so a first run works offline, sitting *last* so
  anything registered shadows it, and convertible to a pinned checkout by one
  explicit fetch. Nothing ever fetches unattended.

This **reaffirms [0002](0002-agent-agnostic-adapters.md) and
[0005](0005-assembled-context-no-agent-memory.md)** rather than loosening them.
chartr still composes its own payload — the role body is concatenated into the
document chartr writes, not handed to a harness's own skill mechanism — and still
assembles the context region fresh at every spawn, never accumulated. What changed
is where the role's *bytes* come from, not who composes or when.

## Trust, and what it costs

**A registered source is executable text that reaches agents chartr runs with
permissions skipped.** That is the sharpest trade-off in the design and it is a
decision, not an oversight.

For a git source, **the only assertion of trust in its entire lifetime is the
moment its URL is typed.** There is deliberately no confirm gate before the clone —
pasting the URL is the deliberate act — and no changed-skills summary after a
refresh. After registration a refresh can move the source arbitrarily far, and the
only visible evidence is a short sha. Both gates were argued and declined in favour
of the fewest steps.

What contains it is narrow and worth stating plainly: nothing fetches unattended,
so a repo trusted on Tuesday cannot silently become something else on Friday
without the operator asking; the checkout is chartr's own directory, not a
workspace, so "I want to edit this" is answered by a `dir` source rather than by
edits a refresh would discard; and a source's URL is **never printed into a
payload**, because a fetchable address in a document handed to a permission-skipped
agent is the one thing that would let the trust boundary move on its own.

chartr validates nothing else about a source's skills. Discovery — a directory
holding a `SKILL.md` — is the whole test, at registration and at resolve. A
malformed skill is a skill that injects whatever it says. Validating someone else's
markdown would mean chartr rejecting a repo for not being written the way chartr
writes, which is exactly what registering your own source exists to avoid.

## Reproducibility is retired

Under the layer model, a skill's identity was a content hash over its directory,
and two machines on the same chartr build resolved identical bytes for the same
ticket. **That guarantee is gone and nothing restores it.** A `dir` source is a
folder the operator can edit between two spawns and that records no version, so
"this exact prompt ran" is no longer verifiable off a teammate's machine.

What replaces it is weaker, and is stated as weaker rather than dressed up:

- The claim trailer records `Skill: <name>=<source>` and appends `@<commit>` where
  the source carries a pin. A source name plus a commit is *fetchable* — a teammate
  reading the history can get the exact bytes — where a content hash could only ever
  have told them that something differed. A `dir` source has no pin and the trailer
  honestly stops at the source's name.
- `Payload-SHA256` is untouched and still fixes the **exact bytes** of the document
  a session was told, for the machine that composed them.

So the audit trail answers "what ran here" completely and "what would run there"
only where the operator chose a pinned source. That asymmetry is the price of the
list being the operator's.

## Consequences

- **Given up: the whole shadowing story as a *fork* story.** There is no
  `forked_from:`, no staleness flag, no "your copy is behind the shipped default"
  warning, and no auto-merge question to answer, because forking is now "register
  your own source above the default". An operator who has never fetched does get
  changed role prompts at upgrade without being asked; that is correct, and matches
  the conventions file's stance — upgrading chartr is the act of taking chartr's
  bytes.
- **Given up: the committed workspace layer.** `<space>/.chartr/skills/` goes inert
  and is left exactly where it is, unread and unremarked. With it goes the one thing
  global sources cannot express — "this project's skills". Whether that need is real
  is deliberately unresolved.
- **chartr stops writing into the operator's repository**, but for `.plan/maps/` and
  the gitignored per-session payload copy. The tracker-adapter offer, which existed
  to reach an agent chartr did not launch, is deleted rather than shrunk.
- **Preferences can defeat the format contract.** `preferences.md` is appended
  verbatim, unranked, and permitted to contradict everything above it, so a
  contradictory preference can make an agent write a map chartr cannot read. The
  operator chose maximum control; this is the accepted consequence.
- **The free payload cannot compel the read.** An agent that never opens the
  conventions file writes whatever its own defaults say, and by construction no
  sentence in the payload would stop it — every sentence chartr writes in its own
  voice must still be true if the agent does nothing about it. Consequence-framing
  is the whole mitigation.
- **The extension point for skills is a directory of markdown.** There is no plugin
  or extension API, and a second mechanism would be speculative bloat.

## Considered options

- **Keep the layers and add a fourth for external repos** — rejected: it preserves
  the closed name set and the fork-in-place workflow that motivated the change,
  while making precedence something inferred from a fixed ladder rather than read
  off a list the operator ordered.
- **Ship the skills but let a source override them** — rejected: shipped skills are
  chartr-specific by construction (they name `.plan/`, `## Answer`, chartr itself),
  so the library could not open up to a generic skill without the format contract
  leaving the skills. Extracting it into `conventions.md` is what made shipping none
  possible.
- **Bare-name role bindings resolved through source order** — rejected: reordering
  sources, or a higher source later shipping its own `grill`, would silently change
  what every grilling ticket runs with no line in any file showing it. That is the
  implicit capture the table exists to prevent.
- **A trust confirm before a clone, and a changed-skills summary after a refresh** —
  rejected in favour of the fewest steps, with the cost recorded above rather than
  smoothed over.
- **Validating a source's skills** — rejected: chartr would be rejecting someone
  else's markdown for not being written the way chartr writes.
