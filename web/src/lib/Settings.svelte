<script lang="ts">
  import type {
    ConfigLayer,
    Layer,
    ResolvedSkill,
    RoleBinding,
    Space,
    Ticket,
  } from './model'
  import { padTicket } from './model'
  import { settingsHash, type SettingsScope } from './route'
  import { openConfigLayer, setBinding } from './actions'
  import PayloadPreview from './PayloadPreview.svelte'
  import { Button } from './components/ui/button'
  import { Badge, type BadgeVariant } from './components/ui/badge'
  import { Input } from './components/ui/input'
  import * as ScrollArea from './components/ui/scroll-area'
  import {
    ArrowCounterClockwise,
    ArrowSquareOut,
    CheckCircle,
    Eye,
    Gear,
    Sparkle,
    Stack,
    User,
    Warning,
    X,
  } from 'phosphor-svelte'

  // The effective config surface (ticket 05, ADR 0014): one screen showing every
  // value the three documented layers resolve, with the layer it came from and
  // the file that layer lives in. Legibility first — it edits exactly one thing
  // (a role binding, into the user layer) and opens everything else in the
  // operator's editor. It is deliberately not a second config store: nothing here
  // is invented state, and every row traces back to a file.
  let {
    spaces,
    config,
    scope,
    onScope,
    onClose,
    onOpenMaps,
  }: {
    spaces: Space[]
    // The layers shared by every space — the operator's local binding file and
    // the two skill libraries that are not a space's own.
    config: ConfigLayer[]
    scope: SettingsScope
    onScope: (scope: SettingsScope) => void
    onClose: () => void
    // Kind is declared from the star-map's picker (ADR 0007); this surface shows
    // it read-only and links there rather than growing a second way to set it.
    onOpenMaps: (spaceId: string) => void
  } = $props()

  // Which layer a value came from, told apart by badge weight rather than hue
  // (the chrome is monochrome): built-in is the lightest touch — the shipped
  // baseline — workspace the shared committed layer, user the operator's own
  // override and so the strongest. The same scale the payload preview uses, so
  // provenance reads identically everywhere.
  const layerVariant: Record<Layer, BadgeVariant> = {
    'built-in': 'outline',
    workspace: 'secondary',
    user: 'default',
  }

  // `#/settings` with no sub-path falls back to the first space, so the screen is
  // never blank; an id that names no space does the same rather than 404-ing.
  const space = $derived<Space | null>(
    scope.kind === 'space' ? (spaces.find((s) => s.id === scope.spaceId) ?? spaces[0] ?? null)
    : scope.kind === 'user' ? null
    : (spaces[0] ?? null),
  )
  const onUser = $derived(scope.kind === 'user')

  // Every layer that participates in resolving this space, shared ones first so
  // the list reads bottom-up the way resolution does.
  const layers = $derived<ConfigLayer[]>(space ? [...config, ...space.layers] : config)

  let busy = $state<string | null>(null)
  let note = $state<string | null>(null)

  // The payload preview is the existing surface for "what would a session
  // actually be told" (ticket 08); this links into it rather than rebuilding it.
  // It needs a ticket, so the link names one: the first map's frontier ticket, or
  // its first ticket when none is takeable.
  const previewTarget = $derived.by<{ slug: string; ticket: Ticket } | null>(() => {
    for (const m of space?.maps ?? []) {
      if (!m.tickets.length) continue
      return { slug: m.slug, ticket: m.tickets.find((t) => t.frontier) ?? m.tickets[0] }
    }
    return null
  })
  let previewRole = $state<string | null>(null)
  // The preview opens on the role a ticket's *type* points at, so a role is
  // handed over as the type that resolves back to it (PayloadPreview.defaultRole).
  const typeForRole: Record<string, string> = {
    grill: 'grilling',
    prototype: 'prototype',
    research: 'research',
    implement: 'task',
  }

  // A binding field being edited. Only one at a time — this is a legibility
  // surface with an edit affordance, not a form.
  let editing = $state<{ role: string; field: 'adapter' | 'model' | 'args' } | null>(null)
  let draft = $state('')

  function fieldValue(b: RoleBinding, field: 'adapter' | 'model' | 'args'): string {
    if (field === 'args') return (b.args ?? []).join(' ')
    return field === 'adapter' ? b.adapter : b.model
  }

  function fieldFrom(b: RoleBinding, field: 'adapter' | 'model' | 'args'): Layer {
    if (field === 'args') return b.argsFrom
    return field === 'adapter' ? b.adapterFrom : b.modelFrom
  }

  function beginEdit(b: RoleBinding, field: 'adapter' | 'model' | 'args') {
    editing = { role: b.role, field }
    draft = fieldValue(b, field)
    note = null
  }

  function isEditing(role: string, field: string): boolean {
    return editing?.role === role && editing.field === field
  }

  // Committing an edit writes the user layer and re-derives; the new value and
  // its new provenance arrive over the control socket, so nothing is held
  // optimistically here.
  async function commit() {
    if (!editing || !space) return
    const { role, field } = editing
    const value = field === 'args' ? draft.trim().split(/\s+/).filter(Boolean) : draft.trim()
    busy = role + '.' + field
    try {
      await setBinding(space.id, role, field, value)
      editing = null
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }

  // Clearing an override reveals the layer beneath it — editing is reversible,
  // never a one-way ratchet (story 42).
  async function clearOverride(role: string, field: 'adapter' | 'model' | 'args') {
    if (!space) return
    busy = role + '.' + field
    try {
      await setBinding(space.id, role, field, null)
      if (isEditing(role, field)) editing = null
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }

  // The escape hatch for everything not editable inline: the server resolves the
  // *named* layer and launches the operator's editor. Where it cannot, the path
  // itself is the answer, surfaced here.
  async function open(layerName: string) {
    if (!space) {
      note = 'Register a space to open a config layer.'
      return
    }
    busy = layerName
    try {
      const r = await openConfigLayer(space.id, layerName)
      note =
        r.opened === 'editor' || r.opened === 'os' ? null
        : r.exists ? `Nothing to open it with — it lives at ${r.path}`
        : `Nothing there yet — it would live at ${r.path}`
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }

  function skillsOf(s: Space): ResolvedSkill[] {
    return s.skills ?? []
  }
</script>

<!-- The settings route renders in place of the space cockpit: its own title bar
     on the same tier as the space header, a scope column, and the resolved
     surface. Esc, the ⚙ button, or selecting a space leaves. -->
<div class="flex h-full min-h-0 flex-col">
  <header class="cockpit-bar justify-between">
    <div class="flex min-w-0 items-baseline gap-2">
      <span class="flex items-center gap-1.5 text-sm font-semibold">
        <Gear class="size-4" aria-hidden="true" /> Settings
      </span>
      <span class="truncate text-[0.7rem] text-muted-foreground">
        every resolved value, the layer it came from, and where that layer lives
      </span>
    </div>
    <Button variant="ghost" size="icon-sm" aria-label="Close settings (Esc)" title="Close (Esc)" onclick={onClose}>
      <X />
    </Button>
  </header>

  <div class="flex min-h-0 flex-1">
    <!-- The scopes: every space, then the one global user file. -->
    <nav class="flex w-56 shrink-0 flex-col gap-1 overflow-y-auto border-r border-border p-2">
      <span class="px-1.5 py-1 text-[0.65rem] font-semibold tracking-wide text-muted-foreground uppercase">
        Spaces
      </span>
      {#each spaces as s (s.id)}
        <a
          href={settingsHash({ kind: 'space', spaceId: s.id })}
          class={[
            'flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-xs',
            !onUser && space?.id === s.id ?
              'bg-accent text-accent-foreground font-medium'
            : 'hover:bg-accent/60',
          ]}
          onclick={(e) => {
            e.preventDefault()
            onScope({ kind: 'space', spaceId: s.id })
          }}
        >
          <Stack class="size-3.5 shrink-0" aria-hidden="true" />
          <span class="truncate">{s.name}</span>
        </a>
      {/each}

      <span class="mt-2 px-1.5 py-1 text-[0.65rem] font-semibold tracking-wide text-muted-foreground uppercase">
        Global
      </span>
      <a
        href={settingsHash({ kind: 'user' })}
        class={[
          'flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-xs',
          onUser ? 'bg-accent text-accent-foreground font-medium' : 'hover:bg-accent/60',
        ]}
        onclick={(e) => {
          e.preventDefault()
          onScope({ kind: 'user' })
        }}
      >
        <User class="size-3.5 shrink-0" aria-hidden="true" />
        <span class="truncate">Your config</span>
      </a>
    </nav>

    <ScrollArea.Root class="min-h-0 flex-1">
      <div class="flex flex-col gap-5 p-4">
        <!-- The explanation in place: one line of layering, then the badges do
             the rest. Deliberately not a diagram or a tutorial — the canonical
             account lives in the skills and the ADR, and these open them. -->
        <p class="flex flex-wrap items-baseline gap-x-2 gap-y-1 text-xs leading-relaxed text-muted-foreground">
          <span>
            Three layers resolve every value: <Badge variant="outline">built-in</Badge>
            ‹ <Badge variant="secondary">workspace</Badge> ‹ <Badge variant="default">user</Badge>. Role
            bindings resolve user-over-workspace (execution is yours); skills resolve
            workspace-over-user (content the project ships wins).
          </span>
          <span class="flex items-center gap-1">
            <Button variant="link" size="xs" disabled={!space} onclick={() => open('skill:core')}>
              how resolution works →
            </Button>
            <span class="font-mono text-[0.65rem]">docs/adr/0009</span>
          </span>
        </p>

        {#if note}
          <p class="rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs">{note}</p>
        {/if}

        {#if onUser}
          <!-- The one global user file: not a space's, so it gets its own scope
               rather than a copy under each space. -->
          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Your config</h2>
            <p class="text-xs leading-relaxed text-muted-foreground">
              Local, never committed, and per-machine. Binding overrides are keyed by space — edit
              them on a space above and they land here. Your skill forks live in a second root:
              whole directories that shadow the shipped default.
            </p>
            {#each config as l (l.name)}
              {@render layerRow(l)}
            {/each}
          </section>
        {:else if !space}
          <p class="text-sm text-muted-foreground">No spaces registered.</p>
        {:else}
          <section class="flex flex-col gap-1.5">
            <h2 class="flex items-baseline gap-2 text-xs font-semibold">
              {space.name}
              <code class="truncate font-mono text-[0.7rem] font-normal text-muted-foreground">{space.path}</code>
            </h2>
          </section>

          {#if space.warnings?.length}
            <!-- Config problems are surfaced where config is read (story 37). -->
            <ul class="flex flex-col gap-1.5 rounded-md border border-border p-2.5">
              {#each space.warnings as w}
                <li class="flex items-start gap-1.5 text-xs leading-relaxed text-muted-foreground">
                  <Warning class="mt-0.5 size-3.5 shrink-0" aria-hidden="true" /> <span>{w}</span>
                </li>
              {/each}
            </ul>
          {/if}

          <!-- Role bindings: the one thing this surface edits, and only into the
               user layer. -->
          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Role bindings</h2>
            <p class="text-xs text-muted-foreground">
              What each role actually runs. Editing writes your <em>user</em> layer; clearing an
              override reveals the layer beneath it.
            </p>
            <ul class="flex flex-col gap-2">
              {#each space.bindings as b (b.role)}
                <li class="rounded-md border border-border p-2.5">
                  <div class="mb-2 flex items-center justify-between gap-2">
                    <span class="text-xs font-semibold">{b.role}</span>
                    <span class="flex items-center gap-1.5">
                      {#if previewTarget}
                        <Button
                          variant="ghost"
                          size="xs"
                          title="Preview what a {b.role} session on #{padTicket(previewTarget.ticket.num)} · {previewTarget.ticket.title} would be told"
                          onclick={() => (previewRole = b.role)}
                        >
                          <Eye /> preview
                        </Button>
                      {/if}
                      {#if b.present}
                        <span class="flex items-center gap-1 text-[0.7rem] text-muted-foreground">
                          <CheckCircle class="size-3.5" /> on PATH
                        </span>
                      {:else}
                        <Badge variant="destructive" class="gap-1"><Warning /> not found</Badge>
                      {/if}
                    </span>
                  </div>
                  <div class="flex flex-col gap-1">
                    {@render field(b, 'adapter')}
                    {@render field(b, 'model')}
                    {@render field(b, 'args')}
                  </div>
                  {#if !b.present && b.missing}
                    <p class="mt-1.5 text-[0.7rem] text-muted-foreground">{b.missing}</p>
                  {/if}
                </li>
              {/each}
            </ul>
          </section>

          <!-- Skills: the positive statement of resolution, not just the
               stale-fork warning. -->
          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Skills</h2>
            <p class="text-xs text-muted-foreground">
              Whole-skill shadowing: the most specific layer defining a skill wins its entire
              directory. Read-value-plus-open-file — a skill is edited on disk.
            </p>
            <ul class="flex flex-col gap-1">
              {#each skillsOf(space) as sk (sk.name)}
                <li class="flex items-center gap-2 rounded-md border border-border px-2.5 py-1.5">
                  <span class="flex min-w-0 flex-1 flex-col">
                    <span class="flex items-center gap-1.5 text-xs font-medium">
                      <Sparkle class="size-3 shrink-0 text-muted-foreground" aria-hidden="true" />
                      <span class="truncate">{sk.name}</span>
                      {#if sk.stale}
                        <Badge variant="destructive" class="gap-1" title="forked from {sk.forkedFrom}; the shipped default has moved on — never auto-merged">
                          <Warning /> behind
                        </Badge>
                      {/if}
                    </span>
                    {#if sk.dir}
                      <code class="truncate font-mono text-[0.65rem] text-muted-foreground">{sk.dir}</code>
                    {:else}
                      <span class="text-[0.65rem] text-muted-foreground">shipped in the binary</span>
                    {/if}
                  </span>
                  <Badge variant={layerVariant[sk.layer]}>{sk.layer}</Badge>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    aria-label="Open the {sk.name} skill"
                    title="Open the winning {sk.name} directory"
                    disabled={busy !== null}
                    onclick={() => open('skill:' + sk.name)}
                  >
                    <ArrowSquareOut />
                  </Button>
                </li>
              {/each}
            </ul>
          </section>

          <!-- Map kinds: read-only here. Kind is declared through the deliberate,
               human-confirmed committed path (ADR 0007), which lives on the
               star-map's picker — this links there. -->
          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Map kinds</h2>
            <p class="text-xs text-muted-foreground">
              Declared in committed config, never inferred, and confirmed by a human on the
              star-map — read-only here.
            </p>
            {#if space.maps.length}
              <ul class="flex flex-col gap-1">
                {#each space.maps as m (m.slug)}
                  <li class="flex items-center gap-2 rounded-md border border-border px-2.5 py-1.5">
                    <span class="min-w-0 flex-1 truncate font-mono text-xs">{m.slug}</span>
                    {#if m.kind}
                      <Badge variant="secondary">{m.kind}</Badge>
                    {:else}
                      <Badge variant="outline">unclassified</Badge>
                    {/if}
                  </li>
                {/each}
              </ul>
              <Button variant="link" size="xs" class="self-start" onclick={() => onOpenMaps(space.id)}>
                classify on the star-map →
              </Button>
            {:else}
              <p class="text-xs text-muted-foreground">No maps in this space.</p>
            {/if}
          </section>

          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Layers on disk</h2>
            <p class="text-xs text-muted-foreground">
              Where each layer lives. Note the split: your binding overrides and your skill forks
              sit under different roots.
            </p>
            {#each layers as l (l.name)}
              {@render layerRow(l)}
            {/each}
          </section>
        {/if}
      </div>
    </ScrollArea.Root>
  </div>
</div>

{#if space && previewRole && previewTarget}
  <PayloadPreview
    open={true}
    spaceId={space.id}
    mapSlug={previewTarget.slug}
    ticketNum={previewTarget.ticket.num}
    ticketTitle={previewTarget.ticket.title}
    ticketType={typeForRole[previewRole] ?? 'task'}
    onClose={() => (previewRole = null)}
  />
{/if}

{#snippet field(b: RoleBinding, name: 'adapter' | 'model' | 'args')}
  {@const from = fieldFrom(b, name)}
  {@const value = fieldValue(b, name)}
  <div class="flex items-center gap-1.5">
    <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">{name}</span>
    {#if isEditing(b.role, name)}
      <Input
        class="h-7 min-w-0 flex-1 font-mono text-xs"
        bind:value={draft}
        spellcheck="false"
        autocapitalize="off"
        autocomplete="off"
        aria-label="{b.role} {name}"
        placeholder={name === 'args' ? 'space-separated flags' : name}
        onkeydown={(e: KeyboardEvent) => {
          if (e.key === 'Enter') commit()
          else if (e.key === 'Escape') editing = null
        }}
      />
      <Button variant="default" size="xs" disabled={busy !== null} onclick={commit}>save</Button>
      <Button variant="ghost" size="xs" onclick={() => (editing = null)}>cancel</Button>
    {:else}
      <button
        class="min-w-0 flex-1 truncate rounded px-1 py-0.5 text-left font-mono text-xs hover:bg-accent/60"
        title="Edit {b.role} {name} in your user layer"
        onclick={() => beginEdit(b, name)}
      >
        {value || '—'}
      </button>
      <Badge variant={layerVariant[from]}>{from}</Badge>
      {#if from === 'user'}
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label="Clear the {name} override"
          title="Clear this override — the layer beneath shows through"
          disabled={busy !== null}
          onclick={() => clearOverride(b.role, name)}
        >
          <ArrowCounterClockwise />
        </Button>
      {/if}
    {/if}
  </div>
{/snippet}

{#snippet layerRow(l: ConfigLayer)}
  <div class="flex items-center gap-2 rounded-md border border-border px-2.5 py-1.5">
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="text-xs font-medium">{l.holds === 'bindings' ? 'bindings & map kinds' : 'skills'}</span>
      <code class="truncate font-mono text-[0.65rem] text-muted-foreground" title={l.path}>{l.path}</code>
    </span>
    {#if !l.exists}
      <span class="shrink-0 text-[0.65rem] text-muted-foreground">not created yet</span>
    {/if}
    <Badge variant={layerVariant[l.layer]}>{l.layer}</Badge>
    <Button
      variant="ghost"
      size="icon-xs"
      aria-label="Open {l.path}"
      title="Open in your editor ($VISUAL / $EDITOR)"
      disabled={busy !== null}
      onclick={() => open(l.name)}
    >
      <ArrowSquareOut />
    </Button>
  </div>
{/snippet}
