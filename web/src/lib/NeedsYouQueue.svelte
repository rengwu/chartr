<script lang="ts">
  import type { Space } from './model'
  import { padTicket } from './model'
  import { needsYouQueue, type QueueEntry } from './attention'
  import * as Sheet from './components/ui/sheet'
  import { ArrowRight, Bell, Warning } from 'phosphor-svelte'

  // The cross-space "Needs you" queue (spec story 63): decision-level signals
  // only — a session halted — across every registered space. Strictly pull: it
  // renders only while summoned (the Sheet's own open state), never surfaced
  // on its own. One click jumps to that
  // space, its map summoned, the ticket in focus (the parent owns the jump —
  // this component only reports which entry was picked).
  let {
    open = $bindable(false),
    spaces,
    onjump,
  }: {
    open?: boolean
    spaces: Space[]
    onjump: (entry: QueueEntry) => void
  } = $props()

  const entries = $derived<QueueEntry[]>(needsYouQueue(spaces))
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="left" class="flex w-full flex-col gap-0 p-0 sm:max-w-sm">
    <Sheet.Header class="border-b border-border px-4 py-3 text-left">
      <Sheet.Title class="flex items-center gap-1.5 text-sm">
        <Bell class="size-4" /> Needs you
      </Sheet.Title>
      <Sheet.Description class="text-xs text-muted-foreground">
        Decision-level signals across every space — a session halted.
      </Sheet.Description>
    </Sheet.Header>

    <div class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
      {#if !entries.length}
        <p class="p-3 text-xs text-muted-foreground">Nothing across your spaces needs a decision.</p>
      {/if}
      {#each entries as entry (entry.spaceId + ':' + entry.mapSlug + ':' + entry.ticketNum + ':' + entry.kind)}
        <button
          class="flex items-center gap-2 rounded-md border border-transparent px-2.5 py-2 text-left hover:border-border hover:bg-accent"
          onclick={() => onjump(entry)}
        >
          <Warning class="size-4 shrink-0 text-destructive" aria-hidden="true" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-xs font-medium">{entry.spaceName} · {entry.mapName}</span>
            <span class="block truncate text-[0.65rem] text-muted-foreground">
              #{padTicket(entry.ticketNum)} {entry.ticketTitle} — session halted
            </span>
          </span>
          <ArrowRight class="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
        </button>
      {/each}
    </div>
  </Sheet.Content>
</Sheet.Root>
