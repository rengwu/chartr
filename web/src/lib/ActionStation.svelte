<script lang="ts">
  import { defaultRole, padTicket, type Agent, type Map as WMap } from './model'
  import { mapActionItems, type ActionItem } from './attention'
  import { spawnSession, ActionError } from './actions'
  import { chooseAgent, type AgentChoice } from './agentchoice'
  import * as Sheet from './components/ui/sheet'
  import { ListChecks, Rocket } from 'phosphor-svelte'

  // The per-map "Next up" action station (spec story 23): a drawer over
  // everything actionable on this map — the frontier ranked by how much each
  // unblocks. Hovering a row highlights its star (via `onhover`, threaded to
  // the island's hover seam).
  //
  // A row names the agent it would spawn with (story 22) and clicking it spawns
  // straight away in the role its own type names — the one-click ethos story 32
  // established, kept for every spawn after the first. With **nothing remembered**
  // a row deliberately does not spawn: it selects its ticket and leaves the
  // operator on the deliberate control, so there is exactly one picker in the
  // codebase and no one-click path that skips the initial choice (story 23).
  let {
    open = $bindable(false),
    map,
    spaceId,
    agents,
    lastAgent,
    onselect,
    onspawned,
    onhover,
  }: {
    open?: boolean
    map: WMap
    spaceId: string
    // The registered library and the space's remembered choice — the same two
    // inputs the action bar decides from, so a row can never name one agent and
    // launch another.
    agents: Agent[]
    lastAgent?: string
    onselect: (ticketNum: number) => void
    onspawned?: (sessionId: string) => void
    onhover?: (ticketNum: number | null) => void
  } = $props()

  const items = $derived<ActionItem[]>(mapActionItems(map))
  const agentChoice = $derived<AgentChoice>(chooseAgent(agents, lastAgent))

  let spawningNum = $state<number | null>(null)
  let spawnError = $state<string | null>(null)

  async function act(item: ActionItem) {
    // Nothing remembered (or a library with nothing in it): route to the
    // deliberate control rather than spawning something the operator never chose.
    if (agentChoice.kind !== 'ready') {
      onselect(item.ticket.num)
      open = false
      return
    }
    const role = defaultRole(item.ticket.type)
    spawningNum = item.ticket.num
    spawnError = null
    try {
      const res = await spawnSession(spaceId, map.slug, item.ticket.num, role, agentChoice.agent.name)
      onspawned?.(res.sessionId)
      onselect(item.ticket.num)
      open = false
    } catch (e) {
      spawnError = e instanceof ActionError ? e.message : (e as Error).message
    } finally {
      spawningNum = null
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="right" class="flex w-full flex-col gap-0 p-0 sm:max-w-sm">
    <Sheet.Header class="border-b border-border px-4 py-3 text-left">
      <Sheet.Title class="flex items-center gap-1.5 text-sm">
        <ListChecks class="size-4" /> Next up
      </Sheet.Title>
      <Sheet.Description class="text-xs text-muted-foreground">
        The frontier, ranked by how much each ticket unblocks.
        {#if agentChoice.kind !== 'ready'}
          Pick an agent on the ticket itself first — a row here never spawns one you haven’t chosen.
        {/if}
      </Sheet.Description>
    </Sheet.Header>

    <div class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
      {#if !items.length}
        <p class="p-3 text-xs text-muted-foreground">Nothing actionable on this map right now.</p>
      {/if}
      {#each items as item (item.ticket.num)}
        <button
          class="flex items-center gap-2 rounded-md border border-transparent px-2.5 py-2 text-left hover:border-border hover:bg-accent disabled:pointer-events-none disabled:opacity-60"
          disabled={spawningNum !== null}
          title={agentChoice.kind === 'ready'
            ? `Start #${padTicket(item.ticket.num)} with ${agentChoice.agent.name}`
            : `Open #${padTicket(item.ticket.num)} to choose an agent`}
          onmouseenter={() => onhover?.(item.ticket.num)}
          onmouseleave={() => onhover?.(null)}
          onfocus={() => onhover?.(item.ticket.num)}
          onblur={() => onhover?.(null)}
          onclick={() => act(item)}
        >
          <Rocket class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-xs font-medium">#{padTicket(item.ticket.num)} {item.ticket.title}</span>
            <span class="block truncate text-[0.65rem] text-muted-foreground">
              unblocks {item.unblockCount} ·
              {#if agentChoice.kind === 'ready'}
                {agentChoice.agent.name}
              {:else if agentChoice.kind === 'empty'}
                no agent registered
              {:else}
                choose an agent
              {/if}
            </span>
          </span>
          {#if spawningNum === item.ticket.num}
            <span class="shrink-0 text-[0.65rem] text-muted-foreground">spawning…</span>
          {/if}
        </button>
      {/each}
      {#if spawnError}
        <p class="px-2.5 py-1 text-[0.7rem] text-destructive">{spawnError}</p>
      {/if}
    </div>
  </Sheet.Content>
</Sheet.Root>
