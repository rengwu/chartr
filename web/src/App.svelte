<script lang="ts">
  import { onMount } from 'svelte'
  import { ControlSocket } from './lib/control.svelte'
  import type { Space, Terminal } from './lib/model'
  import {
    deregisterSpace,
    openTerminal,
    closeTerminal,
    resumeSession,
    respawnSession,
    releaseSession,
    ideate,
  } from './lib/actions'
  import RegisterForm from './lib/RegisterForm.svelte'
  import SpacePane from './lib/SpacePane.svelte'
  import NeedsYouQueue from './lib/NeedsYouQueue.svelte'
  import Modal from './lib/Modal.svelte'
  import { Button } from './lib/components/ui/button'
  import { Input } from './lib/components/ui/input'
  import { needsYouQueue, spaceAttention, spaceLiveness, type QueueEntry } from './lib/attention'
  import { isEditingTarget } from './lib/keys'
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
    Bell,
    Eye,
    Warning,
  } from 'phosphor-svelte'

  // Zero-pad a ticket number for a session row's label (#01), matching the detail
  // pane's ticket ids.
  function pad(n: number): string {
    return n < 10 ? '0' + n : String(n)
  }

  // The control-socket status drives the status-bar dot: on is the neutral "up"
  // primary, connecting a pulsing muted, closed the one true problem (destructive).
  const statusDot: Record<string, string> = {
    open: 'bg-primary',
    connecting: 'bg-muted-foreground animate-pulse',
    closed: 'bg-destructive',
  }

  // The one control socket for this browser. The chrome renders whatever the
  // latest snapshot holds and reacts to every push (ADR 0010).
  const control = new ControlSocket()

  onMount(() => {
    control.connect()
    // A deep link names its space (#s=<id>&…); select it up front so the linked
    // star seats as soon as the space arrives over the socket (ticket 07). The
    // rest of the link — map and star — is applied inside the space's pane.
    const s = new URLSearchParams(location.hash.replace(/^#/, '')).get('s')
    if (s) selectedId = s
    return () => control.close()
  })

  // Spaces arrive already ordered — pinned first, then by recency — so we render
  // them in slice order and never re-sort on the client.
  const spaces = $derived<Space[]>(control.model?.spaces ?? [])

  let selectedId = $state<string | null>(null)
  // The active shell, lifted here from the pane: the sidebar's session rows are
  // now what selects a terminal, so the pane just renders whichever one is active.
  let activeTermId = $state<string | null>(null)
  let filter = $state('')
  let showAdd = $state(false)
  let opening = $state(false)
  // The cross-space "Needs you" queue (ticket 14): summoned here, over the
  // whole cockpit rather than any one space's pane, since its entries can
  // point at a space other than the one currently open.
  let queueOpen = $state(false)
  const queueCount = $derived(needsYouQueue(spaces).length)

  // The effective selection falls back to the first space when the id is stale
  // (e.g. the selected space was just forgotten), so the pane never blanks while
  // spaces remain. No effect mutates state; selection is pure derivation.
  const selected = $derived.by(() => {
    return spaces.find((s) => s.id === selectedId) ?? spaces[0] ?? null
  })

  // The shell the pane shows: the active id within the selected space, falling
  // back to that space's first shell so the pane never shows a blank island while
  // terminals remain (the same stale-id tolerance selection has).
  const activeTerm = $derived.by<Terminal | null>(() => {
    const ts = selected?.terminals ?? []
    return ts.find((t) => t.id === activeTermId) ?? ts[0] ?? null
  })

  // The filter is a pure view over the ordered list — it now reaches into
  // sessions too (a space shows if its own fields or any of its shells match), so
  // the sidebar scales past what a flat list carries without changing order.
  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase()
    if (q === '') return spaces
    return spaces.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.path.toLowerCase().includes(q) ||
        (s.branch ?? '').toLowerCase().includes(q) ||
        s.terminals.some(
          (t) => t.proc.toLowerCase().includes(q) || t.title.toLowerCase().includes(q),
        ),
    )
  })

  async function forget(space: Space) {
    const ok = confirm(
      `Forget “${space.name}”?\n\nThe harness stops tracking it. Nothing in the repository is touched — re-register any time and it picks up exactly as it sits.`,
    )
    if (!ok) return
    if (selectedId === space.id) selectedId = null
    await deregisterSpace(space.id)
  }

  // Selecting a session selects its space and makes that shell active, so one
  // click drives both the sidebar highlight and what the pane renders.
  function selectSession(space: Space, t: Terminal) {
    selectedId = space.id
    activeTermId = t.id
  }

  async function openShell(space: Space) {
    selectedId = space.id
    opening = true
    try {
      const { id } = await openTerminal(space.id)
      activeTermId = id
    } catch (e) {
      alert(`Couldn’t open a shell: ${(e as Error).message}`)
    } finally {
      opening = false
    }
  }

  // The ideate on-ramp (ticket 15): the one opinionated nudge toward charting. A
  // live, ticketless agent tab typed the starter prompt on open — shares only the
  // spawn primitive with a real session, so it opens exactly like a shell (no
  // role picker, no ticket, nothing to gate on).
  async function ideateSpace(space: Space) {
    selectedId = space.id
    opening = true
    try {
      const { id } = await ideate(space.id)
      activeTermId = id
    } catch (e) {
      alert(`Couldn’t start ideating: ${(e as Error).message}`)
    } finally {
      opening = false
    }
  }

  async function endShell(space: Space, t: Terminal) {
    if (activeTermId === t.id) activeTermId = null
    try {
      await closeTerminal(space.id, t.id)
    } catch (e) {
      alert(`Couldn’t end “${t.title}”: ${(e as Error).message}`)
    }
  }

  // The death halt: a dead session offers exactly three choices, and the harness
  // takes none on its own. Resume relaunches it on its own ticket (crash recovery);
  // respawn starts a fresh session on the same ticket; release clears the claim back
  // to the frontier. The resulting state arrives over the control socket.
  async function haltAction(
    space: Space,
    t: Terminal,
    verb: string,
    run: (spaceId: string, sessionId: string) => Promise<unknown>,
  ) {
    selectedId = space.id
    activeTermId = t.id
    try {
      await run(space.id, t.id)
    } catch (e) {
      alert(`Couldn’t ${verb} this session: ${(e as Error).message}`)
    }
  }

  // One click from the queue: select the entry's space and set the deep-link
  // hash naming its map and ticket. The selected space's SpacePane instance
  // persists across space switches (ticket 07) and already listens for
  // hashchange to seat a linked star, so this reuses that exact mechanism
  // rather than reaching into the pane's own state.
  function jumpToQueueEntry(entry: QueueEntry) {
    selectedId = entry.spaceId
    location.hash = `#s=${encodeURIComponent(entry.spaceId)}&m=${encodeURIComponent(entry.mapSlug)}&t=${entry.ticketNum}`
    queueOpen = false
  }

  // Keyboard-first navigation (story 30): space switching and queue summoning,
  // alongside the map's own M/Esc (SpacePane.onKey). `[`/`]` cycle spaces in
  // the same pinned-then-recency order the sidebar renders, never the
  // filtered view — a keyboard shortcut should not depend on what's typed in
  // the filter box. `q` toggles the queue; Esc-to-close comes from the Sheet
  // itself, matching how the bindings drawer already behaves.
  function onGlobalKey(e: KeyboardEvent) {
    if (isEditingTarget() || e.metaKey || e.ctrlKey || e.altKey) return
    if (e.key === 'q' || e.key === 'Q') {
      e.preventDefault()
      queueOpen = !queueOpen
      return
    }
    if ((e.key === '[' || e.key === ']') && spaces.length > 1) {
      e.preventDefault()
      const ids = spaces.map((s) => s.id)
      const i = selected ? ids.indexOf(selected.id) : -1
      const next = ids[(i + (e.key === ']' ? 1 : -1) + ids.length) % ids.length]
      selectedId = next
    }
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

<div class="grid h-full grid-cols-[16rem_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto]">
  <aside
    class="col-start-1 row-start-1 flex min-h-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
  >
    <!-- Branding: a marked home for the cockpit, above the spaces list. -->
    <div class="cockpit-bar gap-2 bg-transparent">
      <span
        class="grid size-5 place-items-center rounded-full border border-sidebar-border text-sidebar-foreground"
      >
        <Compass class="size-3.5" />
      </span>
      <span class="text-sm font-semibold tracking-tight">Wayfinder</span>
    </div>

    <div class="cockpit-bar justify-between bg-transparent">
      <span class="text-xs font-semibold tracking-wide">Spaces</span>
      {#if spaces.length > 0}
        <span class="flex items-center gap-0.5">
          <!-- The cross-space "Needs you" queue (ticket 14): strictly pull —
               it renders only while this Sheet is open, never on its own. -->
          <span class="relative">
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Needs you — {queueCount} across every space"
              title="Needs you — gate-level signals across every space (Q)"
              onclick={() => (queueOpen = !queueOpen)}
            >
              <Bell />
            </Button>
            {#if queueCount > 0}
              <span
                class="pointer-events-none absolute -top-0.5 -right-0.5 grid size-3.5 place-items-center rounded-full bg-primary text-[0.55rem] font-semibold text-primary-foreground"
                >{queueCount}</span
              >
            {/if}
          </span>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="Add a space"
            aria-expanded={showAdd}
            onclick={() => (showAdd = !showAdd)}
          >
            <Plus />
          </Button>
        </span>
      {/if}
    </div>

    {#if control.model === null}
      <p class="px-3 py-2 text-xs text-muted-foreground">Connecting…</p>
    {:else if spaces.length === 0}
      <p class="px-3 py-2 text-xs text-muted-foreground">No spaces yet.</p>
    {:else}
      <div class="p-2">
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

      <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto px-2 pb-2">
        {#each filtered as space (space.id)}
          {@const isSelected = selected?.id === space.id}
          {@const attention = spaceAttention(space)}
          {@const liveness = spaceLiveness(space)}
          <!-- One space, a bordered container on the sidebar surface (its own
               token family — not the bg-card content surface). Selected emphasis
               rides --primary, the one emphasis token; the chrome is monochrome. -->
          <div
            class={[
              'overflow-hidden rounded-lg border',
              isSelected ? 'border-primary/60' : 'border-sidebar-border',
            ]}
          >
            <!-- Header: the space's identity and its forget action. Ambient
                 cross-space attention (ticket 14, story 8) rides here — a
                 wants-you flag (a review waiting, or a session halted) and a
                 liveness dot, both echoing the same signals the queue pulls
                 and the sidebar's own session rows already carry in detail.
                 Neither ever re-sorts the row; muscle memory over this list
                 holds. -->
            <div class="flex items-center gap-0.5 border-b border-sidebar-border pr-0.5">
              <button
                class="min-w-0 flex-1 truncate px-2.5 py-2 text-left text-xs font-semibold"
                title={space.path}
                onclick={() => (selectedId = space.id)}
              >
                <span class="flex items-center gap-1.5">
                  {#if attention === 'review'}
                    <Eye class="size-3.5 shrink-0 text-primary" aria-label="a review is waiting" />
                  {:else if attention === 'halt'}
                    <Warning class="size-3.5 shrink-0 text-destructive" aria-label="a session halted, needs a decision" />
                  {/if}
                  {#if liveness === 'working'}
                    <CircleNotch class="size-3 shrink-0 animate-spin text-primary" aria-label="a session is working" />
                  {:else if liveness === 'quiet'}
                    <CircleNotch
                      class="size-3 shrink-0 animate-spin text-muted-foreground [animation-duration:3s]"
                      aria-label="a session is quiet"
                    />
                  {/if}
                  <span class="truncate">{space.name}</span>
                </span>
              </button>
              <Button
                variant="ghost"
                size="icon-xs"
                class="hover:text-destructive"
                aria-label="Forget space"
                title="Forget (repository untouched)"
                onclick={() => forget(space)}
              >
                <X />
              </Button>
            </div>

            <!-- Sessions: the space's open shells, each a selectable row carrying
                 its foreground process, live status, and a close action. A space
                 with none open still gets a row here, so the header never runs
                 straight into the branch footer with no indication why. -->
            {#if space.terminals.length}
              <ul class="flex flex-col p-1">
                {#each space.terminals as t (t.id)}
                  {@const isActive = isSelected && activeTerm?.id === t.id}
                  <li class="group/session flex items-center gap-1 rounded-md pr-0.5
                    {isActive ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent/60'}">
                    <button
                      class="flex min-w-0 flex-1 items-center gap-2 px-1.5 py-1.5 text-left"
                      onclick={() => selectSession(space, t)}
                    >
                      <!-- Status indicator. A shell: a spinner while working, a tick
                           idle, an error mark once it exits. A session on the session
                           grammar: a spinner working, a slow dimmed crawl when quiet
                           (a hint, not an alarm), a frozen grey mark once dead. -->
                      {#if t.status === 'working'}
                        <CircleNotch class="size-3.5 shrink-0 animate-spin text-primary" aria-label="working" />
                      {:else if t.status === 'quiet'}
                        <CircleNotch
                          class="size-3.5 shrink-0 animate-spin text-muted-foreground [animation-duration:3s]"
                          aria-label="quiet"
                        />
                      {:else if t.status === 'dead'}
                        <XCircle class="size-3.5 shrink-0 text-muted-foreground" aria-label="dead" />
                      {:else if t.status === 'exited'}
                        <XCircle class="size-3.5 shrink-0 text-destructive" aria-label="exited" />
                      {:else}
                        <Check class="size-3.5 shrink-0 text-muted-foreground" aria-label="idle" />
                      {/if}
                      <span class="flex min-w-0 flex-col">
                        {#if t.session}
                          <!-- A session tab: its identity is the ticket it is bound
                               to (role · #num), with the agent and status beneath —
                               told apart from an ad-hoc shell, which shows its
                               foreground process. -->
                          <span class="flex min-w-0 items-center gap-1 text-xs font-medium">
                            <Rocket class="size-3 shrink-0 text-primary" aria-hidden="true" />
                            <span class="truncate">{t.session.role} #{pad(t.session.ticketNum)}</span>
                          </span>
                          <span class="truncate text-[0.65rem] text-muted-foreground">
                            {t.session.agent} · {t.status}
                          </span>
                        {:else}
                          <span class="truncate font-mono text-xs">{t.proc}</span>
                          <span class="truncate text-[0.65rem] text-muted-foreground">{t.status}</span>
                        {/if}
                      </span>
                    </button>
                    {#if t.session && !t.alive}
                      <!-- The death halt: a dead session is pinned to its ticket and
                           offers exactly three choices — resume it (crash recovery),
                           respawn a fresh session, or release the claim. The harness
                           takes none itself. -->
                      <span class="flex shrink-0 items-center pr-0.5">
                        <Button
                          variant="ghost"
                          size="icon-xs"
                          class="hover:text-primary"
                          aria-label="Resume this session"
                          title="Resume — same-ticket crash recovery"
                          onclick={() => haltAction(space, t, 'resume', resumeSession)}
                        >
                          <Play />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon-xs"
                          class="hover:text-primary"
                          aria-label="Respawn a fresh session"
                          title="Respawn — a fresh session on the same ticket"
                          onclick={() => haltAction(space, t, 'respawn', respawnSession)}
                        >
                          <ArrowClockwise />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon-xs"
                          class="hover:text-destructive"
                          aria-label="Release the claim"
                          title="Release — clear the claim back to the frontier"
                          onclick={() => haltAction(space, t, 'release', releaseSession)}
                        >
                          <ArrowUUpLeft />
                        </Button>
                      </span>
                    {:else}
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        class="opacity-0 hover:text-destructive group-hover/session:opacity-100 focus-visible:opacity-100"
                        aria-label="End {t.proc}"
                        title={t.session ? 'End this session' : 'End this shell'}
                        onclick={() => endShell(space, t)}
                      >
                        <X />
                      </Button>
                    {/if}
                  </li>
                {/each}
              </ul>
            {:else}
              <p class="px-2.5 py-1.5 text-xs text-muted-foreground">No sessions open.</p>
            {/if}

            <!-- Footer: the working tree's branch and a new-shell action. -->
            <div class="flex items-center gap-1.5 border-t border-sidebar-border px-2.5 py-1.5">
              <span class="flex min-w-0 flex-1 items-center gap-1.5 text-[0.7rem] text-muted-foreground">
                {#if space.branch}
                  <GitBranch class="size-3.5 shrink-0" />
                  <span class="truncate font-mono" title={space.branch}>{space.branch}</span>
                {/if}
                {#if space.dirty}
                  <!-- A dirty working tree is a badge, never a spawn gate (story 68). -->
                  <span
                    class="flex shrink-0 items-center"
                    title="Uncommitted changes in the working tree"
                  >
                    <GitDiff class="size-3.5" aria-label="uncommitted changes" />
                  </span>
                {/if}
              </span>
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label="Ideate in {space.name}"
                title="Ideate — a live, ticketless chat to think an idea through"
                disabled={opening}
                onclick={() => ideateSpace(space)}
              >
                <Lightbulb />
              </Button>
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label="Open a shell in {space.name}"
                title="Open a shell in {space.name}"
                disabled={opening}
                onclick={() => openShell(space)}
              >
                <Plus />
              </Button>
            </div>
          </div>
        {:else}
          <p class="px-2 py-1.5 text-xs text-muted-foreground">No spaces match “{filter}”.</p>
        {/each}
      </div>
    {/if}
  </aside>

  <main class="col-start-2 row-start-1 min-h-0 min-w-0">
    {#if spaces.length === 0}
      <div class="grid h-full place-items-center p-6">
        <RegisterForm variant="first-run" onRegistered={(id) => (selectedId = id)} />
      </div>
    {:else if selected}
      <SpacePane
        space={selected}
        {activeTerm}
        onOpenShell={() => openShell(selected)}
        onIdeate={() => ideateSpace(selected)}
        onspawned={(id) => (activeTermId = id)}
      />
    {/if}
  </main>

  <footer
    class="col-span-2 row-start-2 flex items-center gap-2 border-t border-border bg-card px-3 py-1.5 text-[0.7rem] text-muted-foreground"
  >
    <span class={['size-2 rounded-full', statusDot[control.status] ?? 'bg-muted-foreground']} aria-hidden="true"></span>
    <span>control socket: {control.status}</span>
  </footer>

  <Modal open={showAdd} title="Add a space" onClose={() => (showAdd = false)}>
    <p class="mb-3 text-xs text-muted-foreground">
      Point the harness at a project folder — paste its absolute path. If it isn’t a git
      repository yet, one is initialized there, announced.
    </p>
    <RegisterForm
      variant="inline"
      onRegistered={(id) => {
        selectedId = id
        showAdd = false
      }}
    />
  </Modal>

  <NeedsYouQueue bind:open={queueOpen} {spaces} onjump={jumpToQueueEntry} />
</div>
