<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ConfigLayer, TerminalPrefs } from "./model";
  import { terminalSettingsSummary } from "./terminalsummary";
  import * as Accordion from "./components/ui/accordion";
  import { Terminal as TerminalIcon } from "phosphor-svelte";

  // The Terminal section of the global scope (ticket 08): what the operator's
  // `terminal.toml` currently puts in force, and a row that opens the file in
  // their own editor. Read-value-plus-open-file — there is deliberately no control
  // here that writes anything, because a second way to set these values would be a
  // second config store (spec, Storage & ownership).
  //
  // Every value comes from the same resolve the terminal island mounts with
  // (`terminalSettingsSummary` over Seam 2), so this can never show one thing while
  // the terminal does another. A row the file did not set still shows the default
  // genuinely in force, in muted text; a row the file *did* set is emphasised.
  let {
    prefs,
    layer,
    layerRow,
  }: {
    // The resolved prefs off the model snapshot. Absent means an untouched machine:
    // every row renders its default, which is exactly the honest answer.
    prefs?: TerminalPrefs;
    // The `terminal.toml` config layer — its path and whether it exists yet. Absent
    // only if an older server did not send it, in which case the open row is simply
    // not offered.
    layer?: ConfigLayer;
    // The shared files-on-disk row, passed in so the open action, its busy state
    // and its editor-ladder note all stay owned by the Settings surface rather than
    // being reimplemented here.
    layerRow: Snippet<[ConfigLayer]>;
  } = $props();

  // Re-resolved whenever the prefs change, off the live tokens — so a colour slot
  // the file left unset tracks the app theme exactly as the terminal's own does.
  const groups = $derived(terminalSettingsSummary(prefs));

  // Which sections are expanded. Start with the first open so the panel is never a
  // wall of headers with nothing showing; the rest collapse away until asked for.
  // Seeded from a one-off resolve (the titles are fixed, so this never goes stale).
  // svelte-ignore state_referenced_locally
  let open = $state<string[]>([terminalSettingsSummary(prefs)[0].title]);
</script>

<section class="flex flex-col gap-2">
  <h2 class="flex items-center gap-1.5 text-xs font-semibold">
    <TerminalIcon class="size-3.5 shrink-0" aria-hidden="true" /> Terminal
  </h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    How every terminal here looks and behaves, from one per-machine
    <code class="font-mono">terminal.toml</code>. Edit it in your own editor and
    every open terminal re-applies it. Values it sets are shown in full;
    everything else is the default in force. A value it gets wrong is ignored
    with a warning on each space's settings.
  </p>

  {#if layer}
    {@render layerRow(layer)}
  {/if}

  <section
    class="flex flex-col gap-1.5 rounded-md border border-border px-2.5 py-2"
  >
    <h3 class="text-xs font-semibold">Parsed config values</h3>
    <Accordion.Root type="multiple" bind:value={open}>
      {#each groups as g (g.title)}
        <Accordion.Item value={g.title}>
          <Accordion.Trigger class="text-[0.65rem] font-semibold tracking-wide">
            {g.title}
          </Accordion.Trigger>
          <Accordion.Content>
            <dl
              class="grid grid-cols-[max-content_1fr] items-baseline gap-x-4 gap-y-0.5"
            >
              {#each g.rows as r (r.label)}
                <dt class="truncate text-[0.7rem] text-muted-foreground">
                  {r.label}
                </dt>
                <dd class="flex min-w-0 items-center gap-1.5">
                  {#if r.swatch}
                    <!-- The one place a concrete colour is painted from data rather than
                       a token: it is the terminal's own resolved slot colour, the same
                       exempt-chromatic class as the star-map's status hues (ADR 0012),
                       fed in at the seam and never inlined as a literal. -->
                    <span
                      class="size-2.5 shrink-0 rounded-[2px] border border-border"
                      style:background-color={r.swatch}
                      aria-hidden="true"
                    ></span>
                  {/if}
                  <span
                    class={[
                      "truncate font-mono text-[0.7rem]",
                      r.set ? "font-medium" : "text-muted-foreground",
                    ]}
                    title={r.set
                      ? `set in terminal.toml: ${r.value}`
                      : `default: ${r.value}`}
                  >
                    {r.value}
                  </span>
                </dd>
              {/each}
            </dl>
          </Accordion.Content>
        </Accordion.Item>
      {/each}
    </Accordion.Root>
  </section>
</section>
