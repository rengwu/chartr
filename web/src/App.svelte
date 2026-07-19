<script lang="ts">
  import { onMount } from 'svelte'
  import { ControlSocket } from './lib/control.svelte'

  // The one control socket for this browser. The chrome renders whatever the
  // latest snapshot holds — near-empty for now.
  const control = new ControlSocket()

  onMount(() => {
    control.connect()
    return () => control.close()
  })

  const spaces = $derived(control.model?.spaces ?? [])
</script>

<div class="cockpit">
  <aside class="sidebar">
    <div class="sidebar-head">Spaces</div>
    {#if control.model === null}
      <p class="hint">Connecting…</p>
    {:else if spaces.length === 0}
      <p class="hint">No spaces yet. Register your first space to begin.</p>
    {:else}
      <ul class="space-list">
        {#each spaces as space (space.id)}
          <li>{space.name}</li>
        {/each}
      </ul>
    {/if}
  </aside>

  <main class="stage">
    <div class="stage-empty">
      <span class="stage-title">wayfinder-harness</span>
      <span class="stage-sub">The cockpit shell is live. Terminals, the star-map, and the review hub land in later tickets.</span>
    </div>
  </main>

  <footer class="statusbar" data-status={control.status}>
    <span class="dot" aria-hidden="true"></span>
    <span>control socket: {control.status}</span>
  </footer>
</div>
