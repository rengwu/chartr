<script lang="ts">
  import Modal from './Modal.svelte'
  import { defaultRole, ROLES, type Agent, type Payload, type PayloadPart, type Role } from './model'
  import { previewPayload } from './actions'
  import { chooseAgent, type AgentChoice } from './agentchoice'
  import { renderMarkdown } from './markdown'
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge'
  import { Button } from '$lib/components/ui/button'
  import * as ScrollArea from '$lib/components/ui/scroll-area'
  import { Warning } from 'phosphor-svelte'
  import { cn } from '$lib/utils'

  // The payload preview (ticket 08, stories 45–49): for a chosen ticket and role,
  // exactly what a session would be told, assembled from the resolved skill
  // library and the context bundle, with per-part layer provenance. The operator
  // reads it here before spawning ever exists — the library is hackable and this
  // is the window onto what an edit actually produces.
  //
  // It answers *what will run it* as well as *what will it read* (ticket 03,
  // story 24): the agent this space would spawn with, and the command line that
  // agent produces.
  let {
    open,
    spaceId,
    mapSlug,
    ticketNum,
    ticketTitle,
    ticketType,
    agents,
    lastAgent,
    onClose,
  }: {
    open: boolean
    spaceId: string
    mapSlug: string
    ticketNum: number
    ticketTitle: string
    ticketType: string
    // The registered library and the space's remembered choice — the same two
    // inputs the spawn control decides from, so the preview names the agent that
    // control would actually launch and cannot disagree with it.
    agents: Agent[]
    lastAgent?: string
    onClose: () => void
  } = $props()

  const agentChoice = $derived<AgentChoice>(chooseAgent(agents, lastAgent))

  // The role a ticket's type points at is the sensible default the preview opens
  // on; the operator can preview — and spawn — any of the four from here. It is
  // the shared default (model.defaultRole), so the preview and the surfaces that
  // spawn one-click agree on which role a ticket is.
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

  const layerLabel: Record<string, string> = {
    'built-in': 'built-in',
    user: 'user',
    workspace: 'workspace',
    context: 'context',
  }

  // The palette has one chromatic token (--destructive); four layers are told
  // apart by weight instead of hue: built-in (the shipped baseline) is the
  // lightest touch, workspace and user step up in emphasis for what a human
  // committed or configured locally, and context (assembled fresh per session)
  // is set apart as the odd one out.
  const layerVariant: Record<string, BadgeVariant> = {
    'built-in': 'outline',
    workspace: 'secondary',
    user: 'default',
    context: 'ghost',
  }

  function partKindLabel(p: PayloadPart): string {
    return p.kind === 'prompt' ? 'prompt' : 'context'
  }
</script>

<Modal {open} title="Payload preview" wide {onClose}>
  <div class="flex h-[65vh] flex-col gap-3">
    <p class="text-xs leading-relaxed text-muted-foreground">
      What a <strong class="font-medium text-foreground">session</strong> on
      <code class="rounded bg-muted px-1 py-0.5 font-mono text-foreground break-words"
        >#{String(ticketNum).padStart(2, '0')} · {ticketTitle}</code
      >
      would be told — the resolved skill library and the context bundle, assembled fresh. Each block is
      tagged with the layer it came from.
    </p>

    <div class="flex flex-wrap gap-1.5" role="group" aria-label="Preview role">
      {#each ROLES as r (r)}
        <Button
          variant={role === r ? 'default' : 'outline'}
          size="sm"
          class="capitalize"
          aria-pressed={role === r}
          onclick={() => (role = r)}>{r}</Button
        >
      {/each}
    </div>

    <!-- What will run it (ticket 03, story 24). The command comes off the agent
         library, which builds it through the same seam that builds the real argv,
         so this preview cannot drift from the launch. -->
    <div class="rounded-md border border-border p-2.5">
      <div class="flex items-baseline justify-between gap-2">
        <span class="text-sm font-medium">
          {#if agentChoice.kind === 'ready'}
            {agentChoice.agent.name}
          {:else if agentChoice.kind === 'empty'}
            No agent registered
          {:else}
            No agent chosen yet
          {/if}
        </span>
        <span class="text-[0.65rem] tracking-wide text-muted-foreground uppercase">runs this</span>
      </div>
      {#if agentChoice.kind === 'ready'}
        <p class="mt-1 font-mono text-[0.7rem] break-all text-muted-foreground">
          {agentChoice.agent.command.join(' ')}
        </p>
        <p class="mt-0.5 text-[0.7rem] text-muted-foreground">
          <span class="font-mono">‹opener›</span> is the read-this-file line pointing at the composed
          payload below.
        </p>
      {:else if agentChoice.kind === 'empty'}
        <p class="mt-1 text-[0.7rem] text-muted-foreground">
          Register an agent in settings before this ticket can be spawned.
        </p>
      {:else}
        <p class="mt-1 text-[0.7rem] text-muted-foreground">
          This space has not spawned yet — the spawn control will ask which agent to run it.
        </p>
      {/if}
    </div>

    {#if loading}
      <p class="text-sm text-muted-foreground">Composing…</p>
    {:else if error}
      <p class="text-sm text-destructive">Couldn’t compose the payload: {error}</p>
    {:else if payload}
      <ScrollArea.Root class="min-h-0 flex-1">
        <div class="flex flex-col gap-3 pr-3">
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

          <ol class="flex flex-col gap-2.5">
            {#each payload.parts as part (part.name)}
              <li class={cn('rounded-md border border-border p-2.5', part.kind === 'context' && 'bg-muted/30')}>
                <div class="mb-1 flex items-baseline justify-between gap-2">
                  <span class="text-sm font-medium">{part.name}</span>
                  <span class="text-[0.65rem] tracking-wide text-muted-foreground uppercase">{partKindLabel(part)}</span>
                </div>
                {#each part.segments as seg, i}
                  <div class={cn(i > 0 && 'mt-1.5 border-t border-dashed border-border pt-1.5')}>
                    <div class="mb-1 flex items-center gap-1.5">
                      <Badge variant={layerVariant[seg.layer] ?? 'outline'}>{layerLabel[seg.layer] ?? seg.layer}</Badge>
                      {#if seg.label}<span class="text-[0.7rem] text-muted-foreground">{seg.label}</span>{/if}
                    </div>
                    <div class="prose-sm">{@html renderMarkdown(seg.text)}</div>
                  </div>
                {/each}
              </li>
            {/each}
          </ol>

          <details class="text-xs">
            <summary class="cursor-pointer text-muted-foreground">Composed document (what gets written to the payload file)</summary>
            <pre
              class="mt-1.5 overflow-x-auto rounded-md bg-muted p-2.5 font-mono text-[0.7rem] leading-relaxed break-words whitespace-pre-wrap">{payload.markdown}</pre>
          </details>
        </div>
      </ScrollArea.Root>
    {/if}
  </div>
</Modal>
