<script lang="ts">
  import type { Map as WMap, Ticket } from './model'
  import { renderMarkdown, sectionOf } from './markdown'
  import PayloadPreview from './PayloadPreview.svelte'

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
  }: {
    map: WMap
    ticket?: Ticket | null
    dock?: 'right' | 'bottom'
    // The space the ticket belongs to — the key the payload preview fetches by.
    spaceId?: string
    onclose: () => void
  } = $props()

  const isMap = $derived(ticket === null)

  // The payload preview (ticket 08): from reading a ticket to seeing exactly what
  // a session on it would be told. Available only with a spaceId in hand.
  let showPreview = $state(false)

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

<aside class="detail-pane" class:bottom={dock === 'bottom'} aria-label={isMap ? 'Map material' : 'Ticket detail'}>
  <header class="dp-bar">
    {#if isMap}
      <span class="dp-eyebrow">Map material</span>
      <span class="dp-title">{map.name}</span>
    {:else if ticket}
      <span class="dp-eyebrow">
        #{pad(ticket.num)} · {ticket.type}
        <span class="dp-status" data-status={ticket.status}>{statusLabel[ticket.status] ?? ticket.status}</span>
        {#if ticket.frontier}<span class="dp-status frontier">frontier</span>{/if}
        {#if spaceId}
          <button
            class="dp-preview"
            title="Preview the payload a session on this ticket would be told"
            onclick={() => (showPreview = true)}>⧉ payload</button
          >
        {/if}
      </span>
      <span class="dp-title">{ticket.title}</span>
    {/if}
    <button class="dp-close" aria-label="Close pane (Esc)" title="Close (Esc)" onclick={onclose}>×</button>
  </header>

  <div class="dp-body">
    {#if isMap}
      {#if map.destination}
        <section class="dp-section">
          <h3 class="dp-h">Destination</h3>
          <div class="dp-md">{@html renderMarkdown(map.destination)}</div>
        </section>
      {/if}
      <section class="dp-section">
        <div class="dp-md">{@html renderMarkdown(stripDestination(map.body ?? ''))}</div>
      </section>
    {:else if ticket}
      <section class="dp-section">
        <div class="dp-md">{@html renderMarkdown(ticket.body ?? '')}</div>
      </section>

      <section class="dp-section">
        <h3 class="dp-h">Blockers</h3>
        {#if blockers.length === 0}
          <p class="dp-empty">None — this ticket depends on nothing.</p>
        {:else}
          <ul class="dp-blockers">
            {#each blockers as b (b.num)}
              <li class="dp-blocker">
                <div class="dp-blocker-head">
                  <span class="dp-blocker-num">#{pad(b.num)}</span>
                  <span class="dp-blocker-title">{b.title}</span>
                  <span class="dp-status" data-status={b.status}>{statusLabel[b.status] ?? b.status}</span>
                </div>
                {#if b.answer}
                  <div class="dp-md dp-blocker-answer">{@html renderMarkdown(b.answer)}</div>
                {:else}
                  <p class="dp-empty">Not yet answered.</p>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="dp-section">
        <h3 class="dp-h">Session history</h3>
        <p class="dp-empty">No sessions on this ticket yet.</p>
      </section>
    {/if}
  </div>
</aside>

<style>
  .dp-preview {
    border: 1px solid var(--border, #2b3242);
    background: transparent;
    color: var(--muted, #8a94a6);
    border-radius: 4px;
    padding: 0 5px;
    font-size: 10px;
    line-height: 1.5;
    cursor: pointer;
    white-space: nowrap;
    font-family: inherit;
  }
  .dp-preview:hover {
    border-color: var(--accent, #6ea8fe);
    color: var(--text, inherit);
  }
</style>

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
