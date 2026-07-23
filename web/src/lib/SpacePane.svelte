<script lang="ts">
  import { onMount, untrack } from 'svelte'
  import type { Agent, Space, Terminal as Term, Map as WMap } from './model'
  import Terminal from './Terminal.svelte'
  import MapCard from './MapCard.svelte'
  import AgentSplitButton from './AgentSplitButton.svelte'
  import { Button } from './components/ui/button'
  import { isEditingTarget } from './keys'
  import { Warning, Sparkle, Lightbulb, Gear } from 'phosphor-svelte'

  // The stage for the selected space: a full-width title bar carrying the space's
  // identity (name and path) plus the stage-level controls — warnings, the
  // star-map toggle, and the cockpit-wide way into config, pinned to the far
  // right of the chrome rather than sitting beside the branding — over the
  // terminal. The sidebar now owns session selection (its space cards list each shell), so the
  // active shell arrives as a prop and this pane simply renders it: no tab strip,
  // no per-pane action bar. A mapless space is fully usable this way (story 29).
  //
  // Over the terminal, the star-map is summoned as a floating card — edge handle,
  // M, Esc — or docked as the terminal-priority split (spec, The interface).
  // Visibility changes only on those explicit acts: switching spaces or focusing
  // a different map never opens or closes it, which is why this state lives on
  // the pane (persisting across space switches) and not per-map.
  let {
    space,
    agents,
    activeTerm,
    active = true,
    onOpenShell,
    onIdeate,
    onOpenSettings,
    onRegisterAgent,
    onspawned,
  }: {
    space: Space
    // The registered agent library, global like the settings surface (ticket 02):
    // passed down to the star-map's detail pane so each spawn button can name and
    // pick the agent it will run.
    agents: Agent[]
    activeTerm: Term | null
    // False while the settings route covers the stage (ticket 05). The pane stays
    // mounted — its terminal and star-map are imperative islands worth keeping
    // alive — but goes inert: it takes no keystrokes and stops reflecting its
    // selection into the URL, which the settings route owns while it is up.
    active?: boolean
    onOpenShell: () => void
    // The ideate on-ramp (ticket 15): a live, ticketless chat, the one
    // opinionated nudge toward charting for a space with no map to spawn onto.
    // It names the agent that runs it (ticket 03), so the control here picks one
    // and hands it up.
    onIdeate: (agent: string) => void
    // The cockpit-wide way into the config surface (ticket 05), owned by the
    // enclosing App — the route is App's, this pane just carries the control at
    // the right end of its title bar.
    onOpenSettings: () => void
    // Where an empty-library spawn or ideate control sends the operator: agent
    // registration (the user scope of the settings surface). Owned by App like the
    // route above; every AgentSplitButton beneath this pane routes its empty state
    // here rather than being a dead button (ticket 04).
    onRegisterAgent: () => void
    // Bubbled from the star-map's detail pane when a session is spawned (ticket
    // 09), so the enclosing App can make the new session's tab active.
    onspawned?: (sessionId: string) => void
  } = $props()

  // A deep link names a star (spec): #s=<spaceId>&m=<mapSlug>&t=<ticketNum>, or
  // &mat=1 for the map material, or &maps=1 for the picker. Parsed once at init — the
  // enclosing App has already selected the space from the same `s` — so the linked
  // star opens and seats on load; manual edits are picked up by a hashchange
  // listener below.
  function parseHash() {
    const p = new URLSearchParams(location.hash.replace(/^#/, ''))
    return { s: p.get('s'), m: p.get('m'), t: p.get('t'), mat: p.get('mat'), maps: p.get('maps') }
  }
  const boot = parseHash()
  // A one-time read at construction: the enclosing App has already selected this
  // space from the same `s`, so the boot link applies here. untrack marks the
  // read deliberate (this must not react to later space switches).
  const bootApplies = !boot.s || boot.s === untrack(() => space.id)

  // Star-map card state (persists across space switches by design). `openSlug`
  // names the open map, or is null for the picker screen.
  let mapShown = $state(bootApplies && (!!boot.t || !!boot.mat || !!boot.maps))
  let dock = $state(true)
  let openSlug = $state<string | null>(bootApplies ? boot.m : null)
  let selectedTicket = $state<number | null>(bootApplies && boot.t ? Number(boot.t) : null)
  let showMaterial = $state(bootApplies && !!boot.mat)
  let dockTermWidth = $state(0)
  let floatWidth = $state(0)
  let bodyEl: HTMLDivElement

  const warnings = $derived<string[]>(space.warnings ?? [])
  const maps = $derived<WMap[]>(space.maps ?? [])

  // A selection belongs to one map: when the open map *changes*, drop it (and any
  // open material) so the island never carries a ticket number from a different
  // graph. The first run only records the slug — it must not clear a selection the
  // deep link just seeded. (undefined is the "not yet recorded" sentinel, since
  // openSlug itself is legitimately null on the picker.)
  let lastOpen: string | null | undefined = undefined
  $effect(() => {
    const s = openSlug
    if (lastOpen === undefined) {
      lastOpen = s
      return
    }
    if (s !== lastOpen) {
      lastOpen = s
      selectedTicket = null
      showMaterial = false
    }
  })

  // Switching spaces while the panel is open: an open slug from the previous
  // space won't match here. Fall to this space's picker — or straight into its
  // one map, the same auto-open a fresh summon does — and drop any stale
  // selection. Guarded on an actual space change so the back button (which nulls
  // openSlug within a space) still lands on, and stays on, the picker.
  let lastSpaceId = untrack(() => space.id)
  $effect(() => {
    if (space.id === lastSpaceId) return
    lastSpaceId = space.id
    if (!maps.some((m) => m.slug === openSlug)) {
      openSlug = maps.length === 1 ? (maps[0]?.slug ?? null) : null
      selectedTicket = null
      showMaterial = false
    }
  })

  // Apply whatever star link the hash currently names to this pane. Shared by
  // the hashchange listener below and the on-activation effect; only ever acts
  // on a link that targets this space (App owns switching to another space's).
  function applyHash() {
    const h = parseHash()
    if (h.s && h.s !== space.id) return
    if (h.m) openSlug = h.m
    // A link names its map and its star together, so record the slug as already
    // seen: the drop-on-map-change guard above must not clear the star that
    // arrived with the slug — the same exemption its first run makes for the
    // boot link, now that a link can also land mid-life (the halt flag's jump).
    if (h.t) {
      selectedTicket = Number(h.t)
      showMaterial = false
      mapShown = true
      lastOpen = openSlug
    } else if (h.mat) {
      showMaterial = true
      selectedTicket = null
      mapShown = true
      lastOpen = openSlug
    } else if (h.maps) {
      // The picker: the space's maps, each one a door in.
      openSlug = null
      selectedTicket = null
      showMaterial = false
      mapShown = true
    }
  }

  // Apply the link the moment this pane swings to another space (or comes back
  // from settings), not only on hashchange. App can switch space and set a star
  // link in one click — the sidebar halt flag's jump — and hashchange is
  // delivered a task later, by which time the reflecting effect below, seeing a
  // pane with nothing open, would already have wiped the link it was about to
  // read. Applying first means the reflection agrees with the link instead of
  // erasing it. Declared above that effect (and below the space-change reset,
  // whose stale selection it overwrites) so the order within one flush is
  // reset → apply → reflect. The hash read is untracked: this fires on a space
  // change and nothing else.
  $effect(() => {
    space.id
    if (!active) return
    untrack(() => applyHash())
  })

  // Reflect the current selection into the URL so a star (or the map material) is
  // a shareable deep link. replaceState never fires hashchange, so this and the
  // listener below do not loop. While the settings route is up it owns the hash,
  // so this stands down and restores its own link when the pane is active again.
  $effect(() => {
    if (!active) return
    const p = new URLSearchParams()
    p.set('s', space.id)
    if (openSlug) p.set('m', openSlug)
    if (selectedTicket !== null) p.set('t', String(selectedTicket))
    else if (showMaterial) p.set('mat', '1')
    const want = mapShown && (selectedTicket !== null || showMaterial) ? '#' + p.toString() : ''
    if (location.hash !== want) {
      history.replaceState(null, '', want || location.pathname + location.search)
    }
  })

  // Manual URL edits and back/forward re-apply, through the same path.
  onMount(() => {
    window.addEventListener('hashchange', applyHash)
    return () => window.removeEventListener('hashchange', applyHash)
  })

  // Freeze the terminal's pixel width at the moment of docking, then let the map
  // absorb every later resize slack — the terminal-priority split holds its width
  // so a window resize never reflows it (planning ticket 08's amendment). The one
  // exception is the small end: once the window is too narrow to also grant the
  // map its floor (min-width 300), the terminal yields the rest so the map never
  // collapses out of view. Both floors are enforced in CSS (the docked term-col
  // is shrinkable to 240; the map card holds 300), so window resizes need no JS.
  function summon() {
    mapShown = true
    // A single-map space has nothing to pick — open straight into its one map.
    // With several, land on the picker (openSlug stays null). Never overrides a
    // slug a deep link or an earlier session already opened.
    if (openSlug === null && maps.length === 1) openSlug = maps[0].slug
  }
  function dismiss() {
    mapShown = false
  }
  function toggleMap() {
    if (mapShown) dismiss()
    else summon()
  }
  $effect(() => {
    if (mapShown && dock && bodyEl && !dockTermWidth) {
      const w = bodyEl.clientWidth
      // First dock: terminal keeps ~60%, always leaving room for the map; clamped
      // so neither pane collapses on a narrow window. A resize below overrides it.
      dockTermWidth = Math.round(Math.min(Math.max(w * 0.6, 320), Math.max(360, w - 360)))
    }
  })

  // Drag the card's left border to resize it — in either mode. Docked, the
  // border is the split: the map's edge moves and the terminal's frozen width
  // follows it. Floating, the card grows leftward while its right edge stays
  // pinned. Clamped so neither pane collapses.
  const MIN_MAP = 300
  const FLOAT_INSET = 10 // matches .map-floating .map-card right offset
  // A floating card is freely draggable wider; its right edge stays pinned and
  // it's only kept within the row so it can't overflow left into the sidebar. CSS
  // max-width holds that same within-the-row bound on resize.
  const FLOAT_MIN_TERM = 30 // sliver of terminal kept visible; matches the CSS 40px bound (FLOAT_INSET + this)
  function startResize(e: MouseEvent) {
    e.preventDefault()
    const rect = bodyEl.getBoundingClientRect()
    const move = (ev: MouseEvent) => {
      if (dock) {
        const minTerm = 240
        dockTermWidth = Math.round(
          Math.min(Math.max(ev.clientX - rect.left, minTerm), Math.max(minTerm, rect.width - MIN_MAP)),
        )
      } else {
        const maxMap = Math.max(MIN_MAP, rect.width - FLOAT_INSET - FLOAT_MIN_TERM)
        floatWidth = Math.round(
          Math.min(Math.max(rect.right - FLOAT_INSET - ev.clientX, MIN_MAP), maxMap),
        )
      }
    }
    const up = () => {
      window.removeEventListener('mousemove', move)
      window.removeEventListener('mouseup', up)
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }
    document.body.style.cursor = 'ew-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', move)
    window.addEventListener('mouseup', up)
  }

  // M summons/dismisses, Esc dismisses — but only when focus is on the chrome,
  // not inside the terminal (whose PTY owns every raw keystroke) or a text field.
  // The edge handle is the always-available path when the shell has the keyboard.
  function onKey(e: KeyboardEvent) {
    // Inert while the settings route covers the stage: its own Esc must not also
    // peel back this pane's map underneath it.
    if (!active) return
    // A summoned Sheet/Dialog (the action station) owns its own Escape; the
    // chrome's M/Esc bindings must not also fire while it holds focus.
    const editing = isEditingTarget()
    if (e.key === 'Escape' && !editing) {
      // Esc peels back one layer: the open detail pane first, then the open map
      // (back to the picker), then the panel.
      if (selectedTicket !== null || showMaterial) {
        selectedTicket = null
        showMaterial = false
        return
      }
      if (openSlug !== null) {
        openSlug = null
        return
      }
      if (mapShown) {
        dismiss()
        return
      }
    }
    if ((e.key === 'm' || e.key === 'M') && !editing && !e.metaKey && !e.ctrlKey) {
      e.preventDefault()
      toggleMap()
    }
  }

</script>

<svelte:window onkeydown={onKey} />

<!-- The space's stage: a full-width title bar (the space's identity — name and
     path) over a row of its subpanes. The identity lives here, one level above
     the panes, so the hierarchy reads "space › {terminals, map}": each pane
     carries only its own chrome. A floating map overlays the panes row but never
     this header — the panes row is its positioning context, and it sits below.

     `isolate` makes this stage one stacking context: every z-index inside (the
     floating card, the map's chrome bar, the resize grips) is then local to the
     stage and cannot climb over a route overlay rendered beside it, such as the
     settings surface. Without it a docked card — `relative`, z-auto, so no
     context of its own — leaked its z-30 chrome through settings. -->
<div class="isolate flex h-full min-h-0 flex-col">
  <header class="cockpit-bar justify-between">
    <div class="flex min-w-0 items-baseline gap-2" title={space.path}>
      <span class="truncate text-sm font-semibold">{space.name}</span>
      <code class="truncate font-mono text-[0.7rem] text-muted-foreground">{space.path}</code>
    </div>

    <!-- The stage-level controls, right-aligned: any surfaced warnings, the one
         star-map show/hide toggle — lifted here now that the terminal has no
         action bar — and, at the far right corner of the chrome, the
         cockpit-wide gear into the config surface (each space card keeps its
         own ⚙ for that space's scope). -->
    <div class="flex items-center gap-1.5">
      {#if warnings.length}
        <span
          class="flex items-center gap-1 text-[0.7rem] text-muted-foreground"
          title={warnings.join('\n')}
          aria-label="{warnings.length} warning(s)"
        >
          <Warning class="size-3.5" /> {warnings.length}
        </span>
      {/if}
      <!-- The one star-map show/hide control for the whole stage, beside the
           bindings; reflects mapShown via aria-pressed. Available even with
           zero maps: the picker itself explains there's nothing yet. -->
      <Button
        variant={mapShown ? 'secondary' : 'ghost'}
        size="sm"
        aria-pressed={mapShown}
        title={mapShown ? 'Hide the star-map (M)' : 'Show the star-map (M)'}
        onclick={toggleMap}
      >
        <Sparkle weight={mapShown ? 'fill' : 'regular'} /> Map
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Config"
        title="Your agents, and where the files behind them live (,)"
        onclick={onOpenSettings}
      >
        <Gear />
      </Button>
    </div>
  </header>

  <!-- The panes row: the terminal column and, over it, the star-map card. It is
       the positioning context for a floating card (relative), and a flex row for
       the docked split — the terminal's frozen width lives in an inline
       flex-basis and the card takes the rest. -->
  <div class="relative flex min-h-0 flex-1" bind:this={bodyEl}>
    <!-- The terminal column: no tab strip, no action bar — the sidebar owns
         session selection now, so this simply renders the active shell. -->
    <div
      class="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden"
      style={mapShown && dock ? `flex: 0 1 ${dockTermWidth}px; min-width: 240px` : ''}
    >
      {#if activeTerm}
        {#key activeTerm.id}
          <Terminal term={activeTerm} />
        {/key}
      {:else}
        <div class="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
          <p class="text-sm text-muted-foreground">No shell open in this space.</p>
          <div class="flex flex-wrap items-center justify-center gap-2">
            <Button variant="outline" size="sm" onclick={onOpenShell}>New Shell</Button>
            <AgentSplitButton
              {agents}
              lastAgent={space.lastAgent}
              label="New Idea"
              title="Think an idea through — a live, ticketless agent tab opened on a starter prompt. Nothing is claimed, nothing is committed, and it ends when you end it."
              onrun={onIdeate}
              onregister={onRegisterAgent}
            >
              {#snippet icon()}<Lightbulb />{/snippet}
            </AgentSplitButton>
            <Button
              variant="outline"
              size="sm"
              aria-pressed={mapShown}
              onclick={toggleMap}
            >
              <Sparkle weight={mapShown ? 'fill' : 'regular'} />
              {mapShown ? 'Hide Maps' : 'View Maps'}
            </Button>
          </div>
        </div>
      {/if}
    </div>

    {#if mapShown}
      <MapCard
        {maps}
        spaceId={space.id}
        lastAgent={space.lastAgent}
        {agents}
        terminals={space.terminals ?? []}
        bind:slug={openSlug}
        bind:dock
        bind:selected={selectedTicket}
        bind:showMaterial
        {floatWidth}
        onclose={dismiss}
        onresizestart={startResize}
        {onRegisterAgent}
        {onspawned}
      />
    {/if}
  </div>
</div>
