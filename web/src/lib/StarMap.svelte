<script lang="ts">
  import { onMount } from 'svelte'
  import type { Map as WMap, Terminal } from './model'
  import { StarMap } from './starmap/starmap'
  import { sessionStates } from './starmap/session'
  import { readColor } from './tokens'

  // The star-map is an imperative island: the Svelte chrome hosts it but never
  // reaches inside (ADR 0010). This wrapper mounts the renderer, feeds it the
  // pushed model, and lifts selection back out through the narrow seam (mount,
  // receive model, emit selection). Everything about how a star looks or moves
  // lives inside the island, not here.
  let { map, terminals = [], selected = $bindable(null), hoverNum = null, insets }: {
    map: WMap
    // The space's tabs, so the island can paint the session overlay (ticket 13):
    // a session's liveness and its pipeline stage are derived from the same push
    // that carries the tickets, never stored.
    terminals?: Terminal[]
    selected?: number | null
    // The action station's hovered row (ticket 14) — highlights a star without
    // selecting it. Driven from outside; the island never originates this itself.
    hoverNum?: number | null
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
    // The card surface colour, resolved off the live tokens at the seam so the
    // field paints the same warm near-black as the reskinned chrome (ticket 04).
    sm.setBackground(readColor('--card'))
    sm.setModel(map.tickets, sessionStates(map, terminals))
    return () => {
      island = null
      sm.destroy()
    }
  })

  // Every control-socket push re-feeds the island — tickets and the session
  // overlay together, so one push is one visual beat. A structure-preserving push
  // (only statuses or session states changed) never moves a star; the island
  // enforces that.
  $effect(() => {
    island?.setModel(map.tickets, sessionStates(map, terminals))
  })

  // A selection set from outside (a deep-link, the queue) eases the star in.
  $effect(() => {
    island?.select(selected)
  })

  // Hovering a row in the action station highlights its star, with no camera
  // move — a lighter echo of selection (ticket 14).
  $effect(() => {
    island?.hover(hoverNum ?? null)
  })

  // The pane's measured footprint re-eases the camera into the free space.
  $effect(() => {
    if (insets) island?.setInsets(insets)
  })
</script>

<div class="starmap-island h-full w-full" bind:this={host}></div>
