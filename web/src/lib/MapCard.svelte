<script lang="ts">
  import type { Map as WMap } from './model'
  import StarMap from './StarMap.svelte'

  // The star-map presented as a card over the terminal (spec, The interface):
  // summoned, never toggled by switching spaces or maps. This is only its chrome
  // — a slim header and the island. Whether it floats over the terminal or docks
  // as the terminal-priority split is the parent's layout decision; the card is
  // the same either way. The action station, ticket pane, and review hub that
  // will share this surface are later tickets.
  let {
    maps,
    slug = $bindable(),
    dock = $bindable(false),
    selected = $bindable(null),
    floatWidth = 0,
    onclose,
    onresizestart,
  }: {
    maps: WMap[]
    slug: string | null
    dock?: boolean
    selected?: number | null
    floatWidth?: number
    onclose: () => void
    onresizestart: (e: MouseEvent) => void
  } = $props()

  const map = $derived<WMap | null>(maps.find((m) => m.slug === slug) ?? maps[0] ?? null)
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
    {:else if map}
      <span class="map-card-name" title={map.destination || map.name}>{map.name}</span>
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

  <div class="map-card-body">
    {#if map}
      {#key map.slug}
        <StarMap {map} bind:selected />
      {/key}
    {:else}
      <p class="map-card-empty">No map to render in this space.</p>
    {/if}
  </div>
</section>
