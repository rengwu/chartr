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
  ]);
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <TextAa class="size-3.5 shrink-0" aria-hidden="true" /> Auto titles
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Summarises an idle agent tab's recent screen into a short title, shown after
    its label. It spends a cheap model on the tab's own agent — a claude session
    is summarised by claude, a codex session by codex — so a session's screen
    never goes to a vendor you didn't run it on.
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
