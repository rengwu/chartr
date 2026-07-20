<script lang="ts">
  import { untrack } from 'svelte'
  import type { Map as WMap, Ticket } from './model'
  import StarMap from './StarMap.svelte'
  import DetailPane from './DetailPane.svelte'
  import { classifyMap } from './actions'
  import { decideDock, type Dock } from './starmap/dock'
  import { Button } from './components/ui/button'
  import { Columns, CornersOut, X } from 'phosphor-svelte'

  // The star-map presented as a card over the terminal (spec, The interface):
  // summoned, never toggled by switching spaces or maps. It hosts the island and,
  // over it, the responsive detail pane (ticket 07) — docked right, re-docking to
  // the bottom when the card is narrow, with the camera easing the selected star
  // into the space the pane leaves free. Whether it floats or docks as the
  // terminal-priority split is the parent's layout decision.
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
  }: {
    maps: WMap[]
    // The space these maps belong to — threaded to the detail pane so its payload
    // preview can fetch (ticket 08).
    spaceId: string
    slug: string | null
    dock?: boolean
    selected?: number | null
    showMaterial?: boolean
    floatWidth?: number
    onclose: () => void
    onresizestart: (e: MouseEvent) => void
  } = $props()

  const map = $derived<WMap | null>(maps.find((m) => m.slug === slug) ?? maps[0] ?? null)

  let bodyEl: HTMLDivElement
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
  // bottom pane the bottom edge; closed, only a small breathing margin.
  const insets = $derived(
    paneOpen
      ? paneDock === 'right'
        ? { top: 16, left: 16, bottom: 16, right: paneSize.w + 20 }
        : { top: 16, left: 16, right: 16, bottom: paneSize.h + 20 }
      : { top: 16, right: 16, bottom: 16, left: 16 },
  )

  $effect(() => {
    if (!bodyEl) return
    const measure = () => {
      bodyWidth = bodyEl.clientWidth
      bodyHeight = bodyEl.clientHeight
    }
    measure()
    const ro = new ResizeObserver(measure)
    ro.observe(bodyEl)
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

  function openMaterial() {
    selected = null
    showMaterial = true
  }
  function closePane() {
    selected = null
    showMaterial = false
  }

  // An unclassified map is inert until a human declares its kind (ADR 0007). The
  // declaration is normally recorded on creation by the wayfinder adapter, so
  // this confirm is the fallback for a map that arrived without one — and it
  // lives here, in the opened panel, never as buttons in the sidebar nav. The
  // convention guess is pre-emphasised; the resulting kind arrives over the
  // control socket and the banner clears itself.
  async function doClassify(kind: 'planning' | 'implementation') {
    if (!map) return
    try {
      await classifyMap(spaceId, map.slug, kind)
    } catch (e) {
      alert(`Couldn’t classify “${map.name}”: ${(e as Error).message}`)
    }
  }
  // p / i confirm the two kinds without reaching for the mouse; the guess is
  // still the pre-emphasised default for a plain click.
  function onKindKey(e: KeyboardEvent) {
    if (e.key === 'p') doClassify('planning')
    else if (e.key === 'i') doClassify('implementation')
  }
</script>

<!-- The star-map card. Docked, it is a flex item in the panes row, taking the
     slack the terminal's frozen basis leaves (min 300). Floating, it is absolute
     over the panes row with its right edge pinned (inset 10px), grown leftward by
     the resize grip; a CSS max-width keeps it within the row on a window resize. -->
<section
  class={[
    'flex min-h-0 flex-col overflow-hidden bg-card',
    dock
      ? // Docked, this is a flush column next to the terminal — a plain square
        // edge like the terminal's own outer edges, with a border only on the
        // seam it shares with the terminal (the split divider). No radius, no
        // border on the other three sides, so there's nothing to misalign by
        // even a pixel against the terminal's borderless top/bottom.
        'relative min-w-[300px] flex-1 border-l border-border'
      : 'absolute inset-y-2 right-2.5 z-20 w-[min(400px,58%)] max-w-[calc(100%-40px)] rounded-lg border border-border shadow-lg',
  ]}
  aria-label="Star-map"
  style={!dock && floatWidth ? `width:${floatWidth}px` : ''}
>
  <!-- Resize from the left border: the split divider when docked, the card's
       leading edge when floating (the right edge stays pinned). A drag grip is a
       pointer affordance; keyboard users size the panes via the dock/float
       toggle's sensible defaults. -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="absolute inset-y-0 left-0 z-30 w-1.5 -translate-x-1/2 cursor-ew-resize transition-colors hover:bg-ring/60"
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize star-map"
    onmousedown={onresizestart}
  ></div>

  <header class="cockpit-bar">
    {#if maps.length > 1}
      <!-- Switching the focused map inside an open card never opens or closes it
           — visibility changes only on an explicit summon/dismiss (spec). -->
      <select
        class="max-w-[12rem] min-w-0 truncate rounded-md border border-border bg-transparent px-2 py-1 text-xs font-medium text-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:outline-none"
        aria-label="Map"
        value={map?.slug}
        onchange={(e) => (slug = (e.currentTarget as HTMLSelectElement).value)}
      >
        {#each maps as m (m.slug)}
          <option value={m.slug}>{m.name}</option>
        {/each}
      </select>
      <Button
        variant={showMaterial ? 'secondary' : 'ghost'}
        size="sm"
        aria-pressed={showMaterial}
        title="Map material"
        onclick={openMaterial}>notes</Button
      >
    {:else if map}
      <Button
        variant={showMaterial ? 'secondary' : 'ghost'}
        size="sm"
        class="min-w-0 truncate"
        aria-pressed={showMaterial}
        title="Open map material — destination, notes, decisions, fog"
        onclick={openMaterial}>{map.name}</Button
      >
    {/if}

    <div class="ml-auto flex items-center gap-1">
      <Button
        variant="ghost"
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
    </div>
  </header>

  <div class="relative min-h-0 flex-1" bind:this={bodyEl}>
    {#if map}
      {#key map.slug}
        <StarMap {map} {insets} bind:selected />
      {/key}

      {#if map.kind === ''}
        <!-- The classify confirm, surfaced only here and only when the opened map
             is unclassified (ADR 0007) — the "deeper in the UI, when it's really
             needed" home, not the sidebar. Pre-emphasise the convention guess;
             either key/button declares the kind, and the map goes live. -->
        <!-- The p/i keys are an enhancement over the two focusable buttons, not the
             only path — the group is a convenience listener, not an interactive
             control. -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="absolute top-3 left-3 right-3 z-20 flex flex-wrap items-center gap-2 rounded-md border border-border bg-card/95 px-3 py-2 text-xs shadow-md backdrop-blur"
          role="group"
          aria-label="Classify {map.name}"
          onkeydown={onKindKey}
          style={paneOpen && paneDock === 'right' ? `right:${paneSize.w + 12}px` : ''}
        >
          <span class="min-w-0 flex-1 text-muted-foreground">
            <span class="font-medium text-foreground">Unclassified.</span>
            No sessions run until you set this map’s kind.
          </span>
          <Button
            size="xs"
            variant={map.kindGuess === 'planning' ? 'default' : 'outline'}
            title="Planning map — tickets resolve live, no review gate (p)"
            onclick={() => doClassify('planning')}>plan</Button
          >
          <Button
            size="xs"
            variant={map.kindGuess === 'implementation' ? 'default' : 'outline'}
            title="Implementation map — tickets pass through review before resolving (i)"
            onclick={() => doClassify('implementation')}>impl</Button
          >
        </div>
      {/if}

      {#if paneOpen}
        <!-- The detail pane overlays one edge of the island: full-height on the
             right by default, half-height along the bottom when the card is narrow
             or tall. Its bg occludes the stars behind it; the camera (insets) eases
             the selected star into the space it leaves free. -->
        <div
          class={[
            'absolute z-10 p-3',
            paneDock === 'bottom' ? 'inset-x-0 bottom-0 h-1/2' : 'inset-y-0 right-0 w-[min(400px,58%)]',
          ]}
          bind:this={paneEl}
        >
          <DetailPane {map} ticket={paneTicket} dock={paneDock} {spaceId} onclose={closePane} />
        </div>
      {/if}
    {:else}
      <p class="grid h-full place-items-center p-6 text-sm text-muted-foreground">
        No map to render in this space.
      </p>
    {/if}
  </div>
</section>
