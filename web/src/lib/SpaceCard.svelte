<script lang="ts">
  import type { Agent, Space, Terminal } from "./model";
  import type { DropEdge } from "./reorder";
  import NewShellButton from "./NewShellButton.svelte";
  import { Button } from "./components/ui/button";
  import { spaceAttention, spaceLiveness } from "./attention";
  import { showsFinishedDot } from "./unseen";
  import {
    X,
    Check,
    XCircle,
    CircleNotch,
    Rocket,
    Play,
    ArrowClockwise,
    ArrowUUpLeft,
    Warning,
    PauseCircle,
    GitBranchIcon,
  } from "phosphor-svelte";

  // The three choices a dead session offers. The card names the choice; the
  // chrome above owns which action each one runs, so the halt actions stay
  // beside every other action call rather than being imported here.
  type HaltVerb = "resume" | "respawn" | "release";

  let {
    space,
    selected,
    activeTermId,
    agents,
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
    onfreesession,
    onregister,
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
    // The registered agent library — global, the same list whatever space is in
    // view — handed straight to the new-shell menu, in registration order.
    agents: Agent[];
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
    // The free session the new-shell menu settled: the agent the operator clicked.
    // It takes no skill and no context line — a free session is told the free
    // payload and nothing else.
    onfreesession: (agent: string) => void;
    // Where an empty agent library sends the operator: the registration surface.
    onregister: () => void;
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
    // branch copy, the new-shell menu, and the session rows, which are click
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

  // Copy-to-clipboard for the branch chip. The card sets `select-none` — it is a
  // click target, and drag-selecting inside one is noise — so this button is the
  // *only* way the branch name leaves the UI, not a shortcut alongside manual
  // selection. That is why the failure is shown rather than swallowed the way the
  // terminal's copy-on-select swallows it: there, a denied clipboard still leaves
  // you a selection to ⌘C; here it would leave nothing at all.
  // `navigator.clipboard` needs a secure context, so a harness opened over plain
  // http on a LAN address lands in exactly that case.
  let copied = $state<{ ok: boolean } | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  async function copyToClipboard(text: string) {
    let ok = false;
    try {
      await navigator.clipboard.writeText(text);
      ok = true;
    } catch {
      ok = false;
    }
    copied = { ok };
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copied = null;
    }, 1200);
  }
</script>

<!-- One space, a bordered container on the sidebar surface (its own
     token family — not the bg-card content surface). The whole card is
     the selection target — clicking anywhere that isn't its own control
     selects the space — so the identity, its sessions and its actions
     all read as one object rather than a header you must aim at.
     Selected emphasis rides --primary, the one emphasis token; the
     chrome is monochrome. Because the whole card is a click target,
     it is `select-none` throughout — name, sessions, branch. Dragging
     a selection across a thing you click is noise, and the branch —
     the one string actually worth lifting — has its own copy button
     instead. The path stays the card's tooltip. -->
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
      ? "border-primary/60 bg-sidebar-accent/30"
      : "border-sidebar-border hover:border-primary/30",
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

  <!-- Identity: the space's name, with the forget
       action pinned top-right (the branch rides the action row below).
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

  <!-- Sessions: the space's open shells, each its own card inside the
       space's — identity over status, with its close action pinned the
       same way the space's is. Clicking one selects the space *and*
       switches to that session, the one click that does both. -->
  {#if space.terminals.length}
    <ul class="flex flex-col gap-1.5">
      {#each space.terminals as t (t.id)}
        {@const isActive = selected && activeTermId === t.id}
        <li>
          <div
            role="button"
            tabindex="0"
            aria-pressed={isActive}
            data-session-row
            class={[
              "flex flex-col gap-0.5 rounded-md border px-2 py-1.5 transition-colors",
              isActive
                ? "border-primary/50 bg-sidebar-accent text-sidebar-accent-foreground"
                : "border-sidebar-border hover:bg-sidebar-accent/60",
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
                      >{t.session.role} #{pad(t.session.ticketNum)}</span
                    >
                  </span>
                {:else}
                  <span class="block truncate font-mono text-xs">{t.proc}</span>
                {/if}
              </span>
              <Button
                variant="ghost"
                size="icon-xs"
                class="-mt-0.5 -mr-1 hover:text-destructive"
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

              {#if showsFinishedDot(t, isActive)}
                <!-- It finished a run worth interrupting you over while you
                     were elsewhere (session-notifications) — the quiet half
                     of the notification the OS already showed. It rides
                     `--primary`, the one emphasis token, and it is a state
                     rather than a decoration: `role="img"` with a name, so
                     the card announces the difference instead of leaving a
                     screen reader a bare circle. Focusing the tab clears it —
                     there is no dismiss — which is why it never shows on the
                     tab in view. -->
                <span
                  role="img"
                  aria-label="finished while you were away"
                  title="Finished while you were away — open it to clear this"
                  class="size-2 shrink-0 rounded-full bg-primary"
                ></span>
              {/if}

              {#if t.session && !t.alive}
                <!-- The death halt: a dead session is pinned to its ticket and
                     offers exactly three choices — resume it (crash recovery),
                     respawn a fresh session, or release the claim. chartr
                     takes none itself. -->
                <span class="-my-0.5 -mr-1 flex shrink-0 items-center">
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
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Actions: the one ticketless on-ramp — the new-shell split button —
       sharing its row with the branch, which doubles as
       the spacer that pushes it right. The branch is a real ghost
       Button rather than a bare span with a handler: it is a control
       inside a control, so it earns the same hover, focus ring and
       keyboard reach every other action in this row has. The size
       variants all set a fixed height and `shrink-0`, so this one
       overrides both — it still has to truncate and still has to be
       the spacer.

       Scratch honours neither: it is not a repository, so a
       branch chip would tell the operator something false, and the
       on-ramp cannot act on a space with no map and no repository. That
       empties the row, so the row itself does not render — the card's
       own `gap-2` would otherwise leave a blank strip under the shells.
       The shell rows above stay, and so does the grab: it is reorderable
       and its shells are selectable like any other space's. -->
  {#if !space.scratch}
    <div class="flex items-center gap-1">
      <span
        class="flex min-w-0 flex-1 items-center gap-1.5 text-[0.6rem] text-muted-foreground"
      >
        {#if space.branch}
          <Button
            variant="ghost"
            class="-mx-1 h-auto min-w-0 shrink justify-start gap-1.5 rounded-sm px-1 py-0.5 text-[0.6rem] font-normal text-muted-foreground hover:text-foreground [&_svg:not([class*='size-'])]:size-3"
            aria-label="Copy branch name"
            title={copied
              ? copied.ok
                ? "Copied"
                : "Couldn’t copy — clipboard unavailable"
              : `Copy branch — ${space.branch}`}
            onclick={(e) => {
              e.stopPropagation();
              void copyToClipboard(space.branch ?? "");
            }}
          >
            {#if copied}
              {#if copied.ok}
                <Check class="text-primary" />
              {:else}
                <Warning class="text-destructive" />
              {/if}
            {:else}
              <GitBranchIcon />
            {/if}
            <span class="truncate font-mono">{space.branch}</span>
          </Button>
        {/if}
      </span>
      <!-- The new-shell control (skill-sources ticket 08): one split button
           where the launcher and the `+` shell button used to sit side by
           side. The caret chooses what the body runs — a plain shell or a
           free session on a registered agent — and the body runs it. The row
           is shorter for it, and the branch chip — which doubles as the
           spacer — gains the width. The row's own click just selects the
           space, which either action does anyway, so the caret deliberately
           does not stop propagation; the body does, because it has a handler
           of its own to protect. -->
      <NewShellButton
        {agents}
        disabled={opening}
        ariaLabel="Open a shell in {space.name}"
        title="Open a plain shell in {space.name} — nothing is injected. Pick an agent from the caret to launch a free session with this button instead."
        onshell={onopenshell}
        onfree={onfreesession}
        {onregister}
      />
    </div>
  {/if}
</div>
