<script lang="ts">
  import { onMount, untrack } from 'svelte'
  import type { Space, Terminal as Term, Map as WMap } from './model'
  import { closeTerminal, openTerminal } from './actions'
  import Terminal from './Terminal.svelte'
  import MapCard from './MapCard.svelte'
  import { Button } from './components/ui/button'
  import { Badge, type BadgeVariant } from './components/ui/badge'
  import * as Tabs from './components/ui/tabs'
  import * as Sheet from './components/ui/sheet'
  import * as ScrollArea from './components/ui/scroll-area'
  import type { Layer } from './model'
  import { Plus, X, Warning, CheckCircle, Sparkle, SlidersHorizontal } from 'phosphor-svelte'

  // Which layer a field was inherited from, told apart by badge weight rather than
  // hue (the palette is monochrome + destructive): built-in is the lightest touch
  // (the shipped baseline), workspace the shared committed layer, user the
  // operator's own override and so the strongest emphasis. Mirrors the payload
  // preview's layer scale (ticket 08) so provenance reads the same everywhere.
  const layerVariant: Record<Layer, BadgeVariant> = {
    'built-in': 'outline',
    workspace: 'secondary',
    user: 'default',
  }

  // The stage for the selected space: a full-width title bar carrying the space's
  // identity (name and path), over a row of the space's subpanes. The identity
  // sits one level above the panes so the hierarchy reads "space › {terminals,
  // map}" — each pane owns only its own chrome. The ticket pane keeps ticket
  // 11's prototype: a shell tab strip flush at the top (the space's ad-hoc shells
  // plus a "+"), the active terminal filling the rest. A mapless space is fully
  // usable this way (story 29). The effective role bindings (stories 39, 40) live
  // in a right-docked drawer summoned from the ticket pane's bar, so they never
  // occupy the terminal's real estate.
  //
  // Over the terminal, the star-map is summoned as a floating card — edge handle,
  // M, Esc — or docked as the terminal-priority split (spec, The interface).
  // Visibility changes only on those explicit acts: switching spaces or focusing
  // a different map never opens or closes it, which is why this state lives on
  // the pane (persisting across space switches) and not per-map.
  let { space }: { space: Space } = $props()

  let activeId = $state<string | null>(null)
  let opening = $state(false)
  let showBindings = $state(false)

  // A deep link names a star (spec): #s=<spaceId>&m=<mapSlug>&t=<ticketNum>, or
  // &mat=1 for the map material. Parsed once at init — the enclosing App has
  // already selected the space from the same `s` — so the linked star opens and
  // seats on load; manual edits are picked up by a hashchange listener below.
  function parseHash() {
    const p = new URLSearchParams(location.hash.replace(/^#/, ''))
    return { s: p.get('s'), m: p.get('m'), t: p.get('t'), mat: p.get('mat') }
  }
  const boot = parseHash()
  // A one-time read at construction: the enclosing App has already selected this
  // space from the same `s`, so the boot link applies here. untrack marks the
  // read deliberate (this must not react to later space switches).
  const bootApplies = !boot.s || boot.s === untrack(() => space.id)

  // Star-map card state (persists across space switches by design).
  let mapShown = $state(bootApplies && (!!boot.t || !!boot.mat))
  let dock = $state(false)
  let mapSlug = $state<string | null>(bootApplies ? boot.m : null)
  let selectedTicket = $state<number | null>(bootApplies && boot.t ? Number(boot.t) : null)
  let showMaterial = $state(bootApplies && !!boot.mat)
  let dockTermWidth = $state(0)
  let floatWidth = $state(0)
  let bodyEl: HTMLDivElement

  const terminals = $derived<Term[]>(space.terminals ?? [])
  const warnings = $derived<string[]>(space.warnings ?? [])
  const maps = $derived<WMap[]>(space.maps ?? [])

  // A stale slug (a map that vanished, or a switch to a space without it) falls
  // back to the first map, so the card always has something to render.
  const focusedMap = $derived<WMap | null>(
    maps.find((m) => m.slug === mapSlug) ?? maps[0] ?? null,
  )

  // A selection belongs to one map: when the focused map *changes*, drop it (and
  // any open material) so the island never carries a ticket number from a
  // different graph. The first run only records the slug — it must not clear a
  // selection the deep link just seeded.
  let lastSlug: string | null = null
  $effect(() => {
    const slug = focusedMap?.slug ?? ''
    if (lastSlug === null) {
      lastSlug = slug
      return
    }
    if (slug !== lastSlug) {
      lastSlug = slug
      selectedTicket = null
      showMaterial = false
    }
  })

  // Reflect the current selection into the URL so a star (or the map material) is
  // a shareable deep link. replaceState never fires hashchange, so this and the
  // listener below do not loop.
  $effect(() => {
    const p = new URLSearchParams()
    p.set('s', space.id)
    if (mapSlug) p.set('m', mapSlug)
    if (selectedTicket !== null) p.set('t', String(selectedTicket))
    else if (showMaterial) p.set('mat', '1')
    const want = mapShown && (selectedTicket !== null || showMaterial) ? '#' + p.toString() : ''
    if (location.hash !== want) {
      history.replaceState(null, '', want || location.pathname + location.search)
    }
  })

  // Manual URL edits and back/forward re-apply, but only when the hash targets
  // this space (App owns switching to another space's link).
  onMount(() => {
    const apply = () => {
      const h = parseHash()
      if (h.s && h.s !== space.id) return
      if (h.m) mapSlug = h.m
      if (h.t) {
        selectedTicket = Number(h.t)
        showMaterial = false
        mapShown = true
      } else if (h.mat) {
        showMaterial = true
        selectedTicket = null
        mapShown = true
      }
    }
    window.addEventListener('hashchange', apply)
    return () => window.removeEventListener('hashchange', apply)
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
    const el = document.activeElement as HTMLElement | null
    const editing =
      !!el &&
      (el.tagName === 'INPUT' ||
        el.tagName === 'TEXTAREA' ||
        el.isContentEditable ||
        el.closest('.terminal-island') !== null ||
        // A summoned Sheet/Dialog (the bindings drawer) owns its own Escape; the
        // chrome's M/Esc bindings must not also fire while it holds focus.
        el.closest('[role="dialog"]') !== null)
    if (e.key === 'Escape' && !editing) {
      // Esc peels back one layer: an open detail pane first, then the card.
      if (selectedTicket !== null || showMaterial) {
        selectedTicket = null
        showMaterial = false
        return
      }
      if (mapShown) {
        dismiss()
        return
      }
    }
    if ((e.key === 'm' || e.key === 'M') && !editing && maps.length && !e.metaKey && !e.ctrlKey) {
      e.preventDefault()
      toggleMap()
    }
  }

  // The active tab falls back to the first terminal when the id is stale — a
  // just-closed shell, or a switch to a different space — so the column never
  // shows a blank island while terminals remain.
  const active = $derived.by(() => {
    return terminals.find((t) => t.id === activeId) ?? terminals[0] ?? null
  })

  async function openShell() {
    opening = true
    try {
      const { id } = await openTerminal(space.id)
      activeId = id
    } catch (e) {
      alert(`Couldn’t open a shell: ${(e as Error).message}`)
    } finally {
      opening = false
    }
  }

  async function endShell(t: Term) {
    if (activeId === t.id) activeId = null
    try {
      await closeTerminal(space.id, t.id)
    } catch (e) {
      alert(`Couldn’t end “${t.title}”: ${(e as Error).message}`)
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<!-- The space's stage: a full-width title bar (the space's identity — name and
     path) over a row of its subpanes. The identity lives here, one level above
     the panes, so the hierarchy reads "space › {terminals, map}": each pane
     carries only its own chrome. A floating map overlays the panes row but never
     this header — the panes row is its positioning context, and it sits below. -->
<div class="flex h-full min-h-0 flex-col">
  <header class="cockpit-bar justify-between">
    <div class="flex min-w-0 items-baseline gap-2" title={space.path}>
      <span class="truncate text-sm font-semibold">{space.name}</span>
      <code class="truncate font-mono text-[0.7rem] text-muted-foreground">{space.path}</code>
    </div>

    {#if maps.length}
      <!-- The one map show/hide control for the whole stage: a toggle in the space
           header, right-aligned, reflecting mapShown via aria-pressed. -->
      <Button
        variant={mapShown ? 'secondary' : 'ghost'}
        size="sm"
        aria-pressed={mapShown}
        title={mapShown ? 'Hide the star-map (M)' : 'Show the star-map (M)'}
        onclick={toggleMap}
      >
        <Sparkle weight={mapShown ? 'fill' : 'regular'} /> Map
      </Button>
    {/if}
  </header>

  <!-- The panes row: the terminal column and, over it, the star-map card. It is
       the positioning context for a floating card (relative), and a flex row for
       the docked split — the terminal's frozen width lives in an inline flex-basis
       and the card takes the rest. -->
  <div class="relative flex min-h-0 flex-1" bind:this={bodyEl}>
    <!-- The ticket pane: its own header is the shell tab strip (the space's ad-hoc
         shells plus a "+") and the pane's actions; below it the active terminal.
         The strip is a Tabs.Root whose value tracks the effective active shell. -->
    <Tabs.Root
      value={active?.id ?? ''}
      onValueChange={(v) => (activeId = v)}
      class="flex min-h-0 min-w-0 flex-1 flex-col gap-0"
      style={mapShown && dock ? `flex: 0 1 ${dockTermWidth}px; min-width: 240px` : ''}
    >
      <div class="cockpit-bar">
        <div class="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto">
          <Tabs.List class="h-auto gap-1 rounded-none bg-transparent p-0">
            {#each terminals as t (t.id)}
              <div
                class={[
                  'group/tab flex items-center rounded-md pr-0.5',
                  active?.id === t.id
                    ? 'bg-muted text-foreground'
                    : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground',
                ]}
              >
                <Tabs.Trigger
                  value={t.id}
                  title={t.alive ? t.title : `${t.title} (ended)`}
                  class={[
                    'h-6 flex-none gap-1.5 px-2 text-xs font-normal after:hidden data-active:bg-transparent dark:data-active:border-transparent dark:data-active:bg-transparent',
                    !t.alive && 'opacity-60',
                  ]}
                >
                  <span
                    class={['size-1.5 rounded-full', t.alive ? 'bg-primary/70' : 'bg-muted-foreground/50']}
                    aria-hidden="true"
                  ></span>
                  {t.title}{#if !t.alive}<span class="text-muted-foreground"> · ended</span>{/if}
                </Tabs.Trigger>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="opacity-0 hover:text-destructive group-hover/tab:opacity-100 focus-visible:opacity-100"
                  aria-label="End {t.title}"
                  title={t.alive ? 'End this shell' : 'Dismiss'}
                  onclick={() => endShell(t)}
                >
                  <X />
                </Button>
              </div>
            {/each}
          </Tabs.List>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="Open a shell in the working tree"
            title="Open a shell in {space.name}"
            disabled={opening}
            onclick={openShell}
          >
            <Plus />
          </Button>
        </div>

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
          <!-- Effective role bindings, summoned into a right Sheet from the bar so
               they never occupy the terminal's real estate. -->
          <Sheet.Root bind:open={showBindings}>
            <Sheet.Trigger>
              {#snippet child({ props })}
                <Button {...props} variant="outline" size="sm" title="Effective role bindings">
                  <SlidersHorizontal /> bindings
                </Button>
              {/snippet}
            </Sheet.Trigger>
            <Sheet.Content side="right" class="w-full gap-0 p-0 sm:max-w-md">
              <Sheet.Header class="border-b border-border px-4 py-3 text-left">
                <Sheet.Title class="text-sm">Effective role bindings</Sheet.Title>
                <Sheet.Description class="text-xs text-muted-foreground">
                  What each role resolves to after merging built-in ‹ workspace ‹ user. The tag on
                  a field names the layer it was inherited from.
                </Sheet.Description>
              </Sheet.Header>
              <ScrollArea.Root class="min-h-0 flex-1">
                <div class="flex flex-col gap-3 p-4">
                  {#if warnings.length}
                    <ul class="flex flex-col gap-1.5 rounded-md border border-border p-2.5">
                      {#each warnings as w}
                        <li class="flex items-start gap-1.5 text-xs text-muted-foreground">
                          <Warning class="mt-0.5 size-3.5 shrink-0" /> <span>{w}</span>
                        </li>
                      {/each}
                    </ul>
                  {/if}

                  <ul class="flex flex-col gap-2">
                    {#each space.bindings as b (b.role)}
                      <li class="rounded-md border border-border p-2.5">
                        <div class="mb-1.5 flex items-center justify-between gap-2">
                          <span class="text-xs font-semibold">{b.role}</span>
                          {#if b.present}
                            <span class="flex items-center gap-1 text-[0.7rem] text-muted-foreground">
                              <CheckCircle class="size-3.5" /> on PATH
                            </span>
                          {:else}
                            <Badge variant="destructive" class="gap-1"><Warning /> not found</Badge>
                          {/if}
                        </div>
                        <div class="flex flex-col gap-1">
                          <div class="flex items-center gap-1.5">
                            <span class="min-w-0 flex-1 truncate font-mono text-xs">{b.adapter}</span>
                            <Badge variant={layerVariant[b.adapterFrom]}>{b.adapterFrom}</Badge>
                          </div>
                          <div class="flex items-center gap-1.5">
                            <span class="min-w-0 flex-1 truncate font-mono text-xs">{b.model}</span>
                            <Badge variant={layerVariant[b.modelFrom]}>{b.modelFrom}</Badge>
                          </div>
                          {#if b.args && b.args.length}
                            <div class="flex items-center gap-1.5">
                              <span class="min-w-0 flex-1 truncate font-mono text-xs">{b.args.join(' ')}</span>
                              <Badge variant={layerVariant[b.argsFrom]}>{b.argsFrom}</Badge>
                            </div>
                          {/if}
                        </div>
                        {#if !b.present && b.missing}
                          <p class="mt-1.5 text-[0.7rem] text-muted-foreground">{b.missing}</p>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                </div>
              </ScrollArea.Root>
            </Sheet.Content>
          </Sheet.Root>
        </div>
      </div>

      <div class="relative min-h-0 flex-1">
        {#if active}
          {#key active.id}
            <Terminal term={active} />
          {/key}
        {:else}
          <div class="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
            <p class="text-sm text-muted-foreground">No shell open in this space.</p>
            <Button variant="outline" size="sm" disabled={opening} onclick={openShell}>Open a shell</Button>
            <p class="max-w-xs text-xs text-muted-foreground">
              A plain shell in <code class="font-mono">{space.name}</code>’s working tree — no ticket, no
              review, ended when you close it.
            </p>
          </div>
        {/if}
      </div>
    </Tabs.Root>

    {#if focusedMap && mapShown}
      <MapCard
        {maps}
        spaceId={space.id}
        bind:slug={mapSlug}
        bind:dock
        bind:selected={selectedTicket}
        bind:showMaterial
        {floatWidth}
        onclose={dismiss}
        onresizestart={startResize}
      />
    {/if}
  </div>
</div>
