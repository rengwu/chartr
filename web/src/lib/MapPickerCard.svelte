<script lang="ts">
  import type { Map as WMap } from "./model";
  import { classifyMap } from "./actions";
  import * as Card from "./components/ui/card";
  import { Badge } from "./components/ui/badge";
  import { Button } from "./components/ui/button";

  // One tile in the map picker (the panel's first screen). It carries a map's
  // identity — name, kind, and a resolution meter — and is the click target that
  // opens the map into the star-map view. An *unclassified* map is the one
  // exception: it is inert until a human declares its kind (ADR 0007), so its
  // tile is not clickable-into. It hosts the classify confirm right here instead,
  // and becomes a live open target the moment the kind arrives over the socket.
  let {
    map,
    spaceId,
    onopen,
  }: {
    map: WMap;
    spaceId: string;
    onopen: () => void;
  } = $props();

  // The kind, worn as a small pill in the tile's bottom-right. Only a classified
  // tile shows it — an unclassified one lives under the "Unclassified maps"
  // heading, which already says as much.
  const typeLabel: Record<string, string> = {
    planning: "planning",
    implementation: "implementation",
  };

  // Resolution progress is resolved / total tickets: out_of_scope tickets count
  // in the denominator but not as progress, so the bar reads "how much of the
  // whole map has actually landed", not "how much is left to think about".
  const total = $derived(map.tickets.length);
  const resolved = $derived(
    map.tickets.filter((t) => t.status === "resolved").length,
  );
  const pct = $derived(total ? Math.round((resolved / total) * 100) : 0);

  const classified = $derived(map.kind !== "");

  async function doClassify(kind: "planning" | "implementation") {
    try {
      await classifyMap(spaceId, map.slug, kind);
    } catch (e) {
      alert(`Couldn’t classify “${map.name}”: ${(e as Error).message}`);
    }
  }
  // p / i confirm the two kinds without reaching for the mouse; the convention
  // guess stays the pre-emphasised default for a plain click.
  function onKindKey(e: KeyboardEvent) {
    if (e.key === "p") doClassify("planning");
    else if (e.key === "i") doClassify("implementation");
  }

  // A classified tile opens on click or Enter/Space; the card is the button, so
  // there is nothing interactive nested inside it to steal the activation.
  function onCardKey(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onopen();
    }
  }
</script>

<Card.Root
  size="sm"
  class={[
    "w-78 gap-2 p-3 text-left transition-colors",
    classified &&
      "cursor-pointer hover:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:outline-none",
  ]}
  role={classified ? "button" : undefined}
  tabindex={classified ? 0 : undefined}
  aria-label={classified ? `Open ${map.name}` : undefined}
  onclick={classified ? onopen : undefined}
  onkeydown={classified ? onCardKey : undefined}
>
  <!-- Title on its own full-width line so it gets the room; the kind pill drops
       to the bottom row next to the resolved count. -->
  <span class="truncate text-sm font-semibold">{map.name}</span>

  <div
    class="progress"
    role="progressbar"
    aria-valuenow={pct}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label="Resolution progress"
  >
    <div class="progress-fill" style="width:{pct}%"></div>
  </div>

  <div class="flex items-center justify-between gap-2">
    <span class="text-[0.7rem] text-muted-foreground"
      >{resolved} / {total} resolved</span
    >
    {#if classified}
      <Badge variant="secondary" class="shrink-0">{typeLabel[map.kind]}</Badge>
    {/if}
  </div>

  {#if !classified}
    <!-- The classify confirm lives on the tile (ADR 0007): the map cannot be
         opened until its kind is set, so this is the door. Pre-emphasise the
         convention guess; either key/button declares the kind and the tile goes
         live. The p/i keys enhance the two focusable buttons — the row is a
         convenience listener, not itself an interactive control. -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="flex flex-col gap-1.5 rounded-md border border-border p-2 text-xs"
      role="group"
      aria-label="Classify {map.name}"
      onkeydown={onKindKey}
    >
      <span class="text-muted-foreground"
        >No sessions run until you set this map’s kind.</span
      >
      <div class="flex gap-1.5">
        <Button
          size="xs"
          variant={map.kindGuess === "planning" ? "default" : "outline"}
          title="Planning map — tickets resolve decisions (p)"
          onclick={() => doClassify("planning")}>plan</Button
        >
        <Button
          size="xs"
          variant={map.kindGuess === "implementation" ? "default" : "outline"}
          title="Implementation map — tickets deliver code (i)"
          onclick={() => doClassify("implementation")}>impl</Button
        >
      </div>
    </div>
  {/if}
</Card.Root>
