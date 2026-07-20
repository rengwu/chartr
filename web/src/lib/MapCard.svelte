<script lang="ts">
  import type { Map as WMap, Ticket } from './model'
  import StarMap from './StarMap.svelte'
  import DetailPane from './DetailPane.svelte'

  // The star-map presented as a card over the terminal (spec, The interface):
  // summoned, never toggled by switching spaces or maps. It hosts the island and,
  // over it, the responsive detail pane (ticket 07) — docked right, re-docking to
  // the bottom when the card is narrow, with the camera easing the selected star
  // into the space the pane leaves free. Whether it floats or docks as the
  // terminal-priority split is the parent's layout decision.
  let {
    maps,
    slug = $bindable(),
    dock = $bindable(false),
    selected = $bindable(null),
    showMaterial = $bindable(false),
    floatWidth = 0,
    onclose,
    onresizestart,
  }: {
    maps: WMap[]
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

  // Responsive docking: right by default, bottom when the card is too narrow to
  // hold a side pane (spec: right dock, re-docking to bottom when narrow).
  const paneDock = $derived<'right' | 'bottom'>(bodyWidth > 0 && bodyWidth < 520 ? 'bottom' : 'right')

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
    const ro = new ResizeObserver(() => (bodyWidth = bodyEl.clientWidth))
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
</script>

<section
  class="map-card"
  class:docked={dock}
  aria-label="Star-map"
  style={!dock && floatWidth ? `width:${floatWidth}px` : ''}
>
  <!-- Resize from the left border: the split divider when docked, the card's
       leading edge when floating (the right edge stays pinned). A drag grip is a
       pointer affordance; keyboard users size the panes via the dock/float
       toggle's sensible defaults. -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="map-resizer"
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize star-map"
    onmousedown={onresizestart}
  ></div>

  <header class="map-card-bar">
    {#if maps.length > 1}
      <!-- Switching the focused map inside an open card never opens or closes it
           — visibility changes only on an explicit summon/dismiss (spec). -->
      <select
        class="map-switch"
        aria-label="Map"
        value={map?.slug}
        onchange={(e) => (slug = (e.currentTarget as HTMLSelectElement).value)}
      >
        {#each maps as m (m.slug)}
          <option value={m.slug}>{m.name}</option>
        {/each}
      </select>
      <button class="map-card-note" class:on={showMaterial} title="Map material" onclick={openMaterial}>notes</button>
    {:else if map}
      <button
        class="map-card-name"
        class:on={showMaterial}
        title="Open map material — destination, notes, decisions, fog"
        onclick={openMaterial}>{map.name}</button
      >
    {/if}

    <div class="map-card-actions">
      <button
        class="map-card-btn"
        aria-pressed={dock}
        title={dock ? 'Float over the terminal' : 'Dock as a split (terminal keeps its width)'}
        onclick={() => (dock = !dock)}>{dock ? '⧉ float' : '⇥ dock'}</button
      >
      <button class="map-card-btn map-card-close" aria-label="Dismiss star-map (Esc)" title="Dismiss (M / Esc)" onclick={onclose}
        >×</button
      >
    </div>
  </header>

  <div class="map-card-body" bind:this={bodyEl}>
    {#if map}
      {#key map.slug}
        <StarMap {map} {insets} bind:selected />
      {/key}

      {#if paneOpen}
        <div class="dp-holder" class:bottom={paneDock === 'bottom'} bind:this={paneEl}>
          <DetailPane {map} ticket={paneTicket} dock={paneDock} onclose={closePane} />
        </div>
      {/if}
    {:else}
      <p class="map-card-empty">No map to render in this space.</p>
    {/if}
  </div>
</section>
