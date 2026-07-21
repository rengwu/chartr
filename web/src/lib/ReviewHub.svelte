<script lang="ts">
  import type { Map as WMap, Role, Ticket } from './model'
  import { rolesForKind } from './model'
  import {
    ActionError,
    abandonTicket,
    approveTicket,
    followUp,
    readReview,
    spawnSession,
    ticketDiff,
    type ApproveResult,
    type DiffScope,
    type ReviewRead,
    type TicketDiff,
  } from './actions'
  import { renderMarkdown, sectionOf } from './markdown'
  import Modal from './Modal.svelte'
  import { Badge } from '$lib/components/ui/badge'
  import { Button } from '$lib/components/ui/button'
  import { Checkbox } from '$lib/components/ui/checkbox'
  import * as ScrollArea from '$lib/components/ui/scroll-area'
  import { ArrowUUpLeft, CaretDown, CaretRight, Check, Rocket, Warning, X } from 'phosphor-svelte'

  // The human review hub (ticket 12): the gate, whole. It takes over the map card
  // and leads with the **brief** — the plain markdown the harness already wrote to
  // disk (ticket 11), rendered here with buttons and nothing else (story 62). The
  // 20% that decides sits on top: what was done, what the reviewer found, and a
  // recommendation the harness derived *mechanically* from the verdict. The full
  // verdict and the diff are one click deeper.
  //
  // Four exits, each a plain HTTP action: Approve (the promotion commit), Send
  // back (a fix-up session briefed with the blocking finding and the operator's
  // note), Take it further (more sessions on the same proposal), and Abandon (the
  // demotion). Esc is the fifth — walking away is legal.
  let {
    spaceId,
    map,
    ticket,
    onclose,
    onspawned,
    onselect,
    approved = $bindable(null),
  }: {
    spaceId: string
    map: WMap
    ticket: Ticket
    onclose: () => void
    // Bubbled up so the chrome can make a follow-up session's tab active.
    onspawned?: (sessionId: string) => void
    // The post-approve strip's Next button falls back to this when the suggested
    // ticket cannot be spawned straight from here — the chrome selects it instead
    // (ticket 17), so the click is never a no-op.
    onselect?: (ticketNum: number) => void
    // Lifted into the parent (ticket 17) so the post-approve strip survives the
    // approve transition: MapCard's own effect, which closes the hub once the
    // ticket leaves `proposed`, exempts whichever ticket this holds a result for.
    approved?: ApproveResult | null
  } = $props()

  let review = $state<ReviewRead | null>(null)
  let loadError = $state<string | null>(null)
  let raw = $state(false)
  let acknowledged = $state(false)
  let busy = $state(false)
  let actionError = $state<string | null>(null)

  // The hub is opened per ticket; a fresh open is a fresh read of the brief off
  // disk, and never inherits the previous ticket's acknowledgement. Guarded by a
  // value key, not merely re-run on every prop identity change: `map`/`ticket`
  // get a fresh object reference on *every* pushed snapshot, including the one
  // approve's own rebuild triggers, and reloading then would wipe `approved` out
  // from under the strip it is about to render (ticket 17).
  let token = 0
  let lastKey: string | null = null
  $effect(() => {
    const num = ticket.num
    const slug = map.slug
    const id = spaceId
    const key = `${id} ${slug} ${num}`
    if (key === lastKey) return
    lastKey = key
    const mine = ++token
    review = null
    loadError = null
    acknowledged = false
    approved = null
    readReview(id, slug, num)
      .then((r) => {
        if (mine === token) review = r
      })
      .catch((e) => {
        if (mine === token) loadError = e instanceof ActionError ? e.message : (e as Error).message
      })
  })

  const rejected = $derived(review?.recommendation === 'Send back')
  // Exactly one tick, and only over a rejection (story 56): a pass approves in one
  // click, and no amount of advisory prose adds friction.
  const needsAck = $derived(rejected && (review?.blocking.length ?? 0) > 0)
  const canApprove = $derived(!!review && !busy && (!needsAck || acknowledged))

  // Pulled out of the brief on disk rather than re-derived here, so what the hub
  // shows and what a CLI reader sees cannot drift.
  const observed = $derived(sectionOf(review?.brief ?? '', ['Observed models']))
  const doneWhen = $derived(sectionOf(review?.brief ?? '', ['Done-when assessment']))
  const advisoriesText = $derived(sectionOf(review?.brief ?? '', ['Advisories']))

  // The recommendation line, in the harness's words. It is a function of the
  // verdict's anchoring — never the agent's prose about what to do.
  const recommendation = $derived.by(() => {
    if (!review) return ''
    if (rejected) {
      const clause = review.blocking[0]?.clause
      return (
        'Send back to fix — the blocking finding breaks the Done-when clause' +
        (clause ? ` “${clause}”` : '') +
        '. One follow-up session should clear it.'
      )
    }
    const n = review.advisories.length
    return `Ready to approve — nothing blocks against a Done-when clause.${
      n ? ` The ${n} advisory note${n > 1 ? 's' : ''} can ride along or wait.` : ''
    }`
  })

  // --- The diff, behind its expander, at three scopes (story 58). ---------------
  let diffOpen = $state(false)
  let verdictOpen = $state(false)
  let scope = $state<DiffScope>('all')
  let diff = $state<TicketDiff | null>(null)
  let diffError = $state<string | null>(null)

  // "Since your last read" is the operator's own bookmark: the head they last
  // looked at, remembered per ticket in this browser. It is a reading aid, never
  // state the gate depends on, so localStorage is the right home for it.
  const readKey = $derived(`wf.lastRead.${spaceId}.${map.slug}.${ticket.num}`)
  function lastRead(): string | undefined {
    try {
      return localStorage.getItem(readKey) ?? undefined
    } catch {
      return undefined
    }
  }
  function rememberRead(head: string) {
    try {
      localStorage.setItem(readKey, head)
    } catch {
      // A browser with storage denied simply loses the bookmark; the other two
      // scopes are anchored in git and still work.
    }
  }

  let diffToken = 0
  $effect(() => {
    if (!diffOpen) return
    const sc = scope
    const num = ticket.num
    const mine = ++diffToken
    diffError = null
    ticketDiff(spaceId, map.slug, num, sc, sc === 'read' ? lastRead() : undefined)
      .then((d) => {
        if (mine !== diffToken) return
        diff = d
        if (d.head) rememberRead(d.head)
      })
      .catch((e) => {
        if (mine !== diffToken) return
        diffError = e instanceof ActionError ? e.message : (e as Error).message
      })
  })

  // A finding names where it bites in prose; when that prose carries a path the
  // diff also shows, the hub jumps to it. A finding with no recognisable path
  // simply opens the diff — the brief is the record, not this convenience.
  function pathIn(text: string): string | null {
    const m = text.match(/[\w./-]+\.[a-zA-Z]{1,5}\b/)
    return m ? m[0] : null
  }
  function jumpToDiff(text: string) {
    diffOpen = true
    const want = pathIn(text)
    requestAnimationFrame(() => {
      const el = want
        ? Array.from(document.querySelectorAll<HTMLElement>('[data-diff-file]')).find((n) =>
            (n.dataset.diffFile ?? '').includes(want),
          )
        : null
      ;(el ?? document.querySelector('[data-diff-root]'))?.scrollIntoView({
        block: 'center',
        behavior: 'smooth',
      })
    })
  }

  // Split the unified patch into per-file blocks so a filename is a jump target
  // and the additions/deletions carry their `+`/`-` prefix as well as their tint.
  const diffFiles = $derived.by(() => {
    const out: { path: string; lines: string[] }[] = []
    for (const line of (diff?.patch ?? '').split('\n')) {
      if (line.startsWith('diff --git ')) {
        const m = line.match(/ b\/(.+)$/)
        out.push({ path: m ? m[1] : line, lines: [] })
      } else if (out.length) {
        out[out.length - 1].lines.push(line)
      }
    }
    return out
  })

  // --- The four exits -----------------------------------------------------------
  type DialogKind = 'sendback' | 'further' | 'abandon'
  let dialog = $state<DialogKind | null>(null)
  let note = $state('')
  let pickedAdvisories = $state<number[]>([])
  let furtherRole = $state('implement')
  let reason = $state('')
  let revert = $state(false)
  let reset = $state(false)

  // The post-approve strip replaces the hub once the gate is passed (story 61).
  // `approved` itself is the bindable prop above, lifted into MapCard.
  let nextEnabled = $state(false)
  let spawningNext = $state(false)
  $effect(() => {
    if (!approved) return
    nextEnabled = false
    // The suggestion is offered, never shoved: the spawn button cannot inherit the
    // approve click that just landed under it.
    const timer = setTimeout(() => (nextEnabled = true), 450)
    return () => clearTimeout(timer)
  })

  async function run<T>(fn: () => Promise<T>): Promise<T | null> {
    busy = true
    actionError = null
    try {
      return await fn()
    } catch (e) {
      actionError = e instanceof ActionError ? e.message : (e as Error).message
      return null
    } finally {
      busy = false
    }
  }

  async function approve() {
    const res = await run(() => approveTicket(spaceId, map.slug, ticket.num, acknowledged))
    if (res) approved = res
  }

  function defaultRoleFor(type: string, offered: Role[]): Role {
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

  // The strip's Next button (story 61): spawn the suggestion straight from here
  // when it is takeable, or fall back to selecting it so the click is never a
  // no-op — a blocked spawn (a live session already running, a missing binding,
  // the ticket having moved on) is exactly when the operator needs to land on the
  // ticket to see why, not watch the button do nothing (ticket 17).
  async function spawnNext() {
    if (!approved?.next || spawningNext) return
    const num = approved.next.num
    const target = map.tickets.find((t) => t.num === num)
    const roles = target?.frontier ? rolesForKind(map.kind) : []
    if (target && roles.length) {
      spawningNext = true
      try {
        const res = await spawnSession(spaceId, map.slug, num, defaultRoleFor(target.type, roles))
        onspawned?.(res.sessionId)
        onclose()
        return
      } catch {
        // Blocked — fall through to navigating there instead.
      } finally {
        spawningNext = false
      }
    }
    onselect?.(num)
    onclose()
  }

  async function sendBack() {
    const res = await run(() =>
      followUp(spaceId, map.slug, ticket.num, {
        role: 'implement',
        note,
        advisories: pickedAdvisories,
      }),
    )
    if (res) {
      dialog = null
      note = ''
      pickedAdvisories = []
      onspawned?.(res.sessionId)
      onclose()
    }
  }

  async function takeFurther() {
    const res = await run(() =>
      followUp(spaceId, map.slug, ticket.num, {
        role: furtherRole,
        note,
        includeFindings: true,
      }),
    )
    if (res) {
      dialog = null
      note = ''
      onspawned?.(res.sessionId)
      onclose()
    }
  }

  async function abandon() {
    const res = await run(() => abandonTicket(spaceId, map.slug, ticket.num, { reason, revert, reset }))
    if (res) {
      dialog = null
      onclose()
    }
  }

  // revert and reset are alternatives, never both (the backend refuses the
  // combination) — ticking one clears the other rather than offering a radio
  // group the vendored primitives don't have.
  function pickRevert(v: boolean) {
    revert = v
    if (v) reset = false
  }
  function pickReset(v: boolean) {
    reset = v
    if (v) revert = false
  }

  function toggleAdvisory(i: number) {
    pickedAdvisories = pickedAdvisories.includes(i)
      ? pickedAdvisories.filter((x) => x !== i)
      : [...pickedAdvisories, i]
  }

  function pad(n: number): string {
    return n < 10 ? '0' + n : String(n)
  }
</script>

<!-- The hub takes over the map card: it covers the island rather than docking
     beside it, because at the gate the map is context and the brief is the work. -->
<section
  class="absolute inset-0 z-30 flex min-h-0 flex-col bg-card"
  aria-label="Review hub"
>
  <header class="flex items-start gap-2 border-b border-border px-3 py-2.5">
    <div class="flex min-w-0 flex-1 flex-col gap-1">
      <span class="flex flex-wrap items-center gap-1.5 text-[0.7rem] text-muted-foreground">
        <Badge variant="secondary">proposed</Badge>
        <span class="font-mono">#{pad(ticket.num)}</span>
        <span aria-hidden="true">·</span>
        <span>human review</span>
      </span>
      <span class="truncate text-sm font-medium">{ticket.title}</span>
      {#if observed}
        <span class="text-[0.7rem] text-muted-foreground">{observed}</span>
      {/if}
    </div>
    <Button
      variant={raw ? 'secondary' : 'ghost'}
      size="xs"
      aria-pressed={raw}
      title="The same brief, as the markdown the harness wrote to disk"
      onclick={() => (raw = !raw)}>{'{ } raw'}</Button
    >
    <Button variant="ghost" size="icon-sm" aria-label="Close the hub (Esc)" title="Close (Esc)" onclick={onclose}>
      <X />
    </Button>
  </header>

  <ScrollArea.Root class="min-h-0 flex-1">
    <div class="flex flex-col gap-4 p-3">
      {#if loadError}
        <p class="flex items-start gap-1.5 text-xs text-muted-foreground">
          <Warning class="mt-0.5 size-3.5 shrink-0" />
          <span>{loadError}</span>
        </p>
      {:else if !review}
        <p class="text-sm text-muted-foreground">Reading the brief…</p>
      {:else if approved}
        <!-- The post-approve strip: what was promoted, what it unblocked, and the
             next best frontier ticket as a suggestion (story 61). -->
        <div class="flex flex-col gap-2 rounded-md border border-border p-3">
          <p class="flex items-start gap-1.5 text-sm">
            <Check class="mt-0.5 size-4 shrink-0" />
            <span>
              #{pad(ticket.num)} resolved — the proposed answer was promoted
              {#if approved.commit}<code class="font-mono text-xs">({approved.commit.slice(0, 8)})</code>{/if}{#if approved.approvedOverRejection}, over a rejecting verdict you acknowledged{/if}.
            </span>
          </p>
          {#if approved.warning}
            <p class="flex items-start gap-1.5 text-xs text-muted-foreground">
              <Warning class="mt-0.5 size-3.5 shrink-0" /><span>{approved.warning}</span>
            </p>
          {/if}
          <p class="text-xs text-muted-foreground">
            {#if approved.unblocked?.length}
              Unblocked {approved.unblocked.map((n) => '#' + pad(n)).join(', ')} — approval is the act
              that lets work compound.
            {:else}
              Nothing newly unblocked; the frontier already had open tickets.
            {/if}
          </p>
          <div class="flex flex-wrap items-center gap-2">
            {#if approved.next}
              <Button
                size="sm"
                disabled={!nextEnabled || spawningNext}
                title={nextEnabled ? '' : 'Offered, never shoved — a moment so this cannot inherit the approve click'}
                onclick={spawnNext}
              >
                <Rocket /> Next: #{pad(approved.next.num)}
                {approved.next.title}
              </Button>
            {/if}
            <Button variant="ghost" size="sm" onclick={onclose}>Not now — Esc</Button>
          </div>
        </div>
      {:else if raw}
        <!-- TUI parity: the exact file, so a CLI-only operator and this hub read
             the same words. -->
        <pre
          class="overflow-x-auto rounded-md bg-muted p-2.5 font-mono text-[0.7rem] leading-relaxed whitespace-pre-wrap">{review.brief}</pre>
      {:else}
        {#if rejected}
          <!-- The forced-arrival banner: a rejection halts here and never loops. -->
          <p class="rounded-md border border-destructive/50 bg-destructive/10 px-2.5 py-2 text-xs leading-relaxed">
            <strong class="font-semibold">The reviewer rejected this.</strong> Rejection never loops back
            to the implementer — it halts here. Every exit below is still yours.
          </p>
        {/if}

        <section class="flex flex-col gap-1.5">
          <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">What was done</h3>
          <div class="prose-sm">{@html renderMarkdown(review.proposedAnswer)}</div>
        </section>

        <section class="flex flex-col gap-1.5">
          <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">
            What the reviewer found
          </h3>
          <p class="text-xs">
            {#if rejected}
              <strong class="font-semibold">Rejected</strong> — {review.blocking.length} blocking finding{review
                .blocking.length > 1
                ? 's'
                : ''}, {review.advisories.length} advisory.
            {:else}
              <strong class="font-semibold">Passed</strong> — no finding cites a Done-when clause it breaks;
              {review.advisories.length} advisory note{review.advisories.length === 1 ? '' : 's'}.
            {/if}
          </p>
          {#each review.blocking as f (f.text)}
            <div class="rounded-md border border-destructive/50 p-2.5 text-xs leading-relaxed">
              <span>{f.text}</span>
              <Button variant="ghost" size="xs" class="mt-1" onclick={() => jumpToDiff(f.text)}>
                see it in the diff
              </Button>
            </div>
          {/each}

          <!-- Everything else is one click deeper. -->
          <div class="rounded-md border border-border">
            <button
              type="button"
              class="flex w-full items-center gap-1.5 px-2.5 py-1.5 text-left text-xs text-muted-foreground"
              aria-expanded={verdictOpen}
              onclick={() => (verdictOpen = !verdictOpen)}
            >
              {#if verdictOpen}<CaretDown class="size-3.5" />{:else}<CaretRight class="size-3.5" />{/if}
              Full verdict — the Done-when check and all findings
            </button>
            {#if verdictOpen}
              <div class="border-t border-border p-2.5">
                {#if review.verdictLine}
                  <p class="mb-1.5 text-xs text-muted-foreground">Reviewer's line: {review.verdictLine}</p>
                {/if}
                {#if doneWhen}
                  <div class="prose-sm">{@html renderMarkdown(doneWhen)}</div>
                {/if}
                {#if advisoriesText}
                  <h4 class="mt-2 text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">
                    Advisories
                  </h4>
                  <div class="prose-sm">{@html renderMarkdown(advisoriesText)}</div>
                {/if}
              </div>
            {/if}
          </div>
        </section>

        <section class="flex flex-col gap-1.5">
          <h3 class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase">Recommended</h3>
          <p class="text-xs leading-relaxed">{recommendation}</p>
        </section>

        <!-- The evidence, collapsed. -->
        <section class="flex flex-col gap-1.5" data-diff-root>
          <div class="rounded-md border border-border">
            <button
              type="button"
              class="flex w-full items-center gap-1.5 px-2.5 py-1.5 text-left text-xs text-muted-foreground"
              aria-expanded={diffOpen}
              onclick={() => (diffOpen = !diffOpen)}
            >
              {#if diffOpen}<CaretDown class="size-3.5" />{:else}<CaretRight class="size-3.5" />{/if}
              Diff — the work under this proposal
            </button>
            {#if diffOpen}
              <div class="border-t border-border p-2.5">
                <div class="mb-2 flex flex-wrap items-center gap-1.5" role="group" aria-label="Diff scope">
                  <span class="text-[0.7rem] text-muted-foreground">scope:</span>
                  {#each [['all', 'all commits'], ['verdict', 'since the verdict'], ['read', 'since your last read']] as [id, label] (id)}
                    <Button
                      variant={scope === id ? 'secondary' : 'ghost'}
                      size="xs"
                      aria-pressed={scope === id}
                      onclick={() => (scope = id as DiffScope)}>{label}</Button
                    >
                  {/each}
                </div>
                {#if diffError}
                  <p class="text-xs text-destructive">{diffError}</p>
                {:else if diff?.note}
                  <p class="text-xs text-muted-foreground">{diff.note}</p>
                {:else if diff && !diffFiles.length}
                  <p class="text-xs text-muted-foreground">Nothing changed in this scope.</p>
                {:else if diff}
                  <p class="mb-1.5 font-mono text-[0.7rem] whitespace-pre-wrap text-muted-foreground">{diff.stat}</p>
                  {#each diffFiles as f (f.path)}
                    <div class="mb-2 overflow-hidden rounded-md border border-border" data-diff-file={f.path}>
                      <div class="border-b border-border bg-muted px-2 py-1 font-mono text-[0.7rem]">{f.path}</div>
                      <div class="overflow-x-auto">
                        {#each f.lines as line}
                          <div
                            class={[
                              'px-2 font-mono text-[0.7rem] leading-relaxed whitespace-pre',
                              line.startsWith('+') && !line.startsWith('+++') && 'bg-primary/10',
                              line.startsWith('-') && !line.startsWith('---') && 'bg-destructive/10 text-destructive',
                            ]}
                          >{line || ' '}</div>
                        {/each}
                      </div>
                    </div>
                  {/each}
                {:else}
                  <p class="text-xs text-muted-foreground">Reading the diff…</p>
                {/if}
              </div>
            {/if}
          </div>
        </section>

        <!-- The exits. Buttons name their outcome. -->
        <section class="flex flex-col gap-2 rounded-md border border-border p-2.5">
          <div class="flex flex-wrap items-center gap-1.5">
            {#if rejected}
              <Button size="sm" disabled={busy} onclick={() => (dialog = 'sendback')}>Send back to fix…</Button>
            {/if}
            <Button
              size="sm"
              variant={rejected ? 'outline' : 'default'}
              disabled={!canApprove}
              title={canApprove ? '' : 'Acknowledge the blocking finding first'}
              onclick={approve}
            >
              <Check /> Approve — resolve #{pad(ticket.num)}
            </Button>
            {#if !rejected}
              <Button variant="outline" size="sm" disabled={busy} onclick={() => (dialog = 'further')}>
                Take it further…
              </Button>
            {/if}
            <Button variant="ghost" size="sm" disabled={busy} onclick={() => (dialog = 'abandon')}>Abandon…</Button>
          </div>

          {#if needsAck}
            <label class="flex items-center gap-2 text-xs">
              <Checkbox bind:checked={acknowledged} aria-label="I've read the blocking finding" />
              <span>I've read the blocking finding</span>
            </label>
          {/if}

          {#if actionError}
            <p class="flex items-start gap-1.5 text-[0.7rem] text-destructive">
              <Warning class="mt-0.5 size-3.5 shrink-0" /><span>{actionError}</span>
            </p>
          {/if}

          <p class="text-[0.7rem] leading-relaxed text-muted-foreground">
            Approve promotes <code class="font-mono">## Proposed Answer</code> →
            <code class="font-mono">## Answer</code> as its own commit, never an amend. Esc walks away — the
            gate keeps waiting. This brief is plain markdown on disk; the buttons are all the GUI adds.
          </p>
        </section>
      {/if}
    </div>
  </ScrollArea.Root>
</section>

<!-- Send back: the human's feedback channel. The dialog shows exactly what the
     fix-up session will be briefed with — the standard bundle, the blocking
     finding always, advisories as opt-in ticks — plus an optional note that rides
     the payload and its archive, never the ticket file (story 59). -->
<Modal open={dialog === 'sendback'} title={`Send #${pad(ticket.num)} back to fix`} onClose={() => (dialog = null)}>
  <div class="flex flex-col gap-3 text-xs">
    <p class="leading-relaxed text-muted-foreground">
      Spawns a fix-up session against this same ticket. Its commits accumulate on the proposal and it comes
      back to this hub — the ticket stays <strong class="font-medium text-foreground">proposed</strong> throughout.
    </p>
    <div class="flex flex-col gap-1.5 rounded-md border border-border p-2.5">
      <span class="font-medium">The session will be briefed with:</span>
      <span class="text-muted-foreground">· the ticket, its Done-when and the spec — the standard bundle</span>
      {#each review?.blocking ?? [] as f (f.text)}
        <span class="text-muted-foreground">· the blocking finding — {f.text}</span>
      {/each}
      {#each review?.advisories ?? [] as f, i (f.text)}
        <label class="flex items-start gap-2">
          <Checkbox
            checked={pickedAdvisories.includes(i)}
            onCheckedChange={() => toggleAdvisory(i)}
            aria-label="Include this advisory"
          />
          <span class="text-muted-foreground">also include the advisory — {f.text}</span>
        </label>
      {/each}
    </div>
    <label class="flex flex-col gap-1">
      <span class="font-medium">Anything to add?</span>
      <textarea
        bind:value={note}
        rows="3"
        placeholder="Steer the fix beyond the findings — optional."
        class="rounded-md border border-border bg-background p-2 text-xs"
      ></textarea>
    </label>
    <p class="text-[0.7rem] text-muted-foreground">
      Your note travels in the injected payload and its archive — not in the ticket file. Only abandonment
      writes there.
    </p>
    <div class="flex justify-end gap-1.5">
      <Button variant="ghost" size="sm" onclick={() => (dialog = null)}>Cancel</Button>
      <Button size="sm" disabled={busy} onclick={sendBack}><Rocket /> Spawn fix-up session</Button>
    </div>
  </div>
</Modal>

<!-- Take it further: the same stacking, without a rejection behind it. -->
<Modal open={dialog === 'further'} title={`Take #${pad(ticket.num)} further`} onClose={() => (dialog = null)}>
  <div class="flex flex-col gap-3 text-xs">
    <p class="leading-relaxed text-muted-foreground">
      Spawns another session against this same ticket. Its commits accumulate on the proposal, the
      <code class="font-mono">## Proposed Answer</code> is rewritten in place (priors live in git), and it comes
      back to this hub.
    </p>
    <div class="flex flex-wrap gap-1.5" role="group" aria-label="Follow-up role">
      {#each ['implement', 'review'] as r (r)}
        <Button
          variant={furtherRole === r ? 'default' : 'outline'}
          size="xs"
          aria-pressed={furtherRole === r}
          onclick={() => (furtherRole = r)}>{r}</Button
        >
      {/each}
    </div>
    <label class="flex flex-col gap-1">
      <span class="font-medium">A note for the session</span>
      <textarea
        bind:value={note}
        rows="3"
        placeholder="What should it push on? — optional."
        class="rounded-md border border-border bg-background p-2 text-xs"
      ></textarea>
    </label>
    <div class="flex justify-end gap-1.5">
      <Button variant="ghost" size="sm" onclick={() => (dialog = null)}>Cancel</Button>
      <Button size="sm" disabled={busy} onclick={takeFurther}><Rocket /> Spawn follow-up session</Button>
    </div>
  </div>
</Modal>

<!-- Abandon: one demand — a reason addressed to the next attempt — and nothing
     destroyed unless a lever is ticked (story 60). -->
<Modal open={dialog === 'abandon'} title={`Abandon #${pad(ticket.num)}`} onClose={() => (dialog = null)}>
  <div class="flex flex-col gap-3 text-xs">
    <p class="leading-relaxed text-muted-foreground">
      Abandon rejects the proposal, not the ticket. It goes back on the frontier for another attempt — armed
      with your reason.
    </p>
    <label class="flex flex-col gap-1">
      <span class="font-medium">Why this fails</span>
      <textarea
        bind:value={reason}
        rows="4"
        placeholder="Written to the next attempt, not to a log. The next session's context bundle carries this verbatim."
        class="rounded-md border border-border bg-background p-2 text-xs"
      ></textarea>
    </label>
    <div class="flex flex-col gap-1 rounded-md border border-border p-2.5 text-muted-foreground">
      <span
        ><strong class="font-medium text-foreground">Will do:</strong> demote
        <code class="font-mono">## Proposed Answer</code> to a dated
        <code class="font-mono">### Rejected</code> section with your reason, as one pathspec-limited commit; #{pad(
          ticket.num,
        )} derives open again.</span
      >
      <span
        ><strong class="font-medium text-foreground">Won't do:</strong> touch the work commits — undoing history
        is yours, below or later in your own terminal.</span
      >
    </div>
    <label class="flex items-start gap-2">
      <Checkbox checked={revert} onCheckedChange={(v) => pickRevert(!!v)} aria-label="Also revert the work commits" />
      <span class="text-muted-foreground">
        Also revert the work commits now — <em>a rejected attempt left in history is a truthful history</em>.
      </span>
    </label>
    {#if review?.resetAvailable}
      <!-- Offered only while the work is verifiably the tip of a clean tree — the
           same guarantee the backend enforces (ticket 17), so this never promises
           what abandon would then refuse. Revert and reset are alternatives. -->
      <label class="flex items-start gap-2">
        <Checkbox checked={reset} onCheckedChange={(v) => pickReset(!!v)} aria-label="Reset to before the work" />
        <span class="text-muted-foreground">
          Reset to before the work instead — <em>discards the commits outright</em> rather than reverting them;
          only offered while nothing else has landed on top.
        </span>
      </label>
    {/if}
    {#if actionError}
      <p class="flex items-start gap-1.5 text-[0.7rem] text-destructive">
        <Warning class="mt-0.5 size-3.5 shrink-0" /><span>{actionError}</span>
      </p>
    {/if}
    <div class="flex justify-end gap-1.5">
      <Button variant="ghost" size="sm" onclick={() => (dialog = null)}>Keep reviewing</Button>
      <Button variant="destructive" size="sm" disabled={busy || !reason.trim()} onclick={abandon}>
        <ArrowUUpLeft /> Abandon — demote & return to the frontier
      </Button>
    </div>
  </div>
</Modal>
