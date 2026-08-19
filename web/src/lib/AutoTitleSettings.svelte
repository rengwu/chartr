<script lang="ts">
  import type { Snippet } from "svelte";
  import type { AutoTitlePrefs, ConfigLayer } from "./model";
  import { TextAa } from "phosphor-svelte";
  import { openExternal } from "./external";

  let {
    prefs,
    layer,
    layerRow,
  }: {
    prefs?: AutoTitlePrefs;
    layer?: ConfigLayer;
    layerRow: Snippet<[ConfigLayer]>;
  } = $props();

  const rows = $derived([
    { label: "Enabled", value: (prefs?.enabled ?? true) ? "true" : "false" },
    {
      label: "Native only",
      value: (prefs?.nativeOnly ?? false) ? "true" : "false",
    },
  ]);
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <TextAa class="size-4 shrink-0" aria-hidden="true" /> Auto titles
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Gives each agent tab a short title from the session running in it, so you
    can tell your tabs apart at a glance.
    <a
      href="https://github.com/rengwu/chartr/blob/main/internal/config/autotitle.scaffold.toml"
      class="text-primary underline-offset-4 hover:underline"
      onclick={(e) => {
        e.preventDefault();
        openExternal(e.currentTarget.href);
      }}>Reference</a
    >
  </p>

  <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
    {#each rows as row (row.label)}
      <dt class="text-muted-foreground">{row.label}</dt>
      <dd class="font-mono">{row.value}</dd>
    {/each}
  </dl>

  {#if layer}
    {@render layerRow(layer)}
  {/if}
</section>
