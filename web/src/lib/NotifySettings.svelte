<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { ConfigLayer, NotifyPrefs } from './model'
  import { Bell } from 'phosphor-svelte'

  let {
    prefs,
    layer,
    layerRow,
  }: {
    prefs?: NotifyPrefs
    layer?: ConfigLayer
    layerRow: Snippet<[ConfigLayer]>
  } = $props()

  const rows = $derived([
    { label: 'Notify after', value: prefs?.after ?? '1m0s' },
    { label: 'Settle delay', value: prefs?.settle ?? '10s' },
    { label: 'Enabled', value: (prefs?.enabled ?? true) ? 'true' : 'false' },
  ])
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <Bell class="size-3.5 shrink-0" aria-hidden="true" /> Notifications
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    When a long-running session stops working. One per-machine
    <code class="font-mono">notify.toml</code> controls the threshold, settle delay,
    and source switch. Edit it in your own editor; invalid values keep their defaults
    and surface a warning on each space.
  </p>

  {#if layer}
    {@render layerRow(layer)}
  {/if}

  <section class="flex flex-col gap-1.5 rounded-md border border-border px-2.5 py-2">
    <h3 class="text-xs font-semibold">Parsed config values</h3>
    <dl class="grid grid-cols-[max-content_1fr] items-baseline gap-x-4 gap-y-0.5">
      {#each rows as row (row.label)}
        <dt class="text-[0.7rem] text-muted-foreground">{row.label}</dt>
        <dd class="truncate font-mono text-[0.7rem] font-medium">{row.value}</dd>
      {/each}
    </dl>
  </section>
</section>
