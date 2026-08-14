<script lang="ts">
  import Modal from './Modal.svelte'
  import { defaultRole, type Agent, type Payload, type PayloadPart } from './model'
  import { previewPayload } from './actions'
  import { renderMarkdown } from './markdown'
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge'
  import * as Accordion from '$lib/components/ui/accordion'
  import * as ScrollArea from '$lib/components/ui/scroll-area'
  import { Warning, Quotes, Cube } from 'phosphor-svelte'
  import { cn } from '$lib/utils'
  import PrototypeSwitcher from './PrototypeSwitcher.svelte' // PROTOTYPE — throwaway

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

  // The palette has one chromatic token (--destructive); origins are told apart
  // by weight instead of hue: chartr's own embedded text is the lightest touch,
  // the operator's preferences step up for what a human wrote, and context
  // (assembled fresh per session) is set apart as the odd one out. Every other
  // origin is a *registered source's name*, an open set, so the lookup keeps its
  // fallback and a new source degrades to a plain outline rather than breaking.
  const originVariant: Record<string, BadgeVariant> = {
    chartr: 'outline',
    operator: 'default',
    context: 'ghost',
  }

  function partKindLabel(p: PayloadPart): string {
    return p.kind === 'prompt' ? 'prompt' : 'context'
  }

  // ── PROTOTYPE (throwaway) ─────────────────────────────────────────────
  // Three structurally-different takes on the collapsed row header, to settle
  // "the type tags are messy — I don't know what is what." Flip with the
  // bottom bar (?variant=A|B|C). Dev-only; fold the winner in and delete the
  // rest (this block, PrototypeSwitcher import + component, the variant name
  // helpers). See PrototypeSwitcher.svelte.
  const VARIANTS = ['A', 'B', 'C']
  const VARIANT_LABELS: Record<string, string> = {
    A: 'Sectioned, cut redundancy',
    B: 'Two-line, colour-coded',
    C: 'Log-line, source token',
  }
  let variant = $state('A')
  $effect(() => {
    const v = new URLSearchParams(window.location.search).get('variant')?.toUpperCase()
    if (v && VARIANTS.includes(v)) variant = v
  })
  function setVariant(v: string) {
    variant = v
    const url = new URL(window.location.href)
    url.searchParams.set('variant', v)
    history.replaceState(null, '', url)
  }

  // Label only earns its place when it says something the bold name doesn't.
  function labelAdds(p: PayloadPart): boolean {
    return !!p.label && p.label.toLowerCase() !== p.name.toLowerCase()
  }
  // The provenance pill is noise when it just echoes the section it's already in.
  function originAdds(p: PayloadPart): boolean {
    return p.origin !== 'context'
  }
  // Rough token weight of a block — chars/4 — so the operator can see which
  // blocks dominate the context window this modal is previewing.
  function tokenEstimate(p: PayloadPart): string {
    const est = Math.round(p.text.length / 4)
    return est < 1000 ? `~${est} tok` : `~${(est / 1000).toFixed(1)}k tok`
  }
  const promptParts = $derived((payload?.parts ?? []).filter((p) => p.kind === 'prompt'))
  const contextParts = $derived((payload?.parts ?? []).filter((p) => p.kind === 'context'))
  // ── /PROTOTYPE ────────────────────────────────────────────────────────
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

          <!-- ── PROTOTYPE (throwaway): three row-header variants ──────────── -->
          {#if variant === 'A'}
            <!-- A · Sectioned, redundancy cut. The prompt/context axis becomes a
                 section header, so the far-right tag and the echoing `context`
                 pill both vanish; name + provenance is all that's left. -->
            {#each [{ head: 'Prompt', items: promptParts }, { head: 'Context', items: contextParts }] as group (group.head)}
              {#if group.items.length}
                <div class="flex flex-col gap-1.5">
                  <span class="px-0.5 text-[0.65rem] font-medium tracking-wider text-muted-foreground uppercase">{group.head}</span>
                  <Accordion.Root type="multiple" class="flex flex-col gap-2.5">
                    {#each group.items as part (part.name)}
                      <Accordion.Item value={part.name} class="rounded-md border border-border">
                        <Accordion.Trigger class="items-center gap-2 p-2.5 hover:no-underline">
                          <span class="text-sm font-medium">{part.name}</span>
                          {#if originAdds(part)}<Badge variant={originVariant[part.origin] ?? 'secondary'}>{part.origin}</Badge>{/if}
                          {#if labelAdds(part)}<span class="truncate text-[0.7rem] text-muted-foreground">{part.label}</span>{/if}
                        </Accordion.Trigger>
                        {@render body(part)}
                      </Accordion.Item>
                    {/each}
                  </Accordion.Root>
                </div>
              {/if}
            {/each}
          {:else if variant === 'B'}
            <!-- B · Two-line, colour-coded. Kind is a left accent bar, not a word;
                 name + provenance sit on line one, the descriptive label drops to a
                 quiet subtitle on line two. -->
            <Accordion.Root type="multiple" class="flex flex-col gap-2.5">
              {#each payload.parts as part (part.name)}
                <Accordion.Item
                  value={part.name}
                  class={cn(
                    'rounded-md border border-border border-l-2',
                    part.kind === 'prompt' ? 'border-l-primary' : 'border-l-muted-foreground/50',
                  )}
                >
                  <Accordion.Trigger class="items-center gap-2 p-2.5 hover:no-underline">
                    <div class="flex min-w-0 flex-col gap-0.5">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-medium">{part.name}</span>
                        <Badge variant={originVariant[part.origin] ?? 'secondary'}>{part.origin}</Badge>
                      </div>
                      {#if labelAdds(part)}<span class="truncate text-left text-[0.7rem] text-muted-foreground">{part.label}</span>{/if}
                    </div>
                  </Accordion.Trigger>
                  {@render body(part)}
                </Accordion.Item>
              {/each}
            </Accordion.Root>
          {:else}
            <!-- C · Log-line. One dense line per block: a leading kind icon, a
                 monospace `source/` token for provenance, the bold name, then the
                 faded label. Reads like a manifest. -->
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
          {/if}
          <!-- ── /PROTOTYPE ───────────────────────────────────────────────── -->

          <details class="text-xs">
            <summary class="cursor-pointer text-muted-foreground">Composed document (what gets written to the payload file)</summary>
            <pre
              class="mt-1.5 rounded-md bg-muted p-2.5 font-mono text-[0.7rem] leading-relaxed break-words whitespace-pre-wrap">{payload.markdown}</pre>
          </details>
        </div>
      </ScrollArea.Root>
    {/if}
  </div>

  <!-- PROTOTYPE (throwaway) — dev-only variant switcher; never ships. -->
  {#if import.meta.env.DEV}
    <PrototypeSwitcher variants={VARIANTS} current={variant} label={VARIANT_LABELS[variant]} onChange={setVariant} />
  {/if}
</Modal>
