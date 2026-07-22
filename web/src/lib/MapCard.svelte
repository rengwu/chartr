<script lang="ts">
  import { untrack } from "svelte";
  import type { Agent, Map as WMap, Terminal, Ticket } from "./model";
  import StarMap from "./StarMap.svelte";
  import DetailPane from "./DetailPane.svelte";
  import MapPickerCard from "./MapPickerCard.svelte";
  import ActionStation from "./ActionStation.svelte";
  import { mapActionCount } from "./attention";
  import { decideDock, type Dock } from "./starmap/dock";
  import { Button } from "./components/ui/button";
  import * as ScrollArea from "./components/ui/scroll-area";
  import {
    CaretLeft,
    Columns,
    CornersOut,
    ListChecks,
    X,
  } from "phosphor-svelte";

  // The star-map panel presented as a card over the terminal (spec, The
  // interface): summoned, never toggled by switching spaces or maps. It carries
  // two screens in the one floating/docked frame:
  //
  //   • the picker — a grid of the space's maps (name, resolution), and the
  //     door in;
  //   • the map — the island, with the back / map-name / dock / close chrome
  //     floating directly over it (no header bar), and the responsive detail pane
  //     (ticket 07) docked right, re-docking to the bottom when the card is narrow.
  //
  // `slug === null` is the picker; a slug names the open map. The parent owns
  // which screen we land on (auto-open for a single-map space, deep links) and
  // whether the frame floats or docks as the terminal-priority split.
  let {
    maps,
    spaceId,
    lastAgent,
    agents,
    terminals = [],
    slug = $bindable(),
    dock = $bindable(false),
    selected = $bindable(null),
    showMaterial = $bindable(false),
    floatWidth = 0,
    onclose,
    onresizestart,
    onspawned,
  }: {
    maps: WMap[];
    // The space's open tabs, threaded to the island so a session paints its moon
    // on the ticket it holds (ticket 13).
    terminals?: Terminal[];
    // The space these maps belong to — threaded to the detail pane so its payload
    // preview can fetch (ticket 08).
    spaceId: string;
    // The space's remembered agent and the global library (ticket 02): handed to
    // the detail pane so the spawn buttons can name and pick which agent runs.
    lastAgent?: string;
    agents: Agent[];
    slug: string | null;
    dock?: boolean;
    selected?: number | null;
    showMaterial?: boolean;
    floatWidth?: number;
    onclose: () => void;
    onresizestart: (e: MouseEvent) => void;
    // Bubbled up from the detail pane's Spawn control (ticket 09) so the chrome can
    // make the new session's tab active.
    onspawned?: (sessionId: string) => void;
  } = $props();

  // The open map, or null for the picker. A stale slug (a map that vanished)
  // falls back to the picker rather than to some other graph.
  const map = $derived<WMap | null>(
    slug ? (maps.find((m) => m.slug === slug) ?? null) : null,
  );

  // Both are bound inside the map screen's `{#if map}`, so they mount only when a
  // map opens — $state so the measuring effects below re-run when they appear.
  let bodyEl = $state<HTMLDivElement | null>(null);
  let paneEl = $state<HTMLDivElement | null>(null);
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

  // The action station (ticket 14): a numbered badge toggling a drawer of
  // everything actionable on the open map. Hovering a row highlights its star
  // (hoverNum, fed to the island); the badge count is echoed onto the map's
  // handle in the parent when the card is tucked away (SpacePane owns that,
  // via spaceActionCount summing every map).
  let stationOpen = $state(false);
  let hoverNum = $state<number | null>(null);
  const actionCount = $derived(map ? mapActionCount(map) : 0);
  // Closing the drawer by any path (Escape, backdrop click) drops a lingering
  // hover ring even if the row's own mouseleave/blur never fired.
  $effect(() => {
    if (!stationOpen) hoverNum = null;
  });
  // A drawer left open belongs to the map it was opened on — navigating back
  // to the picker and into a different map must not carry it along.
  let lastStationSlug: string | null | undefined = undefined;
  $effect(() => {
    const s = map?.slug ?? null;
    if (lastStationSlug === undefined) {
      lastStationSlug = s;
      return;
    }
    if (s !== lastStationSlug) {
      lastStationSlug = s;
      stationOpen = false;
    }
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

<!-- The panel frame. Docked, it is a flex item in the panes row, taking the slack
     the terminal's frozen basis leaves (min 300). Floating, it is absolute over
     the panes row with its right edge pinned (inset 10px), grown leftward by the
     resize grip; a CSS max-width keeps it within the row on a window resize. -->
<section
  class={[
    "flex min-h-0 flex-col overflow-hidden bg-card",
    dock
      ? // Docked, this is a flush column next to the terminal — a plain square
        // edge like the terminal's own outer edges, with a border only on the
        // seam it shares with the terminal (the split divider).
        "relative min-w-[300px] flex-1 border-l border-border"
      : "absolute inset-y-2 right-2.5 z-20 w-[min(560px,64%)] max-w-[calc(100%-40px)] rounded-lg border border-border shadow-lg",
  ]}
  aria-label="Star-map"
  style={!dock && floatWidth ? `width:${floatWidth}px` : ""}
>
  <!-- Resize from the left border: the split divider when docked, the card's
       leading edge when floating (the right edge stays pinned). -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="absolute inset-y-0 left-0 z-40 w-1.5 -translate-x-1/2 cursor-ew-resize transition-colors hover:bg-ring/60"
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize star-map"
    onmousedown={onresizestart}
  ></div>

  <!-- The dock/close chrome is shared by both screens: dock toggles the whole
       panel between the terminal-priority split and floating over the terminal;
       close dismisses it. -->
  {#snippet chrome()}
    <Button
      variant="outline"
      size="icon-sm"
      aria-pressed={dock}
      title={dock
        ? "Float over the terminal"
        : "Dock as a split (terminal keeps its width)"}
      onclick={() => (dock = !dock)}
    >
      {#if dock}<CornersOut />{:else}<Columns />{/if}
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Dismiss star-map (Esc)"
      title="Dismiss (M / Esc)"
      onclick={onclose}
    >
      <X />
    </Button>
  {/snippet}

  {#if map}
    <!-- The map screen: the island fills the frame, its chrome floating directly
         over the top (no header bar). The detail pane overlays one edge below the
         chrome; the camera (insets) eases the selected star into the space it
         leaves free. -->
    <div class="relative min-h-0 flex-1" bind:this={bodyEl}>
      {#key map.slug}
        <StarMap {map} {terminals} {insets} {hoverNum} bind:selected />
      {/key}

      <div
        class="absolute inset-x-0 top-0 z-30 flex h-[var(--bar-h)] items-center gap-1.5 px-2"
      >
        <Button
          variant="outline"
          size="sm"
          title="Back to all maps"
          onclick={back}
        >
          <CaretLeft /> Back
        </Button>
        <Button
          variant={showMaterial ? "secondary" : "ghost"}
          size="sm"
          class="min-w-0 truncate"
          aria-pressed={showMaterial}
          title="Open map material — destination, notes, decisions, fog"
          onclick={openMaterial}>{map.name}</Button
        >
        <div class="ml-auto flex items-center gap-1">
          <Button
            variant="outline"
            size="sm"
            class="gap-1.5"
            title="Next up — the frontier ranked by unblock count"
            onclick={() => (stationOpen = true)}
          >
            <ListChecks />
            {#if actionCount > 0}
              <span
                class="grid size-4 place-items-center rounded-full bg-primary text-[0.6rem] font-semibold text-primary-foreground"
                >{actionCount}</span
              >
            {/if}
          </Button>
          {@render chrome()}
        </div>
      </div>

      {#if paneOpen}
        <!-- Full-height on the right (below the floating chrome) by default,
             half-height along the bottom when the card is narrow or tall. It sits
             flush to the card's edges — a panel sharing a draggable seam with the
             map, not a card floating over it — and its bg occludes the stars. -->
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
            onclose={closePane}
            {onspawned}
          />
        </div>
      {/if}

      <ActionStation
        bind:open={stationOpen}
        {map}
        {spaceId}
        onselect={(num) => (selected = num)}
        {onspawned}
        onhover={(num) => (hoverNum = num)}
      />
    </div>
  {:else}
    <!-- The picker screen: a header bar over one flat auto-fill grid of the
         space's maps, every one of them a live open target. Tiles share the
         width evenly and reach both edges at any pane width. -->
    <header class="cockpit-bar">
      <span class="min-w-0 flex-1 truncate text-sm font-semibold">Maps</span>
      <div class="flex items-center gap-1">
        {@render chrome()}
      </div>
    </header>

    <ScrollArea.Root class="min-h-0 flex-1">
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
          <p class="max-w-xs text-center text-sm text-muted-foreground">
            No maps in this space yet — chart one with <code class="font-mono"
              >/wayfinder</code
            > in a shell.
          </p>
        </div>
      {/if}
    </ScrollArea.Root>
  {/if}
</section>
