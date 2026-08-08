<script lang="ts">
  import type { Space, Terminal } from "./model";
  import type { DropEdge } from "./reorder";
  import { Button } from "./components/ui/button";
  import { spaceAttention, spaceLiveness } from "./attention";
  import { showsFinishedDot } from "./unseen";
  import {
    X,
    Circle,
    XCircle,
    CircleNotch,
    Play,
    Plus,
    ArrowClockwise,
    ArrowUUpLeft,
    Warning,
    PauseCircle,
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
    reorderable = false,
    dragged = false,
    dropEdge = null,
    ongrabstart,
    ongrabmove,
    ongrabdrop,
    ongrabend,
    onselect,
    onselectsession,
    onjumphalt,
    onforget,
    onendshell,
    onhalt,
    onopenshell,
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
    // Whether this row can be dragged to a new place in the sidebar. It is the
    // chrome's call, not the card's: a drop position inside a *filtered* sidebar
    // does not describe a position in the whole list, so the handles go inert
    // while the filter box is non-empty rather than inventing a mapping.
    reorderable?: boolean;
    // This card is the one being dragged: it dims and rides the pointer, so the
    // indicator below reads as where it is going rather than where a second row
    // would go. It is the *chrome's* answer, not this card's own `dragging` — so
    // a cancelled drag (Esc) drops the lift on the spot while the operator is
    // still holding the button.
    dragged?: boolean;
    // Which edge of this row the drop would land on, or null for no indicator.
    // The row must show where it will land *before* the drop, not only after it.
    dropEdge?: DropEdge | null;
    // The gesture reports the pointer's viewport Y and nothing else: which row
    // that lands on is the sidebar's question, because only the sidebar knows the
    // list. See the drag block below for why the card cannot answer it.
    ongrabstart: (clientY: number) => void;
    ongrabmove: (clientY: number) => void;
    ongrabdrop: () => void;
    ongrabend: () => void;
    onselect: () => void;
    onselectsession: (t: Terminal) => void;
    // The halt flag doubles as the jump: the chrome deep-links the halted ticket.
    onjumphalt: () => void;
    onforget: () => void;
    onendshell: (t: Terminal) => void;
    onhalt: (t: Terminal, verb: HaltVerb) => void;
    onopenshell: () => void;
  } = $props();

  // Zero-pad a ticket number for a session row's label (#01), matching the detail
  // pane's ticket ids.
  function pad(n: number): string {
    return n < 10 ? "0" + n : String(n);
  }

  const attention = $derived(spaceAttention(space));
  const liveness = $derived(spaceLiveness(space));

  // --- The reorder drag ------------------------------------------------------
  //
  // Pointer events with pointer capture, not HTML5 drag-and-drop. The API this
  // replaces only delivers a `drop` to an element that preventDefault'd its own
  // `dragover`, which made the *cards* the only surface that could commit a move:
  // releasing over the 8px gutter between two rows — the strip the drop indicator
  // is drawn in — over the terminal pane, or off the window entirely all arrived
  // as `dragend` and threw the move away. Under capture the release is ours
  // wherever it happens, so a drag commits to the last position it showed.
  //
  // The card reports the pointer's Y and never resolves it: capture routes every
  // move to the row that was grabbed, not the row under the cursor, so the card
  // the operator is *over* sees nothing. Only the sidebar knows the list, so the
  // sidebar does the hit test.

  // How far the pointer travels before a press becomes a drag. Below it the
  // gesture is still a click that selects the space — the whole card is both the
  // drag source and the selection target, as it was, and the threshold is what
  // keeps one from eating the other.
  const DRAG_THRESHOLD = 4;

  let cardEl = $state<HTMLElement | null>(null);
  // The press that may become a drag: where it started and which pointer owns it.
  // Non-null from pointerdown, which is *before* there is a drag — `dragging` is
  // what says the threshold has been crossed.
  let press: { x: number; y: number; id: number } | null = null;
  let dragging = $state(false);
  // How far the pointer has moved since the grab, so the row can ride it. Purely
  // cosmetic: the drop position comes from the pointer, never from this offset.
  let liftY = $state(0);
  // A drag ends in a `click` on the card (capture makes it the target), and that
  // click would select the space the operator just finished moving. Set on any
  // release that was a real drag, and cleared by the next press so an undelivered
  // click can never eat a later, genuine one.
  let suppressClick = false;

  function releasePointer() {
    if (press !== null && cardEl?.hasPointerCapture(press.id)) {
      cardEl.releasePointerCapture(press.id);
    }
    press = null;
    dragging = false;
    liftY = 0;
  }

  function onPointerDown(e: PointerEvent) {
    suppressClick = false;
    if (!reorderable || !e.isPrimary || e.button !== 0) return;
    // Controls inside the card own their own gestures — the forget button, the
    // new-shell action, and the session rows, which are click
    // targets that switch the shell in view. A press on one is never a grab, and
    // each shows the pointer rather than the card's open hand: the affordance
    // and what the press actually does have to agree.
    if (
      (e.target as HTMLElement).closest(
        "button, a, input, select, textarea, [data-session-row]",
      )
    ) {
      return;
    }
    press = { x: e.clientX, y: e.clientY, id: e.pointerId };
  }

  function onPointerMove(e: PointerEvent) {
    if (press === null || e.pointerId !== press.id) return;
    if (!dragging) {
      if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < DRAG_THRESHOLD) {
        return;
      }
      dragging = true;
      cardEl?.setPointerCapture(press.id);
      ongrabstart(e.clientY);
      return;
    }
    // The button came up somewhere the browser never told us about — another
    // application's window took the release. Treat it as the drop it was: the
    // operator let go, and the position on screen was the answer.
    if (e.buttons === 0) {
      finishDrag();
      return;
    }
    liftY = e.clientY - press.y;
    ongrabmove(e.clientY);
  }

  // A release *by the pointer that grabbed the row* commits it; a second finger
  // coming up is not this gesture ending, so it is ignored rather than allowed to
  // drop a row the first one is still holding.
  function onPointerUp(e: PointerEvent) {
    if (press === null || e.pointerId !== press.id) return;
    finishDrag();
  }

  function onPointerCancel(e: PointerEvent) {
    if (press === null || e.pointerId !== press.id) return;
    const wasDragging = dragging;
    releasePointer();
    if (!wasDragging) return;
    // The gesture was taken away rather than completed — the row goes back where
    // it was, which is the whole of the cancel: nothing was moved optimistically.
    suppressClick = true;
    ongrabend();
  }

  function finishDrag() {
    const wasDragging = dragging;
    releasePointer();
    // A press that never crossed the threshold is a click, and the click handler
    // below is left to select the space exactly as it did before.
    if (!wasDragging) return;
    suppressClick = true;
    ongrabdrop();
  }

</script>

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
  bind:this={cardEl}
  role="button"
  tabindex="0"
  aria-pressed={selected}
  aria-label="Select {space.name}"
  title={space.path}
  data-space-id={space.id}
  class={[
    "relative flex flex-col gap-2 rounded-lg border p-2 select-none",
    // The whole card *is* the grab target, so the whole card carries the
    // affordance: an open hand that closes while a row is held, the same tell
    // the star-map's empty space uses. There is no separate grip icon — the
    // cursor is the discovery, over a target the size of the card rather than a
    // 14px handle. The card's own click targets opt back out and are excluded
    // from the grab below, so a hand never appears over something that cannot be
    // dragged. `active:` covers the press before the threshold, `dragging` holds
    // the closed hand once the row has left the pointer behind, and the keyboard
    // path (Alt+↑/↓ on the selected space) is unchanged.
    reorderable
      ? "cursor-grab active:cursor-grabbing [&_button]:cursor-pointer [&_[data-session-row]]:cursor-pointer"
      : "cursor-pointer",
    dragging && "cursor-grabbing",
    selected
      ? "border-primary/60 bg-sidebar-accent/50"
      : "border-transparent bg-sidebar-accent/25 hover:bg-sidebar-accent/40",
    // The lift is driven frame by frame from the pointer, so the row must not
    // also be easing towards it — transitions are dropped for its duration and
    // the colour transition stands the rest of the time.
    dragged
      ? "z-10 opacity-60 shadow-lg"
      : "transition-colors",
  ]}
  style={dragged ? `transform: translateY(${liftY}px)` : undefined}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerCancel}
  onclick={() => {
    if (suppressClick) {
      suppressClick = false;
      return;
    }
    onselect();
  }}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onselect();
    }
  }}
>
  {#if dropEdge}
    <!-- Where the row will land, drawn before the drop rather than only after
         it (story 8). It rides the emphasis roles the chrome reserves —
         `--primary` for the bar, `--ring` for the halo that lifts it off the
         sidebar surface — and sits absolutely in the gap between cards, so
         showing it never nudges the list the operator is aiming at. -->
    <div
      aria-hidden="true"
      class={[
        "pointer-events-none absolute right-0 left-0 h-0.5 rounded-full bg-primary ring-2 ring-ring/30",
        dropEdge === "before" ? "-top-1.5" : "-bottom-1.5",
      ]}
    ></div>
  {/if}

  <!-- Identity: the space's name, with the forget action pinned top-right.
       Ambient cross-space attention (ticket 14, story 8) rides on the name
       line — a wants-you flag (a session halted) and a liveness dot,
       both echoing the same signals the queue pulls and the session
       cards below already carry in detail. Neither ever re-sorts the
       card; muscle memory over this list holds — and now the operator's
       own arrangement is the only thing that sets it. -->
  <div class="flex items-start gap-1">
    <span
      class="flex min-w-0 flex-1 items-center gap-1.5 text-xs font-semibold"
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
         it carries no forget control rather than one that would be refused. -->
    {#if !space.scratch}
      <Button
        variant="ghost"
        size="icon-xs"
        class="-mt-0.5 -mr-0.5 hover:text-destructive"
        aria-label="Remove space"
        title="Remove from this list (your files stay put)"
        onclick={(e) => {
          e.stopPropagation();
          onforget();
        }}
      >
        <X />
      </Button>
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
    <ul class="flex flex-col">
      {#each space.terminals as t (t.id)}
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
        <li>
          <div
            role="button"
            tabindex="0"
            aria-pressed={isActive}
            data-session-row
            title={t.session
              ? `${t.session.agent} · ${t.status}`
              : `${t.proc} · ${t.status}`}
            class={[
              "flex items-center gap-1.5 rounded-md px-1.5 py-1 transition-colors",
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
            <!-- Status indicator. A tab with no known agent in front: a
                 spinner while working, an open ring idle, an error mark once
                 it exits. A tab with a known agent reads the agent's own
                 broadcast state — the same spinner and ring, plus a held
                 pause mark when it is blocked waiting on its human. A dead
                 session freezes under a grey mark. Working is the one state
                 that cannot also be unseen — nothing has finished yet — so it
                 alone keeps a fixed weight. -->
            {#if t.status === "working"}
              <CircleNotch
                class="size-3.5 shrink-0 animate-spin text-primary"
                aria-label="working"
              />
            {:else if t.status === "blocked"}
              <PauseCircle
                class="size-3.5 shrink-0 {tone}"
                {weight}
                aria-label="blocked{away}"
              />
            {:else if t.status === "dead"}
              <XCircle
                class="size-3.5 shrink-0 {tone}"
                {weight}
                aria-label="dead{away}"
              />
            {:else if t.status === "exited"}
              <XCircle
                class="size-3.5 shrink-0 {tone}"
                {weight}
                aria-label="exited{away}"
              />
            {:else}
              <Circle
                class="size-3.5 shrink-0 {tone}"
                {weight}
                aria-label="idle{away}"
              />
            {/if}

            {#if t.session}
              <!-- A session: its identity is the ticket it is bound to
                   (role · #num) — told apart from an ad-hoc shell, which
                   shows its foreground process. -->
              <span class="min-w-0 flex-1 truncate text-xs font-medium"
                >{t.session.role} #{pad(t.session.ticketNum)}</span
              >
            {:else}
              <span class="min-w-0 flex-1 truncate font-mono text-xs"
                >{t.proc}</span
              >
            {/if}

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
                  title="Resume — same-ticket crash recovery"
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
                  title="Respawn — a fresh session on the same ticket"
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
                  title="Release — clear the claim back to the frontier"
                  onclick={(e) => {
                    e.stopPropagation();
                    onhalt(t, "release");
                  }}
                >
                  <ArrowUUpLeft />
                </Button>
              </span>
            {/if}

            <Button
              variant="ghost"
              size="icon-xs"
              class="-my-0.5 -mr-0.5 shrink-0 hover:text-destructive"
              aria-label="End {t.proc}"
              title={t.session ? "End this session" : "End this shell"}
              onclick={(e) => {
                e.stopPropagation();
                onendshell(t);
              }}
            >
              <X />
            </Button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- The one ticketless on-ramp: a `+` that opens a plain shell, aligned to
       the card's right edge. Branch information deliberately does not appear in
       the sidebar; the card is for the space and its running sessions. -->
  {#if !space.scratch}
    <div class="flex justify-end">
      <!-- One button, one action: a plain shell. The split button that used to
           stand here also picked an agent to start a free session on, and the
           card is not the place for a choice — a space row is a place you
           *land*, and the agent pickers live where the work is named: the
           star-map's spawn, and the same split button on the stage's empty
           state. The `+` is a secondary fill rather than an outline, because
           the card that hosts it no longer draws borders and an outlined
           control on a borderless plate reads as the last hairline left
           over. The row's own click just selects the space, which opening a
           shell does anyway, but the handler still stops propagation so the
           two never race. -->
      <Button
        variant="secondary"
        size="icon-xs"
        disabled={opening}
        aria-label="Open a shell in {space.name}"
        title="Open a plain shell in {space.name} — nothing is injected."
        onclick={(e) => {
          e.stopPropagation();
          onopenshell();
        }}
      >
        <Plus />
      </Button>
    </div>
  {/if}
</div>
