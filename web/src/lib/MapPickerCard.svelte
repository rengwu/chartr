<script lang="ts">
  import type { Map as WMap } from "./model";
  import * as Card from "./components/ui/card";

  // One tile in the map picker (the panel's first screen). It carries a map's
  // identity — name and a resolution meter — and is the click target that opens
  // the map into the star-map view. Every discovered map is a live open target;
  // there is nothing between a map arriving over the socket and opening it.
  let {
    map,
    onopen,
  }: {
    map: WMap;
    onopen: () => void;
  } = $props();

  // Resolution progress is resolved / total tickets: out_of_scope tickets count
  // in the denominator but not as progress, so the bar reads "how much of the
  // whole map has actually landed", not "how much is left to think about".
  const total = $derived(map.tickets.length);
  const resolved = $derived(
    map.tickets.filter((t) => t.status === "resolved").length,
  );
  const pct = $derived(total ? Math.round((resolved / total) * 100) : 0);

  // A tile opens on click or Enter/Space; the card is the button, so there is
  // nothing interactive nested inside it to steal the activation.
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
    // Width comes from the enclosing auto-fill grid track, not the tile: the
    // picker fills its pane edge to edge at any width rather than leaving a
    // gutter next to a fixed-width row.
    "w-full min-w-0 gap-2 p-3 text-left transition-colors",
    "cursor-pointer hover:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:outline-none",
  ]}
  role="button"
  tabindex={0}
  aria-label="Open {map.name}"
  onclick={onopen}
  onkeydown={onCardKey}
>
  <!-- Title on its own full-width line so it gets the room; the resolved count
       sits on the bottom row under the meter. -->
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

  <span class="text-[0.7rem] text-muted-foreground"
    >{resolved} / {total} resolved</span
  >
</Card.Root>
