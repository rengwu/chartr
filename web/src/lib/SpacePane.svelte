<script lang="ts">
  import type { Space } from './model'

  // The detail pane for the selected space: its effective role bindings and any
  // warnings. Every binding renders what will actually run, with a per-field
  // layer tag so field-level inheritance is visible rather than guessed (story
  // 39), and an absence badge when the adapter is not on PATH (story 40).
  let { space }: { space: Space } = $props()
</script>

<section class="pane">
  <header class="pane-head">
    <h2 class="pane-title">{space.name}</h2>
    <code class="pane-path">{space.path}</code>
  </header>

  {#if space.warnings && space.warnings.length}
    <ul class="warnings">
      {#each space.warnings as w}
        <li class="warning"><span aria-hidden="true">⚠</span> {w}</li>
      {/each}
    </ul>
  {/if}

  <h3 class="pane-sub">Effective role bindings</h3>
  <p class="pane-note">
    What each role resolves to after merging built-in ‹ workspace ‹ user. The tag on a
    field names the layer it was inherited from.
  </p>

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
</section>
