---
type: grilling
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
