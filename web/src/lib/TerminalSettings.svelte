<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ConfigLayer } from "./model";
  import { Terminal as TerminalIcon } from "phosphor-svelte";
  import { openExternal } from "./external";

  // The Terminal section of the global scope (ticket 08): a row that opens
  // `terminal.toml` in the operator's own editor. There is deliberately no control
  // here that writes anything, because a second way to set these values would be a
  // second config store (spec, Storage & ownership).
  let {
    layer,
    layerRow,
  }: {
    // The `terminal.toml` config layer — its path and whether it exists yet. Absent
    // only if an older server did not send it, in which case the open row is simply
    // not offered.
    layer?: ConfigLayer;
    // The shared files-on-disk row, passed in so the open action, its busy state
    // and its editor-ladder note all stay owned by the Settings surface rather than
    // being reimplemented here.
    layerRow: Snippet<[ConfigLayer]>;
  } = $props();
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <TerminalIcon class="icon-size-md shrink-0" aria-hidden="true" /> Terminal
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Customize the look and behavior of the <code>xterm</code> terminal sessions.
    <a
      href="https://github.com/rengwu/chartr/blob/main/internal/config/terminal.scaffold.toml"
      class="text-primary underline-offset-4 hover:underline"
      onclick={(e) => {
        e.preventDefault();
        openExternal(e.currentTarget.href);
      }}
      >Reference</a
    >
  </p>

  {#if layer}
    {@render layerRow(layer)}
  {/if}
</section>
