<script lang="ts">
  import { onMount, untrack } from 'svelte'
  import type { Map as WMap, Terminal } from './model'
  import { StarMap } from './starmap/starmap'
  import { sessionStates } from './starmap/session'
  import { cameraOf, rememberCamera } from './mapstate'
  import { readColor } from './tokens'

  // The star-map is an imperative island: the Svelte chrome hosts it but never
  // reaches inside (ADR 0010). This wrapper mounts the renderer, feeds it the
  // pushed model, and lifts selection back out through the narrow seam (mount,
  // receive model, emit selection). Everything about how a star looks or moves
  // lives inside the island, not here.
  let { map, terminals = [], selected = $bindable(null), insets, cameraKey }: {
    map: WMap
    // The space's tabs, so the island can paint the session overlay (ticket 13):
    // a session's liveness and its pipeline stage are derived from the same push
    // that carries the tickets, never stored.
    terminals?: Terminal[]
    selected?: number | null
    // The detail pane's footprint, so the island eases stars into the space the
    // pane leaves free (ticket 07). Undefined until a pane opens.
    insets?: { top: number; right: number; bottom: number; left: number }
    // Which map, in which space, this island is showing — the key its camera pose
    // is remembered under while the island itself is not alive (mapstate.ts). The
    // wrapper owns this end of it: the renderer knows nothing about spaces, and
    // the chrome knows nothing about cameras.
    cameraKey?: string
  } = $props()

  let host: HTMLDivElement
  let island: StarMap | null = null

  // The pose this map was last left at, if this island is a return rather than a
  // first arrival. Captured at construction and spent by the effect at the foot
  // of this file; untrack marks the one-time read as deliberate, which it safely
  // is — the card keys the island's remount on this same string, so it cannot
  // change under a living island.
  const key = untrack(() => cameraKey)
  let pose = key ? cameraOf(key) : null

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
      // Hand the camera out before the island dies, so switching space (or back
      // to the picker) is somewhere to return to rather than a fresh fit.
      if (key) rememberCamera(key, sm.camera())
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

  // The pane's measured footprint re-eases the camera into the free space.
  $effect(() => {
    if (insets) island?.setInsets(insets)
  })

  // Put the remembered pose back — declared last so it runs last in the mount
  // flush, after the fit, the restored star's seat and the pane's insets have
  // each moved the camera. Every one of those is the island working out where a
  // *new* map should sit; this is not a new map, and the operator already said
  // where it sits. Reads nothing reactive, so it runs exactly once.
  $effect(() => {
    if (!pose) return
    island?.restoreCamera(pose)
    pose = null
  })
</script>

<div class="starmap-island h-full w-full" bind:this={host}></div>
