<script lang="ts">
  import {
    defaultRole,
    ROLES,
    type Agent,
    type Map as WMap,
    type Role,
    type Ticket,
  } from "./model";
  import { renderMarkdown, sectionOf } from "./markdown";
  import { spawnSession, ActionError } from "./actions";
  import PayloadPreview from "./PayloadPreview.svelte";
  import AgentSplitButton from "./AgentSplitButton.svelte";
  import * as Accordion from "$lib/components/ui/accordion";
  import * as Card from "$lib/components/ui/card";
  import * as ScrollArea from "$lib/components/ui/scroll-area";
  import { Badge, type BadgeVariant } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Compass, Eye, X, Rocket, Warning } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The detail pane (ticket 07): from looking at a star to reading it in one
  // click. It renders one ticket — question, Done-when, its blockers with their
  // answers inline, and session history — or, from the map's title, the map's own
  // material. Content is assembled from the derived model (the inlined bodies) so
  // the pane needs no second fetch. Whether it docks right or bottom is the
  // parent's responsive decision; this is only the content.
  let {
    map,
    ticket = null,
    dock = "right",
    spaceId,
    lastAgent,
    agents,
    onclose,
    onRegisterAgent,
    onspawned,
  }: {
    map: WMap;
    ticket?: Ticket | null;
    dock?: "right" | "bottom";
    // The space the ticket belongs to — the key the payload preview fetches by.
    spaceId?: string;
    // The space's remembered agent and the global library (ticket 02): the spawn
    // buttons name and pick from these.
    lastAgent?: string;
    agents: Agent[];
    onclose: () => void;
    // Where the spawn control routes when the library is empty (ticket 04): agent
    // registration, rather than a dead button or a spawn the server would refuse.
    onRegisterAgent: () => void;
    // Called with the new session id after a successful spawn, so the enclosing
    // chrome can make that session's tab active.
    onspawned?: (sessionId: string) => void;
  } = $props();

  const isMap = $derived(ticket === null);

  // The payload preview (ticket 08): from reading a ticket to seeing exactly what
  // a session on it would be told. Available only with a spaceId in hand.
  let showPreview = $state(false);

  // Spawn (tickets 09, 11): a frontier ticket offers a fresh session in any of
  // the four roles — the ticket's type picks the default, and the operator picks
  // from all of them at the gate. The frontier is the whole condition, so the
  // affordance appears exactly where a spawn is actually takeable.
  function offeredRoles(tk: Ticket | null): Role[] {
    return tk?.frontier ? [...ROLES] : [];
  }
  const spawnRoles = $derived<Role[]>(offeredRoles(ticket));
  const canSpawn = $derived(!!spaceId && spawnRoles.length > 0);

  // Every offered role is its own footer action — one click starts it, no
  // pick-then-confirm step. The role the ticket's own type points at is the
  // emphasised one, so the obvious move stays obvious.
  const preferredRole = $derived<Role | null>(
    canSpawn && ticket ? defaultRole(ticket.type) : null,
  );

  // The role currently being spawned — labels its own button and disables the row,
  // so two clicks can't race two sessions onto one ticket.
  let spawningRole = $state<Role | null>(null);
  let spawnError = $state<string | null>(null);

  // A single DetailPane instance is reused as the selection changes ticket, so a
  // block message the operator saw on one ticket must not linger onto the next.
  let lastNum: number | undefined = undefined;
  $effect(() => {
    const n = ticket?.num;
    if (n !== lastNum) {
      lastNum = n;
      spawnError = null;
    }
  });

  async function spawn(role: Role, agent: string) {
    if (!spaceId || !ticket || spawningRole) return;
    spawningRole = role;
    spawnError = null;
    try {
      const res = await spawnSession(spaceId, map.slug, ticket.num, role, agent);
      onspawned?.(res.sessionId);
    } catch (e) {
      // A blocked spawn (absent agent, held ticket) carries chartr's specific
      // message — surface it inline rather than as a silent no-op.
      spawnError = e instanceof ActionError ? e.message : (e as Error).message;
    } finally {
      spawningRole = null;
    }
  }

  function roleLabel(role: Role): string {
    return role.slice(0, 1).toUpperCase() + role.slice(1);
  }

  // The closing-answer section names, in the order a resolved/ruled-out ticket
  // carries them — used to show a blocker's answer inline. An in-flight
  // `## Proposed Answer` is deliberately absent: nothing blessed it, so it is
  // never shown as a blocker's answer.
  const ANSWER_SECTIONS = ["Answer", "Ruled out"];

  // A blocker resolved from the same map, with its answer pulled from its body.
  interface Blocker {
    num: number;
    title: string;
    status: string;
    answer: string;
  }
  const blockers = $derived.by<Blocker[]>(() => {
    if (!ticket?.blockedBy?.length) return [];
    return ticket.blockedBy.map((n) => {
      const b = map.tickets.find((t) => t.num === n);
      if (!b)
        return {
          num: n,
          title: "(missing ticket)",
          status: "unknown",
          answer: "",
        };
      return {
        num: n,
        title: b.title,
        status: b.status,
        answer: sectionOf(b.body ?? "", ANSWER_SECTIONS),
      };
    });
  });

  const statusLabel: Record<string, string> = {
    open: "open",
    claimed: "claimed",
    resolved: "resolved",
    out_of_scope: "out of scope",
    unknown: "missing",
  };

  // resolved reads as the bold/solid "done" state (the palette's only accent
  // besides destructive is the neutral --primary — there is no green to key a
  // literal success tint off); claimed takes the lighter --primary-adjacent
  // secondary emphasis the ticket calls for; out_of_scope stays muted;
  // an unresolved blocker reference is the one true "problem" and gets destructive.
  const statusVariant: Record<string, BadgeVariant> = {
    open: "outline",
    claimed: "secondary",
    resolved: "default",
    out_of_scope: "outline",
    unknown: "destructive",
  };

  function pad(n: number): string {
    return n < 10 ? "0" + n : String(n);
  }

  // The map body leads with its Destination heading; the pane shows that above,
  // so strip the duplicate section from the rendered body.
  function stripDestination(body: string): string {
    const lines = body.split("\n");
    let start = -1;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].trim() === "## Destination") {
        start = i;
        break;
      }
    }
    if (start < 0) return body;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i++) {
      if (/^##\s/.test(lines[i])) {
        end = i;
        break;
      }
    }
    return [...lines.slice(0, start), ...lines.slice(end)].join("\n").trim();
  }
</script>

<!-- The pane is a flush panel, not a floating card: it shares a seam with the map
     (the parent's drag border) rather than hovering inset over it, so it drops the
     card's radius and ring and takes a single border on the seam edge. -->
<Card.Root
  role="complementary"
  aria-label={isMap ? "Map material" : "Ticket detail"}
  class={cn(
    "h-full min-h-0 flex-col gap-0 overflow-hidden rounded-none py-0 ring-0",
    dock === "bottom"
      ? "border-t border-border"
      : "border-l border-border border-t",
  )}
>
  <!-- Two tiers. The identity line — the ticket's number as a struck coin, its
       title, and the way out — reads first and holds the full width, so a long
       title clips rather than wrapping the controls away. Its metadata (type,
       status) sits beneath, with the payload preview pushed to the far end: what
       this ticket *is* on the left, what you can look at on the right. -->
  <!-- items-stretch is load-bearing: Card.Header ships items-start, which in a
       flex *column* shrinks each row to its content — the title would then never
       meet an edge to ellipsis against, and the spacer below would collapse,
       un-pinning the payload button from the right. -->
  <!-- Card.Header ships `[.border-b]:pb-(--card-spacing)` — a two-class selector
       that outranks a plain `py-*`, so adding the rule below the header silently
       reinstated the card's full 1rem bottom padding. Retune the variable rather
       than fight the specificity: pb then matches py-2 on both edges. -->
  <Card.Header
    class="flex flex-col items-stretch gap-1.5 border-b border-border px-3 py-2 [--card-spacing:--spacing(2)]"
  >
    <div class="flex items-center gap-1">
      <span
        class="grid size-6 shrink-0 place-items-center rounded-full border border-border font-mono text-[0.65rem] text-muted-foreground"
        aria-hidden={isMap ? "true" : undefined}
      >
        {#if isMap}<Compass class="size-3.5" />{:else if ticket}{pad(
            ticket.num,
          )}{/if}
      </span>
      <!-- One line, always: a long title clips to an ellipsis rather than wrapping
           the close button onto a second row. The full text stays on the title
           attribute, so nothing is lost — just not spent on height. -->
      <span
        class="min-w-0 flex-1 overflow-hidden text-sm font-medium text-ellipsis whitespace-nowrap"
        title={isMap ? map.name : ticket?.title}
      >
        {isMap ? map.name : (ticket?.title ?? "")}
      </span>
      <Button
        variant="ghost"
        size="icon-sm"
        class="ml-auto shrink-0"
        aria-label="Close pane (Esc)"
        title="Close (Esc)"
        onclick={onclose}
      >
        <X />
      </Button>
    </div>

    <div class="flex items-center gap-1.5">
      {#if isMap}
        <span class="text-[0.7rem] text-muted-foreground">map material</span>
      {:else if ticket}
        <span class="truncate text-[0.7rem] text-muted-foreground"
          >{ticket.type}</span
        >
        <Badge
          variant={statusVariant[ticket.status] ?? "outline"}
          class={ticket.status === "out_of_scope"
            ? "text-muted-foreground"
            : ""}
        >
          {statusLabel[ticket.status] ?? ticket.status}
        </Badge>
        {#if ticket.frontier}
          <Badge variant="outline" class="border-primary/50 text-primary"
            >frontier</Badge
          >
        {/if}
      {/if}
      {#if !isMap && spaceId}
        <Button
          variant="outline"
          size="xs"
          class="ml-auto shrink-0"
          title="Preview the payload a session on this ticket would be told"
          onclick={() => (showPreview = true)}
        >
          <Eye /> View Payload
        </Button>
      {/if}
    </div>
  </Card.Header>

  <ScrollArea.Root class="min-h-0 flex-1">
    <Card.Content class="flex flex-col gap-4 p-3">
      {#if isMap}
        {#if map.destination}
          <section class="flex flex-col gap-1.5">
            <h3
              class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase"
            >
              Destination
            </h3>
            <div class="prose-sm">{@html renderMarkdown(map.destination)}</div>
          </section>
        {/if}
        <section>
          <div class="prose-sm">
            {@html renderMarkdown(stripDestination(map.body ?? ""))}
          </div>
        </section>
      {:else if ticket}
        <section>
          <div class="prose-sm">{@html renderMarkdown(ticket.body ?? "")}</div>
        </section>

        <section class="flex flex-col gap-1.5">
          <h3
            class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase"
          >
            Blockers
          </h3>
          {#if blockers.length === 0}
            <p class="text-xs text-muted-foreground">
              None — this ticket depends on nothing.
            </p>
          {:else}
            <!-- Each blocker collapses to its header row. A resolved blocker's
                 answer is full prose — several of them stacked buried the
                 ticket's own body, so the answers are opened on demand.
                 `multiple` because comparing two blockers is the common read,
                 and nothing is open by default. -->
            <Accordion.Root type="multiple" class="flex flex-col">
              {#each blockers as b (b.num)}
                <Accordion.Item value={String(b.num)}>
                  <!-- items-center overrides the primitive's items-start, which
                       in this row would top-align the badge against the caret;
                       no-underline keeps the hover off the title and badge. -->
                  <Accordion.Trigger
                    class="items-center gap-1.5 p-2 text-xs hover:no-underline"
                  >
                    <span class="font-mono text-muted-foreground"
                      >#{pad(b.num)}</span
                    >
                    <span class="flex-1 truncate text-left font-medium"
                      >{b.title}</span
                    >
                    <Badge variant={statusVariant[b.status] ?? "outline"}
                      >{statusLabel[b.status] ?? b.status}</Badge
                    >
                  </Accordion.Trigger>
                  <Accordion.Content class="pb-2">
                    {#if b.answer}
                      <div class="prose-sm">
                        {@html renderMarkdown(b.answer)}
                      </div>
                    {:else}
                      <p class="text-xs text-muted-foreground">
                        Not yet answered.
                      </p>
                    {/if}
                  </Accordion.Content>
                </Accordion.Item>
              {/each}
            </Accordion.Root>
          {/if}
        </section>

        <section class="flex flex-col gap-1.5">
          <h3
            class="text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase"
          >
            Session history
          </h3>
          <p class="text-xs text-muted-foreground">
            No sessions on this ticket yet.
          </p>
        </section>
      {/if}
    </Card.Content>
  </ScrollArea.Root>

  <!-- The action footer: every session this ticket can start, surfaced on one bar
       that the content scrolls under rather than buried at the end of the body.
       Each role is a split control (ticket 02): the primary action spawns with the
       space's remembered agent and names it, or opens the picker when nothing is
       remembered; the secondary opens the agent list for a one-off override. -->
  {#if canSpawn}
    <div class="flex items-center gap-2 border-t border-border px-3 py-2">
      {#if spawnError}
        <p
          class="flex min-w-0 items-start gap-1.5 text-[0.7rem] text-destructive"
          title={spawnError}
        >
          <Warning class="mt-0.5 size-3.5 shrink-0" />
          <span class="truncate">{spawnError}</span>
        </p>
      {/if}
      <span class="flex-1"></span>
      {#each spawnRoles as r (r)}
        <AgentSplitButton
          {agents}
          {lastAgent}
          label={roleLabel(r)}
          unchosenLabel="Start {roleLabel(r)}"
          busy={spawningRole === r}
          disabled={spawningRole !== null}
          variant={r === preferredRole ? "default" : "outline"}
          title="Start a {r} session on #{ticket ? pad(ticket.num) : ''}"
          onrun={(agent) => spawn(r, agent)}
          onregister={onRegisterAgent}
        >
          {#snippet icon()}
            {#if r === preferredRole}<Rocket />{/if}
          {/snippet}
        </AgentSplitButton>
      {/each}
    </div>
  {/if}
</Card.Root>

{#if !isMap && ticket && spaceId}
  <PayloadPreview
    open={showPreview}
    {spaceId}
    mapSlug={map.slug}
    ticketNum={ticket.num}
    ticketTitle={ticket.title}
    ticketType={ticket.type}
    {agents}
    {lastAgent}
    onClose={() => (showPreview = false)}
  />
{/if}
