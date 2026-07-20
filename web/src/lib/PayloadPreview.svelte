<script lang="ts">
  import Modal from './Modal.svelte'
  import { ROLES, type Payload, type PayloadPart, type Role } from './model'
  import { previewPayload } from './actions'
  import { renderMarkdown } from './markdown'

  // The payload preview (ticket 08, stories 45–49): for a chosen ticket and role,
  // exactly what a session would be told, assembled from the resolved prompt
  // library and the context bundle, with per-part layer provenance. The operator
  // reads it here before spawning ever exists — the library is hackable and this
  // is the window onto what an edit actually produces.
  let {
    open,
    spaceId,
    mapSlug,
    ticketNum,
    ticketTitle,
    ticketType,
    onClose,
  }: {
    open: boolean
    spaceId: string
    mapSlug: string
    ticketNum: number
    ticketTitle: string
    ticketType: string
    onClose: () => void
  } = $props()

  // The role a ticket's type points at, as the sensible default the preview opens
  // on; the operator can preview any role from here regardless.
  function defaultRole(type: string): Role {
    switch (type) {
      case 'research':
        return 'research'
      case 'prototype':
        return 'prototype'
      case 'grilling':
        return 'grill'
      default:
        return 'implement'
    }
  }

  let role = $state<Role>('implement')
  let payload = $state<Payload | null>(null)
  let error = $state<string | null>(null)
  let loading = $state(false)

  // Open fresh on the role the ticket's type points at; the operator then previews
  // any role from there. Seeding on the rising edge of `open` keeps a persistent
  // preview instance from carrying the last ticket's choice into a new ticket.
  let wasOpen = false
  $effect(() => {
    if (open && !wasOpen) role = defaultRole(ticketType)
    wasOpen = open
  })

  // Re-fetch whenever the preview is open and the role (or ticket) changes. The
  // harness reads the library fresh, so re-opening after editing a prompt on disk
  // shows the edit with no reload.
  let token = 0
  $effect(() => {
    if (!open) return
    const num = ticketNum
    const r = role
    const slug = mapSlug
    const id = spaceId
    const mine = ++token
    loading = true
    error = null
    previewPayload(id, slug, num, r)
      .then((p) => {
        if (mine !== token) return
        payload = p
        loading = false
      })
      .catch((e) => {
        if (mine !== token) return
        error = (e as Error).message
        loading = false
      })
  })

  const layerLabel: Record<string, string> = {
    'built-in': 'built-in',
    user: 'user',
    workspace: 'workspace',
    context: 'context',
  }

  function partKindLabel(p: PayloadPart): string {
    return p.kind === 'prompt' ? 'prompt' : 'context'
  }
</script>

<Modal {open} title="Payload preview" wide {onClose}>
  <div class="pp">
    <p class="pp-lede">
      What a <strong>session</strong> on <code>#{String(ticketNum).padStart(2, '0')} · {ticketTitle}</code>
      would be told — the resolved prompt library and the context bundle, assembled fresh. Each
      block is tagged with the layer it came from.
    </p>

    <div class="pp-roles" role="group" aria-label="Preview role">
      {#each ROLES as r (r)}
        <button
          class="pp-role"
          class:active={role === r}
          aria-pressed={role === r}
          onclick={() => (role = r)}>{r}</button
        >
      {/each}
    </div>

    {#if loading}
      <p class="pp-hint">Composing…</p>
    {:else if error}
      <p class="pp-error">Couldn’t compose the payload: {error}</p>
    {:else if payload}
      {#if payload.warnings?.length}
        <ul class="pp-warnings">
          {#each payload.warnings as w}
            <li class="pp-warning"><span aria-hidden="true">⚠</span> {w}</li>
          {/each}
        </ul>
      {/if}

      <ol class="pp-parts">
        {#each payload.parts as part (part.name)}
          <li class="pp-part" class:context={part.kind === 'context'}>
            <div class="pp-part-head">
              <span class="pp-part-name">{part.name}</span>
              <span class="pp-part-kind">{partKindLabel(part)}</span>
            </div>
            {#each part.segments as seg}
              <div class="pp-seg">
                <span class="pp-seg-prov">
                  <span class="pp-chip" data-layer={seg.layer}>{layerLabel[seg.layer] ?? seg.layer}</span>
                  {#if seg.label}<span class="pp-seg-label">{seg.label}</span>{/if}
                </span>
                <div class="pp-md">{@html renderMarkdown(seg.text)}</div>
              </div>
            {/each}
          </li>
        {/each}
      </ol>

      <details class="pp-raw">
        <summary>Composed document (what gets written to the payload file)</summary>
        <pre class="pp-pre">{payload.markdown}</pre>
      </details>
    {/if}
  </div>
</Modal>

<style>
  .pp {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    width: 100%;
    /* Long words, paths, and code spans wrap rather than force the card wider. */
    overflow-wrap: anywhere;
  }
  .pp-lede {
    margin: 0;
    color: var(--muted, #8a94a6);
    font-size: 0.9rem;
    line-height: 1.5;
  }
  .pp-lede code {
    color: var(--text, inherit);
    overflow-wrap: anywhere;
  }
  .pp-roles {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }
  .pp-role {
    border: 1px solid var(--border, #2b3242);
    background: transparent;
    color: var(--fg, inherit);
    border-radius: 999px;
    padding: 0.2rem 0.7rem;
    font-size: 0.82rem;
    cursor: pointer;
    text-transform: capitalize;
  }
  .pp-role:hover {
    border-color: var(--accent, #6ea8fe);
  }
  .pp-role.active {
    background: var(--accent, #6ea8fe);
    border-color: var(--accent, #6ea8fe);
    color: #0b0e14;
    font-weight: 600;
  }
  .pp-hint,
  .pp-error {
    margin: 0;
    font-size: 0.9rem;
  }
  .pp-error {
    color: var(--danger, #f0728a);
  }
  .pp-warnings {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .pp-warning {
    background: color-mix(in srgb, #d9a441 16%, transparent);
    border: 1px solid color-mix(in srgb, #d9a441 45%, transparent);
    border-radius: 6px;
    padding: 0.35rem 0.55rem;
    font-size: 0.83rem;
    line-height: 1.4;
  }
  .pp-parts {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .pp-part {
    border: 1px solid var(--border, #2b3242);
    border-radius: 8px;
    padding: 0.55rem 0.7rem;
    background: var(--panel, #141824);
  }
  .pp-part.context {
    background: color-mix(in srgb, var(--panel, #141824) 88%, #6ea8fe 12%);
  }
  .pp-part-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.3rem;
  }
  .pp-part-name {
    font-weight: 600;
    font-size: 0.9rem;
  }
  .pp-part-kind {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted, #8a94a6);
  }
  .pp-seg + .pp-seg {
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px dashed var(--border, #2b3242);
  }
  .pp-seg-prov {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.25rem;
  }
  .pp-chip {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: 4px;
    padding: 0.05rem 0.4rem;
    border: 1px solid var(--border, #2b3242);
    color: var(--muted, #8a94a6);
  }
  .pp-chip[data-layer='built-in'] {
    border-color: #4a5568;
  }
  .pp-chip[data-layer='user'] {
    border-color: #6ea8fe;
    color: #6ea8fe;
  }
  .pp-chip[data-layer='workspace'] {
    border-color: #57c98a;
    color: #57c98a;
  }
  .pp-chip[data-layer='context'] {
    border-color: #b48ce8;
    color: #b48ce8;
  }
  .pp-seg-label {
    font-size: 0.72rem;
    color: var(--muted, #8a94a6);
  }
  .pp-md {
    font-size: 0.86rem;
    line-height: 1.5;
  }
  .pp-md :global(h3),
  .pp-md :global(h4),
  .pp-md :global(h5) {
    font-size: 0.9rem;
    margin: 0.4rem 0 0.2rem;
  }
  .pp-md :global(p) {
    margin: 0.3rem 0;
  }
  .pp-md :global(pre) {
    overflow-x: auto;
    background: color-mix(in srgb, #000 25%, transparent);
    padding: 0.5rem;
    border-radius: 6px;
  }
  .pp-raw summary {
    cursor: pointer;
    font-size: 0.82rem;
    color: var(--muted, #8a94a6);
  }
  .pp-pre {
    margin: 0.4rem 0 0;
    background: color-mix(in srgb, #000 25%, transparent);
    padding: 0.6rem;
    border-radius: 6px;
    font-size: 0.78rem;
    line-height: 1.45;
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
  }
</style>
