<script lang="ts">
  import { rolesForKind, type Kind, type Map as WMap, type Role, type Ticket } from './model'
  import { renderMarkdown, sectionOf } from './markdown'
  import { spawnSession, ActionError } from './actions'
  import PayloadPreview from './PayloadPreview.svelte'
  import * as Card from '$lib/components/ui/card'
  import * as ScrollArea from '$lib/components/ui/scroll-area'
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge'
  import { Button } from '$lib/components/ui/button'
  import { Eye, X, Rocket, Warning } from 'phosphor-svelte'
  import { cn } from '$lib/utils'

  // The detail pane (ticket 07): from looking at a star to reading it in one
  // click. It renders one ticket — question, Done-when, its blockers with their
  // answers inline, and session history — or, from the map's title, the map's own
  // material. Content is assembled from the derived model (the inlined bodies) so
  // the pane needs no second fetch. Whether it docks right or bottom is the
  // parent's responsive decision; this is only the content.
  let {
    map,
    ticket = null,
    dock = 'right',
    spaceId,
    onclose,
    onspawned,
  }: {
    map: WMap
    ticket?: Ticket | null
    dock?: 'right' | 'bottom'
    // The space the ticket belongs to — the key the payload preview fetches by.
    spaceId?: string
    onclose: () => void
    // Called with the new session id after a successful spawn, so the enclosing
    // chrome can make that session's tab active.
    onspawned?: (sessionId: string) => void
  } = $props()

  const isMap = $derived(ticket === null)

  // The payload preview (ticket 08): from reading a ticket to seeing exactly what
  // a session on it would be told. Available only with a spaceId in hand.
  let showPreview = $state(false)

  // Spawn (tickets 09, 11): a frontier ticket on a classified map offers a fresh
  // session in any of the kind's roles; a `proposed` ticket offers exactly a review
  // — the state a review hangs on (its dependents stay blocked until a human
  // approves). An unclassified map offers none. So the affordance appears only where
  // a spawn is actually takeable, and the default role is the one the ticket's type
  // points at, clamped to what is offered (which resolves to review on a proposal).
  function offeredRoles(kind: Kind, tk: Ticket | null): Role[] {
    if (!tk) return []
    const roles = rolesForKind(kind)
    if (tk.frontier) return roles
    if (tk.status === 'proposed' && roles.includes('review')) return ['review']
    return []
  }
  const spawnRoles = $derived<Role[]>(offeredRoles(map.kind, ticket))
  const canSpawn = $derived(!!spaceId && spawnRoles.length > 0)

  function defaultRole(type: string, offered: Role[]): Role {
    const guess: Role =
      type === 'research'
        ? 'research'
        : type === 'prototype'
          ? 'prototype'
          : type === 'grilling'
            ? 'grill'
            : 'implement'
    return offered.includes(guess) ? guess : offered[0]
  }

  let spawnRole = $state<Role | null>(null)
  let spawning = $state(false)
  let spawnError = $state<string | null>(null)

  // A single DetailPane instance is reused as the selection changes ticket, so a
  // role the operator picked (or a block message they saw) on one ticket must not
  // linger onto the next: reset both when the ticket number changes.
  let lastNum: number | undefined = undefined
  $effect(() => {
    const n = ticket?.num
    if (n !== lastNum) {
      lastNum = n
      spawnRole = null
      spawnError = null
    }
  })

  // The effective role: the operator's pick if it is still one the kind offers,
  // else the default the ticket's type points at. Only meaningful when canSpawn,
  // so spawnRoles is non-empty wherever this resolves to a role.
  const activeRole = $derived<Role | null>(
    !canSpawn || !ticket
      ? null
      : spawnRole && spawnRoles.includes(spawnRole)
        ? spawnRole
        : defaultRole(ticket.type, spawnRoles),
  )

  async function spawn() {
    if (!spaceId || !ticket || !activeRole) return
    spawning = true
    spawnError = null
    try {
      const res = await spawnSession(spaceId, map.slug, ticket.num, activeRole)
      onspawned?.(res.sessionId)
    } catch (e) {
      // A blocked spawn (absent agent, held ticket) carries the harness's specific
      // message — surface it inline rather than as a silent no-op.
      spawnError = e instanceof ActionError ? e.message : (e as Error).message
    } finally {
      spawning = false
    }
  }

  // The closing-answer section names, in the order a resolved/proposed/ruled-out
  // ticket carries them — used to show a blocker's answer inline.
  const ANSWER_SECTIONS = ['Answer', 'Proposed Answer', 'Ruled out']

  // A blocker resolved from the same map, with its answer pulled from its body.
  interface Blocker {
    num: number
    title: string
    status: string
    answer: string
  }
  const blockers = $derived.by<Blocker[]>(() => {
    if (!ticket?.blockedBy?.length) return []
    return ticket.blockedBy.map((n) => {
      const b = map.tickets.find((t) => t.num === n)
      if (!b) return { num: n, title: '(missing ticket)', status: 'unknown', answer: '' }
      return { num: n, title: b.title, status: b.status, answer: sectionOf(b.body ?? '', ANSWER_SECTIONS) }
    })
  })

  const statusLabel: Record<string, string> = {
    open: 'open',
    claimed: 'claimed',
    proposed: 'proposed',
    resolved: 'resolved',
    out_of_scope: 'out of scope',
    unknown: 'missing',
  }

  // resolved reads as the bold/solid "done" state (the palette's only accent
  // besides destructive is the neutral --primary — there is no green to key a
  // literal success tint off); proposed/claimed share the lighter --primary-
  // adjacent secondary emphasis the ticket calls for; out_of_scope stays muted;
  // an unresolved blocker reference is the one true "problem" and gets destructive.
  const statusVariant: Record<string, BadgeVariant> = {
    open: 'outline',
    claimed: 'secondary',
    proposed: 'secondary',
    resolved: 'default',
    out_of_scope: 'outline',
    unknown: 'destructive',
  }

  function pad(n: number): string {
    return n < 10 ? '0' + n : String(n)
  }

  // The map body leads with its Destination heading; the pane shows that above,
  // so strip the duplicate section from the rendered body.
  function stripDestination(body: string): string {
    const lines = body.split('\n')
    let start = -1
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].trim() === '## Destination') {
        start = i
        break
      }
    }
    if (start < 0) return body
    let end = lines.length
    for (let i = start + 1; i < lines.length; i++) {
      if (/^##\s/.test(lines[i])) {
        end = i
        break
      }
    }
    return [...lines.slice(0, start), ...lines.slice(end)].join('\n').trim()
  }
</script>

<Card.Root
  role="complementary"
  aria-label={isMap ? 'Map material' : 'Ticket detail'}
  class={cn('detail-pane h-full min-h-0 flex-col gap-0 overflow-hidden py-0', dock === 'bottom' && 'bottom')}
>
  <Card.Header class="flex flex-row items-start justify-between gap-2 border-b border-border px-3 py-2.5">
    <div class="flex min-w-0 flex-col gap-1">
      {#if isMap}
        <span class="text-[0.7rem] font-medium tracking-wide text-muted-foreground uppercase">Map material</span>
        <span class="truncate text-sm font-medium">{map.name}</span>
      {:else if ticket}
        <span class="flex flex-wrap items-center gap-1.5 text-[0.7rem] text-muted-foreground">
          <span class="font-mono">#{pad(ticket.num)}</span>
          <span aria-hidden="true">·</span>
          <span>{ticket.type}</span>
          <Badge variant={statusVariant[ticket.status] ?? 'outline'} class={ticket.status === 'out_of_scope' ? 'text-muted-foreground' : ''}>
            {statusLabel[ticket.status] ?? ticket.status}
          </Badge>
          {#if ticket.frontier}
            <Badge variant="outline" class="border-primary/50 text-primary">frontier</Badge>
          {/if}
          {#if spaceId}
            <Button
              variant="ghost"
              size="xs"
              title="Preview the payload a session on this ticket would be told"
              onclick={() => (showPreview = true)}
            >
              <Eye /> payload
            </Button>
          {/if}
        </span>
        <span class="truncate text-sm font-medium">{ticket.title}</span>
      {/if}
    </div>
    <Button variant="ghost" size="icon-sm" aria-label="Close pane (Esc)" title="Close (Esc)" onclick={onclose}>
      <X />
    </Button>
  </Card.Header>

  <ScrollArea.Root class="min-h-0 flex-1">
    <Card.Content class="flex flex-col gap-4 p-3">
      {#if isMap}
        {#if map.destination}
          <section class="flex flex-col gap-1.5">
            <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">Destination</h3>
            <div class="prose-sm">{@html renderMarkdown(map.destination)}</div>
          </section>
        {/if}
        <section>
          <div class="prose-sm">{@html renderMarkdown(stripDestination(map.body ?? ''))}</div>
        </section>
      {:else if ticket}
        <section>
          <div class="prose-sm">{@html renderMarkdown(ticket.body ?? '')}</div>
        </section>

        <section class="flex flex-col gap-1.5">
          <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">Blockers</h3>
          {#if blockers.length === 0}
            <p class="text-xs text-muted-foreground">None — this ticket depends on nothing.</p>
          {:else}
            <ul class="flex flex-col gap-2">
              {#each blockers as b (b.num)}
                <li class="rounded-md border border-border p-2.5">
                  <div class="mb-1 flex items-center gap-1.5 text-xs">
                    <span class="font-mono text-muted-foreground">#{pad(b.num)}</span>
                    <span class="flex-1 truncate font-medium">{b.title}</span>
                    <Badge variant={statusVariant[b.status] ?? 'outline'}>{statusLabel[b.status] ?? b.status}</Badge>
                  </div>
                  {#if b.answer}
                    <div class="prose-sm">{@html renderMarkdown(b.answer)}</div>
                  {:else}
                    <p class="text-xs text-muted-foreground">Not yet answered.</p>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </section>

        {#if canSpawn}
          <section class="flex flex-col gap-2 rounded-md border border-border p-2.5">
            <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">Spawn a session</h3>
            <div class="flex flex-wrap gap-1.5" role="group" aria-label="Session role">
              {#each spawnRoles as r (r)}
                <Button
                  variant={activeRole === r ? 'default' : 'outline'}
                  size="xs"
                  aria-pressed={activeRole === r}
                  onclick={() => (spawnRole = r)}>{r}</Button
                >
              {/each}
            </div>
            <Button size="sm" disabled={spawning || !activeRole} onclick={spawn}>
              <Rocket />
              {spawning ? 'Spawning…' : `Spawn ${activeRole ?? ''}`}
            </Button>
            {#if spawnError}
              <p class="flex items-start gap-1.5 text-[0.7rem] text-destructive">
                <Warning class="mt-0.5 size-3.5 shrink-0" /> <span>{spawnError}</span>
              </p>
            {/if}
          </section>
        {/if}

        <section class="flex flex-col gap-1.5">
          <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">Session history</h3>
          <p class="text-xs text-muted-foreground">No sessions on this ticket yet.</p>
        </section>
      {/if}
    </Card.Content>
  </ScrollArea.Root>
</Card.Root>

{#if !isMap && ticket && spaceId}
  <PayloadPreview
    open={showPreview}
    {spaceId}
    mapSlug={map.slug}
    ticketNum={ticket.num}
    ticketTitle={ticket.title}
    ticketType={ticket.type}
    onClose={() => (showPreview = false)}
  />
{/if}
