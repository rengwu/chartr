<script lang="ts">
  import { untrack } from 'svelte'
  import type { Map as WMap, Ticket } from './model'
  import StarMap from './StarMap.svelte'
  import DetailPane from './DetailPane.svelte'
  import MapPickerCard from './MapPickerCard.svelte'
  import { decideDock, type Dock } from './starmap/dock'
  import { Button } from './components/ui/button'
  import * as ScrollArea from './components/ui/scroll-area'
  import { CaretLeft, Columns, CornersOut, X } from 'phosphor-svelte'

  // The star-map panel presented as a card over the terminal (spec, The
  // interface): summoned, never toggled by switching spaces or maps. It carries
  // two screens in the one floating/docked frame:
  //
  //   • the picker — a grid of the space's maps (name, kind, resolution), and the
  //     door in; an unclassified map is classified from its tile, never opened
  //     until it has a kind (ADR 0007);
  //   • the map — the island, with the back / map-name / dock / close chrome
  //     floating directly over it (no header bar), and the responsive detail pane
  //     (ticket 07) docked right, re-docking to the bottom when the card is narrow.
  //
  // `slug === null` is the picker; a slug names the open map. The parent owns
  // which screen we land on (auto-open for a single-map space, deep links) and
  // whether the frame floats or docks as the terminal-priority split.
  let {
    maps,
    spaceId,
    slug = $bindable(),
    dock = $bindable(false),
    selected = $bindable(null),
    showMaterial = $bindable(false),
    floatWidth = 0,
    onclose,
    onresizestart,
    onspawned,
  }: {
    maps: WMap[]
    // The space these maps belong to — threaded to the detail pane so its payload
    // preview can fetch (ticket 08), and to the picker tiles for classify.
    spaceId: string
    slug: string | null
    dock?: boolean
    selected?: number | null
    showMaterial?: boolean
    floatWidth?: number
    onclose: () => void
    onresizestart: (e: MouseEvent) => void
    // Bubbled up from the detail pane's Spawn control (ticket 09) so the chrome can
    // make the new session's tab active.
    onspawned?: (sessionId: string) => void
  } = $props()

  // The open map, or null for the picker. A stale slug (a map that vanished)
  // falls back to the picker rather than to some other graph.
  const map = $derived<WMap | null>(slug ? (maps.find((m) => m.slug === slug) ?? null) : null)

  // The picker splits the maps into two sections. Beyond reading as "ready to
  // open" vs "needs a kind first", it keeps the taller unclassified tiles (they
  // carry the classify confirm) from stretching the shorter classified tiles they
  // would otherwise share a flex-wrap row with.
  const classified = $derived(maps.filter((m) => m.kind !== ''))
  const unclassified = $derived(maps.filter((m) => m.kind === ''))

  // Both are bound inside the map screen's `{#if map}`, so they mount only when a
  // map opens — $state so the measuring effects below re-run when they appear.
  let bodyEl = $state<HTMLDivElement | null>(null)
  let paneEl = $state<HTMLDivElement | null>(null)
  let bodyWidth = $state(0)
  let bodyHeight = $state(0)
  let paneSize = $state({ w: 0, h: 0 })

  // Selecting a star wins over the map-material pane; opening material clears any
  // ticket selection. The two are one pane showing one thing.
  $effect(() => {
    if (selected !== null) showMaterial = false
  })

  const paneTicket = $derived<Ticket | null>(
    !showMaterial && selected !== null ? (map?.tickets.find((t) => t.num === selected) ?? null) : null,
  )
  const paneOpen = $derived(showMaterial || paneTicket !== null)

  // The floating chrome sits over the top of the island. Keep the camera's stars
  // clear of it — a taller top inset than the other three edges — so a star or
  // label never hides behind the back / dock buttons.
  const TOP_INSET = 52

  // Responsive docking: right by default, re-docking to bottom when the card is
  // either too narrow to hold a side pane or tall enough that a right pane would
  // ribbon the map — the hybrid signal (spec: right dock, re-docking to bottom
  // when narrow). A dead-band makes the switch sticky: `prev` feeds the next
  // decision, so dragging the card through the boundary holds the current side
  // rather than flip-flopping.
  let paneDock = $state<Dock>('right')
  $effect(() => {
    const next = decideDock('hybrid', bodyWidth, bodyHeight, untrack(() => paneDock), true)
    if (next !== untrack(() => paneDock)) paneDock = next
  })

  // The camera measures the pane's actual footprint and eases the star into the
  // rest (planning ticket 08 as amended): a right pane insets the right edge, a
  // bottom pane the bottom edge; closed, only a small breathing margin. The top
  // always clears the floating chrome.
  const insets = $derived(
    paneOpen
      ? paneDock === 'right'
        ? { top: TOP_INSET, left: 16, bottom: 16, right: paneSize.w + 20 }
        : { top: TOP_INSET, left: 16, right: 16, bottom: paneSize.h + 20 }
      : { top: TOP_INSET, right: 16, bottom: 16, left: 16 },
  )

  $effect(() => {
    const el = bodyEl
    if (!el) return
    const measure = () => {
      bodyWidth = el.clientWidth
      bodyHeight = el.clientHeight
    }
    measure()
    const ro = new ResizeObserver(measure)
    ro.observe(el)
    return () => ro.disconnect()
  })

  $effect(() => {
    const el = paneEl
    if (!el) return
    const measure = () => (paneSize = { w: el.clientWidth, h: el.clientHeight })
    measure()
    const ro = new ResizeObserver(measure)
    ro.observe(el)
    return () => ro.disconnect()
  })

  function back() {
    selected = null
    showMaterial = false
    slug = null
  }
  function openMaterial() {
    selected = null
    showMaterial = true
  }
  function closePane() {
    selected = null
    showMaterial = false
  }
</script>

<!-- The panel frame. Docked, it is a flex item in the panes row, taking the slack
     the terminal's frozen basis leaves (min 300). Floating, it is absolute over
     the panes row with its right edge pinned (inset 10px), grown leftward by the
     resize grip; a CSS max-width keeps it within the row on a window resize. -->
<section
  class={[
    'flex min-h-0 flex-col overflow-hidden bg-card',
    dock
      ? // Docked, this is a flush column next to the terminal — a plain square
        // edge like the terminal's own outer edges, with a border only on the
        // seam it shares with the terminal (the split divider).
        'relative min-w-[300px] flex-1 border-l border-border'
      : 'absolute inset-y-2 right-2.5 z-20 w-[min(560px,64%)] max-w-[calc(100%-40px)] rounded-lg border border-border shadow-lg',
  ]}
  aria-label="Star-map"
  style={!dock && floatWidth ? `width:${floatWidth}px` : ''}
>
  <!-- Resize from the left border: the split divider when docked, the card's
       leading edge when floating (the right edge stays pinned). -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="absolute inset-y-0 left-0 z-40 w-1.5 -translate-x-1/2 cursor-ew-resize transition-colors hover:bg-ring/60"
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize star-map"
    onmousedown={onresizestart}
  ></div>

  <!-- The dock/close chrome is shared by both screens: dock toggles the whole
       panel between the terminal-priority split and floating over the terminal;
       close dismisses it. -->
  {#snippet chrome()}
    <Button
      variant="outline"
      size="icon-sm"
      aria-pressed={dock}
      title={dock ? 'Float over the terminal' : 'Dock as a split (terminal keeps its width)'}
      onclick={() => (dock = !dock)}
    >
      {#if dock}<CornersOut />{:else}<Columns />{/if}
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Dismiss star-map (Esc)"
      title="Dismiss (M / Esc)"
      onclick={onclose}
    >
      <X />
    </Button>
  {/snippet}

  {#if map}
    <!-- The map screen: the island fills the frame, its chrome floating directly
         over the top (no header bar). The detail pane overlays one edge below the
         chrome; the camera (insets) eases the selected star into the space it
         leaves free. -->
    <div class="relative min-h-0 flex-1" bind:this={bodyEl}>
      {#key map.slug}
        <StarMap {map} {insets} bind:selected />
      {/key}

      <div class="absolute inset-x-0 top-0 z-30 flex h-[var(--bar-h)] items-center gap-1.5 px-2">
        <Button variant="outline" size="sm" title="Back to all maps" onclick={back}>
          <CaretLeft /> back
        </Button>
        <Button
          variant={showMaterial ? 'secondary' : 'ghost'}
          size="sm"
          class="min-w-0 truncate"
          aria-pressed={showMaterial}
          title="Open map material — destination, notes, decisions, fog"
          onclick={openMaterial}>{map.name}</Button
        >
        <div class="ml-auto flex items-center gap-1">
          {@render chrome()}
        </div>
      </div>

      {#if paneOpen}
        <!-- Full-height on the right (below the floating chrome) by default,
             half-height along the bottom when the card is narrow or tall. Its bg
             occludes the stars behind it. -->
        <div
          class={[
            'absolute z-10 p-3',
            paneDock === 'bottom'
              ? 'inset-x-0 bottom-0 h-1/2'
              : 'top-[var(--bar-h)] right-0 bottom-0 w-[min(400px,58%)]',
          ]}
          bind:this={paneEl}
        >
          <DetailPane {map} ticket={paneTicket} dock={paneDock} {spaceId} onclose={closePane} {onspawned} />
        </div>
      {/if}
    </div>
  {:else}
    <!-- The picker screen: a header bar, over the space's maps split into a
         classified section (openable) and an unclassified one (classify in place
         first, ADR 0007). Each section is its own flex-wrap row, so the taller
         unclassified tiles never stretch the classified ones. -->
    <header class="cockpit-bar">
      <span class="min-w-0 flex-1 truncate text-sm font-semibold">Maps</span>
      <div class="flex items-center gap-1">
        {@render chrome()}
      </div>
    </header>

    <ScrollArea.Root class="min-h-0 flex-1">
      {#if maps.length}
        <div class="flex min-h-full flex-col gap-3 p-3">
          {#if classified.length}
            <!-- The openable maps flow from the top, unheaded. -->
            <div class="flex flex-wrap items-start gap-3">
              {#each classified as m (m.slug)}
                <MapPickerCard map={m} {spaceId} onopen={() => (slug = m.slug)} />
              {/each}
            </div>
          {/if}
          {#if unclassified.length}
            <!-- The maps still awaiting a kind, pinned to the bottom of the pane
                 (mt-auto) behind a divider — a standing to-do, out of the way of
                 the maps you actually open. -->
            <section class="mt-auto flex flex-col gap-2 border-t border-border pt-3">
              <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">Unclassified maps</h3>
              <div class="flex flex-wrap items-start gap-3">
                {#each unclassified as m (m.slug)}
                  <MapPickerCard map={m} {spaceId} onopen={() => (slug = m.slug)} />
                {/each}
              </div>
            </section>
          {/if}
        </div>
      {:else}
        <p class="grid h-full place-items-center p-6 text-sm text-muted-foreground">
          No maps in this space.
        </p>
      {/if}
    </ScrollArea.Root>
  {/if}
</section>
