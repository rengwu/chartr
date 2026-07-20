<script lang="ts">
  import { onMount } from 'svelte'
  import type { Map as WMap } from './model'
  import { StarMap } from './starmap/starmap'

  // The star-map is an imperative island: the Svelte chrome hosts it but never
  // reaches inside (ADR 0010). This wrapper mounts the renderer, feeds it the
  // pushed model, and lifts selection back out through the narrow seam (mount,
  // receive model, emit selection). Everything about how a star looks or moves
  // lives inside the island, not here.
  let { map, selected = $bindable(null), insets }: {
    map: WMap
    selected?: number | null
    // The detail pane's footprint, so the island eases stars into the space the
    // pane leaves free (ticket 07). Undefined until a pane opens.
    insets?: { top: number; right: number; bottom: number; left: number }
  } = $props()

  let host: HTMLDivElement
  let island: StarMap | null = null

  onMount(() => {
    const sm = new StarMap()
    island = sm
    sm.onSelect((n) => (selected = n))
    sm.mount(host)
    sm.setModel(map.tickets)
    return () => {
      island = null
      sm.destroy()
    }
  })

  // Every control-socket push re-feeds the island. A structure-preserving push
  // (only statuses changed) never moves a star; the island enforces that.
  $effect(() => {
    island?.setModel(map.tickets)
  })

  // A selection set from outside (a deep-link, the queue) eases the star in.
  $effect(() => {
    island?.select(selected)
  })

  // The pane's measured footprint re-eases the camera into the free space.
  $effect(() => {
    if (insets) island?.setInsets(insets)
  })
</script>

<div class="starmap-island" bind:this={host}></div>
