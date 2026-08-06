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
