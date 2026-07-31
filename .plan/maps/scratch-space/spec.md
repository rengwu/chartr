# Scratch space

## Problem Statement

Every terminal chartr can open belongs to a space, and a space is a git
repository the operator has registered. That is the right shape for the work
chartr exists to drive — a map, its tickets, the sessions against them all need
a working tree. It is the wrong shape for the other half of what a cockpit is
used for: a shell, right now, somewhere that is not a project.

Today that costs the operator a registration. Registering runs `git init` on any
folder that is not already a repository — announced, never silent, but still a
repository the operator did not ask for — and it seats a permanent row in the
sidebar that has to be explicitly forgotten afterwards. An operator who wants a
shell in their home directory, or in a downloads folder, or anywhere they are
merely poking around, has to either accept a junk repository and a junk space or
leave chartr and open a terminal somewhere else.

The mismatch is not really about git. It is that an ad-hoc shell has never
needed anything a space provides. It has no map, no ticket, no claim, no
lifecycle and no agent; it is ended by the human and by nothing else. The space
around it is supplying two things only: a working directory, and somewhere for
the tab to live.

## Solution

One built-in **Scratch** space, present from first run, that exists to hold
ad-hoc shells and nothing else.

It is not registered and cannot be. It is not a git repository and is never
made into one. Its working directory is the operator's home directory. It has no
maps, no branch, no skills, no remembered agent, and no committed config, and
every repo-scoped action refuses it — there is nothing there to spawn a session
against, and pretending otherwise would rebuild a space badly.

It is hidden until it has something in it. With no shells open it is not in the
sidebar at all, so an operator who never wants it never meets it. Opening a
scratch shell makes it appear; ending the last one makes it disappear again, and
a Scratch space that vanishes underfoot is treated exactly as a forgotten space
is — the selection falls to the neighbouring row, the same path and the same
code.

It sits in the operator's own sidebar arrangement like any other row: it can be
dragged, it can be moved with the keyboard, and where it sits is remembered. It
holds that slot even while it is hidden, so it comes back where it was left
rather than at the bottom of the list. Visibility is a question the chrome
answers at render time; the order is a fact the registry keeps either way.

It cannot be removed, so it carries no forget control. There is one new way in:
a **New Scratch Shell** control in the sidebar footer, beneath **New Space** —
one click, no folder chooser, because a shell that made you pick a folder first
is a space you wanted.

## User Stories

1. As an operator, I want to open a terminal without registering a space, so
   that a quick shell does not cost me a new row in my sidebar.
2. As an operator, I want that terminal to open without `git init` running
   anywhere, so that poking around in a folder does not leave a repository
   behind.
3. As an operator, I want a scratch shell to open in my home directory, so that
   it starts somewhere predictable and I do not have to answer a question first.
4. As an operator, I want to open a scratch shell in one click from the sidebar
   footer, so that it is as cheap as the thing it replaces — leaving chartr for
   a real terminal.
5. As an operator, I want to open more than one scratch shell, so that the
   Scratch space multiplexes the way every other space does.
6. As an operator, I want each scratch shell listed in the sidebar with its
   foreground process, so that I can tell two of them apart at a glance.
7. As an operator, I want to click a scratch shell in the sidebar to bring it up
   in the pane, so that switching between them is the one click it is everywhere
   else.
8. As an operator, I want scratch shells to look and behave exactly like the
   ad-hoc shells I already open in a space, so that there is no second kind of
   terminal to learn.
9. As an operator, I want the Scratch space to stay out of my sidebar until I
   use it, so that a feature I have not asked for does not take up a row.
10. As an operator, I want the Scratch space to appear the moment I open a
    scratch shell, so that I can see where the thing I just opened went.
11. As an operator, I want the Scratch space to disappear when I close its last
    shell, so that it does not linger as an empty row.
12. As an operator, I want the cockpit to move me to another space when Scratch
    disappears under me, so that I am not left looking at a blank stage.
13. As an operator, I want to drag the Scratch space to a position in my
    sidebar, so that it sits where I think of it rather than where chartr puts
    it.
14. As an operator, I want to move the Scratch space with the keyboard, so that
    it is not the one row in the sidebar that is mouse-only.
15. As an operator, I want the Scratch space to come back where I left it after
    it has been hidden, so that its position is a setting rather than an
    accident of when I last used it.
16. As an operator, I want that position to survive a restart, so that I arrange
    my sidebar once.
17. As an operator, I want my existing sidebar arrangement untouched by the
    arrival of this feature, so that an upgrade does not rearrange my workspace.
18. As an operator, I want no remove control on the Scratch space, so that I am
    not offered an action that cannot happen.
19. As an operator, I want no map controls on the Scratch space, so that the
    cockpit does not offer to chart a folder that has no repository.
20. As an operator, I want no branch shown on the Scratch space, so that I am
    not told something false about a folder that is not a repository.
21. As an operator, I want no skills or spawn controls on the Scratch space, so
    that I cannot start a session that has nothing to work against.
22. As an operator, I want the Scratch space kept out of the settings surface's
    list of spaces, so that I am not offered config layers that do not exist.
23. As an operator, I want a scratch shell to survive a browser reload and
    re-attach with its scrollback, so that it is as durable as any other shell.
24. As an operator, I want a second browser window to see the Scratch space and
    its shells, so that it is part of the same pushed model as everything else.
25. As an operator, I want the sidebar filter to match scratch shells by what is
    running in them, so that a filtered sidebar does not silently hide a shell I
    have open.
26. As an operator, I want a stale or hand-written client that asks the Scratch
    space to spawn, launch, or chart to be refused, so that a bug cannot write
    into my home directory.

## Implementation Decisions

**A synthetic registry entry, not a second concept.** The Scratch space is one
entry the space registry holds in memory at all times, alongside the registered
ones. It carries a fixed identity that cannot collide with a registered space's
(those are derived from the absolute path), and it is flagged so every consumer
that needs to treat it differently can ask. Because it is an ordinary entry, the
registry's listing, ordering and lookup all work on it unchanged, and the
terminal manager — which only ever used the space id as a grouping tag and the
path as a working directory — needs no change at all.

**Its working directory is the operator's home directory**, resolved at load. It
is never registered, never `git init`ed, and its path is not stored: it is
re-resolved on every run, so it follows the machine rather than the file.

**It is written to the registry file as a single order value, never as a space
row.** The file's `[[space]]` rows continue to mean "a folder the operator
registered", which is what makes a deleted registry file recoverable by
re-adding folders. Scratch's sidebar slot is the only thing about it worth
persisting, so it persists as a scalar beside the rows.

**It always holds a slot in the order, hidden or not.** The registry densifies
the whole arrangement — registered spaces and Scratch together — on every save.
Registered rows are written with the orders that densification gave them, which
means the file may carry a gap where Scratch sits; the existing load path
already compacts a gapped-but-unique arrangement without disturbing its
sequence, so Scratch is re-inserted at its recorded index and the whole list
densifies again. A file written before this feature has no recorded index and
Scratch appends at the end, which leaves the operator's existing arrangement
byte-for-byte as it was — the same invisible-upgrade rule the stored order
itself was introduced under.

**Reordering tolerates a list that omits Scratch.** Today a reorder must name
every entry exactly once; that contract is what makes the write idempotent and
unable to leave the sidebar half-moved. The chrome posts what it can see, and
when Scratch is hidden it cannot see it — so the registry now accepts a list
that omits *the Scratch entry only*, and splices it back at its stored slot.
Every registered space must still be named exactly once; omitting one of those
remains a client bug and is still refused whole. When Scratch is visible it is
named like any other row and rides along.

**The derived model always carries the Scratch space, flagged.** It is not
filtered server-side. Sending it unconditionally is what keeps the ordering
server-authoritative — the one authority the sidebar has — and puts the
visibility question where the selection it interacts with already lives. Its
derived form is deliberately thin: its open shells, and nothing else. No maps
are discovered for it, no branch or dirty state is read, no skill library or
config layers are resolved, no tracker-adapter offer is classified, and no
remembered agent is reported.

**Repo-scoped actions refuse it at the server.** Spawning, launching an on-ramp
skill, ideating, previewing a payload, the death-halt verbs, releasing a ticket,
the tracker-adapter offer, opening a per-space config layer, setting the
remembered agent, and deregistering all resolve a space before they act; each
now refuses the Scratch identity with a bad-request rather than acting on the
home directory. Opening, closing and acknowledging a terminal accept it, which
is the whole of what it is for. Refusal is a server rule, not merely an absent
button: the chrome will not offer these, but a stale client must not be able to
spawn an agent into `$HOME`.

**Visibility is one predicate in the chrome: Scratch is shown when it has at
least one open shell.** There is no "or it is currently selected" clause. The
selection already falls back to the first remaining space when the selected one
leaves the list — the path a forgotten space takes — so a Scratch space that
empties is handled by machinery that already exists and is already understood.
Because everything downstream of the sidebar reads the filtered list, the
first-run screen, the filter, the keyboard space-cycling and the reorder write
all follow without further change.

**The settings surface reads the unfiltered list minus Scratch.** It enumerates
spaces as config scopes, and Scratch has no config to scope.

**The Scratch space's card carries no branch chip, no skill launcher, no
new-shell control and no remove control** — which empties its action row
entirely, so the row is not rendered rather than rendered blank. The drag handle
and the shell rows stay: it is reorderable and its shells are selectable like
any other space's.

**Its stage carries no map controls.** The star-map toggle, the map card and the
tracker-adapter banner do not render, and the map's keyboard bindings are inert
while it is on show. The star-map's open state is a standing preference held on
the stage rather than per space, so this is enforced where the map renders, not
only where it is toggled. The stage's empty state is unreachable by
construction — a Scratch space with no shells is not on show.

**The new control is a lesser action.** It sits beneath **New Space** in the
sidebar footer with less visual weight than it, because it is the secondary of
the two and two equally-weighted full-width buttons would read as a choice
rather than a primary and an alternative.

**Nothing changes in the terminal.** The terminal socket, the scrollback replay,
the resize control frame, the OSC-derived status and the terminal customization
seam are all untouched. A scratch shell is an ad-hoc shell that happens to be
tagged with a different space id.

**Relationship to existing decisions.** ADR 0003 makes the space the unit of
serialisation because it owns one working tree; the Scratch space owns no map
and can hold no session, so there is nothing to serialise and the live-session
gate never applies to it. ADR 0009's split between execution and content config
is unaffected: Scratch resolves no committed layers, and the agent library it
would have drawn on is global and unchanged. ADR 0010's chrome/island split is
unaffected: this is entirely a chrome and model change.

## Testing Decisions

A good test here asserts what an operator or a browser can observe — what comes
back from an action, what the pushed snapshot says, what the file on disk says
after a restart. It does not assert on the shape of internal state, and it does
not reach inside the terminal manager or the chrome's components. Three seams,
all of them already in use.

**Through the API and the pushed model** — the primary seam, and the one the
existing server tests already work at (the terminal, agent, config-surface and
tracker-adapter suites are prior art). Covered there: the Scratch space is
present in every snapshot and flagged; asking it for a terminal opens a shell
whose working directory is the home directory; the repo-scoped actions each
refuse it; deregistering it is refused; a reorder that omits it succeeds and
leaves it in its slot; a reorder that omits a registered space is still refused
whole. Its derived space is asserted to be thin — no maps, no branch, no skills,
no layers, no tracker offer — which is what proves nothing is being discovered
or classified against the home directory.

**Through the registry's load and save** — the on-disk order format, which the
snapshot cannot observe. The registry's existing suite is prior art. Covered:
Scratch's slot round-trips through a save and a reload; a file whose registered
rows carry a gap where Scratch sat loads back into the same sequence; a file
written before this feature loads with the operator's arrangement unchanged and
Scratch appended at the end.

**Through one pure frontend module** — the show/hide rule, extracted into a
small function so that it is testable at all. The chrome has no component tests
and this does not introduce them; the pattern is the existing pure modules
beside it (the attention derivation, the reorder resolution, the unseen-dot
rule), each a plain module with a vitest file. Covered: a Scratch space with no
shells is filtered out, one with shells is kept, and registered spaces are never
filtered regardless of their shells.

## Out of Scope

- **Choosing a folder for a scratch shell.** The working directory is the home
  directory, full stop. A per-shell folder chooser is the registration flow with
  the git step removed, which is a different feature and probably a better
  answer to a different problem.
- **More than one scratch space.** One is a home for loose shells; several is a
  space registry without the registry.
- **Renaming it, pinning it, or configuring it.** It has no settings.
- **Persisting scratch shells across a restart.** Shells die with the process
  today; scratch shells are no different, and a restart simply leaves the
  Scratch space hidden again.
- **Running skills, agents or sessions in it.** Deliberately refused, not
  deferred: a Scratch space that could spawn is a space.
- **Making the Scratch space reachable by a deep link.** Star deep links name a
  map and a ticket, and it has neither.
- **A shell anywhere but a space or Scratch.** This does not introduce a general
  "terminal at an arbitrary path" surface.

## Further Notes

The home directory is the least-protected place a shell can start: no
repository, so no branch shown, no dirty badge, and nothing to `git checkout`
back to. That is the honest deal with a terminal and it is not a reason to gate
anything — but it is the default working directory for every scratch shell, and
worth stating plainly rather than discovering.

The sidebar filter matches a space on its name, path, branch and its shells'
processes and titles. The Scratch space has a constant name and no branch, and
its path is the home directory, so in practice it will match on what is running
inside it. That is the useful behaviour and no special case is needed for it.

The registry file's `[[space]]` rows keep meaning exactly what they have always
meant: folders the operator registered. That is what keeps "delete the registry
and re-add your folders" a complete recovery — Scratch needs no recovering,
because it is rebuilt from nothing on every run.
