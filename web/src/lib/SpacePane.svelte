<script lang="ts">
  import type { Space, Terminal as Term } from './model'
  import { closeTerminal, openTerminal } from './actions'
  import Terminal from './Terminal.svelte'

  // The stage for the selected space, built as the terminal column of ticket
  // 11's prototype: a tab strip flush at the top (the space's ad-hoc shells plus
  // a "+"), and the active terminal filling the rest of the height. A mapless
  // space is fully usable this way (story 29). Space identity is a slim leading
  // label, not a heading that pushes the terminal down; the effective role
  // bindings (stories 39, 40) live in a right-docked drawer summoned from the
  // bar, so they never occupy the terminal's real estate. The star-map card and
  // session tabs are later tickets.
  let { space }: { space: Space } = $props()

  let activeId = $state<string | null>(null)
  let opening = $state(false)
  let showBindings = $state(false)

  const terminals = $derived<Term[]>(space.terminals ?? [])
  const warnings = $derived<string[]>(space.warnings ?? [])

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

<section class="term-col">
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
