<script lang="ts">
  import type { Space, Terminal as Term, Map as WMap } from './model'
  import { closeTerminal, openTerminal } from './actions'
  import Terminal from './Terminal.svelte'
  import MapCard from './MapCard.svelte'

  // The stage for the selected space, built as the terminal column of ticket
  // 11's prototype: a tab strip flush at the top (the space's ad-hoc shells plus
  // a "+"), and the active terminal filling the rest of the height. A mapless
  // space is fully usable this way (story 29). Space identity is a slim leading
  // label, not a heading that pushes the terminal down; the effective role
  // bindings (stories 39, 40) live in a right-docked drawer summoned from the
  // bar, so they never occupy the terminal's real estate.
  //
  // Over the terminal, the star-map is summoned as a floating card — edge handle,
  // M, Esc — or docked as the terminal-priority split (spec, The interface).
  // Visibility changes only on those explicit acts: switching spaces or focusing
  // a different map never opens or closes it, which is why this state lives on
  // the pane (persisting across space switches) and not per-map.
  let { space }: { space: Space } = $props()

  let activeId = $state<string | null>(null)
  let opening = $state(false)
  let showBindings = $state(false)

  // Star-map card state (persists across space switches by design).
  let mapShown = $state(false)
  let dock = $state(false)
  let mapSlug = $state<string | null>(null)
  let selectedTicket = $state<number | null>(null)
  let dockTermWidth = $state(0)
  let floatWidth = $state(0)
  let bodyEl: HTMLDivElement
  let termColEl: HTMLElement

  const terminals = $derived<Term[]>(space.terminals ?? [])
  const warnings = $derived<string[]>(space.warnings ?? [])
  const maps = $derived<WMap[]>(space.maps ?? [])

  // A stale slug (a map that vanished, or a switch to a space without it) falls
  // back to the first map, so the card always has something to render.
  const focusedMap = $derived<WMap | null>(
    maps.find((m) => m.slug === mapSlug) ?? maps[0] ?? null,
  )

  // A selection belongs to one map: when the focused map changes, drop it so the
  // island never carries a ticket number from a different graph.
  let lastSlug = ''
  $effect(() => {
    const slug = focusedMap?.slug ?? ''
    if (slug !== lastSlug) {
      lastSlug = slug
      selectedTicket = null
    }
  })

  // Freeze the terminal's pixel width at the moment of docking, then let the map
  // absorb every later resize slack — the terminal-priority split holds its width
  // so a window resize never reflows it (planning ticket 08's amendment).
  function summon() {
    mapShown = true
  }
  function dismiss() {
    mapShown = false
  }
  function toggleMap() {
    if (mapShown) dismiss()
    else summon()
  }
  $effect(() => {
    if (mapShown && dock && bodyEl && !dockTermWidth) {
      const w = bodyEl.clientWidth
      // First dock: terminal keeps ~60%, always leaving room for the map; clamped
      // so neither pane collapses on a narrow window. A resize below overrides it.
      dockTermWidth = Math.round(Math.min(Math.max(w * 0.6, 320), Math.max(360, w - 360)))
    }
  })

  // Drag the card's left border to resize it — in either mode. Docked, the
  // border is the split: the map's edge moves and the terminal's frozen width
  // follows it. Floating, the card grows leftward while its right edge stays
  // pinned. Clamped so neither pane collapses.
  const MIN_MAP = 300
  const FLOAT_INSET = 10 // matches .map-floating .map-card right offset
  function startResize(e: MouseEvent) {
    e.preventDefault()
    const rect = bodyEl.getBoundingClientRect()
    const move = (ev: MouseEvent) => {
      if (dock) {
        const minTerm = 240
        dockTermWidth = Math.round(
          Math.min(Math.max(ev.clientX - rect.left, minTerm), Math.max(minTerm, rect.width - MIN_MAP)),
        )
      } else {
        const maxMap = Math.max(MIN_MAP, rect.width - 120)
        floatWidth = Math.round(
          Math.min(Math.max(rect.right - FLOAT_INSET - ev.clientX, MIN_MAP), maxMap),
        )
      }
    }
    const up = () => {
      window.removeEventListener('mousemove', move)
      window.removeEventListener('mouseup', up)
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }
    document.body.style.cursor = 'ew-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', move)
    window.addEventListener('mouseup', up)
  }

  // M summons/dismisses, Esc dismisses — but only when focus is on the chrome,
  // not inside the terminal (whose PTY owns every raw keystroke) or a text field.
  // The edge handle is the always-available path when the shell has the keyboard.
  function onKey(e: KeyboardEvent) {
    const el = document.activeElement as HTMLElement | null
    const editing =
      !!el &&
      (el.tagName === 'INPUT' ||
        el.tagName === 'TEXTAREA' ||
        el.isContentEditable ||
        el.closest('.terminal-island') !== null)
    if (e.key === 'Escape' && mapShown && !editing) {
      dismiss()
      return
    }
    if ((e.key === 'm' || e.key === 'M') && !editing && maps.length && !e.metaKey && !e.ctrlKey) {
      e.preventDefault()
      toggleMap()
    }
  }

  // The active tab falls back to the first terminal when the id is stale — a
  // just-closed shell, or a switch to a different space — so the column never
  // shows a blank island while terminals remain.
  const active = $derived.by(() => {
    return terminals.find((t) => t.id === activeId) ?? terminals[0] ?? null
  })

  async function openShell() {
    opening = true
    try {
      const { id } = await openTerminal(space.id)
      activeId = id
    } catch (e) {
      alert(`Couldn’t open a shell: ${(e as Error).message}`)
    } finally {
      opening = false
    }
  }

  async function endShell(t: Term) {
    if (activeId === t.id) activeId = null
    try {
      await closeTerminal(space.id, t.id)
    } catch (e) {
      alert(`Couldn’t end “${t.title}”: ${(e as Error).message}`)
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div
  class="space-body"
  class:map-docked={mapShown && dock}
  class:map-floating={mapShown && !dock}
  bind:this={bodyEl}
>
  <section
    class="term-col"
    bind:this={termColEl}
    style={mapShown && dock ? `flex: 0 0 ${dockTermWidth}px` : ''}
  >
  <div class="term-bar">
    <div class="term-id" title={space.path}>
      <span class="term-id-name">{space.name}</span>
      <code class="term-id-path">{space.path}</code>
    </div>

    <div class="term-tabs" role="tablist">
      {#each terminals as t (t.id)}
        <div class="term-tab" class:active={active?.id === t.id} class:dead={!t.alive}>
          <button
            class="term-tab-main"
            role="tab"
            aria-selected={active?.id === t.id}
            title={t.alive ? t.title : `${t.title} (ended)`}
            onclick={() => (activeId = t.id)}
          >
            <span class="term-dot" aria-hidden="true"></span>
            {t.title}{#if !t.alive}<span class="term-ended"> · ended</span>{/if}
          </button>
          <button
            class="term-close"
            aria-label="End {t.title}"
            title={t.alive ? 'End this shell' : 'Dismiss'}
            onclick={() => endShell(t)}>×</button
          >
        </div>
      {/each}
      <button
        class="term-add"
        aria-label="Open a shell in the working tree"
        title="Open a shell in {space.name}"
        disabled={opening}
        onclick={openShell}>＋</button
      >
    </div>

    <div class="term-actions">
      {#if warnings.length}
        <span class="term-warn-pip" title={warnings.join('\n')} aria-label="{warnings.length} warning(s)">⚠ {warnings.length}</span>
      {/if}
      <button
        class="term-config"
        aria-pressed={showBindings}
        title="Effective role bindings"
        onclick={() => (showBindings = !showBindings)}>bindings</button
      >
    </div>
  </div>

  <div class="term-body">
    {#if active}
      {#key active.id}
        <Terminal term={active} />
      {/key}
    {:else}
      <div class="term-empty">
        <p>No shell open in this space.</p>
        <button class="term-empty-open" disabled={opening} onclick={openShell}
          >Open a shell</button
        >
        <p class="term-empty-note">
          A plain shell in <code>{space.name}</code>’s working tree — no ticket, no review,
          ended when you close it.
        </p>
      </div>
    {/if}

    {#if showBindings}
      <aside class="bindings-drawer" aria-label="Effective role bindings">
        <header class="drawer-head">
          <h3 class="drawer-title">Effective role bindings</h3>
          <button class="drawer-close" aria-label="Close bindings" onclick={() => (showBindings = false)}>×</button>
        </header>
        <p class="pane-note">
          What each role resolves to after merging built-in ‹ workspace ‹ user. The tag on a
          field names the layer it was inherited from.
        </p>

        {#if warnings.length}
          <ul class="warnings">
            {#each warnings as w}
              <li class="warning"><span aria-hidden="true">⚠</span> {w}</li>
            {/each}
          </ul>
        {/if}

        <ul class="bindings">
          {#each space.bindings as b (b.role)}
            <li class="binding" class:absent={!b.present}>
              <div class="binding-role">{b.role}</div>
              <div class="binding-fields">
                <span class="field">
                  <span class="field-val">{b.adapter}</span>
                  <span class="field-src" data-layer={b.adapterFrom}>{b.adapterFrom}</span>
                </span>
                <span class="field">
                  <span class="field-val">{b.model}</span>
                  <span class="field-src" data-layer={b.modelFrom}>{b.modelFrom}</span>
                </span>
                {#if b.args && b.args.length}
                  <span class="field">
                    <span class="field-val">{b.args.join(' ')}</span>
                    <span class="field-src" data-layer={b.argsFrom}>{b.argsFrom}</span>
                  </span>
                {/if}
              </div>
              {#if b.present}
                <div class="binding-status ok"><span aria-hidden="true">●</span> on PATH</div>
              {:else}
                <div class="binding-status missing"><span aria-hidden="true">▲</span> not found</div>
              {/if}
            </li>
            {#if !b.present && b.missing}
              <li class="binding-missing">{b.missing}</li>
            {/if}
          {/each}
        </ul>
      </aside>
    {/if}
  </div>
  </section>

  {#if maps.length && !mapShown}
    <!-- The always-available summon: the edge handle, live even while the shell
         owns the keyboard. A later ticket hangs the action-station badge here. -->
    <button
      class="map-handle"
      aria-label="Summon the star-map (M)"
      title="Star-map (M)"
      onclick={summon}
    >
      <span class="map-handle-glyph" aria-hidden="true">✦</span>
      <span class="map-handle-label">MAP</span>
    </button>
  {/if}

  {#if focusedMap && mapShown}
    <MapCard
      {maps}
      bind:slug={mapSlug}
      bind:dock
      bind:selected={selectedTicket}
      {floatWidth}
      onclose={dismiss}
      onresizestart={startResize}
    />
  {/if}
</div>
