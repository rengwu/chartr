<script lang="ts">
  import type { TrackerAdapterOffer } from './model'
  import { installTrackerAdapter, dismissTrackerAdapter } from './actions'
  import { Button } from './components/ui/button'
  import Modal from './Modal.svelte'
  import { MapTrifold, ArrowClockwise, Warning, X } from 'phosphor-svelte'

  // chartr's standing offer to write its tracker adapter into a space, rendered as
  // a dismissible banner under the space header (ADR 0010 chrome — plain Svelte +
  // tokens + primitives, no island). The offer is authoritative and gating: it
  // rides the snapshot only while there is something to act on, so this component
  // is mounted iff `space.trackerAdapter` exists, and every action's result arrives
  // as the *next* snapshot dropping the offer — never a client-side mutation. The
  // buttons stay disabled from the moment one is pressed until that snapshot
  // unmounts us; a refusal instead re-enables and shows chartr's own message.
  let {
    spaceId,
    offer,
  }: {
    spaceId: string
    offer: TrackerAdapterOffer
  } = $props()

  let busy = $state(false)
  let error = $state<string | null>(null)
  // The one destructive path — overwriting a foreign file — earns a confirm step
  // (the dialog primitive) rather than a one-click clobber.
  let confirming = $state(false)

  // A cosmetic read of what foreign file is in the way — phrasing only.
  const hintLabel: Record<string, string> = { gh: 'GitHub', glab: 'GitLab', linear: 'Linear' }
  const foreignHint = $derived(offer.remoteHint ? hintLabel[offer.remoteHint] : undefined)

  async function run(fn: () => Promise<unknown>) {
    busy = true
    error = null
    try {
      await fn()
      // Success: leave `busy` set — the offer's own disappearance from the next
      // snapshot unmounts this banner. Re-enabling would only flicker.
    } catch (e) {
      error = (e as Error).message
      busy = false
    }
  }

  const install = () => run(() => installTrackerAdapter(spaceId))
  const dismiss = () => run(() => dismissTrackerAdapter(spaceId))
  function replace() {
    confirming = false
    run(() => installTrackerAdapter(spaceId))
  }
</script>

<!-- A quiet banner, emphasised (border-ring) while chartr wants a write and neutral
     when it's only asking to leave a foreign file be. Not a hand-rolled .card: a
     utility-composed container in the house style (cf. Settings warnings), with
     every action a Button primitive. -->
<div
  class="mx-3 mt-2 flex flex-col gap-1.5 rounded-md border p-2.5 {offer.state === 'foreign'
    ? 'border-border'
    : 'border-ring'}"
>
  <div class="flex items-start gap-2.5">
    <div class="mt-0.5 shrink-0 text-muted-foreground">
      {#if offer.state === 'absent'}
        <MapTrifold class="size-4" aria-hidden="true" />
      {:else if offer.state === 'stale'}
        <ArrowClockwise class="size-4" aria-hidden="true" />
      {:else}
        <Warning class="size-4" aria-hidden="true" />
      {/if}
    </div>

    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      {#if offer.state === 'absent'}
        <p class="text-xs leading-relaxed">Let chartr's skills write maps here.</p>
      {:else if offer.state === 'stale'}
        <p class="text-xs leading-relaxed">chartr's tracker adapter has an update.</p>
      {:else}
        <p class="text-xs leading-relaxed">
          An existing tracker config is here{foreignHint ? ` — looks like ${foreignHint}` : ''}.
        </p>
      {/if}
      <code class="truncate font-mono text-[0.7rem] text-muted-foreground" title={offer.path}>
        {offer.path}
      </code>
      {#if error}
        <p class="mt-0.5 text-[0.7rem] text-destructive">{error}</p>
      {/if}
    </div>

    <div class="flex shrink-0 items-center gap-1.5">
      {#if offer.state === 'foreign'}
        <!-- Leave (= dismiss) is the safe default; Replace overwrites a foreign
             file, so it's the destructive secondary and goes through a confirm. -->
        <Button variant="secondary" size="xs" disabled={busy} onclick={dismiss}>Leave</Button>
        <Button variant="destructive" size="xs" disabled={busy} onclick={() => (confirming = true)}>
          Replace
        </Button>
      {:else}
        <Button variant="default" size="xs" disabled={busy} onclick={install}>
          {offer.state === 'stale' ? 'Refresh' : 'Install'}
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label="Dismiss"
          title="Don't offer this again for this space"
          disabled={busy}
          onclick={dismiss}
        >
          <X />
        </Button>
      {/if}
    </div>
  </div>
</div>

<Modal open={confirming} title="Replace the existing tracker config?" onClose={() => (confirming = false)}>
  <div class="flex flex-col gap-3">
    <p class="text-sm leading-relaxed text-muted-foreground">
      This overwrites the file already at
      <code class="font-mono text-xs text-foreground">{offer.path}</code>
      with chartr's tracker adapter. There's no undo.
    </p>
    <div class="flex items-center justify-end gap-2">
      <Button variant="ghost" size="sm" onclick={() => (confirming = false)}>Cancel</Button>
      <Button variant="destructive" size="sm" onclick={replace}>Replace</Button>
    </div>
  </div>
</Modal>
