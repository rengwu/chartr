# Getting started

From a fresh machine to your first star-map. About five minutes, most of it
waiting on an agent to think.

chartr ships **no agent CLIs**. You bring your own — the one hard prerequisite.

## 1. Install

**macOS (Apple silicon)** — download
[`chartr_darwin_arm64.dmg`](https://github.com/rengwu/chartr/releases/latest/download/chartr_darwin_arm64.dmg)
and drag it to Applications. The `.dmg` is unsigned, so macOS blocks the first
launch:

1. Open chartr, click **Done** (_not_ **Move to Trash**).
2. **System Settings → Privacy & Security → Security → Open Anyway**.

Or skip the dance entirely:

```
xattr -d com.apple.quarantine /Applications/chartr.app
```

**Everywhere else** — grab your platform's archive from the
[releases page](https://github.com/rengwu/chartr/releases), unpack it, and put
`chartr` on your `PATH`. Linux and Windows get the binary only; there is no
packaged app yet. On Windows, **WSL2 is the sure path**.

**From source** — Go 1.26+ and Node 22+, then `make build` → `bin/chartr`.

## 2. Install at least one agent CLI

chartr drives whatever is on your `PATH`. These six are detected for you with no
configuration:

`claude` · `codex` · `opencode` · `kimi` · `grok` · `pi`

Install one the normal way for that tool and confirm it runs in your own shell
before continuing. Anything else on `PATH` works too — press `,` for settings and
register it under **Agents**.

## 3. Launch

```
chartr
```

The cockpit is at <http://127.0.0.1:8787>. Two flags matter:

```
chartr -addr :9000       # serve somewhere else — not loopback, see below
chartr -data-dir ~/work  # session/runtime root (default: cwd)
```

chartr has **no authentication**: anyone who can reach the port can open shells,
run commands and spawn agents as you. `-addr :9000` (or any non-loopback
address) gives that to everyone who can reach your machine on that port, so keep
the default `127.0.0.1` unless you mean to expose it. chartr warns at startup
when the address it bound is not loopback.

Your config lives at `~/.config/chartr` (or `$XDG_CONFIG_HOME/chartr`) — the
registry of spaces, your agent library, your skill sources, terminal theme. That
path is **global**, not per-`-data-dir`, so `-data-dir` moves runtime state but
every invocation shares one set of registered spaces.

## 4. Register your first space

The cockpit opens on **Register your first space**. Paste an absolute project
folder path and hit **Register**.

> If the folder isn't a git repository, chartr initializes one there. It says so
> in the confirmation. Point it at a real project if you don't want that.

You now have a plain multiplexer: the space in the sidebar, shells and agent
CLIs in tabs. That is a perfectly good place to stop — everything below is the
map half.

## 5. Chart a map

Open the space card's **new shell** menu and pick an agent. That starts a **free
session**: an agent in a plain shell, told what chartr is, the file format it
should write, your own preferences, and what skills your sources hold — and
nothing about how to behave. Ask it to run **`wayfinder`** and describe what you
want built. It interviews you, then writes a map to `.plan/maps/<slug>/`.

Whatever it writes draws as a **star-map the moment it hits disk** — you don't
reload or import anything.

When the plan is settled, ask it to run **`to-tickets`** to graduate the map into
numbered ticket files. Those tickets, and their blocker edges, are what the
star-map draws.

The button's body — rather than its caret — opens a plain shell with nothing
injected at all, for when you just want a terminal.

## 6. Spawn off the frontier

The **frontier** is every ticket whose blockers are all answered — the work you
could actually start right now.

1. Click an unblocked star.
2. Pick a role and an agent.
3. The session opens in that agent's own TUI with the map, the ticket, and its
   blockers' answers **already submitted** into the buffer.

You don't paste context. That's the whole point.

## 7. Finish a ticket

A ticket resolves when its **`## Answer`** section appears in the file. The
agent writes it; the star turns; the tickets it was blocking join the frontier.

chartr's only writes to your repo are the **claim** and **release** commits on
the ticket file. The map on disk is the state — delete chartr tomorrow and your
plan is still sitting in markdown.

## 8. Bring your own skills

chartr ships **no skills**. What a session is told to do comes from a **source**:
a folder on your machine, or a git repository chartr clones and pins. You
register them in settings under **Skill sources**, and the list's order *is*
resolution order — the first enabled source holding a name wins, and the loser
stays reachable as `Source/skill`.

One source is always there: `chartr-skills`, chartr's own set, seated last so
anything you register outranks it. It ships inside the binary so a first run
works offline; refresh it once and it becomes an ordinary pinned checkout.

Four roles — grill, prototype, research, implement — are each **bound** to one
source-qualified skill, seeded on your first run. Rebind one by editing
`user.toml`; if you delete a row that role refuses to spawn until you put it
back, and **Restore default** in the same section is how.

Two cautions the settings screen repeats:

- A **git** source is chartr's checkout, not a workspace. A refresh discards
  anything you edited inside it. If you want to edit skills, fork them into a
  folder and register that as a `dir` source.
- Those checkouts live under `sources/`. If you lose `sources.toml` they are
  orphaned — chartr does not collect them, and deleting them is yours to do.

**Free session payload** in the same section shows exactly what a free session is
told, `preferences.md` included. It is the one place you can watch your own
standing instructions land in an assembled document.

## Where things live

| Thing | Path |
| --- | --- |
| Registered spaces, agent library, terminal theme | `~/.config/chartr/` |
| Your skill sources, in resolution order | `~/.config/chartr/sources.toml` |
| Git checkouts chartr owns, and its own seeded set | `~/.config/chartr/sources/` |
| chartr's file-format contract (generated — read it, don't edit it) | `~/.config/chartr/conventions.md` |
| Your standing instructions, appended to every payload | `~/.config/chartr/preferences.md` |
| Maps and tickets | `<your repo>/.plan/maps/` |

Press `,` or the ⚙ in the sidebar header to reach all of it, and to open any of
these files in `$EDITOR`.

## Next

- [Design system](design-system.md) — if you're touching the UI
- [ADRs](adr/) — why the thing is shaped the way it is; start with
  [0017](adr/0017-skills-come-from-registered-sources.md) for where skills come from
