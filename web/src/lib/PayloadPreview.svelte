<script lang="ts">
  import Modal from './Modal.svelte'
  import { defaultRole, type Agent, type Payload, type PayloadPart } from './model'
  import { previewPayload } from './actions'
  import { renderMarkdown } from './markdown'
  import * as Accordion from '$lib/components/ui/accordion'
  import * as ScrollArea from '$lib/components/ui/scroll-area'
  import { Warning, Quotes, Cube } from 'phosphor-svelte'

  // The payload preview (ticket 08, stories 45–49): for a chosen ticket and role,
  // exactly what a session would be told, assembled from the resolved skill
  // library and the context bundle, with per-part layer provenance. The operator
  // reads it here before spawning ever exists — the library is hackable and this
  // is the window onto what an edit actually produces.
  let {
    open,
    spaceId = '',
    mapSlug = '',
    ticketNum = 0,
    ticketTitle = '',
    ticketType = '',
    agents = [],
    lastAgent,
    onClose,
  }: {
    open: boolean
    spaceId?: string
    mapSlug?: string
    ticketNum?: number
    ticketTitle?: string
    ticketType?: string
    // The registered library and the space's remembered choice — the same two
    // inputs the spawn control decides from, so the preview names the agent that
    // control would actually launch and cannot disagree with it.
    agents?: Agent[]
    lastAgent?: string
    onClose: () => void
  } = $props()

  // The preview shows exactly the role the ticket's type is bound to — the shared
  // default (model.defaultRole), so the preview and the surfaces that spawn
  // one-click agree on which role a ticket is. There is no switcher: a session is
  // spawned in its ticket's role, and the preview mirrors that single reality.
  const role = $derived(defaultRole(ticketType))
  let payload = $state<Payload | null>(null)
  let error = $state<string | null>(null)
  let loading = $state(false)

  // Re-fetch whenever the preview is open and the role (or ticket) changes. The
  // chartr reads the library fresh, so re-opening after editing a prompt on disk
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

  // Rough token weight of a block — chars/4 — so the operator can see at a glance
  // which blocks dominate the context window this modal previews.
  function tokenEstimate(p: PayloadPart): string {
    const est = Math.round(p.text.length / 4)
    return est < 1000 ? `~${est} tok` : `~${(est / 1000).toFixed(1)}k tok`
  }
</script>

<Modal {open} title="Payload preview" wide {onClose}>
  <div class="flex h-[65vh] flex-col gap-3">
    <p class="text-xs leading-relaxed text-muted-foreground">
      What a session on
      <code class="rounded bg-muted px-1 py-0.5 font-mono text-foreground break-words"
        >#{String(ticketNum).padStart(2, '0')}</code
      >
      is told. Each block shows its source.
    </p>

    {#if loading}
      <p class="text-sm text-muted-foreground">Composing…</p>
    {:else if error}
      <p class="text-sm text-destructive">Couldn’t compose the payload: {error}</p>
    {:else if payload}
      <ScrollArea.Root class="min-h-0 flex-1">
        <div class="flex flex-col gap-3">
          {#if payload.warnings?.length}
            <ul class="flex flex-col gap-1.5">
              {#each payload.warnings as w}
                <li class="flex items-start gap-2 rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs leading-relaxed">
                  <Warning class="mt-0.5 shrink-0 text-muted-foreground" aria-hidden="true" />
                  <span>{w}</span>
                </li>
              {/each}
            </ul>
          {/if}

          {#snippet body(part: PayloadPart)}
            <Accordion.Content class="px-2.5 pb-2.5">
              <div class="prose-sm">{@html renderMarkdown(part.text)}</div>
            </Accordion.Content>
          {/snippet}

          <!-- One dense line per block, read like a manifest: a leading kind icon
               (prompt vs context), a monospace `source/` token for provenance, the
               block name, then its token weight. All collapsed by default; the
               grouped card and `type="multiple"` let several open at once. -->
          <Accordion.Root type="multiple" class="flex flex-col overflow-hidden rounded-md border border-border">
            {#each payload.parts as part (part.name)}
              <Accordion.Item value={part.name}>
                <Accordion.Trigger class="items-center gap-2 p-2 hover:no-underline">
                  {#if part.kind === 'prompt'}
                    <Quotes class="shrink-0 text-muted-foreground" aria-label="prompt" />
                  {:else}
                    <Cube class="shrink-0 text-muted-foreground" aria-label="context" />
                  {/if}
                  <span class="flex min-w-0 flex-1 items-baseline">
                    <span class="shrink-0 font-mono text-xs text-muted-foreground">{part.origin}/</span>
                    <span class="text-xs font-medium">{part.name}</span>
                  </span>
                  <span class="shrink-0 font-mono text-[0.7rem] text-muted-foreground tabular-nums">{tokenEstimate(part)}</span>
                </Accordion.Trigger>
                {@render body(part)}
              </Accordion.Item>
            {/each}
          </Accordion.Root>

          <details class="text-xs">
            <summary class="cursor-pointer text-muted-foreground">Composed document (what gets written to the payload file)</summary>
            <pre
              class="mt-1.5 rounded-md bg-muted p-2.5 font-mono text-[0.7rem] leading-relaxed break-words whitespace-pre-wrap">{payload.markdown}</pre>
          </details>
        </div>
      </ScrollArea.Root>
    {/if}
  </div>
</Modal>
