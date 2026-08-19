<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ConfigLayer } from "./model";
  import { Brain } from "phosphor-svelte";

  // The System Prompts section: the prompts chartr composes into every session —
  // its two embedded cores (the per-ticket core and the standing-space core) and
  // the operator's own preferences. Each traces to a file under the config root;
  // absent, chartr composes its shipped default (an empty file, for preferences).
  // The row itself — open, Create from defaults, its busy and note state — is the
  // Settings surface's shared snippet, so nothing about resolution is reimplemented
  // here.
  let {
    layers,
    layerRow,
  }: {
    // The three prompt layers, in display order: ticket core, space core,
    // preferences. Any the server did not send are already filtered out upstream.
    layers: ConfigLayer[];
    // The shared files-on-disk row, so the open and create actions, their busy
    // state and the editor-ladder note all stay owned by the Settings surface.
    layerRow: Snippet<[ConfigLayer]>;
  } = $props();
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <Brain class="size-4 shrink-0" aria-hidden="true" /> System Prompts
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    The prompts chartr injects into spawned sessions. Delete the file to reset
    it to default.
  </p>

  {#each layers as layer (layer.name)}
    {@render layerRow(layer)}
  {/each}
</section>
