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
before continuing. Anything else on `PATH` works too — register it from the
sidebar's skill launcher via **Register an agent…**.

## 3. Launch

```
chartr
```

The cockpit is at <http://127.0.0.1:8787>. Two flags matter:

```
chartr -addr :9000       # serve somewhere else
chartr -data-dir ~/work  # session/runtime root (default: cwd)
```

Your config lives at `~/.config/chartr` (or `$XDG_CONFIG_HOME/chartr`) — the
registry of spaces, your agent library, terminal theme. That path is **global**,
not per-`-data-dir`, so `-data-dir` moves runtime state but every invocation
shares one set of registered spaces.

## 4. Register your first space

The cockpit opens on **Register your first space**. Paste an absolute project
folder path and hit **Register**.

> If the folder isn't a git repository, chartr initializes one there. It says so
> in the confirmation. Point it at a real project if you don't want that.

You now have a plain multiplexer: the space in the sidebar, shells and agent
CLIs in tabs. That is a perfectly good place to stop — everything below is the
map half.

## 5. Chart a map

From the sidebar's skill launcher, run **`wayfinder`** and describe what you want
built in the context box. The planning agent interviews you, then writes a map
to `.plan/maps/<slug>/`.

Whatever it writes draws as a **star-map the moment it hits disk** — you don't
reload or import anything.

When the plan is settled, run **`to-tickets`** to graduate it into numbered
ticket files. Those tickets, and their blocker edges, are what the star-map
draws.

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

## Where things live

| Thing | Path |
| --- | --- |
| Registered spaces, agent library, terminal theme | `~/.config/chartr/` |
| Your skill forks | `~/.config/chartr/skills/` |
| Maps and tickets | `<your repo>/.plan/maps/` |

Press `,` or the ⚙ in the sidebar header to see what the config layers actually
resolve to, and to open any of these files in `$EDITOR`.

## Next

- [Design system](design-system.md) — if you're touching the UI
- [ADRs](adr/) — why the thing is shaped the way it is
- [`skills/README.md`](../skills/README.md) — the skill layer and how to fork one
