<script lang="ts">
  import { onMount } from 'svelte'
  import { ControlSocket } from './lib/control.svelte'
  import type { Space, Terminal } from './lib/model'
  import { deregisterSpace, openTerminal, closeTerminal } from './lib/actions'
  import RegisterForm from './lib/RegisterForm.svelte'
  import SpacePane from './lib/SpacePane.svelte'
  import Modal from './lib/Modal.svelte'
  import { Button } from './lib/components/ui/button'
  import { Input } from './lib/components/ui/input'
  import { Plus, X, Check, XCircle, CircleNotch, Compass, GitBranch } from 'phosphor-svelte'

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

  async function endShell(space: Space, t: Terminal) {
    if (activeTermId === t.id) activeTermId = null
    try {
      await closeTerminal(space.id, t.id)
    } catch (e) {
      alert(`Couldn’t end “${t.title}”: ${(e as Error).message}`)
    }
  }
</script>

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
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Add a space"
          aria-expanded={showAdd}
          onclick={() => (showAdd = !showAdd)}
        >
          <Plus />
        </Button>
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
          <!-- One space, a bordered container on the sidebar surface (its own
               token family — not the bg-card content surface). Selected emphasis
               rides --primary, the one emphasis token; the chrome is monochrome. -->
          <div
            class={[
              'overflow-hidden rounded-lg border',
              isSelected ? 'border-primary/60' : 'border-sidebar-border',
            ]}
          >
            <!-- Header: the space's identity and its forget action. -->
            <div class="flex items-center gap-0.5 border-b border-sidebar-border pr-0.5">
              <button
                class="min-w-0 flex-1 truncate px-2.5 py-2 text-left text-xs font-semibold"
                title={space.path}
                onclick={() => (selectedId = space.id)}
              >
                <span class="flex items-center gap-1.5">
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
                      <!-- Status indicator: a spinner while working, a tick when
                           idle at the prompt, an error mark once the shell exits. -->
                      {#if t.status === 'working'}
                        <CircleNotch class="size-3.5 shrink-0 animate-spin text-primary" aria-label="working" />
                      {:else if t.status === 'exited'}
                        <XCircle class="size-3.5 shrink-0 text-destructive" aria-label="exited" />
                      {:else}
                        <Check class="size-3.5 shrink-0 text-muted-foreground" aria-label="idle" />
                      {/if}
                      <span class="flex min-w-0 flex-col">
                        <span class="truncate font-mono text-xs">{t.proc}</span>
                        <span class="truncate text-[0.65rem] text-muted-foreground">{t.status}</span>
                      </span>
                    </button>
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      class="opacity-0 hover:text-destructive group-hover/session:opacity-100 focus-visible:opacity-100"
                      aria-label="End {t.proc}"
                      title="End this shell"
                      onclick={() => endShell(space, t)}
                    >
                      <X />
                    </Button>
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
              </span>
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
      <SpacePane space={selected} {activeTerm} onOpenShell={() => openShell(selected)} />
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
</div>
