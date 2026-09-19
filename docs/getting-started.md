# Getting started

Build and launch chartr using the [installation guide](installation.md), then
set up your workspace:

1. **Open a space.** Add a project folder, or use Free sessions for a shell
   outside a project. The `+` button opens a terminal.
2. **Register an agent.** Open the **Agent** settings gear in **Settings → Plugins** and
   add an installed CLI agent and its launch settings.
3. **Register your skills.** Open the **Skill sources** settings gear in **Settings → Plugins**
   and add local folders or Git repositories containing your skills.
4. **Chart your work.** Open the Agent surface using the **New surface** button
   beside `+`. Work with your agent to write a plan under `.plan/maps/`, following
   the [tracker convention](../plugins/wayfinder/TRACKER-CONVENTION.md).
5. **Drive the map.** Open Wayfinder, choose a map and a ready ticket, then use
   **Review & launch** to inspect the prompt and start its agent session.

With the Agent and Skill sources plugins enabled, Wayfinder can browse existing
maps before their registries are configured. It allows one claimed ticket per
space at a time; ordinary agent sessions and terminals remain independent. If a
launched session ends without completing its ticket, use **Release claim…** in
the ticket pane before retrying.

Open the bundled Browser from the workspace’s **New surface** menu.
See [Browser](../plugins/browser/README.md) for its capabilities and limits.

See the [workspace reference](workspace.md) for views, panes, settings, and
[data locations](workspace.md#your-data).
