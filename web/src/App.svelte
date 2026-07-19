<script lang="ts">
  import { onMount } from 'svelte'
  import { ControlSocket } from './lib/control.svelte'
  import { needsAgents, type Space } from './lib/model'
  import { deregisterSpace, setPin } from './lib/actions'
  import RegisterForm from './lib/RegisterForm.svelte'
  import SpacePane from './lib/SpacePane.svelte'

  // The one control socket for this browser. The chrome renders whatever the
  // latest snapshot holds and reacts to every push (ADR 0010).
  const control = new ControlSocket()

  onMount(() => {
    control.connect()
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
</script>

<div class="cockpit">
  <aside class="sidebar">
    <div class="sidebar-head">
      <span>Spaces</span>
      {#if spaces.length > 0}
        <button
          class="icon-btn"
          aria-label="Add a space"
          aria-expanded={showAdd}
          onclick={() => (showAdd = !showAdd)}>＋</button
        >
      {/if}
    </div>

    {#if control.model === null}
      <p class="hint">Connecting…</p>
    {:else if spaces.length === 0}
      <p class="hint">No spaces yet.</p>
    {:else}
      {#if showAdd}
        <div class="sidebar-add">
          <RegisterForm variant="inline" onRegistered={(id) => { selectedId = id; showAdd = false }} />
        </div>
      {/if}

      <input
        class="filter"
        type="text"
        placeholder="Filter spaces…"
        bind:value={filter}
        spellcheck="false"
        autocapitalize="off"
        autocomplete="off"
        aria-label="Filter spaces"
      />

      <ul class="space-list">
        {#each filtered as space (space.id)}
          <li class="space-row" class:active={selected?.id === space.id}>
            <button class="row-main" onclick={() => (selectedId = space.id)}>
              <span class="row-name">
                {space.name}
                {#if needsAgents(space)}
                  <span class="row-badge" title="An agent for one or more roles isn’t on your PATH">
                    <span aria-hidden="true">▲</span> agent
                  </span>
                {/if}
              </span>
              <span class="row-path">{space.path}</span>
            </button>
            <button
              class="icon-btn row-pin"
              class:pinned={space.pinned}
              aria-pressed={space.pinned}
              aria-label={space.pinned ? 'Unpin space' : 'Pin space'}
              title={space.pinned ? 'Unpin' : 'Pin to top'}
              onclick={() => togglePin(space)}>📌</button
            >
            <button
              class="icon-btn row-forget"
              aria-label="Forget space"
              title="Forget (repository untouched)"
              onclick={() => forget(space)}>×</button
            >
          </li>
        {:else}
          <li class="space-empty">No spaces match “{filter}”.</li>
        {/each}
      </ul>
    {/if}
  </aside>

  <main class="stage">
    {#if spaces.length === 0}
      <div class="stage-center">
        <RegisterForm variant="first-run" onRegistered={(id) => (selectedId = id)} />
      </div>
    {:else if selected}
      <SpacePane space={selected} />
    {/if}
  </main>

  <footer class="statusbar" data-status={control.status}>
    <span class="dot" aria-hidden="true"></span>
    <span>control socket: {control.status}</span>
  </footer>
</div>
