<script lang="ts">
  import type { Space, Terminal, Agent } from "./model";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { Button } from "./components/ui/button";
  import Marquee from "./Marquee.svelte";
  import NewShellButton from "./NewShellButton.svelte";
  import * as ContextMenu from "./components/ui/context-menu";
  import { spaceAttention, spaceLiveness } from "./attention";
  import { showsFinishedDot } from "./unseen";
  import {
    X,
    XCircle,
    Circle,
    CircleNotch,
    Play,
    Plus,
    ArrowClockwise,
    ArrowUUpLeft,
    Warning,
    PauseCircle,
    PencilSimple,
    Skull,
    Spinner,
  } from "phosphor-svelte";

  // The three choices a dead session offers. The card names the choice; the
  // chrome above owns which action each one runs, so the halt actions stay
  // beside every other action call rather than being imported here.
  type HaltVerb = "resume" | "respawn" | "release";

  let {
    space,
    selected,
    activeTermId,
    opening = false,
    onselect,
    onselectsession,
    onjumphalt,
    onforget,
    onrename,
    onendshell,
    onhalt,
    onopenshell,
    onfree,
    onregister,
    onreorder,
    agents,
  }: {
    space: Space;
    // Whether this card is the chrome's current selection. Passed rather than
    // derived: selection is the shell's business, and a card only reports the
    // click that would change it.
    selected: boolean;
    // The shell that is actually showing, already resolved by the chrome (which
    // falls back to a space's first terminal when the active id is stale). A row
    // reads active only within the selected card, so a background space never
    // paints one.
    activeTermId: string | null;
    // A free session or shell-open is in flight somewhere in the chrome; the
    // control disables while it is, so one click cannot start two tabs.
    opening?: boolean;
    onselect: () => void;
    onselectsession: (t: Terminal) => void;
    // The halt flag doubles as the jump: the chrome deep-links the halted ticket.
    onjumphalt: () => void;
    onforget: () => void;
    // Right-click → Rename: the chrome owns the little name editor and the action,
    // so the card only reports the intent, the same way the forget button does.
    onrename: () => void;
    onendshell: (t: Terminal) => void;
    onhalt: (t: Terminal, verb: HaltVerb) => void;
    onopenshell: () => void;
    // A free session on the agent the operator picked from the shell control's
    // caret — launched bare, exactly as the space pane's control does it.
    onfree: (agent: string) => void;
    // Where an empty agent library routes the operator: settings.
    onregister: () => void;
    // A drop (or a keyboard move) reordered this space's session tabs: the whole
    // ordered list of the card's terminal ids, scoped to this card — the chrome
    // posts it, exactly as it posts a sidebar reorder. The card holds the new order
    // on screen until the confirming snapshot lands (below); a rejected promise
    // means the write failed, and the card drops the held order back to the
    // server's truth.
    onreorder: (ids: string[]) => void | Promise<void>;
    // The registered agent library, for the shell control's caret list.
    agents: Agent[];
  } = $props();

  // Zero-pad a ticket number for a session row's label (#01), matching the detail
  // pane's ticket ids.
  function pad(n: number): string {
    return n < 10 ? "0" + n : String(n);
  }

  const attention = $derived(spaceAttention(space));
  const liveness = $derived(spaceLiveness(space));

  // Two reorder gestures nest here, both svelte-dnd-action, both the same method.
  // The *outer* zone lives in the sidebar (App.svelte): the whole card is one drag
  // item and the click target for selecting the space. The *inner* zone is the
  // session list below — the card owns it, so a space's tabs are the operator's to
  // arrange within the one card. The library keeps the two apart: a press-and-drag
  // that starts on a session row moves the row, not the card. A click that never
  // moved is still the select.
  //
  // The inner reorder mirrors the sidebar's exactly (App.svelte's dndItems dance):
  // the library owns the list live during a drag, the drop writes the whole list
  // and holds it on screen until the confirming snapshot arrives, and an incoming
  // snapshot never yanks the rows out from under the pointer. Scoped to this card,
  // so `space.terminals` is both the source of truth and the id set the drag can
  // only ever permute — a tab never leaves for another space.
  let dndItems = $state<Terminal[]>([]);
  // True between the first `consider` and the `finalize`: the window in which the
  // library, not an incoming snapshot, owns `dndItems`.
  let dragging = $state(false);
  // The order a just-dropped move committed, held until the confirming snapshot
  // shows it — without it the rows would snap back to the pre-drop order the instant
  // the drag ends, then forward again when it lands.
  let pendingOrder = $state<string[] | null>(null);

  // Reorder a list by a list of ids, dropping ids that have since left and
  // appending any that arrived — so a snapshot landing mid-hold can neither drop
  // nor duplicate a row.
  function orderBy(list: Terminal[], ids: string[]): Terminal[] {
    const byId = new Map(list.map((t) => [t.id, t]));
    const held = ids.filter((id) => byId.has(id)).map((id) => byId.get(id)!);
    const extra = list.filter((t) => !ids.includes(t.id));
    return [...held, ...extra];
  }

  // Keep `dndItems` in step with the space's terminals — except while the library
  // holds it (a snapshot must not yank the rows out from under the pointer), and
  // except across the drop-to-snapshot gap, where the committed order stands over
  // the freshest data until the snapshot catches up (an exact id match) or a failed
  // write releases it.
  $effect(() => {
    const current = space.terminals;
    if (dragging) return;
    if (pendingOrder !== null) {
      const ids = current.map((t) => t.id);
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
  // asked for less motion — shared by the dndzone and the rows' `animate:flip`.
  const reduceMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;
  const flipDurationMs = reduceMotion ? 0 : 120;

  // Nothing to arrange with fewer than two tabs, so the drag stays off — a lone
  // row gains no grab affordance it could do nothing with.
  const reorderable = $derived(space.terminals.length > 1);

  // The library reorders `dndItems` and hands it back: live during the drag
  // (consider) and once on the drop (finalize). Only the drop commits.
  function handleSessionConsider(e: CustomEvent<DndEvent<Terminal>>) {
    dragging = true;
    dndItems = e.detail.items;
  }
  function handleSessionFinalize(e: CustomEvent<DndEvent<Terminal>>) {
    dndItems = e.detail.items;
    dragging = false;
    const ids = e.detail.items.map((t) => t.id);
    const current = space.terminals.map((t) => t.id);
    // A drop back where it started (or a keyboard move that clamped): the honest
    // response is to write nothing, and to hold nothing.
    if (
      ids.length === current.length &&
      ids.every((id, i) => id === current[i])
    )
      return;
    // Hold the committed order on screen until the snapshot confirms it (the effect
    // above), so the drop does not bounce.
    pendingOrder = ids;
    void Promise.resolve(onreorder(ids)).catch(() => {
      // The write failed, so the held order is a fiction. A refusal pushes no
      // snapshot to fall back on, so snap the rows back to the space's own
      // terminals here rather than waiting for one that isn't coming.
      pendingOrder = null;
      dndItems = space.terminals;
    });
  }
</script>

<!-- The card plate is a snippet so it can render either bare (Scratch, which
     has no repository to act on) or wrapped in a right-click menu (every
     registered space). The `data-space-id` and the whole drag gesture live on
     the plate itself, so the sidebar's hit test and reorder are unchanged
     whichever way it renders. -->
{#snippet cardPlate()}
  <!-- One space, a faintly filled plate on the sidebar surface (its own
     token family — not the bg-card content surface). The fill, not a
     hairline, is what separates one card from the next: the sidebar
     carries a card per space and a row per session, and a border on
     every one of them turned the list into a grid. The border is kept
     for exactly one job — saying which space is selected — so it means
     something when it appears. Transparent rather than absent when it
     is not, so gaining it never nudges the list by a pixel.

     The whole card is the selection target — clicking anywhere that
     isn't its own control selects the space — so the identity, its
     sessions and its actions all read as one object rather than a
     header you must aim at. Selected emphasis rides --primary, the one
     emphasis token; the chrome is monochrome. Because the whole card is a click target,
     it is `select-none` throughout. Dragging a selection across a thing
     you click is noise. The path stays the card's tooltip. -->
  <div
    role="button"
    tabindex="0"
    aria-pressed={selected}
    aria-label="Select {space.name}"
    title={space.path}
    data-space-id={space.id}
    class={[
      "relative flex flex-col rounded-lg border p-1 transition-colors select-none",
      // The whole card is both the drag item (svelte-dnd-action, in the sidebar)
      // and the click target that selects the space — a click that never moves.
      "cursor-pointer",
      selected
        ? "border-primary/60 bg-sidebar-accent/50"
        : "border-transparent bg-sidebar-accent/25 hover:bg-sidebar-accent/40",
    ]}
    onclick={onselect}
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        // Keep the drop zone's own keyboard-drag (Space to lift) from also firing
        // when the card itself is focused: here, Enter/Space selects the space.
        e.stopPropagation();
        onselect();
      }
    }}
  >
    <!-- Identity: the space's name, with the open-shell and forget actions
       pinned top-right, beside the title rather than on a row of their own
       — the `+` is the one action a card exists to offer, so it rides where
       the eye already lands. Ambient cross-space attention (ticket 14, story
       8) rides on the name line — a wants-you flag (a session halted) and a
       liveness dot, both echoing the same signals the queue pulls and the
       session cards below already carry in detail. Neither ever re-sorts the
       card; muscle memory over this list holds — and now the operator's
       own arrangement is the only thing that sets it. -->
    <div class="flex items-start gap-1">
      <span
        class="flex p-1 min-w-0 flex-1 items-center gap-1.5 text-xs font-semibold"
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
              onjumphalt();
            }}
            onkeydown={(e) => {
              // The card handles Enter/Space itself and preventDefaults
              // it; let the button's own activation win instead.
              if (e.key === "Enter" || e.key === " ") e.stopPropagation();
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
      <!-- Scratch cannot be removed — it is rebuilt from nothing on every run — so
         it carries no open-shell control. Removing a space lives on the
         right-click menu now (rename/close), not a button on the card. The `+`
         is the body of the same split control the space pane carries: it opens a
         plain shell, and its caret lists the registered agents to launch a free
         session on — icon-only here, where the card header has no room for a
         label. -->
      {#if !space.scratch}
        <span class="-mt-0.5 -mr-0.5 flex shrink-0 items-center p-1">
          <NewShellButton
            {agents}
            variant="ghost"
            size="icon-xs"
            disabled={opening}
            ariaLabel="Open a shell in {space.name}"
            title="Open a plain shell in {space.name} — nothing is injected."
            onshell={onopenshell}
            {onfree}
            {onregister}
          >
            <Plus />
          </NewShellButton>
        </span>
      {/if}
    </div>

    <!-- Sessions: the space's open shells, one line each — status, name,
       close — on no border at all. A session is a *row in a list*, not a
       card within a card, and the second line it used to carry (agent ·
       status) said in words what the status glyph says in a shape; it is
       the row's tooltip now. The identity keeps the width it needs to
       truncate late. Clicking one selects the space *and* switches to
       that session, the one click that does both, and the fill — the
       same emphasis the card takes, one step stronger — is what says
       which row is showing. -->
    {#if space.terminals.length}
      <!-- The session tabs are a svelte-dnd-action zone of their own — the same
         method the sidebar reorders spaces with, nested one level down and scoped
         to this card. Each row is an item keyed by its terminal id; `consider`
         drives the live reflow, `finalize` commits, and `animate:flip` (same
         duration) does the slide. The default drop outline is cleared — the chrome
         is monochrome — and cursor detection tracks the hand rather than a row's
         midpoint, matching the sidebar. The whole zone is a nested drag surface, so
         `onpointerdown` stops the press from reaching the card's own select.

         A session belongs to exactly one space and never moves between them, so
         each card's list gets its *own* zone type keyed by the space id. Zones of
         different types do not interconnect: a dragged row cannot leave for another
         card (or vanish into the gap between them, which is what a shared type let
         it do) — it only ever reorders within the card it started in. -->
      <ul
        class="flex flex-col gap-0.5"
        onpointerdown={(e) => {
          if (reorderable) e.stopPropagation();
        }}
        use:dndzone={{
          items: dndItems,
          type: `sessions-${space.id}`,
          flipDurationMs,
          dragDisabled: !reorderable,
          dropFromOthersDisabled: true,
          dropTargetStyle: {},
          useCursorForDetection: true,
        }}
        onconsider={handleSessionConsider}
        onfinalize={handleSessionFinalize}
      >
        {#each dndItems as t (t.id)}
          {@const isActive = selected && activeTermId === t.id}
          {@const unseen = showsFinishedDot(t, isActive)}
          <!-- The finished-while-you-were-away signal (session-notifications) is
             carried by the *status glyph* rather than a second dot beside it:
             the mark fills and takes `--primary`. One line has room for one
             mark, and the two were always about the same tab — the status says
             what it is doing, the fill says you have not looked since it
             stopped. Focusing the tab clears it — there is no dismiss — which
             is why it never shows on the tab in view, and the label carries
             both halves so a screen reader hears the difference too. -->
          {@const weight = unseen ? "fill" : "regular"}
          {@const away = unseen ? " — finished while you were away" : ""}
          {@const tone = unseen
            ? "text-primary"
            : t.status === "blocked"
              ? "text-primary"
              : t.status === "exited"
                ? "text-destructive"
                : "text-muted-foreground"}
          <li animate:flip={{ duration: flipDurationMs }}>
            <div
              role="button"
              tabindex="0"
              aria-pressed={isActive}
              data-session-row
              title={t.session
                ? `${t.session.agent} · ${t.status}`
                : `${t.proc} · ${t.status}`}
              class={[
                "group flex items-center gap-1 rounded-md px-0.5 py-1 transition-colors",
                isActive
                  ? "bg-sidebar-accent text-sidebar-accent-foreground"
                  : "hover:bg-sidebar-accent/60",
              ]}
              onclick={(e) => {
                e.stopPropagation();
                onselectsession(t);
              }}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  e.stopPropagation();
                  onselectsession(t);
                }
              }}
            >
              <!-- Status indicator, pinned to the row's leading edge so every tab's
                 state reads down a single column regardless of how its name or title
                 wraps. It is always present — idle shows a filled neutral dot, so no
                 row is ever missing its mark and the column never gaps. A tab with no
                 known agent in front: that neutral dot when idle, a muted static
                 pulse mark while a plain process holds the foreground (a dev server,
                 a build — running, not an agent working), an error mark once it
                 exits. A tab with a known agent reads the agent's own broadcast
                 state — the primary spinner racing while it works, plus a held pause
                 mark when it is blocked waiting on its human. The fast primary
                 spinner and the slowly turning muted asterisk are what tell an agent
                 churning a task apart from a process that merely runs. A dead session
                 freezes under a grey mark. Working is the one state that cannot also
                 be unseen — nothing has finished yet — so it alone keeps a fixed
                 weight. -->
              <!-- Fixed-size slot so the glyph occupies the same box in every
                 state: the mark swaps but the column before the name never
                 reflows. -->
              <span class="flex size-3.5 shrink-0 items-center justify-center">
                {#if t.status === "working"}
                  <Spinner
                    class="size-3.5 animate-spin [animation-duration:2s] text-primary"
                    aria-label="working"
                  />
                {:else if t.status === "running"}
                  <CircleNotch
                    class="size-3.5 animate-spin [animation-duration:5s] text-muted-foreground"
                    aria-label="running"
                  />
                {:else if t.status === "blocked"}
                  <PauseCircle
                    class="size-3.5 {tone}"
                    {weight}
                    aria-label="blocked{away}"
                  />
                {:else if t.status === "dead"}
                  <Skull
                    class="size-3.5 {tone}"
                    {weight}
                    aria-label="dead{away}"
                  />
                {:else if t.status === "exited"}
                  <XCircle
                    class="size-3.5 {tone}"
                    {weight}
                    aria-label="exited{away}"
                  />
                {:else}
                  <Circle
                    class="size-2.5 text-muted-foreground/10"
                    weight="fill"
                    aria-label="idle"
                  />
                {/if}
              </span>

              <!-- The tab's identity, and after it the generated conversation
                 title: the label keeps the width it needs to read, the title takes
                 what's left and scrolls only when it overflows. The title rides
                 after whatever label the row already shows — the ticket for a
                 session, the foreground agent (claude/codex) for an ad-hoc shell —
                 and is present only on a tab whose agent produced one. -->
              <span class="flex min-w-0 flex-1 items-center gap-1.5">
                {#if t.session}
                  <!-- A session: its identity is the ticket it is bound to
                     (role · #num) — told apart from an ad-hoc shell, which
                     shows its foreground process. -->
                  <span
                    class="max-w-[60%] shrink-0 truncate text-xs font-medium"
                    >{t.session.role} #{pad(t.session.ticketNum)}</span
                  >
                {:else}
                  <span class="max-w-[60%] shrink-0 truncate font-mono text-xs"
                    >{t.proc}</span
                  >
                {/if}
                {#if t.autoTitle}
                  <Marquee
                    text={t.autoTitle}
                    class="min-w-0 flex-1 text-[10px] leading-none text-muted-foreground"
                  />
                {/if}
              </span>

              {#if t.session && !t.alive}
                <!-- The death halt: a dead session is pinned to its ticket and
                   offers exactly three choices — resume it (crash recovery),
                   respawn a fresh session, or release the claim. chartr takes
                   none itself. Three buttons is a lot for one line, so they
                   ride *inside* it rather than on a second row that would
                   exist for the one state in five: the identity beside them
                   truncates instead, and it is the only row in the list where
                   the name gives up width. -->
                <span class="-my-0.5 flex shrink-0 items-center">
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="hover:text-primary"
                    aria-label="Resume this session"
                    title="Attempt to resume session."
                    onclick={(e) => {
                      e.stopPropagation();
                      onhalt(t, "resume");
                    }}
                  >
                    <Play />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="hover:text-primary"
                    aria-label="Respawn a fresh session"
                    title="Respawn a fresh session on the same ticket."
                    onclick={(e) => {
                      e.stopPropagation();
                      onhalt(t, "respawn");
                    }}
                  >
                    <ArrowClockwise />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="hover:text-destructive"
                    aria-label="Release the claim"
                    title="Release the ticket claim back to the frontier."
                    onclick={(e) => {
                      e.stopPropagation();
                      onhalt(t, "release");
                    }}
                  >
                    <ArrowUUpLeft />
                  </Button>
                </span>
              {/if}

              <!-- The close button stays out of the way on a background tab and
                 surfaces only where ending one is a deliberate act: the row the
                 pointer is over, or the tab that is selected and in view. A row of
                 always-present ✕es across every background tab read as clutter and
                 invited a mis-click, so it is shown on the active row and fades in
                 on hover / keyboard focus everywhere else.

                 The wrapper always reserves the button's width, so nothing shifts
                 when the ✕ appears — only its opacity animates (motion-reduce drops
                 the transition). It stays in the DOM the whole time; while hidden it
                 is opacity-0 and pointer-events-none so an invisible ✕ can't be
                 mis-clicked. -->
              <span
                class={[
                  "-my-0.5 flex w-5 shrink-0 overflow-hidden transition-opacity duration-100 ease-out motion-reduce:transition-none",
                  isActive
                    ? "opacity-100"
                    : "pointer-events-none opacity-0 group-hover:pointer-events-auto group-hover:opacity-100 focus-within:pointer-events-auto focus-within:opacity-100",
                ]}
              >
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="shrink-0 hover:text-destructive"
                  aria-label="End {t.proc}"
                  title={t.session ? "End this session" : "End this shell"}
                  onclick={(e) => {
                    e.stopPropagation();
                    onendshell(t);
                  }}
                >
                  <X />
                </Button>
              </span>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/snippet}

{#if space.scratch}
  <!-- Scratch is rebuilt from nothing on every run, so there is nothing to
       rename and nothing to close — it renders as a bare plate with no menu. -->
  {@render cardPlate()}
{:else}
  <!-- Right-clicking the plate opens the space's actions. `class="contents"`
       keeps the trigger out of layout entirely — the plate stays the sidebar's
       flex item and its measured `[data-space-id]` row — so adding the menu
       changes nothing about spacing, dragging, or the drop indicator. The
       contextmenu event bubbles from the plate up to this ancestor, so the
       card's own pointer/click handlers are left completely untouched. -->
  <ContextMenu.Root>
    <ContextMenu.Trigger class="contents">
      {@render cardPlate()}
    </ContextMenu.Trigger>
    <ContextMenu.Content class="w-40">
      <ContextMenu.Item onSelect={onrename}>
        <PencilSimple />
        Rename
      </ContextMenu.Item>
      <ContextMenu.Item variant="destructive" onSelect={onforget}>
        <X />
        Close
      </ContextMenu.Item>
    </ContextMenu.Content>
  </ContextMenu.Root>
{/if}
