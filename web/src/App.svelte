<script lang="ts">
  import { onMount } from 'svelte'
  import { ControlSocket } from './lib/control.svelte'
  import { needsAgents, type Map, type Space } from './lib/model'
  import { deregisterSpace, setPin } from './lib/actions'
  import RegisterForm from './lib/RegisterForm.svelte'
  import SpacePane from './lib/SpacePane.svelte'
  import Modal from './lib/Modal.svelte'
  import { Button } from './lib/components/ui/button'
  import { Input } from './lib/components/ui/input'
  import { Badge } from './lib/components/ui/badge'
  import { Plus, PushPin, X, Warning, WarningDiamond, Check, CircleDashed } from 'phosphor-svelte'

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
  let filter = $state('')
  let showAdd = $state(false)

  // The effective selection falls back to the first space when the id is stale
  // (e.g. the selected space was just forgotten), so the pane never blanks while
  // spaces remain. No effect mutates state; selection is pure derivation.
  const selected = $derived.by(() => {
    return spaces.find((s) => s.id === selectedId) ?? spaces[0] ?? null
  })

  // The always-present filter is a pure view over the ordered list; it scales
  // the sidebar past what a flat list carries without changing order (story 7).
  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase()
    if (q === '') return spaces
    return spaces.filter(
      (s) => s.name.toLowerCase().includes(q) || s.path.toLowerCase().includes(q),
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

  async function togglePin(space: Space) {
    await setPin(space.id, !space.pinned)
  }

  // How many of a map's tickets sit on the stricter frontier — the takeable
  // edge. Shown as a small count so the most spawnable maps read at a glance.
  function frontierCount(m: Map): number {
    return m.tickets.filter((t) => t.frontier).length
  }

</script>

<div class="grid h-full grid-cols-[15rem_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto]">
  <aside class="col-start-1 row-start-1 flex min-h-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
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
          placeholder="Filter spaces…"
          bind:value={filter}
          spellcheck="false"
          autocapitalize="off"
          autocomplete="off"
          aria-label="Filter spaces"
        />
      </div>

      <ul class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-1.5 pb-2">
        {#each filtered as space (space.id)}
          <li class="group/row">
            <div
              class={[
                'flex items-center gap-0.5 rounded-md pr-0.5',
                selected?.id === space.id ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent/60',
              ]}
            >
              <button
                class="flex min-w-0 flex-1 flex-col items-start gap-0.5 py-1.5 pl-2 text-left"
                onclick={() => (selectedId = space.id)}
              >
                <span class="flex max-w-full items-center gap-1.5 truncate text-xs font-medium">
                  {space.name}
                  {#if needsAgents(space)}
                    <Badge
                      variant="outline"
                      class="gap-1 border-border text-muted-foreground"
                      title="An agent for one or more roles isn’t on your PATH"
                    >
                      <Warning /> agent
                    </Badge>
                  {/if}
                </span>
                <span class="max-w-full truncate font-mono text-[0.65rem] text-muted-foreground">{space.path}</span>
              </button>
              <Button
                variant="ghost"
                size="icon-xs"
                class={[
                  space.pinned
                    ? 'text-primary'
                    : 'opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100',
                ]}
                aria-pressed={space.pinned}
                aria-label={space.pinned ? 'Unpin space' : 'Pin space'}
                title={space.pinned ? 'Unpin' : 'Pin to top'}
                onclick={() => togglePin(space)}
              >
                <PushPin weight={space.pinned ? 'fill' : 'regular'} />
              </Button>
              <Button
                variant="ghost"
                size="icon-xs"
                class="opacity-0 hover:text-destructive group-hover/row:opacity-100 focus-visible:opacity-100"
                aria-label="Forget space"
                title="Forget (repository untouched)"
                onclick={() => forget(space)}
              >
                <X />
              </Button>
            </div>

            <!-- Maps nest under their space; they arrive already ordered
                 (finished last) so we render in slice order. -->
            {#if space.maps.length}
              <ul class="mt-0.5 mb-1 ml-3 flex flex-col gap-px border-l border-sidebar-border pl-2">
                {#each space.maps as m (m.slug)}
                  <li
                    class={[
                      'flex items-center gap-1.5 py-0.5 pr-1 text-[0.7rem]',
                      m.finished && 'text-muted-foreground',
                      m.kind === '' && 'text-muted-foreground italic',
                    ]}
                  >
                    <span class="min-w-0 flex-1 truncate" title={m.name}>{m.name}</span>
                    {#if m.kind === ''}
                      <!-- Undeclared: inert until classified (ADR 0007). The
                           declaration is meant to be recorded on creation (the
                           wayfinder adapter, docs/wayfinder-adapter.md); this quiet
                           marker is the fallback for a map that arrived without one.
                           The confirm itself lives in the star-map panel — never
                           hoisted into the nav as a pair of buttons per row. -->
                      <span
                        class="text-muted-foreground"
                        title="Unclassified — open the map to set its kind"
                        aria-label="unclassified"
                      >
                        <CircleDashed class="size-3.5" />
                      </span>
                    {:else}
                      {#if frontierCount(m) > 0}
                        <Badge variant="secondary" title="{frontierCount(m)} ticket(s) at the frontier">
                          {frontierCount(m)}
                        </Badge>
                      {/if}
                      {#if m.finished}
                        <span class="text-muted-foreground" title="every ticket resolved" aria-label="finished">
                          <Check class="size-3.5" />
                        </span>
                      {/if}
                    {/if}
                    {#if m.malformations?.length}
                      <span
                        class="text-muted-foreground"
                        title={m.malformations.join('\n')}
                        aria-label="{m.malformations.length} malformation(s) surfaced"
                      >
                        <WarningDiamond class="size-3.5" />
                      </span>
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}
          </li>
        {:else}
          <li class="px-2 py-1.5 text-xs text-muted-foreground">No spaces match “{filter}”.</li>
        {/each}
      </ul>
    {/if}
  </aside>

  <main class="col-start-2 row-start-1 min-h-0 min-w-0">
    {#if spaces.length === 0}
      <div class="grid h-full place-items-center p-6">
        <RegisterForm variant="first-run" onRegistered={(id) => (selectedId = id)} />
      </div>
    {:else if selected}
      <SpacePane space={selected} />
    {/if}
  </main>

  <footer class="col-span-2 row-start-2 flex items-center gap-2 border-t border-border bg-card px-3 py-1.5 text-[0.7rem] text-muted-foreground">
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
