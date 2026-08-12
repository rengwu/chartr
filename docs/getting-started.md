# Getting started

This guide shows how to configure chartr, create a map, and start a ticket
session.

Before you begin, install:

- chartr for your platform. See [Installation](../README.md#installation).
- At least one agent CLI, such as Claude Code, Codex, or OpenCode. Confirm that
  the CLI runs from your shell before you add it to chartr.

## 1. Launch chartr

Open the desktop app, or start the CLI:

```sh
chartr
```

The CLI serves chartr at <http://127.0.0.1:8787>.

chartr does not provide authentication. Keep the server bound to the default
loopback address unless you intend to make it accessible over a network. See
[CLI and source builds](cli-and-source-builds.md) for command-line options.

## 2. Register an agent

Register an installed agent CLI before using it with chartr.

1. Open **Settings**.
2. Under **Agents**, select **New Agent**.
3. Enter a name for the agent and the CLI executable in **adapter**.
4. Add any required arguments, environment variables, or prompt-delivery
   settings.
5. Select **Save**.

See [Agent registration templates](agent-registration-templates.md) for tested examples.

## 3. Add a space

A space is a project folder that contains your terminal sessions and maps.

On the first-run screen, select **Choose a folder**. If a folder picker is not
available, enter the absolute path to the project and select **Register**.

chartr creates a `CHARTR.md` file in the space to provide local context and
resources for agents.

Ask an agent to read `CHARTR.md` when it needs information about chartr. The file
lists the available skills and points to the conventions for writing maps and
tickets that chartr can read.

## 4. Configure skills and roles

Ticket sessions use skills assigned to four roles: grill, prototype, research,
and implement.

A fresh installation normally registers the `chartr-skills` repository as a
skill source. Open **Settings** and check the following:

1. Under **Skill sources**, confirm that an enabled source contains the skills
   you want to use. Add a source if the list is empty.
2. Under **Role bindings**, select **no preference** for each role, or assign a
   specific skill.

Free sessions do not require role bindings. Ticket sessions require a binding
for the selected role.

## 5. Create a map

1. Select a space.
2. Open the menu next to **New Shell** and select a registered agent. This starts
   a free session in the space.
3. Ask the agent to use `wayfinder` to plan the work.
4. Review the plan with the agent.
5. Ask the agent to run `to-tickets` when the plan is ready.

The agent writes the map to `.plan/maps/<slug>/`. chartr watches this directory
and displays valid maps automatically.

## 6. Work a ticket

1. Select an unblocked ticket on the map.
2. Select a role and an agent.
3. Start the session.

The session receives the map, the selected ticket, and the resolved answers from
its blockers. Work with the agent in its terminal tab.

A ticket is complete when its file contains a non-empty `## Answer` or
`## Ruled out` section. chartr updates the map and makes newly unblocked tickets
available.

## Related documentation

- [CLI and source builds](cli-and-source-builds.md)
- [Security](../SECURITY.md)
