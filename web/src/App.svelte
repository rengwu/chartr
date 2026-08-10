<script lang="ts">
  import { onMount } from "svelte";
  import { flip } from "svelte/animate";
  import { ControlSocket } from "./lib/control.svelte";
  import type { Space, Terminal } from "./lib/model";
  import {
    deregisterSpace,
    openTerminal,
    closeTerminal,
    markTerminalSeen,
    resumeSession,
    respawnSession,
    releaseSession,
    launchFree,
    pickFolder,
    registerSpace,
    reorderSpaces,
    renameSpace,
    ActionError,
    LIVE_SESSION,
  } from "./lib/actions";
  import { reorder } from "./lib/reorder";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import RegisterForm from "./lib/RegisterForm.svelte";
  import SpaceCard from "./lib/SpaceCard.svelte";
  import SpacePane from "./lib/SpacePane.svelte";
  import Settings from "./lib/Settings.svelte";
  import Modal from "./lib/Modal.svelte";
  import { Button } from "./lib/components/ui/button";
  import { Input } from "./lib/components/ui/input";
  import { Toaster, toast } from "./lib/components/ui/sonner";
  import { spaceHaltTarget } from "./lib/attention";
  import { acknowledgesFinishedRun } from "./lib/unseen";
  import { isEditingTarget } from "./lib/keys";
  import { forgetSpace } from "./lib/mapstate";
  import { nativeTitleBarHeight, trackTitleBarButtons } from "./lib/titlebar";
  import { configurableSpaces, visibleSpaces } from "./lib/spacevisibility";
  import { parseRoute, settingsHash, type SettingsScope } from "./lib/route";
  import {
    Plus,
    CircleNotch,
    Gear,
    FolderOpen,
    MagnifyingGlass,
    TerminalWindow,
  } from "phosphor-svelte";

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

  // The last settings scope the operator was on, remembered across leaving the
  // surface (the hash is cleared on exit, so the scope would otherwise be lost).
  // The sidebar Settings control reopens onto this, so a click lands you back where you
  // were rather than forcing any particular submenu. Null until the first visit.
  let lastSettingsScope = $state<SettingsScope | null>(null);
  $effect(() => {
    if (route.settings && route.scope) lastSettingsScope = route.scope;
  });

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
    const stopTrackingTitleBarButtons = trackTitleBarButtons(titleBarH);
    // A deep link names its space (#s=<id>&…); select it up front so the linked
    // star seats as soon as the space arrives over the socket (ticket 07). The
    // rest of the link — map and star — is applied inside the space's pane.
    const s = new URLSearchParams(location.hash.replace(/^#/, "")).get("s");
    if (s) selectedId = s;
    const onHash = () => (hash = location.hash);
    window.addEventListener("hashchange", onHash);
    return () => {
      window.removeEventListener("hashchange", onHash);
      stopTrackingTitleBarButtons();
      control.close();
    };
  });

  // Spaces arrive already ordered — the operator's own stored arrangement, the
  // one authority. The snapshot always includes Scratch; the one predicate below
  // removes it while empty, and every cockpit behavior consumes that visible
  // ordered list without re-sorting it.
  const snapshotSpaces = $derived<Space[]>(control.model?.spaces ?? []);
  const spaces = $derived<Space[]>(visibleSpaces(snapshotSpaces));
  // The settings surface enumerates spaces as config scopes, and Scratch has none
  // — so it reads the unfiltered snapshot minus Scratch rather than the sidebar's
  // list above. A Scratch space with a shell open is visible in the sidebar and
  // still absent here.
  const scopeSpaces = $derived<Space[]>(configurableSpaces(snapshotSpaces));
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

  // Whether a manual skill sync would write anything: at least one enabled source
  // that resolved and yields a skill. The mirror copies exactly this set, so with
  // nothing here a sync is a no-op — the cockpit's sync control disables itself
  // and says "Nothing to sync" rather than spinning on empty. Global like the
  // sources list, so it is read once and handed to whichever space is in view.
  const canSyncSkills = $derived(
    (control.model?.sources ?? []).some(
      (s) => s.enabled && s.status === "ok" && s.skills.length > 0,
    ),
  );

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
  // The register outcome — the announced `git init` (story 2) and every refusal —
  // surfaces as a toast now (centered over the main panel), not an inline line
  // beside the button. The git-init caveat rides along as the toast's description.
  async function addSpace() {
    if (picking) return;
    if (!nativePicker) {
      showAdd = true;
      return;
    }
    picking = true;
    try {
      const picked = await pickFolder();
      // Dismissing the chooser is an ordinary outcome, not a failure: say nothing
      // and leave the sidebar exactly as it was.
      if (picked.cancelled || !picked.path) return;
      const res = await registerSpace(picked.path);
      toast.success(`Added ${picked.path}`, {
        // Announced, never silent (story 2) — but only when it actually happened.
        description: res.gitInited
          ? "Wasn’t a git repository — a new one was initialized there."
          : undefined,
      });
      selectedId = res.id;
    } catch (err) {
      toast.error(err instanceof ActionError ? err.message : String(err));
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

  // Acknowledging a run that finished while the operator was elsewhere
  // (session-notifications): the tab in front of them clears its own dot. Focus is
  // the only acknowledgement there is — no dismiss, no clear-all — and "focused"
  // here is exactly what the sidebar paints as the active row: the shell the pane
  // is showing, with the settings surface not standing over it. The flag itself is
  // server state, so this posts and lets the cleared snapshot come back; a failure
  // is silent, because a dot that outlives one failed post clears on the next look.
  $effect(() => {
    const space = selected;
    const term = activeTerm;
    if (!space || !term) return;
    if (!acknowledgesFinishedRun(term, !route.settings)) return;
    void markTerminalSeen(space.id, term.id).catch(() => {
      // Nothing to say: the dot is the server's, and the next look posts again.
    });
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
        s.terminals.some(
          (t) =>
            t.proc.toLowerCase().includes(q) ||
            t.title.toLowerCase().includes(q),
        ),
    );
  });

  // --- Reordering the sidebar ------------------------------------------------
  //
  // The operator's arrangement is theirs to set, and there is exactly *one* write
  // path for it: resolve the move to a complete ordered list of ids and post that
  // (`applyOrder`). The pointer drag and the keyboard both end there, so the
  // keyboard is not a second implementation.
  //
  // The drag itself is svelte-dnd-action (`use:dndzone` on the list below): it owns
  // the whole gesture — a card lifts under the pointer while the rest slide apart
  // around the gap (FLIP), on pointer, touch, and its own keyboard mode. We hand it
  // the visible list and take the reordered one back on `consider` (live, during
  // the drag) and `finalize` (the drop). The reorder is optimistic: the cards
  // settle into the new order at once, and the drop still writes the whole list and
  // waits for the fresh snapshot over the control socket to confirm it — that
  // snapshot is fast because the server republishes the one it holds permuted
  // rather than rebuilding (hub.reorderSpaces).
  //
  // This was hand-rolled on pointer capture before, which fought the FLIP reorder:
  // moving the captured node into its new slot mid-drag dropped the capture, and
  // the drag cut off as it crossed a row. The library owns the gesture now, so the
  // cards are plain again — no per-card pointer handlers.

  // The list svelte-dnd-action owns: the visible list, but assignable, because the
  // library reorders it in place through the events below. It tracks `filtered`
  // whenever no drag is in flight (the effect), and the library drives it while one
  // is.
  let dndItems = $state<Space[]>([]);
  // True between the first `consider` and the `finalize`: the window in which the
  // library, not an incoming snapshot, owns `dndItems`.
  let dndDragging = $state(false);
  // The order a just-dropped move committed, held until the confirming snapshot
  // shows it. Without it the cards would snap back to the pre-drop order the instant
  // the drag ends (the snapshot not yet here), then forward again when it lands.
  let pendingOrder = $state<string[] | null>(null);

  // Reorder a list by a list of ids, dropping ids that have since left and
  // appending any that arrived — so a snapshot landing mid-hold can neither drop
  // nor duplicate a card.
  function orderBy(list: Space[], ids: string[]): Space[] {
    const byId = new Map(list.map((s) => [s.id, s]));
    const held = ids.filter((id) => byId.has(id)).map((id) => byId.get(id)!);
    const extra = list.filter((s) => !ids.includes(s.id));
    return [...held, ...extra];
  }

  // Keep `dndItems` in step with the source of truth — except while the library
  // holds it (a snapshot must not yank the cards out from under the pointer), and
  // except across the drop-to-snapshot gap, where the committed order stands over
  // the freshest data until the snapshot catches up (an exact id match) or a failed
  // write releases it.
  $effect(() => {
    const current = filtered;
    if (dndDragging) return;
    if (pendingOrder !== null) {
      const ids = current.map((s) => s.id);
      const landed =
        ids.length === pendingOrder.length &&
        ids.every((id, i) => id === pendingOrder![i]);
      if (landed) {
        pendingOrder = null;
      } else {
        dndItems = orderBy(current, pendingOrder);
        return;
      }
    }
    dndItems = current;
  });

  // The flip duration for the reflow, dropped to nothing when the operator has
  // asked for less motion. Shared by the dndzone and the `animate:flip` on each row.
  const reduceMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;
  const flipDurationMs = reduceMotion ? 0 : 120;

  // Dragging is inert while the filter box is non-empty. A drop position within a
  // filtered subset does not describe a position in the whole list, and mapping one
  // onto the other would invent a rule nothing else in the product makes — so the
  // drag is disabled instead. It gates the keyboard move as well, since both write
  // the whole list and so both need the whole list in view.
  const reorderable = $derived(filter.trim() === "");

  // The library reorders `dndItems` and hands it back: live during the drag
  // (consider) and once on the drop (finalize). Only the drop commits.
  function handleDndConsider(e: CustomEvent<DndEvent<Space>>) {
    dndDragging = true;
    dndItems = e.detail.items;
  }
  function handleDndFinalize(e: CustomEvent<DndEvent<Space>>) {
    dndItems = e.detail.items;
    dndDragging = false;
    const ids = e.detail.items.map((s) => s.id);
    // Hold the committed order on screen until the snapshot confirms it (the effect
    // above), so the drop does not bounce.
    pendingOrder = ids;
    void applyOrder(ids);
  }

  async function applyOrder(ids: string[]) {
    const current = spaces.map((s) => s.id);
    // A drop back where it started, or ⌥↑ on the first row: an ordinary outcome,
    // and the honest response is to write nothing at all.
    if (ids.length === current.length && ids.every((id, i) => id === current[i]))
      return;
    try {
      await reorderSpaces(ids);
    } catch (e) {
      // The write failed, so the held order is a fiction — drop it and let the
      // sidebar fall back to the server's truth.
      pendingOrder = null;
      actionError = `Couldn’t save the new order: ${(e as Error).message}`;
    }
  }

  // ⌥↑ / ⌥↓ move the selected space, emitting the same whole-list write a drag
  // does. `reorder` clamps, so a nudge past either end simply moves nothing.
  function moveSelected(delta: number) {
    if (!reorderable || !selected) return;
    const ids = spaces.map((s) => s.id);
    const i = ids.indexOf(selected.id);
    if (i < 0) return;
    void applyOrder(reorder(ids, i, i + delta));
  }

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

  // The rename editor: the space being renamed, the draft label its field holds,
  // and a handle to the field so opening the modal seats the caret in it with the
  // current name selected — overtyping is the common case. Renaming is
  // presentation only (it persists per-path and changes nothing on disk), so it
  // is a light one-field editor rather than a confirmation.
  let renaming = $state<Space | null>(null);
  let renameDraft = $state("");
  let renameInput = $state<HTMLInputElement | null>(null);

  function startRename(space: Space) {
    renaming = space;
    renameDraft = space.name;
  }

  async function confirmRename() {
    const space = renaming;
    if (!space) return;
    const name = renameDraft.trim();
    renaming = null;
    // Nothing to persist if the label is unchanged; an empty name clears the
    // override, so it is sent whenever it differs from what the card shows.
    if (name === space.name) return;
    try {
      await renameSpace(space.id, name);
    } catch (e) {
      actionError = `Couldn’t rename “${space.name}”: ${(e as Error).message}`;
    }
  }

  // The folder basename behind a space's path — the label a cleared rename falls
  // back to, shown as the field's placeholder so the empty-to-revert path reads.
  function baseName(path: string): string {
    return path.replace(/[/\\]+$/, "").split(/[/\\]/).pop() ?? path;
  }

  async function confirmForget() {
    const space = pendingForget;
    pendingForget = null;
    if (!space) return;
    if (selectedId === space.id) selectedId = null;
    // Its star-map pane state goes with it — nothing should be held, or written
    // to storage, for a space the cockpit no longer knows.
    forgetSpace(space.id);
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

  // Scratch is always present in the server snapshot even while its empty row is
  // hidden. The footer finds that flagged entry and uses the ordinary terminal
  // action; no registration or folder chooser is involved.
  async function openScratchShell() {
    const scratch = snapshotSpaces.find((space) => space.scratch);
    if (!scratch) return;
    await openShell(scratch);
  }

  // The new-shell control's agent rows (skill-sources ticket 08): a free session
  // on the agent the operator clicked — a live, ticketless tab told the free
  // payload, which shares only the spawn primitive with a real session, so it
  // opens exactly like a shell (no role picker, no ticket, nothing to gate on).
  // It names the agent that runs it (agent-selection ticket 03) and sends nothing
  // else: a free session takes no skill and no context line.
  async function freeSession(space: Space, agent: string) {
    selectSpace(space.id);
    opening = true;
    try {
      const { id } = await launchFree(space.id, agent);
      activeTermId = id;
    } catch (e) {
      actionError = `Couldn’t start a session on ${agent}: ${(e as Error).message}`;
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
  //
  // The card names the choice and this maps it to the call, so which endpoint each
  // verb reaches stays here beside every other action rather than being imported
  // into the card.
  const HALT_ACTIONS = {
    resume: resumeSession,
    respawn: respawnSession,
    release: releaseSession,
  } as const;

  // The halt choice the operator has been warned about and not yet answered:
  // resuming or respawning would seat a second live session in a space that already
  // has one (ADR 0003 as amended). Release never appears here — it clears a claim
  // and seats nothing, so it meets no such gate.
  let pendingHalt = $state<{
    space: Space;
    t: Terminal;
    verb: string;
    run: (
      spaceId: string,
      sessionId: string,
      force?: boolean,
    ) => Promise<unknown>;
  } | null>(null);

  async function haltAction(
    space: Space,
    t: Terminal,
    verb: string,
    run: (
      spaceId: string,
      sessionId: string,
      force?: boolean,
    ) => Promise<unknown>,
    force = false,
  ) {
    selectSpace(space.id);
    activeTermId = t.id;
    try {
      await run(space.id, t.id, force);
      pendingHalt = null;
    } catch (e) {
      // The one refusal the operator can overrule opens the warning; everything else
      // is a refusal of fact and lands as an error.
      if (e instanceof ActionError && e.code === LIVE_SESSION) {
        pendingHalt = { space, t, verb, run };
      } else {
        actionError = `Couldn’t ${verb} this session: ${(e as Error).message}`;
      }
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
  // own M/Esc (SpacePane.onKey). `[`/`]` cycle spaces in the operator's own
  // stored sidebar order, never the filtered view — a keyboard shortcut should
  // not depend on what's typed in the filter box.
  function onGlobalKey(e: KeyboardEvent) {
    // A pointer/keyboard reorder in flight is svelte-dnd-action's own modal
    // gesture now, and it cancels on Esc itself — the chrome no longer tracks a
    // drag to abandon here.
    if (isEditingTarget()) return;
    // ⌥↑ / ⌥↓ move the selected space in the sidebar (story 9) — the same write
    // the drag emits, so reordering is not mouse-only. It is read ahead of the
    // modifier bail-out below, being the one binding that *wants* a modifier:
    // ⌥ keeps it clear of the bare arrow keys the panes use, and ⌘/⌃ are the
    // platform's. Like every global binding it sits behind `isEditingTarget`, so
    // a keystroke aimed at a terminal, a text field or a dialog is never stolen.
    if (
      e.altKey &&
      !e.metaKey &&
      !e.ctrlKey &&
      (e.key === "ArrowUp" || e.key === "ArrowDown")
    ) {
      e.preventDefault();
      moveSelected(e.key === "ArrowDown" ? 1 : -1);
      return;
    }
    if (e.metaKey || e.ctrlKey || e.altKey) return;
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
  <div
    class="grid min-h-0 flex-1 grid-cols-[16rem_minmax(0,1fr)]"
    style={titleBarH ? `--bar-h: ${Math.max(titleBarH, 40)}px` : undefined}
  >
    <aside
      class="col-start-1 row-start-1 flex min-h-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
    >
      <!-- The brand and the active space now share the one top tier, matching
           the sketch's split header. In the native macOS shell this tier also
           fills the title-bar strip; the left inset leaves the real traffic-light
           buttons clear and seats the wordmark immediately beside them. -->
      <div class="brand-bar justify-start gap-2" class:pl-20={titleBarH > 0}>
        <img
          src="/brandmark.svg"
          alt=""
          width="20"
          height="20"
          class="size-5 shrink-0 grayscale"
        />
        <span class="truncate text-sm font-semibold tracking-tight">chartr</span
        >
      </div>

      <!-- Search is its own compact row below the brand, followed by the
           section label and the two global creation actions. This keeps adding
           spaces and opening Scratch close to the list they affect. -->
      <div class="flex items-center gap-2 px-2 pt-2">
        <div class="relative min-w-0 flex-1">
          <MagnifyingGlass
            aria-hidden="true"
            class="pointer-events-none absolute top-1/2 left-2 z-10 size-3.5 -translate-y-1/2 text-muted-foreground"
          />
          <Input
            type="text"
            class="h-8 pl-7"
            placeholder="Search"
            bind:value={filter}
            spellcheck="false"
            autocapitalize="off"
            autocomplete="off"
            aria-label="Filter spaces and sessions"
          />
        </div>
      </div>

      <div class="flex items-center justify-between gap-2 px-2 pt-2 pb-1">
        <span
          class="px-1 text-[0.65rem] font-semibold tracking-wide text-muted-foreground uppercase"
        >
          Spaces
        </span>
        <div class="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="icon-sm"
            disabled={opening || control.model === null}
            aria-label="Open a new Scratch shell"
            title="Open a new Scratch shell"
            onclick={openScratchShell}
          >
            {#if opening}
              <CircleNotch class="animate-spin" />
            {:else}
              <TerminalWindow />
            {/if}
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            disabled={picking || control.model === null}
            aria-label="Add a space"
            title="Add a space"
            aria-expanded={nativePicker ? undefined : showAdd}
            onclick={addSpace}
          >
            {#if picking}
              <CircleNotch class="animate-spin" />
            {:else}
              <Plus />
            {/if}
          </Button>
        </div>
      </div>

      {#if control.model === null}
        <p class="flex-1 px-3 py-2 text-xs text-muted-foreground">
          Connecting…
        </p>
      {:else if spaces.length === 0}
        <p class="flex-1 px-3 py-2 text-xs text-muted-foreground">
          No spaces yet.
        </p>
      {:else if filtered.length === 0}
        <p class="px-3 py-2 text-xs text-muted-foreground">
          No spaces match “{filter}”.
        </p>
      {:else}
        <!-- The reorder is svelte-dnd-action: the section is the drop zone, each
             row is an item keyed by space id, and `animate:flip` (same duration)
             does the slide. `consider` drives the live reflow, `finalize` commits.
             Disabled while filtering — a position within a subset does not describe
             one in the whole list. The default drop-zone outline is cleared; the
             chrome is monochrome and marks nothing with a raw colour. -->
        <section
          class="sidebar-scroll mr-1 flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-2 pr-1"
          use:dndzone={{
            items: dndItems,
            flipDurationMs,
            dragDisabled: !reorderable,
            dropTargetStyle: {},
            // Resolve the drop slot from the cursor, not the dragged card's
            // centre. Space cards vary a lot in height (a card with several
            // sessions dwarfs a bare one), and centre-based detection makes a tall
            // card's midpoint sweep across several short ones at once — the drop
            // overshoots and jumps rows. Cursor-based detection tracks the hand.
            useCursorForDetection: true,
          }}
          onconsider={handleDndConsider}
          onfinalize={handleDndFinalize}
        >
          {#each dndItems as space (space.id)}
            <div animate:flip={{ duration: flipDurationMs }}>
              <SpaceCard
                {space}
                {opening}
                selected={selected?.id === space.id}
                activeTermId={activeTerm?.id ?? null}
                onselect={() => selectSpace(space.id)}
                onselectsession={(t) => selectSession(space, t)}
                onjumphalt={() => jumpToHalt(space)}
                onforget={() => forget(space)}
                onrename={() => startRename(space)}
                onendshell={(t) => endShell(space, t)}
                onhalt={(t, verb) =>
                  haltAction(space, t, verb, HALT_ACTIONS[verb])}
                onopenshell={() => openShell(space)}
              />
            </div>
          {/each}
        </section>
      {/if}

      <!-- Settings is the persistent footer destination from the sketch. -->
      <div class="border-t border-sidebar-border p-2">
        <Button
          variant={route.settings ? "secondary" : "ghost"}
          size="sm"
          class="w-full justify-start"
          aria-pressed={route.settings}
          onclick={() => {
            if (route.settings) leaveSettings();
            else openSettings(lastSettingsScope ?? { kind: "default" });
          }}
        >
          <Gear /> Settings
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
          {canSyncSkills}
          terminalPrefs={control.model?.terminal}
          active={!route.settings}
          onOpenShell={() => openShell(selected)}
          onFreeSession={(agent) => freeSession(selected, agent)}
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
            spaces={scopeSpaces}
            config={configLayers}
            agents={agentLibrary}
            {detected}
            sources={control.model?.sources ?? []}
            roles={control.model?.roles ?? []}
            gitAvailable={control.model?.gitAvailable ?? false}
            terminalPrefs={control.model?.terminal}
            notifyPrefs={control.model?.notify}
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
      No folder chooser was found on this machine, so point chartr at a project
      folder by pasting its absolute path. If it isn’t a git repository yet, one
      is initialized there, announced.
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
      <Button
        variant="outline"
        size="sm"
        onclick={() => (pendingForget = null)}
      >
        Cancel
      </Button>
      <Button variant="destructive" size="sm" onclick={confirmForget}>
        Remove
      </Button>
    </div>
  </Modal>

  <!-- Renaming a space is presentation only — it changes the label the sidebar
       shows and nothing on disk — so it is a light one-field editor rather than a
       confirmation. The caret opens in the field with the current name selected;
       submitting empty clears the custom name back to the folder basename. -->
  <Modal
    open={renaming !== null}
    title="Rename “{renaming?.name ?? ''}”"
    onClose={() => (renaming = null)}
    onOpenAutoFocus={(e) => {
      e.preventDefault();
      renameInput?.focus();
      renameInput?.select();
    }}
  >
    <form
      onsubmit={(e) => {
        e.preventDefault();
        confirmRename();
      }}
    >
      <Input
        bind:ref={renameInput}
        bind:value={renameDraft}
        placeholder={renaming ? baseName(renaming.path) : ""}
        aria-label="Space name"
      />
      <div class="mt-4 flex justify-end gap-2">
        <Button
          variant="outline"
          size="sm"
          type="button"
          onclick={() => (renaming = null)}
        >
          Cancel
        </Button>
        <Button variant="default" size="sm" type="submit">Save</Button>
      </div>
    </form>
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

  <!-- Resuming or respawning into a space that already has a live session runs
       both agents in one working tree (ADR 0003 as amended). Same warning the
       spawn path shows, at the halt's own gate. -->
  <Modal
    open={pendingHalt !== null}
    title="This space already has a live session"
    onClose={() => (pendingHalt = null)}
  >
    <div class="space-y-4 text-sm">
      <p class="text-muted-foreground">
        {pendingHalt ? `To ${pendingHalt.verb} this session` : "This"} would run
        two agents in
        <strong class="font-medium text-foreground"
          >the same working tree</strong
        >. There is no branch or worktree between them, so they can overwrite
        each other's uncommitted edits with no conflict to resolve.
      </p>
      <div class="flex justify-end gap-2">
        <Button variant="ghost" size="sm" onclick={() => (pendingHalt = null)}
          >Cancel</Button
        >
        <Button
          variant="default"
          size="sm"
          onclick={() => {
            const p = pendingHalt;
            if (p) haltAction(p.space, p.t, p.verb, p.run, true);
          }}
        >
          {pendingHalt
            ? `${pendingHalt.verb[0].toUpperCase()}${pendingHalt.verb.slice(1)} anyway`
            : "Continue"}
        </Button>
      </div>
    </div>
  </Modal>

  <!-- The one toast surface, centered over the main panel (app.css offsets it
       past the sidebar). Every transient notice — the register outcome here, the
       source and agent form refusals — surfaces through it now. -->
  <Toaster />
</div>
