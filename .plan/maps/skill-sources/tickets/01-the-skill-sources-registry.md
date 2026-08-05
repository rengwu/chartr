---
type: grilling
claimed_by: sce0c1186772b
claimed_at: 2026-08-05T10:45:34Z
---

# The skill sources registry

## Question

The layer model is dead and a flat ordered list replaces it. This ticket designs that list.

`internal/registry` already solves a strikingly similar problem for spaces — a TOML file under the data dir, entries keyed by a hash of their path, a dense operator-owned `order` rewritten wholesale on every save, atomic writes at `0600`, and a deliberate stance that losing the file costs re-registration and never work. How much of that is precedent to copy versus a different problem wearing the same clothes is the first thing to settle.

Settle:

- **The row.** What identifies a source — a hash of its path like a space, or the operator's chosen name (`"Matt Pocock Skills"`)? Names are what a free session addresses skills *through*, so they are load-bearing in a way a space's id never was. Are they unique, and what happens on a collision? What are the kinds — is `dir` versus `git` a kind field or two different tables? Where does `enabled` sit, and what exactly does the pinned default row look like on disk given it is not removable and not reorderable?
- **Discovery.** A source is a directory scanned for `*/SKILL.md`. How deep — top level only, or nested (Pocock's repo is flat, others may not be)? What makes a directory *not* a skill? Is discovery eager at load, cached, or re-walked per resolve, and what does a source whose path has vanished read as — an error, an empty source, or a row flagged in the surface?
- **Resolution.** `resolve(name)` walks enabled sources in order and takes the first hit. State it precisely: is a disabled source skipped entirely, is the pinned default always last, and can resolution be addressed *through* a source (`"Matt Pocock Skills"/prototype`) as well as bare? The latter is what a free session needs when two sources ship the same name, and it decides whether a qualified form exists in the model at all.
- **Git sources.** Clone depth, where the cache lives (chartr's config root, or a cache dir the OS may reap), what is recorded at registration beyond the pin, and what refresh reports. What happens when `git` is absent from PATH, when a clone fails partway, or when the repo turns out to contain no skills at all — that last one is the honest signal that someone pasted the wrong URL.
- **The trust moment.** Registering a git source imports text that will be injected into agents the operator runs with permissions skipped. What is shown before the clone, and is there anything to show *after* it that is not a full diff review (which was weighed and deferred).

## Done when

The registry's file shape, identity rule, kind set, discovery walk, resolution order and git lifecycle are all written down precisely enough that an implementation ticket can be cut without a second design conversation — including the named failure modes (vanished path, absent `git`, failed clone, empty source, duplicate skill name, duplicate source name) and what each one reads as in the model.

## Answer

**The registry is a new chartr-owned `sources.toml` under the config root: an array of `[[source]]` tables whose position in the file *is* resolution order, each row identified by the operator's own name, with the pinned `chartr-skills` row synthetic — never written as a row, only its toggle persisted — and always last.** Discovery is a bounded, uncached walk for `SKILL.md`. Resolution takes two forms: a bare name searching enabled sources top-down, and a source-qualified `Source/skill` that addresses one source exactly and never falls through. Git sources are shallow clones under the config root keyed by a hash of their URL, registered with no confirm gate and refreshed as an explicit, quiet fetch-and-apply.

It borrows less from `internal/registry` than the ticket's framing expects. The two files look alike and are not the same problem: `spaces.toml` stores an `order` int only because it serialises its rows sorted by path, and it needs a hashed id only because per-space state is keyed elsewhere by it. Neither pressure exists here, so both mechanisms — the dense order, the densify-on-save, the duplicate/missing-order degradation, the twelve-hex id — are dropped rather than copied. What *is* copied is the stance: atomic temp-then-rename writes at `0600`, a missing file as the first-run state rather than an error, and losing the file costing re-registration and never work.

### The file

```toml
# ~/.config/chartr/sources.toml
default_enabled = true

[[source]]
name    = "Matt Pocock Skills"
kind    = "git"
url     = "https://github.com/mattpocock/skills"
ref     = "main"
commit  = "9e8b5ea1c4d7"
path    = "/Users/x/.config/chartr/sources/3f2a1b9c4d7e"
fetched = 2026-08-05T10:12:00Z

[[source]]
name    = "House skills"
kind    = "dir"
path    = "/Users/x/work/skills"
enabled = false
```

- **Position is precedence.** Reordering is moving lines, and the file reads as the resolution order it is. There is no `order` field, and therefore no densification, no duplicate-order case, and no legacy-row decode.
- **`default_enabled` is a scalar declared before the array**, because TOML cannot hold a scalar after an array of tables — the same constraint `registry.file.ScratchOrder` already documents. It is written on every save.
- **Written atomically at `0600` under a `0700` root**, temp-file-then-rename. The file holds the absolute path of every folder the operator sources skills from, which is nobody else's business on a shared machine.
- **A missing or unparseable file is the default row alone** — the first-run state, not an error, and exactly what a lost file recovers to.
- **Losing it costs re-registration, never work** — with one honest asymmetry against the space registry: git checkouts under `<configDir>/sources/` are orphaned by the loss. They are inert directories, chartr does not GC them, and the settings surface says so where it names the file.

### The row

- **Identity is the operator's `name`.** No hashed id: a source's order is its position and its git state is on its own row, so there is no per-source state keyed elsewhere for an id to key. The name is already load-bearing — it is what a free session addresses a skill *through* and what ticket 03's binding can qualify with — so identity by name is identity by the thing that already had to be stable.
- **Name rule:** trimmed, 1–64 characters, letters, digits, space, hyphen, underscore. **No `/`** — that character is reserved by the qualified form. Uniqueness is case-insensitive.
- **`kind` is declared, never inferred** from which of `path`/`url` happens to be present. Two kinds: `dir` and `git`. One table, not two, because resolution is one ordered list and two tables cannot interleave.
- **`enabled` sits on the row**, written only when `false`; absent means enabled.
- **A `dir` row carries `path`**, stored absolute and cleaned at registration (`registry.Register`'s precedent). A leading `~/` an operator hand-writes expands on read; chartr never writes one back.
- **A `git` row carries `url`, `ref`, `commit`, `path`, `fetched`** — the pin (`commit`) is what the row is refreshed *against*, `ref` is the branch or tag a refresh follows, `path` is the local checkout, and `fetched` is when it last moved.
- **The default row is not in the file.** It is synthesized at load exactly as `registry.ScratchID` is: fixed name `chartr-skills`, kind `dir`, path `<configDir>/sources/chartr-skills`, always last, not removable, not reorderable. Only `default_enabled` persists. What lands at that path, and how the seed and the pin interact, is [ticket 05](05-chartr-skills-and-how-it-ships.md)'s — this ticket fixes only the row's shape and position.

### Discovery

- **A skill is any directory at depth 1–3 below the source root that contains `SKILL.md` at its top level.** That is the entire test, and it is the test `prompt.dirFiles` already applies.
- **Never descend into a directory that has `SKILL.md`** — everything below it is that skill's supporting files.
- **Skipped while walking:** entries whose name starts with `.`, and `node_modules`. Nothing else is special-cased.
- **A skill's name is its directory's basename, not its path**, so names stay the short tokens a free session says out loud and a `[roles]` binding writes. The bound of 3 exists to admit the nested layout (`document-skills/pdf/SKILL.md`) that a top-level glob would read as an empty source.
- **Discovery is uncached — every caller walks.** The walk stats directories and reads no file, and depth is bounded, so it is a shallow stat over a handful of roots. What that buys: a skill folder created a second ago is usable in the very next spawn, with no invalidation rule, no rescan action, and no staleness to explain. It extends the property the current library already has — editing a resolved skill changes what the next launch reads — from the skill's bytes to the list itself. `Resolve` still reads `SKILL.md` at spawn time, as today.

### Resolution

```
resolve(ref):
  ref contains "/"  →  split at it: (source, skill)
                       that source, if it exists and is enabled.
                       miss = not found. never a fall-through.
  otherwise         →  each enabled source in file order, then the
                       default row if enabled. first hit wins.
```

- **A disabled source is skipped by both forms.** It is not searched and it is not reachable by qualification — *disabled* means one thing. A qualified reference into a disabled source reads as not-found **and names the source as disabled**, because that is the one failure the operator fixes in a click.
- **A qualified miss never falls through.** Naming a source and silently receiving a different source's skill is worse than an error.
- **The split is unambiguous** because neither a source name nor a skill basename may contain `/`.
- **The default row is always last**, so any registered source shadows it. That is safe rather than alarming precisely because ticket 03's `[roles]` table is explicit: shadowing changes what a free session finds by bare name, never which prompt a ticket session gets.

### The seam

```go
package sources

type Kind string // "dir" | "git"

type Source struct {
    Name    string
    Kind    Kind
    Enabled bool
    Path    string   // dir: registered path. git: the local checkout.
    URL     string   // git only
    Ref     string   // git only
    Commit  string   // git only — the pin
    Fetched time.Time

    Default bool     // the synthetic chartr-skills row
    Status  Status   // ok | unavailable | empty
}

type Registry struct{ /* path + mu + rows, as internal/registry */ }

func Load(configDir string) (*Registry, error)
func (r *Registry) List() []Source                  // file order, default row last
func (r *Registry) Skills(name string) []Skill      // walk one source now
func (r *Registry) Resolve(ref string) (Skill, error)
func (r *Registry) Register(s Source) error         // refuses duplicate names
func (r *Registry) Remove(name string) error
func (r *Registry) SetEnabled(name string, on bool) error
func (r *Registry) Reorder(names []string) error    // whole list, as registry.Reorder
func (r *Registry) Refresh(name string) (string, error) // new short sha
```

`prompt.Names()`, `prompt.Roots{}`, `prompt.RootsFor`, `Resolve(name, roots)` and the `LayerBuiltin`/`LayerUser`/`LayerWorkspace` tags are replaced by this package. The map settled that; it is named here only so the implementation ticket knows what it is deleting into.

### Git sources

- **The cache lives at `<configDir>/sources/<12 hex of sha256(url)>/`.** Under chartr's own config root, never an OS cache dir — a reaped cache would silently empty a source that a role binding resolves through. Keyed by URL hash rather than by name (`registry.spaceID`'s trick) so renaming a source is a pure metadata edit and two rows can never collide on disk.
- **Clone:** `git clone --depth 1 --single-branch [--branch <ref>] <url> <tmp>`, then the resolved `HEAD` is recorded as `commit` and the temp directory is renamed into place. **The row is written only after that rename**, so nothing half-cloned is ever a source and a failed clone leaves neither a row nor a directory — the same discipline `saveLocked` uses for the file.
- **No gate before the clone.** Pasting a URL into the registration field *is* the deliberate act; a confirm screen restating what was just typed is a dialog people click through. Afterwards the row reads its commit and its skill count, which is where the operator learns whether they pasted the right thing.
- **Refresh is explicit and quiet:** `git fetch --depth 1 origin <ref>`, hard-reset the checkout to `FETCH_HEAD`, record the new `commit` and `fetched`, and report the new short sha. Nothing ever fetches unattended.
- **The checkout is chartr's, not a workspace** — a refresh discards local edits inside it. Stated here because it will bite someone: the settings row for a git source must say it where it shows the path, and the answer to "I want to edit this" is a `dir` source.
- **A bare URL is enough.** `ref` defaults to whatever the clone resolved, and the commit it landed on becomes the pin.

### Failure modes

| case | what it reads as |
| --- | --- |
| dir path has vanished | row survives, status `unavailable`, contributes zero skills, flagged in the list. Never auto-removed — an unmounted volume is not a deregistration |
| `git` absent from PATH | registering a `git` source is refused at the gate, naming why, before a row is written. Existing checkouts keep resolving — a checkout is just a directory and discovery shells out to nothing. Only refresh fails, with the same message |
| clone fails partway | no row, no directory. Registration reports git's own error verbatim |
| source yields zero skills | row kept, status `empty`, reading `0 skills` with a remove action beside it. One rule covers a mis-pasted URL, a dir source registered before it is populated, and a source that empties out later |
| duplicate skill name across sources | not an error — it is what the order is *for*. The lower one stays reachable by qualification, and the list marks it shadowed |
| duplicate skill basename inside one source | sorted walk order wins; the loser resolves to nothing and is named on the source's row. A source is someone else's repo and chartr does not get to reject its layout |
| duplicate source name at registration | refused before anything is cloned |
| duplicate source name in a hand-edited file | first row wins, later ones dropped with a warning naming them — `ResolveAgents`' stance that one bad table must not cost the operator the rest of the list |
| unknown `kind`, or a row with neither `path` nor `url` | row dropped with a warning; the rest of the list stands |
| a `[[source]]` row named `chartr-skills` | dropped with a warning — the name belongs to the synthetic default |
| `sources.toml` missing or unparseable | the default row alone. First-run state, not an error |
| `sources.toml` lost | costs re-registration, never work. Git checkouts under `sources/` are orphaned and inert; chartr does not delete them |

### Rejected

- **Cloning the space registry wholesale** — hashed id, dense `order` int, densify-on-save, the legacy-row second decode. Rejected because the pressure that produced all of it is absent: `spaces.toml` needs a stored order only because it writes its rows sorted by path, and needs an id only because state elsewhere is keyed by it. Copying the mechanism without the pressure would import three failure modes (duplicate order, missing order, hand-edit degradation) that file position makes unrepresentable.
- **Sources as a `[sources.*]` map in `user.toml`** — duplicate names would be unrepresentable by construction, which is a real gain. Rejected because chartr rewrites the sources list on every register, toggle, reorder and refresh, and doing that inside the file whose hand-written `[agents.*]` tables the operator maintains puts a machine writer and a human writer in one file for no resolution benefit. The uniqueness that shape gave for free is enforced by the writer instead.
- **Top-level-only discovery (`<root>/*/SKILL.md`)** — one glob, no depth constant, no collision rule. Rejected because a nested repo could then only be registered by pointing at a subdirectory, which a `dir` source can do and a `git` source cannot without the row growing a `subdir` field. The bounded walk buys nested repos for the price of one collision rule.
- **Path-qualified skill names (`document-skills/pdf`)** — eliminates collisions entirely, and rejected because the long form propagates into `[roles]` and into what a free session has to say out loud. It trades a rare collision for permanent verbosity.
- **A cached index with a rescan action** — cheaper on a large source. Rejected because the invalidation rule and the "why isn't my new skill showing up" surprise both cost more than the walk does.
- **Bare-name resolution only** — one lookup form, one rule. Rejected because a shadowed skill becomes unreachable except by reordering, and ticket 03's role bindings would be order-sensitive: dragging a source up the list could change which prompt every grilling ticket gets.
- **A trust confirm before the clone, and a changed-skills summary after a refresh.** Both were argued for on the grounds that a source is executable text injected into agents run with permissions skipped, and both were declined by the operator in favour of the fewest steps. **Knowingly accepted: the only assertion of trust in a git source's entire lifetime is the moment its URL is typed.** After that a `refresh` can move the source arbitrarily far and the only visible evidence is a short sha. This is the sharpest trade-off on this ticket and it is recorded as one rather than smoothed over.

### What this hands the neighbouring tickets

- **[03](03-binding-a-ticket-type-to-a-skill.md)** gets the qualified form, and with it the option of a binding that reordering cannot change (`grilling = "chartr-skills/grill"`), plus the failure-mode table above to decide spawn behaviour against.
- **[02](02-the-two-core-payloads.md)** gets a source list that is as cheap to enumerate *with* its skill names as without, since discovery is an uncached bounded walk — so listing skills in the free-session payload is a payload-size question, not a performance one. It also gets `url` and `path` as separate fields, so the payload can name either.
- **[05](05-chartr-skills-and-how-it-ships.md)** owns everything inside `<configDir>/sources/chartr-skills`. This ticket fixes only that the default row is synthetic, sits last, is toggleable, and materializes at that path.
- **[07](07-migrating-off-the-layer-model.md)** gets: an auto-registered legacy `<configDir>/skills/` would be an ordinary `[[source]]` row of kind `dir`, and the absence of `sources.toml` is the first-run state — so migration writes a file that did not exist rather than editing one.
- **The map's "not yet specified" entry on the settings surface** now has its inputs: a row renders name, kind, enabled, position, status (`ok`/`unavailable`/`empty`), skill count, and for git the url, commit, fetched time and a refresh action. **Naming collisions in free-session lookup** is answered — the qualified form is the mechanism.

### Revisit triggers

- **The uncached walk** is revisited the first time a registered source is large enough that the walk is visible in a model rebuild. What would change is the depth bound or the caller list, not the model.
- **The no-gate registration and the quiet refresh** are revisited the first time a source changes under an operator in a way they did not expect. The summary is three lines of `git diff --name-status` folded to skill directories and can be added later without disturbing anything decided here.
- **Basename collisions inside one source** are revisited if a real registered source has them habitually; the escape is path-qualified names, and the qualified form's syntax already leaves room for them.
- **The depth bound of 3** is revisited the first time a real repo nests deeper. It is a constant, not a model change.
