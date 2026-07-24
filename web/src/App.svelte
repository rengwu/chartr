<script lang="ts">
  import { onMount } from "svelte";
  import { ControlSocket } from "./lib/control.svelte";
  import type { Space, Terminal } from "./lib/model";
  import {
    deregisterSpace,
    openTerminal,
    closeTerminal,
    resumeSession,
    respawnSession,
    releaseSession,
    launch,
    pickFolder,
    registerSpace,
    ActionError,
  } from "./lib/actions";
  import RegisterForm from "./lib/RegisterForm.svelte";
  import SpacePane from "./lib/SpacePane.svelte";
  import Settings from "./lib/Settings.svelte";
  import SkillLauncher from "./lib/SkillLauncher.svelte";
  import Modal from "./lib/Modal.svelte";
  import { Button } from "./lib/components/ui/button";
  import { Input } from "./lib/components/ui/input";
  import {
    spaceAttention,
    spaceHaltTarget,
    spaceLiveness,
  } from "./lib/attention";
  import { isEditingTarget } from "./lib/keys";
  import { nativeTitleBarHeight } from "./lib/titlebar";
  import { parseRoute, settingsHash, type SettingsScope } from "./lib/route";
  import {
    Plus,
    X,
    Check,
    XCircle,
    CircleNotch,
    Compass,
    GitBranch,
    GitDiff,
    Rocket,
    Lightbulb,
    Play,
    ArrowClockwise,
    ArrowUUpLeft,
    Gear,
    Warning,
    PauseCircle,
    GitBranchIcon,
    GitDiffIcon,
    FolderOpen,
  } from "phosphor-svelte";

  // Zero-pad a ticket number for a session row's label (#01), matching the detail
  // pane's ticket ids.
  function pad(n: number): string {
    return n < 10 ? "0" + n : String(n);
  }

  // The one control socket for this browser. The chrome renders whatever the
  // latest snapshot holds and reacts to every push (ADR 0010).
  const control = new ControlSocket();

  // The window's own title bar, when the native shell handed us its top strip
  // (macOS). Read once at startup — it is a property of the window we booted in,
  // not state — and zero everywhere else, which is what keeps this out of a
  // browser tab entirely.
  const titleBarH = nativeTitleBarHeight();

  // The cockpit's one route besides itself: the effective config surface, on a
  // `#/settings` hash prefix (ticket 05). The star deep link (`#s=…`, never a
  // leading slash) is a disjoint scheme, so the two share the bar without
  // colliding. No routing library — a parser and a `$derived`.
  let hash = $state(typeof location === "undefined" ? "" : location.hash);
  const route = $derived(parseRoute(hash));

  // Navigation is a hash assignment; the hashchange listener below catches every
  // other way the bar changes (manual edits, back/forward). The local state is
  // set here too, synchronously: hashchange is delivered a task later, and until
  // it lands `route` would still read the *old* hash. That stale window is real —
  // navigating and switching spaces in one click let the space pane's own
  // URL-reflecting effect (which stands down only while settings is up) flush
  // first and overwrite the route we just set. Setting it here closes the window;
  // the echoing hashchange then finds the same value and changes nothing.
  function navigate(next: string) {
    if (next !== location.hash) location.hash = next;
    hash = next;
  }

  function openSettings(scope?: SettingsScope) {
    navigate(
      settingsHash(
        scope ??
          (selected
            ? { kind: "space", spaceId: selected.id }
            : { kind: "default" }),
      ),
    );
  }

  // Leaving settings is Esc, the ⚙ again, or selecting a space: the surface is a
  // place you visit, never a mode you get stuck in.
  function leaveSettings() {
    if (route.settings) navigate("");
  }

  onMount(() => {
    control.connect();
    // A deep link names its space (#s=<id>&…); select it up front so the linked
    // star seats as soon as the space arrives over the socket (ticket 07). The
    // rest of the link — map and star — is applied inside the space's pane.
    const s = new URLSearchParams(location.hash.replace(/^#/, "")).get("s");
    if (s) selectedId = s;
    const onHash = () => (hash = location.hash);
    window.addEventListener("hashchange", onHash);
    return () => {
      window.removeEventListener("hashchange", onHash);
      control.close();
    };
  });

  // Spaces arrive already ordered — pinned first, then by recency — so we render
  // them in slice order and never re-sort on the client.
  const spaces = $derived<Space[]>(control.model?.spaces ?? []);
  // The config layers shared by every space — the operator's local binding file
  // and the two skill libraries that are not a space's own.
  const configLayers = $derived(control.model?.config ?? []);
  // The registered agent library. Global — the same list whatever space is in
  // view — so it is read once here and handed to the settings surface, which lists
  // and edits it on the global scope, and to every spawn picker.
  const agentLibrary = $derived(control.model?.agents ?? []);
  // The known agent CLIs found on this machine's PATH — the advisory hint the
  // registration surface renders beneath the adapter input. A machine property,
  // resolved server-side, so a fresh operator sees real suggestions.
  const detected = $derived(control.model?.detected ?? []);

  let selectedId = $state<string | null>(null);
  // The active shell, lifted here from the pane: the sidebar's session rows are
  // now what selects a terminal, so the pane just renders whichever one is active.
  let activeTermId = $state<string | null>(null);
  let filter = $state("");
  let opening = $state(false);

  // Adding a space is the operator's own OS folder chooser, raised server-side
  // (chartr always serves on loopback, so the dialog lands on their desktop
  // in the native shell and in a plain browser alike). The typed-path modal
  // survives only as the fallback for a machine with no chooser at all — a Linux
  // box with neither zenity nor kdialog — where it is the only way in.
  const nativePicker = $derived(control.model?.nativePicker ?? false);
  let showAdd = $state(false);
  let picking = $state(false);
  // The register outcome, shown inline beside the button that started it: the
  // announced `git init` (story 2) and every refusal. There is no modal left to
  // carry them.
  // Structured rather than a pre-formatted string, so the template can front-
  // truncate the path on its own (the operator needs the project name at the
  // end of it, not the drive/user segments at the front) without swallowing
  // the git-init caveat into the same truncated run.
  let addNotice = $state<{ path: string; gitInited: boolean } | null>(null);
  let addError = $state<string | null>(null);

  // addSpace is the whole add flow: name a folder in the native chooser, then
  // register it. The two are separate calls on purpose — registration keeps the
  // one action, and one response shape, it has always had.
  async function addSpace() {
    if (picking) return;
    if (!nativePicker) {
      showAdd = true;
      return;
    }
    picking = true;
    addNotice = null;
    addError = null;
    try {
      const picked = await pickFolder();
      // Dismissing the chooser is an ordinary outcome, not a failure: say nothing
      // and leave the sidebar exactly as it was.
      if (picked.cancelled || !picked.path) return;
      const res = await registerSpace(picked.path);
      addNotice = { path: picked.path, gitInited: res.gitInited };
      selectedId = res.id;
    } catch (err) {
      addError = err instanceof ActionError ? err.message : String(err);
    } finally {
      picking = false;
    }
  }
  // The effective selection falls back to the first space when the id is stale
  // (e.g. the selected space was just forgotten), so the pane never blanks while
  // spaces remain. No effect mutates state; selection is pure derivation.
  const selected = $derived.by(() => {
    return spaces.find((s) => s.id === selectedId) ?? spaces[0] ?? null;
  });

  // The shell the pane shows: the active id within the selected space, falling
  // back to that space's first shell so the pane never shows a blank island while
  // terminals remain (the same stale-id tolerance selection has).
  const activeTerm = $derived.by<Terminal | null>(() => {
    const ts = selected?.terminals ?? [];
    return ts.find((t) => t.id === activeTermId) ?? ts[0] ?? null;
  });

  // The filter is a pure view over the ordered list — it now reaches into
  // sessions too (a space shows if its own fields or any of its shells match), so
  // the sidebar scales past what a flat list carries without changing order.
  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (q === "") return spaces;
    return spaces.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.path.toLowerCase().includes(q) ||
        (s.branch ?? "").toLowerCase().includes(q) ||
        s.terminals.some(
          (t) =>
            t.proc.toLowerCase().includes(q) ||
            t.title.toLowerCase().includes(q),
        ),
    );
  });

  // Confirmations and failures are the chrome's own surfaces, never the browser's
  // `confirm()`/`alert()`. The native shell's webview implements one WKUIDelegate
  // method — the file-open panel — so a JS dialog there is a silent no-op and
  // `confirm()` returns false on the spot: the forget action simply did nothing,
  // and every failure message vanished. Both now render as Modals, which also
  // keeps them on the design system instead of an OS-drawn box.
  let pendingForget = $state<Space | null>(null);
  let actionError = $state<string | null>(null);

  function forget(space: Space) {
    pendingForget = space;
  }

  async function confirmForget() {
    const space = pendingForget;
    pendingForget = null;
    if (!space) return;
    if (selectedId === space.id) selectedId = null;
    try {
      await deregisterSpace(space.id);
    } catch (e) {
      actionError = `Couldn’t remove “${space.name}”: ${(e as Error).message}`;
    }
  }

  // Selecting a space is also how you leave the settings route — it is a place
  // you visit, not a mode.
  function selectSpace(id: string) {
    selectedId = id;
    leaveSettings();
  }

  // Selecting a session selects its space and makes that shell active, so one
  // click drives both the sidebar highlight and what the pane renders.
  function selectSession(space: Space, t: Terminal) {
    selectSpace(space.id);
    activeTermId = t.id;
  }

  async function openShell(space: Space) {
    selectSpace(space.id);
    opening = true;
    try {
      const { id } = await openTerminal(space.id);
      activeTermId = id;
    } catch (e) {
      actionError = `Couldn’t open a shell: ${(e as Error).message}`;
    } finally {
      opening = false;
    }
  }

  // The skill launcher (skill-launcher map): the space card's on-ramp control runs
  // any self-driving skill on a chosen agent as a live, ticketless tab — shares
  // only the spawn primitive with a real session, so it opens exactly like a shell
  // (no role picker, no ticket, nothing to gate on). ideate is now just the
  // `skill=ideate` case of this one launch. It names the agent that runs it
  // (ticket 03) and passes no context — the optional context box is 03's next step.
  async function launchSpace(space: Space, agent: string, skill: string) {
    selectSpace(space.id);
    opening = true;
    try {
      const { id } = await launch(space.id, agent, skill);
      activeTermId = id;
    } catch (e) {
      actionError = `Couldn’t launch ${skill}: ${(e as Error).message}`;
    } finally {
      opening = false;
    }
  }

  async function endShell(space: Space, t: Terminal) {
    if (activeTermId === t.id) activeTermId = null;
    try {
      await closeTerminal(space.id, t.id);
    } catch (e) {
      actionError = `Couldn’t end “${t.title}”: ${(e as Error).message}`;
    }
  }

  // The death halt: a dead session offers exactly three choices, and chartr
  // takes none on its own. Resume relaunches it on its own ticket (crash recovery);
  // respawn starts a fresh session on the same ticket; release clears the claim back
  // to the frontier. The resulting state arrives over the control socket.
  async function haltAction(
    space: Space,
    t: Terminal,
    verb: string,
    run: (spaceId: string, sessionId: string) => Promise<unknown>,
  ) {
    selectSpace(space.id);
    activeTermId = t.id;
    try {
      await run(space.id, t.id);
    } catch (e) {
      actionError = `Couldn’t ${verb} this session: ${(e as Error).message}`;
    }
  }

  // One click on a sidebar card's halt flag: select that space and set the
  // deep-link hash naming the halted session's map and ticket. The selected
  // space's SpacePane instance persists across space switches (ticket 07) and
  // already listens for hashchange to seat a linked star, so this reuses that
  // exact mechanism rather than reaching into the pane's own state.
  function jumpToHalt(space: Space) {
    const target = spaceHaltTarget(space);
    if (!target) return;
    selectedId = space.id;
    navigate(
      `#s=${encodeURIComponent(space.id)}&m=${encodeURIComponent(target.mapSlug)}&t=${target.ticketNum}`,
    );
  }

  // Keyboard-first navigation (story 30): space switching, alongside the map's
  // own M/Esc (SpacePane.onKey). `[`/`]` cycle spaces in the same
  // pinned-then-recency order the sidebar renders, never the filtered view — a
  // keyboard shortcut should not depend on what's typed in the filter box.
  function onGlobalKey(e: KeyboardEvent) {
    if (isEditingTarget() || e.metaKey || e.ctrlKey || e.altKey) return;
    // `,` enters the settings route (the conventional preferences key); Esc
    // leaves it, ahead of the map's own Esc, which the pane suppresses while
    // settings is up.
    if (e.key === ",") {
      e.preventDefault();
      if (route.settings) leaveSettings();
      else openSettings();
      return;
    }
    if (e.key === "Escape" && route.settings) {
      e.preventDefault();
      leaveSettings();
      return;
    }
    if ((e.key === "[" || e.key === "]") && spaces.length > 1) {
      e.preventDefault();
      const ids = spaces.map((s) => s.id);
      const i = selected ? ids.indexOf(selected.id) : -1;
      const next =
        ids[(i + (e.key === "]" ? 1 : -1) + ids.length) % ids.length];
      selectedId = next;
    }
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

<div class="flex h-full min-h-0 flex-col">
  {#if titleBarH}
    <!-- The window's title bar, ours to draw (macOS shell only). Its height is
         the strip the shell freed, so the three native window buttons — still
         AppKit's, drawn over this — sit centred at the left, and the branding
         centres in the full window width, clear of them. The whole strip drags
         the window because AppKit still owns its mouse events; nothing
         interactive belongs here for that same reason. -->
    <header
      class="flex shrink-0 select-none items-center justify-center border-b border-border bg-card"
      style="height: {titleBarH}px"
    >
      <span class="flex min-w-0 items-center gap-2">
        <span
          class="grid size-5 shrink-0 place-items-center rounded-full border border-border text-foreground"
        >
          <Compass class="size-3.5" />
        </span>
        <span class="truncate text-sm font-semibold tracking-tight">chartr</span>
      </span>
    </header>
  {/if}

  <div class="grid min-h-0 flex-1 grid-cols-[16rem_minmax(0,1fr)]">
    <aside
      class="col-start-1 row-start-1 flex min-h-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
    >
      <!-- Branding: a marked home for the cockpit, above the spaces list. Just the
           mark and the name — the cockpit-wide way into the config surface (ticket
           05) sits at the far top-right of the chrome instead, past the stage's
           Map toggle (SpacePane).

           In the macOS shell the window's title bar carries the branding instead,
           centred over the whole window; repeating it here would be the same mark
           twice within 40px. -->
      {#if !titleBarH}
        <div class="cockpit-bar gap-2 bg-transparent">
          <span class="flex min-w-0 items-center gap-2">
            <span
              class="grid size-5 shrink-0 place-items-center rounded-full border border-sidebar-border text-sidebar-foreground"
            >
              <Compass class="size-3.5" />
            </span>
            <span class="truncate text-sm font-semibold tracking-tight">chartr</span>
          </span>
        </div>
      {/if}

      {#if spaces.length > 0}
        <div class="cockpit-bar justify-between gap-2 bg-transparent">
          <Input
            type="text"
            class="h-7"
            placeholder="Filter spaces and sessions…"
            bind:value={filter}
            spellcheck="false"
            autocapitalize="off"
            autocomplete="off"
            aria-label="Filter spaces and sessions"
          />
        </div>
      {/if}

      {#if control.model === null}
        <p class="flex-1 px-3 py-2 text-xs text-muted-foreground">Connecting…</p>
      {:else if spaces.length === 0}
        <p class="flex-1 px-3 py-2 text-xs text-muted-foreground">No spaces yet.</p>
      {:else}
        <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-2">
          {#each filtered as space (space.id)}
            {@const isSelected = selected?.id === space.id}
            {@const attention = spaceAttention(space)}
            {@const liveness = spaceLiveness(space)}
            <!-- One space, a bordered container on the sidebar surface (its own
                 token family — not the bg-card content surface). The whole card is
                 the selection target — clicking anywhere that isn't its own control
                 selects the space — so the identity, its sessions and its actions
                 all read as one object rather than a header you must aim at.
                 Selected emphasis rides --primary, the one emphasis token; the
                 chrome is monochrome. -->
            <div
              role="button"
              tabindex="0"
              aria-pressed={isSelected}
              aria-label="Select {space.name}"
              title={space.path}
              class={[
                "flex cursor-pointer flex-col gap-2 rounded-lg border p-2 transition-colors",
                isSelected
                  ? "border-primary/60 bg-sidebar-accent/30"
                  : "border-sidebar-border hover:border-primary/30",
              ]}
              onclick={() => selectSpace(space.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  selectSpace(space.id);
                }
              }}
            >
              <!-- Identity: the space's name over its branch, with the forget
                   action pinned top-right. Ambient cross-space attention (ticket
                   14, story 8) rides on the name line — a wants-you flag (a
                   session halted) and a liveness dot, both echoing the same
                   signals the queue pulls and the session cards below already
                   carry in detail. Neither ever re-sorts the card; muscle memory
                   over this list holds. -->
              <div class="flex items-start gap-1">
                <div class="flex min-w-0 flex-1 flex-col">
                  <span
                    class="flex min-w-0 items-center gap-1.5 text-xs font-semibold"
                  >
                    {#if attention === "halt"}
                      <!-- The flag is also the jump: one click selects the space
                           and deep-links its halted ticket. Inside a card that is
                           itself role="button", so the handler stops propagation
                           the way the forget action does. -->
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        class="-my-0.5 shrink-0 text-destructive hover:text-destructive"
                        aria-label="a session halted — go to the halted ticket"
                        title="A session halted, needs a decision — go to it"
                        onclick={(e) => {
                          e.stopPropagation();
                          jumpToHalt(space);
                        }}
                        onkeydown={(e) => {
                          // The card handles Enter/Space itself and preventDefaults
                          // it; let the button's own activation win instead.
                          if (e.key === "Enter" || e.key === " ")
                            e.stopPropagation();
                        }}
                      >
                        <Warning />
                      </Button>
                    {/if}
                    {#if liveness === "working"}
                      <CircleNotch
                        class="size-3 shrink-0 animate-spin text-primary"
                        aria-label="a session is working"
                      />
                    {:else if liveness === "blocked"}
                      <PauseCircle
                        class="size-3 shrink-0 text-primary"
                        aria-label="a session is blocked"
                      />
                    {/if}
                    <span class="truncate">{space.name}</span>
                  </span>
                  {#if space.branch}
                    <div
                      class="mt-0.5 flex min-w-0 items-center gap-1.5 text-[0.6rem] text-muted-foreground"
                    >
                      <GitBranchIcon class="size-3 shrink-0" />
                      <span class="truncate font-mono" title={space.branch}
                        >{space.branch}</span
                      >
                    </div>
                  {/if}
                </div>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="-mt-0.5 -mr-0.5 hover:text-destructive"
                  aria-label="Remove space"
                  title="Remove from this list (your files stay put)"
                  onclick={(e) => {
                    e.stopPropagation();
                    forget(space);
                  }}
                >
                  <X />
                </Button>
              </div>

              <!-- Sessions: the space's open shells, each its own card inside the
                   space's — identity over status, with its close action pinned the
                   same way the space's is. Clicking one selects the space *and*
                   switches to that session, the one click that does both. -->
              {#if space.terminals.length}
                <ul class="flex flex-col gap-1.5">
                  {#each space.terminals as t (t.id)}
                    {@const isActive = isSelected && activeTerm?.id === t.id}
                    <li>
                      <div
                        role="button"
                        tabindex="0"
                        aria-pressed={isActive}
                        class={[
                          "flex flex-col gap-0.5 rounded-md border px-2 py-1.5 transition-colors",
                          isActive
                            ? "border-primary/50 bg-sidebar-accent text-sidebar-accent-foreground"
                            : "border-sidebar-border hover:bg-sidebar-accent/60",
                        ]}
                        onclick={(e) => {
                          e.stopPropagation();
                          selectSession(space, t);
                        }}
                        onkeydown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            e.stopPropagation();
                            selectSession(space, t);
                          }
                        }}
                      >
                        <div class="flex items-start gap-1">
                          <span class="min-w-0 flex-1">
                            {#if t.session}
                              <!-- A session: its identity is the ticket it is bound
                                   to (role · #num) — told apart from an ad-hoc
                                   shell, which shows its foreground process. -->
                              <span
                                class="flex min-w-0 items-center gap-1 text-xs font-medium"
                              >
                                <Rocket
                                  class="size-3 shrink-0 text-primary"
                                  aria-hidden="true"
                                />
                                <span class="truncate"
                                  >{t.session.role} #{pad(
                                    t.session.ticketNum,
                                  )}</span
                                >
                              </span>
                            {:else}
                              <span class="block truncate font-mono text-xs"
                                >{t.proc}</span
                              >
                            {/if}
                          </span>
                          <Button
                            variant="ghost"
                            size="icon-xs"
                            class="-mt-0.5 -mr-1 hover:text-destructive"
                            aria-label="End {t.proc}"
                            title={t.session
                              ? "End this session"
                              : "End this shell"}
                            onclick={(e) => {
                              e.stopPropagation();
                              endShell(space, t);
                            }}
                          >
                            <X />
                          </Button>
                        </div>

                        <div class="flex items-center gap-1.5">
                          <!-- Status indicator. A tab with no known agent in front: a
                               spinner while working, a tick idle, an error mark once it
                               exits. A tab with a known agent reads the agent's own
                               broadcast state — the same spinner and tick, plus a held
                               pause mark when it is blocked waiting on its human. A dead
                               session freezes under a grey mark. -->
                          {#if t.status === "working"}
                            <CircleNotch
                              class="size-3.5 shrink-0 animate-spin text-primary"
                              aria-label="working"
                            />
                          {:else if t.status === "blocked"}
                            <PauseCircle
                              class="size-3.5 shrink-0 text-primary"
                              aria-label="blocked"
                            />
                          {:else if t.status === "dead"}
                            <XCircle
                              class="size-3.5 shrink-0 text-muted-foreground"
                              aria-label="dead"
                            />
                          {:else if t.status === "exited"}
                            <XCircle
                              class="size-3.5 shrink-0 text-destructive"
                              aria-label="exited"
                            />
                          {:else}
                            <Check
                              class="size-3.5 shrink-0 text-muted-foreground"
                              aria-label="idle"
                            />
                          {/if}
                          <span
                            class="min-w-0 flex-1 truncate text-[0.65rem] text-muted-foreground"
                          >
                            {#if t.session}{t.session.agent} · {t.status}{:else}{t.status}{/if}
                          </span>

                          {#if t.session && !t.alive}
                            <!-- The death halt: a dead session is pinned to its ticket and
                                 offers exactly three choices — resume it (crash recovery),
                                 respawn a fresh session, or release the claim. chartr
                                 takes none itself. -->
                            <span
                              class="-my-0.5 -mr-1 flex shrink-0 items-center"
                            >
                              <Button
                                variant="ghost"
                                size="icon-xs"
                                class="hover:text-primary"
                                aria-label="Resume this session"
                                title="Resume — same-ticket crash recovery"
                                onclick={(e) => {
                                  e.stopPropagation();
                                  haltAction(space, t, "resume", resumeSession);
                                }}
                              >
                                <Play />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon-xs"
                                class="hover:text-primary"
                                aria-label="Respawn a fresh session"
                                title="Respawn — a fresh session on the same ticket"
                                onclick={(e) => {
                                  e.stopPropagation();
                                  haltAction(space, t, "respawn", respawnSession);
                                }}
                              >
                                <ArrowClockwise />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon-xs"
                                class="hover:text-destructive"
                                aria-label="Release the claim"
                                title="Release — clear the claim back to the frontier"
                                onclick={(e) => {
                                  e.stopPropagation();
                                  haltAction(space, t, "release", releaseSession);
                                }}
                              >
                                <ArrowUUpLeft />
                              </Button>
                            </span>
                          {/if}
                        </div>
                      </div>
                    </li>
                  {/each}
                </ul>
              {/if}

              <!-- Actions: this space's own way into the config surface (ticket 05),
                   and the two ticketless on-ramps — ideate and a plain shell. -->
              <div class="flex items-center gap-1">
                <Button
                  class="-ml-1"
                  variant="ghost"
                  size="icon-xs"
                  aria-label="{space.name}’s settings"
                  title="This space's effective config — bindings, skills, and where each layer lives"
                  onclick={(e) => {
                    e.stopPropagation();
                    // Selects the space *and* opens its config — so this sets the
                    // selection directly rather than going through selectSpace,
                    // whose job is to leave the settings route we are entering.
                    selectedId = space.id;
                    openSettings({ kind: "space", spaceId: space.id });
                  }}
                >
                  <Gear class="size-3.5" />
                </Button>
                <span class="flex-1"></span>
                <!-- The skill launcher (skill-launcher map): one `Skills ▾` menu —
                     the agent picker over the on-ramp skills. The row's own click
                     just selects the space, which launchSpace does anyway, so this
                     one deliberately does not stop propagation — the dropdown
                     trigger has no click handler of its own to protect. The agent's
                     model and the skill name live in the menu, not on this cramped
                     label. -->
                <SkillLauncher
                  agents={agentLibrary}
                  lastAgent={space.lastAgent}
                  skills={space.skills}
                  label="Skills"
                  disabled={opening}
                  size="xs"
                  ariaLabel="Launch a skill in {space.name}"
                  title="Launch a self-driving skill in {space.name} — a live, ticketless agent tab. Nothing is claimed, nothing is committed, and it ends when you end it."
                  onrun={(agent, skill) => launchSpace(space, agent, skill)}
                  onregister={() => openSettings({ kind: "user" })}
                >
                  {#snippet icon()}<Plus />{/snippet}
                </SkillLauncher>
                <Button
                  variant="outline"
                  size="xs"
                  aria-label="Open a shell in {space.name}"
                  title="Open a shell in {space.name}"
                  disabled={opening}
                  onclick={(e) => {
                    e.stopPropagation();
                    openShell(space);
                  }}
                >
                  <Plus /> Shell
                </Button>
              </div>
            </div>
          {:else}
            <p class="px-2 py-1.5 text-xs text-muted-foreground">
              No spaces match “{filter}”.
            </p>
          {/each}
        </div>
      {/if}

      <!-- The effective config surface (ticket 05) is entered per space — each
           space card carries its own ⚙ — or with `,`; this stickied footer
           keeps only what is genuinely cross-space: adding a new one. Always
           available, even with zero spaces registered — it's the only way in. -->
      <div class="flex flex-col gap-2 border-t border-sidebar-border p-2">
        <!-- The register outcome lands here, next to the control that caused it:
             the announced `git init` and every refusal. Dismissible, because it
             is a report on a finished action and nothing depends on it. -->
        {#if addNotice || addError}
          <div class="flex flex-col gap-0.5" role={addError ? "alert" : "status"}>
            <div class="flex items-center gap-1.5 text-[0.7rem]">
              {#if addError}
                <p class="min-w-0 flex-1 truncate text-destructive" title={addError}>
                  {addError}
                </p>
              {:else if addNotice}
                <!-- The path front-truncates (dir="rtl" flips which end the
                     ellipsis eats from) so the project name at its end stays
                     visible instead of the drive/user segments at its front. -->
                <p class="flex min-w-0 flex-1 items-baseline gap-1 text-muted-foreground">
                  <span class="shrink-0">Added</span>
                  <span
                    dir="rtl"
                    class="min-w-0 flex-1 truncate text-left font-mono"
                    title={addNotice.path}
                  >{addNotice.path}</span
                  ><span class="shrink-0">.</span>
                </p>
              {/if}
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label="Dismiss"
                onclick={() => {
                  addNotice = null;
                  addError = null;
                }}
              >
                <X />
              </Button>
            </div>
            {#if addNotice?.gitInited}
              <p class="text-[0.65rem] leading-snug text-muted-foreground">
                Wasn’t a git repository — a new one was initialized there.
              </p>
            {/if}
          </div>
        {/if}
        <Button
          variant="outline"
          size="sm"
          class="w-full"
          disabled={picking || control.model === null}
          aria-expanded={nativePicker ? undefined : showAdd}
          onclick={addSpace}
        >
          {#if picking}
            <CircleNotch class="animate-spin" /> Choosing…
          {:else if nativePicker}
            <FolderOpen /> New Space
          {:else}
            <Plus /> New Space
          {/if}
        </Button>
      </div>
    </aside>

    <main class="relative col-start-2 row-start-1 min-h-0 min-w-0">
      {#if spaces.length === 0}
        <div class="grid h-full place-items-center p-6">
          <!-- First run is the same add action as the sidebar's, so it is the same
               chooser — a native picker the operator would only meet on their
               second space would be a picker they never meet. The typed form is
               still what a machine with no chooser gets. -->
          {#if nativePicker}
            <div class="flex w-full max-w-sm flex-col items-start gap-3">
              <h1 class="text-lg font-semibold">Register your first space</h1>
              <p class="text-sm text-muted-foreground">
                Point chartr at a project folder. If it isn’t a git repository
                yet, one is initialized there — announced, never silent.
              </p>
              <Button disabled={picking} onclick={addSpace}>
                {#if picking}
                  <CircleNotch class="animate-spin" /> Choosing…
                {:else}
                  <FolderOpen /> Choose a folder…
                {/if}
              </Button>
              {#if addError}
                <p class="text-xs text-destructive" role="alert">{addError}</p>
              {/if}
            </div>
          {:else}
            <RegisterForm
              variant="first-run"
              onRegistered={(id) => (selectedId = id)}
            />
          {/if}
        </div>
      {:else if selected}
        <SpacePane
          space={selected}
          agents={agentLibrary}
          {activeTerm}
          terminalPrefs={control.model?.terminal}
          active={!route.settings}
          onOpenShell={() => openShell(selected)}
          onLaunch={(agent, skill) => launchSpace(selected, agent, skill)}
          onOpenSettings={() => openSettings()}
          onRegisterAgent={() => openSettings({ kind: "user" })}
          onspawned={(id) => (activeTermId = id)}
        />
      {/if}

      <!-- The settings route renders over the space cockpit rather than replacing
           it in the tree: the terminal and the star-map are imperative islands
           (ADR 0010), and tearing them down to read config would cost a re-attach
           and the map's open state. The pane below goes inert while this is up —
           it takes no keystrokes and stops reflecting itself into the URL, and it
           is a single isolated stacking context (SpacePane), so this one z-index
           is all it takes to sit over the whole stage, chrome included. -->
      {#if route.settings && route.scope}
        <div class="absolute inset-0 z-30 bg-background">
          <Settings
            {spaces}
            config={configLayers}
            agents={agentLibrary}
            {detected}
            terminalPrefs={control.model?.terminal}
            scope={route.scope}
            onScope={(s) => navigate(settingsHash(s))}
            onClose={leaveSettings}
          />
        </div>
      {/if}
    </main>
  </div>

  <!-- The typed-path modal is now the fallback and nothing else: it opens only on
       a machine with no native folder chooser (Linux without zenity or kdialog,
       or Windows), where pasting a path is the only way in. Everywhere else the
       operator gets their own OS chooser and never sees this. -->
  <Modal open={showAdd} title="Add a space" onClose={() => (showAdd = false)}>
    <p class="mb-3 text-xs text-muted-foreground">
      No folder chooser was found on this machine, so point chartr at a
      project folder by pasting its absolute path. If it isn’t a git repository
      yet, one is initialized there, announced.
    </p>
    <RegisterForm
      variant="inline"
      onRegistered={(id) => {
        selectedId = id;
        showAdd = false;
      }}
    />
  </Modal>

  <!-- Removing a space is destructive-sounding enough to confirm, and the
       confirmation is ours: dismissal (Esc, backdrop, ✕) is Cancel, so the only
       way through is the explicit button. -->
  <Modal
    open={pendingForget !== null}
    title="Remove “{pendingForget?.name ?? ''}”?"
    onClose={() => (pendingForget = null)}
  >
    <p class="text-xs text-muted-foreground">
      This only takes it off your list here. Your files stay exactly where they
      are, and you can add it back any time.
    </p>
    <div class="mt-4 flex justify-end gap-2">
      <Button variant="outline" size="sm" onclick={() => (pendingForget = null)}>
        Cancel
      </Button>
      <Button variant="destructive" size="sm" onclick={confirmForget}>
        Remove
      </Button>
    </div>
  </Modal>

  <!-- Every action failure that used to be an `alert()`. One surface, dismissed
       the ordinary way. -->
  <Modal
    open={actionError !== null}
    title="That didn’t work"
    onClose={() => (actionError = null)}
  >
    <p class="text-xs text-muted-foreground">{actionError}</p>
    <div class="mt-4 flex justify-end">
      <Button variant="outline" size="sm" onclick={() => (actionError = null)}>
        Close
      </Button>
    </div>
  </Modal>
</div>
