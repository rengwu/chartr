<script lang="ts">
  import { untrack } from "svelte";
  import type {
    Agent,
    Map as WMap,
    Terminal,
    Ticket,
  } from "./model";
  import StarMap from "./StarMap.svelte";
  import DetailPane from "./DetailPane.svelte";
  import MapPickerCard from "./MapPickerCard.svelte";
  import { decideDock, type Dock } from "./starmap/dock";
  import { cameraKey } from "./mapstate";
  import { Button } from "./components/ui/button";
  import * as ScrollArea from "./components/ui/scroll-area";
  import { CaretLeft, GpsFix } from "phosphor-svelte";

  // The star-map panel presented as a card over the terminal (spec, The
  // interface): summoned, never toggled by switching spaces or maps. It carries
  // two screens in the auxiliary pane's frame (SpacePane owns the frame, its
  // Map/Prompts tabs, and the dock/close chrome; this is only what the Map tab
  // shows):
  //
  //   • the picker — a grid of the space's maps (name, resolution), and the
  //     door in;
  //   • the map — the island, with the back / map-name chrome floating directly
  //     over it (no header bar of its own), and the responsive detail pane
  //     (ticket 07) docked right, re-docking to the bottom when the card is narrow.
  //
  // `slug === null` is the picker; a slug names the open map. The parent owns
  // which screen we land on (auto-open for a single-map space, deep links).
  let {
    maps,
    spaceId,
    lastAgent,
    agents,
    terminals = [],
    slug = $bindable(),
    selected = $bindable(null),
    showMaterial = $bindable(false),
    onRegisterAgent,
    onspawned,
  }: {
    maps: WMap[];
    // The space's open tabs, threaded to the island so a session paints its moon
    // on the ticket it holds (ticket 13), and to the detail pane so a claimed
    // ticket can say whether anything here is still running it.
    terminals?: Terminal[];
    // The space these maps belong to — threaded to the detail pane so its payload
    // preview can fetch (ticket 08).
    spaceId: string;
    // The space's remembered agent and the global library (ticket 02): handed to
    // the detail pane so the spawn buttons can name and pick which agent runs.
    lastAgent?: string;
    agents: Agent[];
    slug: string | null;
    selected?: number | null;
    showMaterial?: boolean;
    // Where the detail pane's spawn control sends the operator when the library is
    // empty: agent registration (ticket 04), owned by App and carried through.
    onRegisterAgent: () => void;
    // Bubbled up from the detail pane's Spawn control (ticket 09) so the chrome can
    // make the new session's tab active.
    onspawned?: (sessionId: string) => void;
  } = $props();

  // The open map, or null for the picker. A stale slug (a map that vanished)
  // falls back to the picker rather than to some other graph.
  const map = $derived<WMap | null>(
    slug ? (maps.find((m) => m.slug === slug) ?? null) : null,
  );

  // This island's identity across its lifetimes (mapstate.ts): the handle its
  // camera pose is filed under when the island is torn down, and the key it is
  // rebuilt on.
  const camera = $derived(map ? cameraKey(spaceId, map.slug) : "");

  // Both are bound inside the map screen's `{#if map}`, so they mount only when a
  // map opens — $state so the measuring effects below re-run when they appear.
  let bodyEl = $state<HTMLDivElement | null>(null);
  let paneEl = $state<HTMLDivElement | null>(null);
  // The island wrapper, for the one call the chrome makes into it: recentre.
  let island = $state<ReturnType<typeof StarMap> | null>(null);

  // The map screen's controls float directly over the canvas rather than on a
  // bar, so the ones that read as solid chips lift themselves off the starfield
  // with a faint blur — enough that a glyph stays readable over a dense
  // constellation, well short of a frosted panel. Only the chips: the ghosted
  // title and close are meant to sit *in* the field, and blurring a control with
  // no surface of its own just prints its rectangle onto the map. Only the map
  // screen, too — on the picker the same chrome sits on an opaque card, where
  // there is nothing behind it to blur.
  const OVER_MAP = "backdrop-blur-[2px]";
  let bodyWidth = $state(0);
  let bodyHeight = $state(0);
  let paneSize = $state({ w: 0, h: 0 });

  // Selecting a star wins over the map-material pane; opening material clears any
  // ticket selection. The two are one pane showing one thing.
  $effect(() => {
    if (selected !== null) showMaterial = false;
  });

  const paneTicket = $derived<Ticket | null>(
    !showMaterial && selected !== null
      ? (map?.tickets.find((t) => t.num === selected) ?? null)
      : null,
  );
  const paneOpen = $derived(showMaterial || paneTicket !== null);

  // The floating chrome sits over the top of the island. Keep the camera's stars
  // clear of it — a taller top inset than the other three edges — so a star or
  // label never hides behind the back / dock buttons.
  const TOP_INSET = 52;

  // Responsive docking: right by default, re-docking to bottom when the card is
  // either too narrow to hold a side pane or tall enough that a right pane would
  // ribbon the map — the hybrid signal (spec: right dock, re-docking to bottom
  // when narrow). A dead-band makes the switch sticky: `prev` feeds the next
  // decision, so dragging the card through the boundary holds the current side
  // rather than flip-flopping.
  let paneDock = $state<Dock>("right");
  $effect(() => {
    const next = decideDock(
      "hybrid",
      bodyWidth,
      bodyHeight,
      untrack(() => paneDock),
      true,
    );
    if (next !== untrack(() => paneDock)) paneDock = next;
  });

  // The camera measures the pane's actual footprint and eases the star into the
  // rest (planning ticket 08 as amended): a right pane insets the right edge, a
  // bottom pane the bottom edge; closed, only a small breathing margin. The top
  // always clears the floating chrome.
  const insets = $derived(
    paneOpen
      ? paneDock === "right"
        ? { top: TOP_INSET, left: 16, bottom: 16, right: paneSize.w + 20 }
        : { top: TOP_INSET, left: 16, right: 16, bottom: paneSize.h + 20 }
      : { top: TOP_INSET, right: 16, bottom: 16, left: 16 },
  );

  // The pane's own size, dragged from the seam it shares with the map. Each dock
  // side remembers its own figure, so re-docking and coming back restores what was
  // set rather than snapping to the default fraction. 0 means "not yet dragged" —
  // the CSS default (half the height, or 400px/58% of the width) still applies.
  let paneH = $state(0);
  let paneW = $state(0);
  const MIN_PANE = 120; // below this the header and the action footer collide
  const MIN_MAP_H = 140; // stars left visible above a bottom pane, chrome included
  const MIN_MAP_W = 220;

  // Drag the seam to resize: the top border when the pane is docked bottom, the
  // left border when it is docked right. Clamped so neither the pane nor the map
  // can be dragged shut — dismissing the pane is the close button's job.
  function startPaneResize(e: MouseEvent) {
    e.preventDefault();
    const el = bodyEl;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const vertical = paneDock === "bottom";
    const move = (ev: MouseEvent) => {
      if (vertical) {
        const max = Math.max(MIN_PANE, rect.height - TOP_INSET - MIN_MAP_H);
        paneH = Math.round(
          Math.min(Math.max(rect.bottom - ev.clientY, MIN_PANE), max),
        );
      } else {
        const max = Math.max(MIN_PANE, rect.width - MIN_MAP_W);
        paneW = Math.round(
          Math.min(Math.max(rect.right - ev.clientX, MIN_PANE), max),
        );
      }
    };
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    document.body.style.cursor = vertical ? "ns-resize" : "ew-resize";
    document.body.style.userSelect = "none";
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  $effect(() => {
    const el = bodyEl;
    if (!el) return;
    const measure = () => {
      bodyWidth = el.clientWidth;
      bodyHeight = el.clientHeight;
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  });

  $effect(() => {
    const el = paneEl;
    if (!el) return;
    const measure = () =>
      (paneSize = { w: el.clientWidth, h: el.clientHeight });
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  });

  function back() {
    selected = null;
    showMaterial = false;
    slug = null;
  }
  function openMaterial() {
    selected = null;
    showMaterial = true;
  }
  function closePane() {
    selected = null;
    showMaterial = false;
  }
</script>

<!-- What the pane's Map tab shows: the picker, or the open map. The frame
     around it — the docked/floating card, its resize grip, and the Map/Prompts
     tabs with the dock and close chrome — belongs to SpacePane, which is what
     keeps Map and Prompts from ever competing for the same auxiliary space. -->
{#if map}
  <!-- The map screen: the island fills the frame, its chrome floating directly
       over the top (no header bar). The detail pane overlays one edge below the
       chrome; the camera (insets) eases the selected star into the space it
       leaves free. -->
  <div class="relative min-h-0 flex-1" bind:this={bodyEl}>
    <!-- Keyed on space *and* map: the island is rebuilt when either changes,
         and the same string is the key its camera pose is remembered under
         while it is gone. Two spaces can hold maps of one name, so the space
         has to be in it — on the key alone, switching between them would show
         one map's stars under the other's camera. -->
    {#key camera}
      <StarMap
        {map}
        {terminals}
        {insets}
        cameraKey={camera}
        bind:selected
        bind:this={island}
      />
    {/key}

    <div
      class="absolute inset-x-0 top-0 z-30 flex h-[var(--bar-h)] items-center gap-1.5 px-2"
    >
      <Button
        variant="outline"
        size="sm"
        class={OVER_MAP}
        title="Back to all maps"
        onclick={back}
      >
        <CaretLeft /> Back
      </Button>
      <Button
        variant={showMaterial ? "secondary" : "ghost"}
        size="sm"
        class="min-w-0 shrink flex-1"
        aria-pressed={showMaterial}
        title="Open map material — destination, notes, decisions, fog"
        onclick={openMaterial}
      >
        <!-- The button itself is a flex container (justify-center), so
             `truncate` on it clips the centred text from both ends with no
             ellipsis. Truncating this inner, left-aligned span instead lets
             the name end in a normal ellipsis. -->
        <span class="min-w-0 flex-1 truncate text-left">{map.name}</span>
      </Button>
    </div>

    <!-- Recentre. Closing the pane leaves the camera where the operator put it,
         so this is the way back to the whole constellation. Only offered with
         the pane closed: with one open the camera belongs to the selected star,
         and a bottom-docked pane would sit on top of this corner anyway. -->
    {#if !paneOpen}
      <div class="absolute bottom-2 left-2 z-30">
        <Button
          variant="outline"
          size="icon-sm"
          class={OVER_MAP}
          aria-label="Recenter map"
          title="Recenter map"
          onclick={() => island?.fit()}
        >
          <GpsFix />
        </Button>
      </div>
    {/if}

    {#if paneOpen}
      <!-- Full-height on the right (below the floating chrome) by default,
           half-height along the bottom when the card is narrow or tall. It sits
           flush to the card's edges — a panel sharing a draggable seam with the
           map, not a card floating over it. Its surface is translucent (see
           DetailPane), so the stars behind it stay faintly visible; the camera
           still keeps them clear of the footprint. -->
      <div
        class={[
          "absolute z-10",
          paneDock === "bottom"
            ? "inset-x-0 bottom-0 h-1/2"
            : "top-[var(--bar-h)] right-0 bottom-0 w-[min(400px,58%)]",
        ]}
        style={paneDock === "bottom"
          ? paneH
            ? `height:${paneH}px`
            : ""
          : paneW
            ? `width:${paneW}px`
            : ""}
        bind:this={paneEl}
      >
        <!-- The seam, straddling the shared edge: drag to resize the pane. -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class={[
            "absolute z-20 transition-colors hover:bg-ring/60",
            paneDock === "bottom"
              ? "inset-x-0 top-0 h-1.5 -translate-y-1/2 cursor-ns-resize"
              : "inset-y-0 left-0 w-1.5 -translate-x-1/2 cursor-ew-resize",
          ]}
          role="separator"
          aria-orientation={paneDock === "bottom" ? "horizontal" : "vertical"}
          aria-label="Resize the detail pane"
          onmousedown={startPaneResize}
        ></div>

        <DetailPane
          {map}
          ticket={paneTicket}
          dock={paneDock}
          {spaceId}
          {lastAgent}
          {agents}
          {terminals}
          onclose={closePane}
          {onRegisterAgent}
          {onspawned}
        />
      </div>
    {/if}

  </div>
{:else}
  <!-- The picker screen: one flat auto-fill grid of the space's maps, every
       one of them a live open target. Tiles share the width evenly and reach
       both edges at any pane width. -->
  <!-- `auto`, not the default `hover`: a space with more maps than fit is the
       normal case here, and the grid gives no other hint that it runs on past
       the fold — so the bar stands while the content overflows rather than
       appearing only once the pointer is already inside. -->
  <ScrollArea.Root type="auto" class="min-h-0 flex-1">
    {#if maps.length}
      <div class="flex min-h-full flex-col gap-3 p-3">
        <div
          class="grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] items-start gap-3"
        >
          {#each maps as m (m.slug)}
            <MapPickerCard map={m} onopen={() => (slug = m.slug)} />
          {/each}
        </div>
      </div>
    {:else}
      <div class="grid h-full place-items-center p-6">
        <p class="max-w-xs text-center text-xs text-muted-foreground">
          No maps in this space yet — chart one with <code class="font-mono"
            >/wayfinder</code
          > in a shell.
        </p>
      </div>
    {/if}
  </ScrollArea.Root>
{/if}
