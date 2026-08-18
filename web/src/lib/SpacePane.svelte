<script lang="ts">
  import { onMount, untrack } from "svelte";
  import type {
    Agent,
    Prompt,
    Space,
    Terminal as Term,
    TerminalPrefs,
    Map as WMap,
  } from "./model";
  import Terminal from "./Terminal.svelte";
  import MapCard from "./MapCard.svelte";
  import PromptsCard from "./PromptsCard.svelte";
  import NewShellButton from "./NewShellButton.svelte";
  import { openSpaceFolder, syncSkills } from "./actions";
  import { Button } from "./components/ui/button";
  import * as Tooltip from "./components/ui/tooltip";
  import { isEditingTarget } from "./keys";
  import { openingFor, paneState, rememberPane } from "./mapstate";
  import {
    Warning,
    Columns,
    CornersOut,
    SidebarSimple,
    FolderOpen,
    Copy,
    ArrowsClockwise,
    CircleNotch,
    X,
  } from "phosphor-svelte";

  // The folder button's label names the operator's own file browser rather than
  // a generic "folder" — Finder on a Mac, "file browser" everywhere else, since
  // that is the one native-picker split the app already draws elsewhere
  // (`nativePicker` in App.svelte). Read once: the platform does not change
  // under a running session.
  const isMac =
    typeof navigator !== "undefined" &&
    /Mac/.test(navigator.platform ?? navigator.userAgent ?? "");

  // The stage for the selected space: a full-width title bar carrying the space's
  // identity (name plus folder/path actions) and the stage-level controls —
  // warnings and the star-map toggle, pinned to the far right — over the
  // terminal. The sidebar now owns session selection (its space cards list each shell), so the
  // active shell arrives as a prop and this pane simply renders it: no tab strip,
  // no per-pane action bar. A mapless space is fully usable this way (story 29).
  //
  // Over the terminal, the star-map is summoned as a floating card — edge handle,
  // M, Esc — or docked as the terminal-priority split (spec, The interface).
  // Visibility changes only on those explicit acts: switching spaces or focusing
  // a different map never opens or closes it, which is why this state lives on
  // the pane (persisting across space switches) and not per-map.
  let {
    space,
    agents,
    prompts,
    activeTerm,
    canSyncSkills = false,
    terminalPrefs,
    active = true,
    onOpenShell,
    onFreeSession,
    onRegisterAgent,
    onspawned,
  }: {
    space: Space;
    // The registered agent library, global like the settings surface (ticket 02):
    // passed down to the star-map's detail pane so each spawn button can name and
    // pick the agent it will run.
    agents: Agent[];
    // The operator's prompt catalog, global in exactly the same way (prompt-presets
    // ticket 04): the Prompts tab renders it, and this space carries only which of
    // it applies at launch (`space.prompts`).
    prompts: Prompt[];
    activeTerm: Term | null;
    // Whether a manual skill sync would write anything — at least one enabled
    // source yields a skill (App reads the sources list once and hands it down).
    // The sync control disables itself and says "Nothing to sync" when false,
    // rather than firing a reconcile that copies nothing.
    canSyncSkills?: boolean;
    // The operator's resolved terminal customization off the model snapshot —
    // global (per-machine `terminal.toml`), fed into the terminal island at the
    // token seam. A change to it remounts the island (see the `{#key}` below).
    terminalPrefs?: TerminalPrefs;
    // False while the settings route covers the stage (ticket 05). The pane stays
    // mounted — its terminal and star-map are imperative islands worth keeping
    // alive — but goes inert: it takes no keystrokes and stops reflecting its
    // selection into the URL, which the settings route owns while it is up.
    active?: boolean;
    onOpenShell: () => void;
    // A free session (skill-sources ticket 08): a live, ticketless agent tab for a
    // space with no shell open, told the free payload and nothing else. The
    // control here picks the agent and hands the name up; there is no skill and no
    // context line to settle.
    onFreeSession: (agent: string) => void;
    // Where an empty-library spawn or launch control sends the operator: agent
    // registration (the user scope of the settings surface). Owned by App like the
    // route above; every agent picker beneath this pane routes its empty state
    // here rather than being a dead button (ticket 04).
    onRegisterAgent: () => void;
    // Bubbled from the star-map's detail pane when a session is spawned (ticket
    // 09), so the enclosing App can make the new session's tab active.
    onspawned?: (sessionId: string) => void;
  } = $props();

  // The space name copies its path, with the folder action beside it. Feedback
  // stays on those controls so the compact header does not grow a transient
  // second line.
  let pathFeedback = $state<"copied" | "copy-error" | "open-error" | null>(
    null,
  );
  let pathFeedbackTimer: ReturnType<typeof setTimeout> | undefined;

  function showPathFeedback(next: typeof pathFeedback) {
    pathFeedback = next;
    clearTimeout(pathFeedbackTimer);
    pathFeedbackTimer = setTimeout(() => (pathFeedback = null), 1400);
  }

  async function revealSpace() {
    try {
      await openSpaceFolder(space.id);
    } catch {
      showPathFeedback("open-error");
    }
  }

  // The manual skill sync (beside the folder action): reconcile this space's
  // `.chartr/skills` mirror against the enabled sources now. The button spins and
  // goes inert for the duration — one sync at a time — and a failure surfaces on
  // the control itself for a beat, the same shape the folder action uses, so the
  // header never grows a second line for it. The refreshed state rides back on
  // the control snapshot; there is nothing to reflect here beyond the outcome.
  let syncing = $state(false);
  let syncError = $state(false);
  let syncErrorTimer: ReturnType<typeof setTimeout> | undefined;

  async function runSkillSync() {
    if (syncing) return;
    syncing = true;
    syncError = false;
    clearTimeout(syncErrorTimer);
    try {
      await syncSkills(space.id);
    } catch {
      syncError = true;
      syncErrorTimer = setTimeout(() => (syncError = false), 1400);
    } finally {
      syncing = false;
    }
  }

  async function copySpacePath() {
    try {
      await navigator.clipboard.writeText(space.path);
      showPathFeedback("copied");
    } catch {
      showPathFeedback("copy-error");
    }
  }

  // A deep link names a star (spec): #s=<spaceId>&m=<mapSlug>&t=<ticketNum>, or
  // &mat=1 for the map material, or &maps=1 for the picker. Parsed once at init — the
  // enclosing App has already selected the space from the same `s` — so the linked
  // star opens and seats on load; manual edits are picked up by a hashchange
  // listener below.
  function parseHash() {
    const p = new URLSearchParams(location.hash.replace(/^#/, ""));
    return {
      s: p.get("s"),
      m: p.get("m"),
      t: p.get("t"),
      mat: p.get("mat"),
      maps: p.get("maps"),
    };
  }
  const boot = parseHash();
  // A one-time read at construction: the enclosing App has already selected this
  // space from the same `s`, so the boot link applies here. untrack marks the
  // read deliberate (this must not react to later space switches).
  const bootApplies = !boot.s || boot.s === untrack(() => space.id);

  // What this space's pane last had open, from the store that outlives these
  // components (mapstate.ts). On the first run of a sitting only the map survives
  // — the selection and the camera belong to the sitting that made them — so this
  // is a whole restore on a space switch and a slug on a cold start, through the
  // one path. A deep link outranks it: it named a map explicitly.
  const remembered = untrack(() =>
    openingFor(paneState(space.id), space.maps ?? []),
  );

  // Star-map card state. Which map, which star, and (inside the island) where the
  // camera sits are all per space, held in the store above across a switch; the
  // panel's own visibility and docking are the operator's standing preference for
  // the whole cockpit, so they simply live here and never reset.
  //
  // A boot link that names anything about the map owns the whole opening state;
  // one that names only the space (or none at all) leaves it to the store.
  const linked =
    bootApplies && (!!boot.m || !!boot.t || !!boot.mat || !!boot.maps);
  let paneShown = $state(bootApplies && (!!boot.t || !!boot.mat || !!boot.maps));
  // Which tool the one auxiliary pane is showing. Map and Prompts share the
  // frame rather than competing for it, so this is a switch and not a second
  // visibility flag. A standing choice for the whole stage, like the frame's own
  // visibility and docking: switching spaces does not reset it.
  let paneTab = $state<"map" | "prompts">("map");
  let dock = $state(true);
  let openSlug = $state<string | null>(linked ? boot.m : remembered.slug);
  let selectedTicket = $state<number | null>(
    linked ? (boot.t ? Number(boot.t) : null) : remembered.selected,
  );
  let showMaterial = $state(linked ? !!boot.mat : remembered.showMaterial);
  let dockTermWidth = $state(0);
  let floatWidth = $state(0);
  let bodyEl: HTMLDivElement;

  const warnings = $derived<string[]>(space.warnings ?? []);
  const maps = $derived<WMap[]>(space.maps ?? []);

  // The Scratch space has no repository, so it has nothing to chart: no star-map
  // toggle, no map card, and inert M/Esc. `paneShown` is
  // a standing preference for the whole stage rather than per space, so this is
  // enforced *here*, where the card renders — an operator who leaves the map open
  // on a registered space and switches to Scratch must not carry it across, and
  // gating only the toggle would do exactly that.
  const mapless = $derived(space.scratch === true);

  // Whether the star-map is actually on the stage — the standing preference, minus
  // the spaces that cannot hold a map. Everything that follows from a *rendered*
  // card reads this rather than `paneShown`: the docked split's frozen terminal
  // width included, or a Scratch shell would sit in 60% of the stage with nothing
  // beside it.
  const paneOpen = $derived(paneShown && !mapless);

  // A stable identity for the current terminal prefs: the retained pool keys its
  // remount on this string, so editing `terminal.toml` (a new snapshot with
  // different prefs) tears its islands down and mounts them afresh with the new
  // options resolved at the seam. The whole prefs object goes in — preset, font,
  // and every colour slot — so any field the file changes forces the remount
  // without this list having to track the growing slot set.
  const terminalPrefsKey = $derived(JSON.stringify(terminalPrefs ?? {}));

  // A terminal becomes part of this space's retained pool on first visit. Keeping
  // the record lazy avoids opening sockets/renderers for sessions the operator has
  // never looked at; switching spaces clears it, which bounds the pool naturally
  // to the one space on stage. Preference changes need no special state here: the
  // existing key below remounts every retained island as one simple operation.
  let retainedSpaceId = $state<string | undefined>(undefined);
  let retainedTerminals = $state<Record<string, true>>({});
  $effect(() => {
    if (space.id !== retainedSpaceId) {
      retainedSpaceId = space.id;
      retainedTerminals = {};
    }
    if (activeTerm) retainedTerminals[activeTerm.id] = true;
  });

  // A selection belongs to one map: when the open map *changes*, drop it (and any
  // open material) so the island never carries a ticket number from a different
  // graph. The first run only records the slug — it must not clear a selection the
  // deep link just seeded. (undefined is the "not yet recorded" sentinel, since
  // openSlug itself is legitimately null on the picker.)
  let lastOpen: string | null | undefined = undefined;
  $effect(() => {
    const s = openSlug;
    if (lastOpen === undefined) {
      lastOpen = s;
      return;
    }
    if (s !== lastOpen) {
      lastOpen = s;
      selectedTicket = null;
      showMaterial = false;
    }
  });

  // Switching spaces while the panel is open: the state on show belongs to the
  // space we are leaving, so swap in the arriving space's own (the effect below
  // has been keeping it filed as it changed). A space never opened before, or one
  // whose remembered map has since been deleted, gets what a fresh summon gets:
  // its one map, or the picker. Guarded on an actual space change so the back
  // button (which nulls openSlug within a space) still lands on, and stays on,
  // the picker.
  let lastSpaceId = untrack(() => space.id);
  $effect(() => {
    if (space.id === lastSpaceId) return;
    lastSpaceId = space.id;
    const opening = openingFor(paneState(space.id), maps);
    openSlug = opening.slug;
    selectedTicket = opening.selected;
    showMaterial = opening.showMaterial;
    // The restored star arrived *with* its slug, so mark the slug as already
    // seen: the drop-on-map-change guard above must not read this as a map
    // switch and clear what we just put back (the same exemption a link gets).
    lastOpen = openSlug;
  });

  // Keep the store current for whichever space is on show, so the swap above
  // always has something faithful to hand back — and so the open map is on disk
  // for the next run. Declared after the swap: on a space change both are dirty
  // in one flush and effects run in declaration order, so this writes the
  // arriving space's state under the arriving space's id, never the outgoing
  // pane's under the newcomer's name.
  $effect(() => {
    rememberPane(space.id, {
      slug: openSlug,
      selected: selectedTicket,
      showMaterial,
    });
  });

  // Apply whatever star link the hash currently names to this pane. Shared by
  // the hashchange listener below and the on-activation effect; only ever acts
  // on a link that targets this space (App owns switching to another space's).
  function applyHash() {
    const h = parseHash();
    if (h.s && h.s !== space.id) return;
    if (h.m) openSlug = h.m;
    // A link names its map and its star together, so record the slug as already
    // seen: the drop-on-map-change guard above must not clear the star that
    // arrived with the slug — the same exemption its first run makes for the
    // boot link, now that a link can also land mid-life (the halt flag's jump).
    if (h.t) {
      selectedTicket = Number(h.t);
      showMaterial = false;
      paneShown = true;
      paneTab = "map";
      lastOpen = openSlug;
    } else if (h.mat) {
      showMaterial = true;
      selectedTicket = null;
      paneShown = true;
      paneTab = "map";
      lastOpen = openSlug;
    } else if (h.maps) {
      // The picker: the space's maps, each one a door in.
      openSlug = null;
      selectedTicket = null;
      showMaterial = false;
      paneShown = true;
      paneTab = "map";
    }
  }

  // Apply the link the moment this pane swings to another space (or comes back
  // from settings), not only on hashchange. App can switch space and set a star
  // link in one click — the sidebar halt flag's jump — and hashchange is
  // delivered a task later, by which time the reflecting effect below, seeing a
  // pane with nothing open, would already have wiped the link it was about to
  // read. Applying first means the reflection agrees with the link instead of
  // erasing it. Declared above that effect (and below the space-change reset,
  // whose stale selection it overwrites) so the order within one flush is
  // reset → apply → reflect. The hash read is untracked: this fires on a space
  // change and nothing else.
  $effect(() => {
    space.id;
    if (!active) return;
    untrack(() => applyHash());
  });

  // Reflect the current selection into the URL so a star (or the map material) is
  // a shareable deep link. replaceState never fires hashchange, so this and the
  // listener below do not loop. While the settings route is up it owns the hash,
  // so this stands down and restores its own link when the pane is active again.
  $effect(() => {
    if (!active) return;
    const p = new URLSearchParams();
    p.set("s", space.id);
    if (openSlug) p.set("m", openSlug);
    if (selectedTicket !== null) p.set("t", String(selectedTicket));
    else if (showMaterial) p.set("mat", "1");
    const want =
      paneOpen && (selectedTicket !== null || showMaterial)
        ? "#" + p.toString()
        : "";
    if (location.hash !== want) {
      history.replaceState(
        null,
        "",
        want || location.pathname + location.search,
      );
    }
  });

  // Manual URL edits and back/forward re-apply, through the same path.
  onMount(() => {
    window.addEventListener("hashchange", applyHash);
    return () => window.removeEventListener("hashchange", applyHash);
  });

  // Freeze the terminal's pixel width at the moment of docking, then let the map
  // absorb every later resize slack — the terminal-priority split holds its width
  // so a window resize never reflows it (planning ticket 08's amendment). The one
  // exception is the small end: once the window is too narrow to also grant the
  // map its floor (min-width 300), the terminal yields the rest so the map never
  // collapses out of view. Both floors are enforced in CSS (the docked term-col
  // is shrinkable to 240; the map card holds 300), so window resizes need no JS.
  function summon() {
    paneShown = true;
    // A single-map space has nothing to pick — open straight into its one map.
    // With several, land on the picker (openSlug stays null). Never overrides a
    // slug a deep link or an earlier session already opened.
    if (openSlug === null && maps.length === 1) openSlug = maps[0].slug;
  }
  function dismiss() {
    paneShown = false;
  }
  function togglePane() {
    if (paneShown) dismiss();
    else summon();
  }
  $effect(() => {
    if (paneOpen && dock && bodyEl && !dockTermWidth) {
      const w = bodyEl.clientWidth;
      // First dock: terminal keeps ~60%, always leaving room for the map; clamped
      // so neither pane collapses on a narrow window. A resize below overrides it.
      dockTermWidth = Math.round(
        Math.min(Math.max(w * 0.6, 320), Math.max(360, w - 360)),
      );
    }
  });

  // Drag the card's left border to resize it — in either mode. Docked, the
  // border is the split: the map's edge moves and the terminal's frozen width
  // follows it. Floating, the card grows leftward while its right edge stays
  // pinned. Clamped so neither pane collapses.
  const MIN_MAP = 300;
  const FLOAT_INSET = 10; // matches .map-floating .map-card right offset
  // A floating card is freely draggable wider; its right edge stays pinned and
  // it's only kept within the row so it can't overflow left into the sidebar. CSS
  // max-width holds that same within-the-row bound on resize.
  const FLOAT_MIN_TERM = 30; // sliver of terminal kept visible; matches the CSS 40px bound (FLOAT_INSET + this)
  function startResize(e: MouseEvent) {
    e.preventDefault();
    const rect = bodyEl.getBoundingClientRect();
    const move = (ev: MouseEvent) => {
      if (dock) {
        const minTerm = 240;
        dockTermWidth = Math.round(
          Math.min(
            Math.max(ev.clientX - rect.left, minTerm),
            Math.max(minTerm, rect.width - MIN_MAP),
          ),
        );
      } else {
        const maxMap = Math.max(
          MIN_MAP,
          rect.width - FLOAT_INSET - FLOAT_MIN_TERM,
        );
        floatWidth = Math.round(
          Math.min(
            Math.max(rect.right - FLOAT_INSET - ev.clientX, MIN_MAP),
            maxMap,
          ),
        );
      }
    };
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    document.body.style.cursor = "ew-resize";
    document.body.style.userSelect = "none";
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  // M summons/dismisses, Esc dismisses — but only when focus is on the chrome,
  // not inside the terminal (whose PTY owns every raw keystroke) or a text field.
  // The edge handle is the always-available path when the shell has the keyboard.
  function onKey(e: KeyboardEvent) {
    // Inert while the settings route covers the stage: its own Esc must not also
    // peel back this pane's map underneath it.
    if (!active) return;
    // Inert over Scratch for the same reason the toggle is absent there: there is
    // no map to summon, and a binding that silently changes state nothing renders
    // is worse than one that does nothing.
    if (mapless) return;
    // A summoned Sheet/Dialog (the action station) owns its own Escape; the
    // chrome's M/Esc bindings must not also fire while it holds focus.
    const editing = isEditingTarget();
    if (e.key === "Escape" && !editing) {
      // Esc peels back one layer: the open detail pane first, then the open map
      // (back to the picker), then the pane itself. The two inner layers belong
      // to the map, so they are skipped when Prompts is the tab on show.
      if (paneShown && paneTab === "map") {
        if (selectedTicket !== null || showMaterial) {
          selectedTicket = null;
          showMaterial = false;
          return;
        }
        if (openSlug !== null) {
          openSlug = null;
          return;
        }
      }
      if (paneShown) {
        dismiss();
        return;
      }
    }
    if (
      (e.key === "m" || e.key === "M") &&
      !editing &&
      !e.metaKey &&
      !e.ctrlKey
    ) {
      e.preventDefault();
      togglePane();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<!-- The space's stage: a full-width title bar (the space's name and path
     actions) over a row of its subpanes. The identity lives here, one level above
     the panes, so the hierarchy reads "space › {terminals, map}": each pane
     carries only its own chrome. A floating map overlays the panes row but never
     this header — the panes row is its positioning context, and it sits below.

     `isolate` makes this stage one stacking context: every z-index inside (the
     floating card, the map's chrome bar, the resize grips) is then local to the
     stage and cannot climb over a route overlay rendered beside it, such as the
     settings surface. Without it a docked card — `relative`, z-auto, so no
     context of its own — leaked its z-30 chrome through settings. -->
<div class="isolate flex h-full min-h-0 flex-col">
  <header class="cockpit-bar justify-between">
    <div class="flex min-w-0 items-center gap-1">
      <!-- disableCloseOnTriggerClick: the click *is* the copy action, and its
           feedback ("Copied") lives in this same tooltip — bits-ui's default
           close-on-click would hide it before the operator ever saw it. -->
      <Tooltip.Root disableCloseOnTriggerClick>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button
              {...props}
              variant="ghost"
              size="sm"
              class="-ml-1 min-w-0 shrink justify-start px-1 text-sm font-semibold"
              aria-label="Copy path for {space.name}"
              onclick={copySpacePath}
            >
              <span class="truncate">{space.name}</span>
              {#if pathFeedback === "copy-error"}
                <Warning class="shrink-0 text-destructive" />
              {/if}
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>
          <div class="flex flex-col gap-1">
            <code
              class="rounded bg-muted px-1.5 py-1 font-mono text-[0.65rem] break-all text-foreground"
              >{space.path}</code
            >
            <span class="flex items-center justify-end gap-1">
              {#if pathFeedback === "copy-error"}
                <Warning class="size-3 shrink-0 text-destructive" /> Couldn’t copy
                — clipboard unavailable
              {:else}
                <Copy class="size-3 shrink-0" />
                {pathFeedback === "copied" ? "Copied" : "Click to copy"}
              {/if}
            </span>
          </div>
        </Tooltip.Content>
      </Tooltip.Root>
      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button
              {...props}
              variant="ghost"
              size="icon-sm"
              aria-label="Open {space.name} in the folder browser"
              onclick={revealSpace}
            >
              {#if pathFeedback === "open-error"}
                <Warning class="text-destructive" />
              {:else}
                <FolderOpen />
              {/if}
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>
          {pathFeedback === "open-error"
            ? "Couldn’t open the folder"
            : isMac
              ? "Open in Finder"
              : "Open in file browser"}
        </Tooltip.Content>
      </Tooltip.Root>
      <!-- Manual skill sync, beside the folder action. Absent over Scratch (no
           repository to mirror into, like the map toggle), disabled when no
           enabled source yields a skill — there is nothing to copy — and inert
           while a sync is in flight. -->
      {#if !mapless}
        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="ghost"
                size="icon-sm"
                disabled={syncing || !canSyncSkills}
                aria-label="Sync skills into this space"
                onclick={runSkillSync}
              >
                {#if syncing}
                  <CircleNotch class="animate-spin" />
                {:else if syncError}
                  <Warning class="text-destructive" />
                {:else}
                  <ArrowsClockwise />
                {/if}
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content>
            {syncError
              ? "Couldn’t sync skills"
              : canSyncSkills
                ? "Sync skills into this space"
                : "Nothing to sync — no enabled source yields a skill"}
          </Tooltip.Content>
        </Tooltip.Root>
      {/if}
    </div>

    <!-- The stage-level controls, right-aligned: surfaced warnings and the one
         star-map show/hide toggle. Settings lives at the bottom of the sidebar,
         so this end of the bar stays dedicated to the active space. -->
    <div class="flex items-center gap-1.5">
      {#if warnings.length}
        <span
          class="flex items-center gap-1 text-[0.7rem] text-muted-foreground"
          title={warnings.join("\n")}
          aria-label="{warnings.length} warning(s)"
        >
          <Warning class="size-3.5" />
          {warnings.length}
        </span>
      {/if}
      <!-- The one show/hide control for the stage's auxiliary pane, beside the
           bindings; reflects paneShown via aria-pressed. Which tool the pane
           shows — Map or Prompts — is the pane's own tab switcher, so this stays
           one button. Available even with zero maps (the picker explains there is
           nothing yet, and the catalog is global) — but not over Scratch, which
           has no repository to hold a `.plan/` and takes no launch selection. -->
      {#if !mapless}
        <Button
          variant={paneShown ? "secondary" : "ghost"}
          size="sm"
          aria-pressed={paneShown}
          title={paneShown ? "Hide the pane (M)" : "Show the pane (M)"}
          onclick={togglePane}
        >
          <SidebarSimple weight={paneShown ? "fill" : "regular"} /> Show Pane
        </Button>
      {/if}
    </div>
  </header>

  <!-- The panes row: the terminal column and, over it, the star-map card. It is
       the positioning context for a floating card (relative), and a flex row for
       the docked split — the terminal's frozen width lives in an inline
       flex-basis and the card takes the rest. -->
  <div class="relative flex min-h-0 flex-1" bind:this={bodyEl}>
    <!-- The terminal column: no tab strip, no action bar — the sidebar owns
         session selection now, so this simply renders the active shell. -->
    <div
      class="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden"
      style={paneOpen && dock
        ? `flex: 0 1 ${dockTermWidth}px; min-width: 240px`
        : ""}
    >
      {#if activeTerm}
        <!-- Retain every visited terminal in this space and swap only visibility:
             its xterm buffer, viewport, selection, find state, and socket survive a
             session switch. The one prefs key remains deliberately broad and
             simple — any resolved terminal.toml change remounts the whole retained
             pool so all islands read the new mount-time options. -->
        {#key terminalPrefsKey}
          {#each space.terminals ?? [] as term (term.id)}
            {#if retainedTerminals[term.id]}
              {@const isActive = term.id === activeTerm.id}
              <div
                class="absolute inset-0"
                class:invisible={!isActive}
                class:pointer-events-none={!isActive}
                aria-hidden={isActive ? undefined : "true"}
                inert={!isActive}
              >
                <Terminal {term} prefs={terminalPrefs} active={isActive} />
              </div>
            {/if}
          {/each}
        {/key}
      {:else}
        <div
          class="flex h-full flex-col items-center justify-center gap-2 p-6 text-center"
        >
          <p class="text-sm text-muted-foreground mb-1">No open sessions</p>
          <div class="flex flex-wrap items-center justify-center gap-2">
            <!-- The same one control the space card carries (skill-sources
                 ticket 08): the body opens a plain shell, and the caret lists the
                 registered agents — clicking one launches a free session on it. -->
            <NewShellButton
              {agents}
              size="sm"
              title="Open a plain shell — nothing is injected. Pick an agent from the caret to launch a free session on it instead."
              onshell={onOpenShell}
              onfree={onFreeSession}
              onregister={onRegisterAgent}
            />
            <Button
              variant="outline"
              size="sm"
              aria-pressed={paneShown}
              onclick={togglePane}
            >
              <SidebarSimple weight={paneShown ? "fill" : "regular"} />
              Show Pane
            </Button>
          </div>
        </div>
      {/if}
    </div>

    {#if paneOpen}
      <!-- The auxiliary pane. Docked, it is a flex item in the panes row, taking
           the slack the terminal's frozen basis leaves (min 300). Floating, it is
           absolute over the row with its right edge pinned (inset 10px), grown
           leftward by the resize grip; a CSS max-width keeps it within the row on
           a window resize.

           The frame lives here rather than in either tool, which is what makes
           Map and Prompts one auxiliary space with a switch in it instead of two
           cards racing for the same corner. -->
      <section
        class={[
          "flex min-h-0 flex-col overflow-hidden bg-card",
          dock
            ? // Docked, this is a flush column next to the terminal — a plain
              // square edge like the terminal's own outer edges, with a border
              // only on the seam it shares with the terminal (the split divider).
              "relative min-w-[300px] flex-1 border-l border-border"
            : "absolute inset-y-2 right-2.5 z-20 w-[min(560px,64%)] max-w-[calc(100%-40px)] rounded-lg border border-border shadow-lg",
        ]}
        aria-label="Space pane"
        style={!dock && floatWidth ? `width:${floatWidth}px` : ""}
      >
        <!-- Resize from the left border: the split divider when docked, the
             card's leading edge when floating (the right edge stays pinned). -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="absolute inset-y-0 left-0 z-40 w-1.5 -translate-x-1/2 cursor-ew-resize transition-colors hover:bg-ring/60"
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize the pane"
          onmousedown={startResize}
        ></div>

        <!-- The pane's own bar: which tool is on show, then the dock and close
             chrome that belong to the frame around both of them. -->
        <header class="cockpit-bar">
          <Button
            variant={paneTab === "map" ? "secondary" : "ghost"}
            size="sm"
            aria-pressed={paneTab === "map"}
            onclick={() => (paneTab = "map")}
          >
            Map
          </Button>
          <Button
            variant={paneTab === "prompts" ? "secondary" : "ghost"}
            size="sm"
            aria-pressed={paneTab === "prompts"}
            onclick={() => (paneTab = "prompts")}
          >
            Prompts
          </Button>
          <div class="ml-auto flex items-center gap-1">
            <Button
              variant="outline"
              size="icon-sm"
              aria-pressed={dock}
              title={dock
                ? "Float over the terminal"
                : "Dock as a split (terminal keeps its width)"}
              onclick={() => (dock = !dock)}
            >
              {#if dock}<CornersOut />{:else}<Columns />{/if}
            </Button>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Dismiss the pane (Esc)"
              title="Dismiss (M / Esc)"
              onclick={dismiss}
            >
              <X />
            </Button>
          </div>
        </header>

        <div class="relative flex min-h-0 flex-1 flex-col">
          {#if paneTab === "map"}
            <MapCard
              {maps}
              spaceId={space.id}
              lastAgent={space.lastAgent}
              {agents}
              terminals={space.terminals ?? []}
              bind:slug={openSlug}
              bind:selected={selectedTicket}
              bind:showMaterial
              {onRegisterAgent}
              {onspawned}
            />
          {:else}
            <PromptsCard
              {prompts}
              spaceId={space.id}
              selected={space.prompts ?? []}
              {activeTerm}
            />
          {/if}
        </div>
      </section>
    {/if}
  </div>
</div>
