---
type: grilling
blocked_by: [01, 03, 05]
claimed_by: s2d0aa507bc6e
claimed_at: 2026-08-06T08:02:14Z
---

# Migrating off the layer model

## Question

An operator upgrading into this has three directories chartr created and one it wrote into their repos: `<configDir>/builtin-skills/` (materialized shipped skills, possibly edited), `<configDir>/skills/` (their own forks), `<space>/.chartr/skills/` (committed workspace skills), and `docs/agents/issue-tracker.md` in every space that accepted the tracker adapter. All four are being deleted as concepts. This ticket decides what happens to the bytes.

The registry's own precedent is instructive and may or may not apply: losing `spaces.toml` is explicitly not an error, because re-registering folders costs nothing and no work is lost. Skills are not obviously in that category — an operator who forked `implement` and edited it for six months has work in `<configDir>/skills/`.

Settle:

- **`<configDir>/skills/`.** Auto-register it as a source on upgrade, leave it on disk untouched and unregistered, or prompt. Auto-registering preserves the operator's edits with zero action and is the only option where a fork keeps working; it also silently seeds a source above the default, which is a behavioural change arriving by upgrade. Note that with an explicit `[roles]` table seeded against the default source, an auto-registered fork of `implement` would *not* take effect — which is either the safe outcome or a confusing one.
- **`<configDir>/builtin-skills/`.** These are chartr's own shipped skills, possibly edited in place — the old model made editing them the supported way to customise. Deleted, left as an orphan directory, or preserved as a source. Whichever it is, an operator who customised a shipped prompt must be able to find their text afterwards.
- **`<space>/.chartr/skills/`.** Committed, so it is in the operator's git history and deleting it is a commit in *their* repo, which chartr must not make. Presumably chartr simply stops reading it and says so. Confirm, and decide whether anything tells the operator that a directory in their repo is now inert.
- **`docs/agents/issue-tracker.md`.** Same problem, same repo, same rule — chartr does not commit to the operator's repository. Does chartr stop writing it and leave existing ones, offer a removal, or is the file harmless enough to ignore forever. Also: `TrackerDismissed` in the space registry and the `TrackerAdapterBanner` component both exist only to serve it.
- **What the upgrade actually does on first run.** Seed `[roles]`, materialize `chartr-skills` from the seed, materialize the ruleset, and possibly register a legacy source — several writes, all on a first run after upgrade. Is there one migration step that reports what it did, or does each piece quietly do its own thing? The registry's stance is that a `git init` is announced and never silent; the same standard plausibly applies here.
- **Downgrade.** An operator who upgrades and goes back. What breaks, and is that acceptable to state rather than solve.

## Done when

Each of the four directories has a stated fate, the first-run upgrade sequence is written down as an ordered list of writes with what is reported to the operator, and anything left inert in the operator's repo is either cleaned up by their own hand or explicitly declared harmless.

## Answer

**The line that organizes all four fates is who owns the bytes.** `<configDir>/skills/` and `<configDir>/builtin-skills/` sit under chartr's own config root — chartr's turf, the same turf `sources.toml` itself lives on — so chartr may register, rewrite, or delete inside it without breaking the ground rule. `<space>/.chartr/skills/` and `docs/agents/issue-tracker.md` sit inside the operator's repo, in their git history, so the ground rule ("chartr writes nothing into the operator's repo but `.plan/maps/`," reaffirmed by this map) means chartr may only *stop touching* them and *say what it did*, never delete or rewrite. That split answers all four before the details do: the config-root pair gets an active migration; the in-repo pair gets a stated-and-left-alone fate.

### `<configDir>/skills/` — auto-register, conditionally

**Auto-register it as an ordinary `dir` source, named `Legacy skills`, only if the directory exists and a bounded discovery walk (ticket 01's own depth-1–3 `SKILL.md` test) finds at least one skill in it. An empty or absent directory contributes no row.**

The ticket's own worry — role bindings are always qualified (ticket 03), so an auto-registered fork of `implement` stops taking effect for `task` tickets — is real, but it is bounded to exactly that: **free-session bare-name lookups only.** No ticket session can be silently recaptured, because ticket 03 already closed that door for every source, migrated or not. That converts "auto-register or prompt" into a much smaller question than the ticket poses it as: the risk is a fork going quiet for the one workflow (role-typed tickets) it was never going to keep serving anyway under this map's model, not a fork silently taking over anything. Auto-registering is the only option where the fork keeps existing anywhere chartr can find it with zero operator action, and it costs nothing to undo (removing a `dir` source is free — the bytes are untouched on disk either way).

The confusing half is not registering it — it's registering it *silently*. That is handled in "What the upgrade actually does," below: the migration names the fork, names the source it became, and says in the same sentence that it no longer drives any role until rebound in Settings. Silence would make this the confusing outcome the ticket worried about; one sentence in a report makes it the safe one.

### `<configDir>/builtin-skills/` — deleted if untouched, preserved as a source if not

**Compare it against the shipped copy chartr is retiring, one last time, using the comparison this effort is otherwise deleting. Byte-identical (or empty/absent): delete it — chartr wrote it, chartr owns it, and an untouched copy is disk litter with a stale README describing a layer model that no longer exists. Diverges anywhere: leave the directory exactly where it is and add a `dir` source row for it, named `Migrated built-in skills`, so the edit is still findable.**

This is not new machinery kept alive past its purpose — it is a **one-time migration check**, run once against the file the shipped-skills embed is about to lose, never a permanently maintained feature. `hashFiles`/`ShippedHash` die with this ticket's work either way; the migration just gets to use them once before they go, which is cheaper than reinventing a bytes-differ check from scratch for a single call site.

Deleting the untouched case is safe under downgrade for a reason worth stating plainly: `Materialize` (`internal/prompt/prompt.go:415`) **never overwrites an existing file** — "existing files are never overwritten... an operator's edits are the point." An old binary reached by downgrade recreates a deleted-but-untouched `builtin-skills/` from its own embed on the next startup, exactly as if it had never been removed. Leaving an *edited* copy in place downgrades even more cleanly: the old binary's `RootsFor`/`Resolve` still point straight at that path and read the operator's edits back untouched, because migration never moved or renamed the directory — it only added a pointer to it that the old binary doesn't know how to read and therefore ignores.

**Registering an edited copy is placed before the migrated `<configDir>/skills/` row would sit, and both are placed before the default `chartr-skills` row** — old resolution order was workspace › user › built-in (`prompt.go:323-325`); with the workspace layer gone (next section), the surviving relative order between what used to be "user" and what used to be "built-in" carries forward as the order between the two migrated rows, and both still beat the shipped default they're standing in front of.

### `<space>/.chartr/skills/` — chartr stops reading it, and says so live, never stored

**Confirmed: chartr simply stops resolving it — `Workspace` drops out of `Roots` entirely, this ticket does not touch the directory.** Something does tell the operator, and it is not new stored state: a space whose repo still has a non-empty `.chartr/skills/` gets a line in its existing `Warnings` (`internal/model/model.go` — the same field that already carries the stale-fork and unresolvable-role notices) saying the directory is no longer read and any fork inside it is inert. This is computed live off the filesystem on every rebuild, exactly the way `prompt.LibraryWarnings` already checks for staleness today — **no dismiss flag, no persisted acknowledgement.** The warning clears itself the moment the operator removes the directory in their own commit, which is the only correct action here and the one chartr cannot take for them. A `TrackerDismissed`-style stored dismissal would be the wrong shape: that field exists because the tracker offer is an action chartr can *complete* on the operator's behalf and the dismissal silences an offer with nothing left to act on; this warning has no action to silence, only a live fact to stop stating once the fact stops being true.

### `docs/agents/issue-tracker.md` — chartr stops writing and offering it; existing files are declared harmless and left alone

**Chartr stops writing new ones, stops refreshing existing ones, and deletes the whole offer surface: `handleInstallTrackerAdapter`, `handleDismissTrackerAdapter`, `tracker.Classify`/`tracker.Install` call sites in `spaces.go`, `TrackerAdapterOffer` (Go and TS), `TrackerDismissed` and `SetTrackerDismissed` in the registry, and `TrackerAdapterBanner.svelte` plus its wiring in `MapCard.svelte`. The `internal/tracker` package and the embedded `assets/issue-tracker.md` template go with it.** This is not a smaller version of the workspace-skills fate — it is a full deletion of the *offering* machinery, because the offer's entire reason to exist is reaching an agent chartr did not launch (a vanilla wayfinder-family skill in a terminal chartr didn't spawn), and this map's own Out of scope already refused to build anything for that case going forward. Keeping the offer alive while refusing its purpose everywhere else in this effort would be incoherent.

**Existing files at that path are declared harmless and left alone — no warning, no removal offer, unlike `.chartr/skills/`.** The asymmetry is real and worth stating: `.chartr/skills/` going inert is a *behavior regression* an operator can be surprised by (a fork they rely on silently stops applying). `docs/agents/issue-tracker.md` going inert is not — its content (redirect map reads to `.plan/maps/` in chartr's format) stays true, because this map does not move or reshape `.plan/maps/`; the file simply stops being the mechanism that keeps that content current. A stale-but-still-correct file that chartr no longer refreshes needs no notice, because nothing about it becomes wrong. If a future map ever *does* change the on-disk map format, that file becomes actively misleading and the "harmless enough to ignore forever" call is the thing to revisit — not this ticket's problem, this ticket's problem is only whether it's true today, and it is.

### What the upgrade actually does on first run

**One trigger, ticket 01's own: the absence of `sources.toml`.** That file didn't exist under the old layer model, so its absence is indistinguishable between "brand-new install" and "upgrading from three layers" — and that's the right property, because it means migration is not a separate flag or a version check, it's the same first-run path a from-scratch install already takes, just with something to actually migrate.

1. **Scan.** Walk `<configDir>/skills/` and `<configDir>/builtin-skills/` (bounded, uncached, ticket 01's discovery rule) to decide what this migration contributes, and diff `builtin-skills/` against the shipped copy being retired.
2. **Delete `<configDir>/builtin-skills/` if it's empty, absent, or byte-identical to shipped.** Chartr's own directory, chartr's own cleanup.
3. **Write `sources.toml`** — the file that did not exist, exactly as ticket 01 anticipated handing this ticket ("migration writes a file that did not exist rather than editing one"). `default_enabled = true`, plus a `Legacy skills` row if step 1 found anything in `<configDir>/skills/`, plus a `Migrated built-in skills` row if step 2 didn't delete `builtin-skills/`, migrated-user row before migrated-builtin row.
4. **Reconcile `<configDir>/sources/chartr-skills`** from the embedded seed (ticket 05) — the default row's toggle now fixed by step 3.
5. **Seed `[roles]`** in `user.toml` if the table is absent (ticket 03) — runs after the default source exists to point into.
6. **Materialize `conventions.md`** (ticket 04) — order-independent, runs every startup regardless of the trigger above.
7. **Report once, in the existing warnings channel, never a new surface.** Only steps 2 and 3 produce anything worth a sentence — a from-scratch install with neither legacy directory present says nothing at all, matching the registry's own "a `git init` is announced, there's nothing to announce if there was nothing to do" posture. When there is something: name the directory, name the source it became (or that it was removed as an untouched copy), and — for the `Legacy skills` row specifically — say plainly that it no longer drives any role binding until rebound in Settings. Steps 4–6 stay quiet, exactly as their own tickets already decided (a reconciled default and a seeded `[roles]` table are deterministic, reversible-by-deleting infrastructure with no data at stake); step 3's legacy rows are the one place real disposal or a real behavior change happens, and that's what earns the sentence.

### Downgrade — stated, not solved

An operator who upgrades and goes back loses nothing that matters, because nothing this ticket does moves or rewrites bytes the old binary reads:

- **`sources.toml`, `[roles]`, `default_commit`/`default_fetched`** are files and tables the old binary has never heard of. BurntSushi/toml (`internal/config`'s decoder throughout) ignores unknown tables by default, so a downgraded `user.toml` with a `[roles]` table the old binary doesn't parse is not a parse error — it's inert, exactly like the directories above.
- **A deleted, untouched `builtin-skills/`** is recreated verbatim by the old binary's own `Materialize` on its next startup, per the never-overwrite guarantee already quoted above.
- **A preserved, edited `builtin-skills/` or a migrated `<configDir>/skills/`** is read by the old binary exactly as before — migration never moved either directory, only pointed a new file at them that the old binary can't see.
- **What actually breaks:** any source registered *after* upgrading (a real git source added through the new UI, say) is invisible to the old binary and its role bindings, if the operator rebound any, revert to whatever `RootsFor`'s three-layer resolution finds instead — silently, from the old binary's perspective, because it has no concept of a binding to warn about. That is the one real loss, and it is acceptable to state rather than solve: it is symmetric with every other "config format moved on" downgrade chartr already doesn't guard against elsewhere, and guarding it would mean the new binary writing old-format layer files nobody asked for, just in case a downgrade happens.

### Rejected

- **Prompting the operator before auto-registering `<configDir>/skills/`.** A confirm dialog restating "we found a directory, register it?" is the same dialog ticket 01 already refused before a git clone — an extra step for a reversible, zero-cost action (removing a `dir` source touches nothing on disk).
- **Leaving `<configDir>/skills/` on disk untouched and unregistered.** The only option that guarantees a six-month fork stops being reachable *anywhere*, not just as a role binding — strictly worse than auto-registering plus a report, for no gain the ticket named.
- **Preserving `<configDir>/builtin-skills/` as a source unconditionally, edited or not.** Most operators never touch it; registering an untouched duplicate of the seven names `chartr-skills` already ships is pure clutter with no fork to protect, and it's exactly the "two rows shipping the same seven names" shadowing papercut ticket 05 already flagged as a real cost when arguing against a different design.
- **A stored dismissal for the `.chartr/skills/` warning.** Rejected in favor of a live filesystem check for the reason given above: the warning has no action to silence, only a fact to stop stating once it stops being true, and a stored flag would be a second thing that could drift from the directory's actual state.
- **Offering an in-app removal action for `docs/agents/issue-tracker.md` or `.chartr/skills/`.** Both are commits in the operator's own repo; chartr does not make commits there, full stop, and "chartr deletes a file so you can review the diff" is still chartr writing to a repo it does not own.
- **Silently overwriting `[roles]`, reconciling the default source, or materializing `conventions.md` with a per-piece report on every startup.** Rejected because their own blockers already settled them as quiet, deterministic, self-healing infrastructure (delete-to-reset covers repair); adding a report to routine no-op startups would be noise on every single launch for the common case where migration did nothing.

### Revisit trigger

**If `.plan/maps/` or the tracker-convention format itself ever changes shape,** `docs/agents/issue-tracker.md`'s "harmless because still true" call is what breaks, and existing files become actively misleading rather than merely stale — that's the moment to reopen "declared harmless" and decide whether chartr owes those repos an active notice. **If an operator ever reports losing a fork they didn't know was auto-registered under `Legacy skills`,** the one-time migration report is the piece to strengthen first — a persistent settings-surface banner rather than a report that scrolls away, not a return to prompting.
