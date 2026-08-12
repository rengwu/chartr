<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ConfigLayer, NotifyPrefs } from "./model";
  import { Bell } from "phosphor-svelte";
  import { openExternal } from "./external";

  let {
    prefs,
    layer,
    layerRow,
  }: {
    prefs?: NotifyPrefs;
    layer?: ConfigLayer;
    layerRow: Snippet<[ConfigLayer]>;
  } = $props();

  const rows = $derived([
    { label: "Notify after", value: prefs?.after ?? "1m0s" },
    { label: "Settle delay", value: prefs?.settle ?? "10s" },
    { label: "Enabled", value: (prefs?.enabled ?? true) ? "true" : "false" },
  ]);
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <Bell class="size-3.5 shrink-0" aria-hidden="true" /> Notifications
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Controls when to notify a user when a long-running session stops working.
    <a
      href="https://github.com/rengwu/chartr/blob/main/internal/config/notify.scaffold.toml"
      class="text-primary underline-offset-4 hover:underline"
      onclick={(e) => {
        e.preventDefault();
        openExternal(e.currentTarget.href);
      }}>Reference</a
    >
  </p>

  {#if layer}
    {@render layerRow(layer)}
  {/if}
</section>
