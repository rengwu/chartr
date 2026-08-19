<script lang="ts">
  import {
    defaultRole,
    heldLive,
    sinceLabel,
    ROLES,
    type Agent,
    type Map as WMap,
    type Role,
    type Terminal,
    type Ticket,
  } from "./model";
  import { renderMarkdown, sectionOf } from "./markdown";
  import {
    spawnSession,
    setSpaceAgent,
    releaseTicket,
    ActionError,
    LIVE_SESSION,
  } from "./actions";
  import { chooseAgent, type AgentChoice } from "./agentchoice";
  import PayloadPreview from "./PayloadPreview.svelte";
  import AgentSelector from "./AgentSelector.svelte";
  import Modal from "./Modal.svelte";
  import * as Accordion from "$lib/components/ui/accordion";
  import * as Card from "$lib/components/ui/card";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as ScrollArea from "$lib/components/ui/scroll-area";
  import { Badge, type BadgeVariant } from "$lib/components/ui/badge";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import {
    ArrowUUpLeft,
    Compass,
    Eye,
    X,
    Rocket,
    Warning,
    CaretDown,
  } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The detail pane (ticket 07): from looking at a star to reading it in one
  // click. It renders one ticket — question, Done-when, and its blockers with
  // their answers inline — or, from the map's title, the map's own material.
  // Session management is left to the operator: the pane spawns sessions but keeps
  // no history of them. Content is assembled from the derived model (the inlined
  // bodies) so
  // the pane needs no second fetch. Whether it docks right or bottom is the
  // parent's responsive decision; this is only the content.
  let {
    map,
    ticket = null,
    dock = "right",
    spaceId,
    lastAgent,
    agents,
    terminals = [],
    onclose,
    onRegisterAgent,
    onspawned,
  }: {
    map: WMap;
    ticket?: Ticket | null;
    dock?: "right" | "bottom";
    // The space the ticket belongs to — the key the payload preview fetches by.
    spaceId?: string;
    // The space's open tabs — read only to tell whether a live session is working
    // this ticket, which is what separates a claim being honoured from one left
    // behind by a session that is gone.
    terminals?: Terminal[];
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

  // Release: the one way back off a claimed ticket. A claim keeps the ticket off
  // the frontier, so a session that died without answering — or one whose tab this
  // chartr lost to a restart — leaves the ticket held by nobody and takeable by
  // nobody. The dead tab's own release covers it only while that tab exists; this
  // one is addressed to the ticket, so it survives the tab.
  //
  // It is offered while no *live* session holds the ticket. A dead pinned tab is
  // deliberately included: both its own release and this one clear the same claim,
  // and the operator should not have to find the tab to do it.
  const claimHeldLive = $derived(
    !!ticket && heldLive(terminals, map.slug, ticket.num),
  );
  const canRelease = $derived(
    !!spaceId && ticket?.status === "claimed" && !claimHeldLive,
  );
  // The claim line, shown on any claimed ticket: how long it has been held, and
  // whether anything here is actually holding it. An old claim with no session is
  // precisely the stuck ticket, and naming it is what makes the button findable.
  const claimAge = $derived(sinceLabel(ticket?.claimedAt));
  let releasing = $state(false);

  // The role the ticket's own type points at is the emphasised action, surfaced
  // as the one plain button; the rest live behind "More" so the footer reads as a
  // single obvious move, not a spread of equal buttons.
  const preferredRole = $derived<Role | null>(
    canSpawn && ticket ? defaultRole(ticket.type) : null,
  );
  const otherRoles = $derived<Role[]>(
    spawnRoles.filter((r) => r !== preferredRole),
  );

  // The agent every action here spawns with (ticket 02): a per-space choice, not a
  // per-role one, so it lives on one quiet selector rather than a caret on each
  // action. `agentOverride` is the operator's pick this session; until they pick,
  // the space's remembered `lastAgent` stands. A successful spawn is what the
  // server remembers, so the pick needs no separate persist step.
  let agentOverride = $state<string | undefined>(undefined);
  const agentChoice = $derived<AgentChoice>(
    chooseAgent(agents, agentOverride ?? lastAgent),
  );
  // No agent resolved (nothing chosen, or nothing registered) means no one-click
  // path may spawn — the selector is the deliberate first choice, and the actions
  // wait on it. `actAgent` is the name they spawn with, empty only while disabled.
  const canAct = $derived(agentChoice.kind === "ready");
  const actAgent = $derived(
    agentChoice.kind === "ready" ? agentChoice.agent.name : "",
  );

  // The role currently being spawned — labels its own button and disables the row,
  // so two clicks can't race two sessions onto one ticket.
  let spawningRole = $state<Role | null>(null);
  // One error line for the footer, shared by every action on it: whichever the
  // operator just took is the one whose refusal they are reading.
  let actionError = $state<string | null>(null);

  // A single DetailPane instance is reused as the selection changes ticket, so a
  // block message the operator saw on one ticket must not linger onto the next —
  // and neither must an unanswered concurrency warning, which names a role on the
  // ticket that raised it and would otherwise confirm a spawn onto a different one.
  let lastNum: number | undefined = undefined;
  $effect(() => {
    const n = ticket?.num;
    if (n !== lastNum) {
      lastNum = n;
      actionError = null;
      pendingSpawn = null;
    }
  });

  // The agent override is a per-space pick; when the pane is reused for a different
  // space its remembered agent (`lastAgent`) takes over, so a stale override from
  // the previous space must not shadow it.
  let lastSpace: string | undefined = undefined;
  $effect(() => {
    if (spaceId !== lastSpace) {
      lastSpace = spaceId;
      agentOverride = undefined;
    }
  });

  // The spawn the operator has been warned about and not yet answered: the space
  // already has a live session, and this is the {role, agent} they asked for
  // (ADR 0003 as amended). Held rather than retried immediately, because the
  // decision — two agents in one working tree — is only the operator's to make.
  let pendingSpawn = $state<{ role: Role; agent: string } | null>(null);

  async function spawn(role: Role, agent: string, force = false) {
    if (!spaceId || !ticket || spawningRole) return;
    spawningRole = role;
    actionError = null;
    try {
      const res = await spawnSession(
        spaceId,
        map.slug,
        ticket.num,
        role,
        agent,
        force,
      );
      pendingSpawn = null;
      onspawned?.(res.sessionId);
    } catch (e) {
      // One refusal is the operator's to overrule — a space that already has a live
      // session — so it opens the warning rather than landing as an error. Every
      // other blocked spawn (absent agent, held ticket) is a refusal of fact and
      // carries chartr's specific message inline.
      if (e instanceof ActionError && e.code === LIVE_SESSION) {
        pendingSpawn = { role, agent };
      } else {
        actionError =
          e instanceof ActionError ? e.message : (e as Error).message;
      }
    } finally {
      spawningRole = null;
    }
  }

  // Release the claim, then let the snapshot say so: the ticket derives open again
  // off the commit chartr just made, and the spawn footer takes over on the very
  // same pane. No confirmation — the claim is one commit in the log either way, and
  // the only claim worth protecting (a live session's) the server refuses outright.
  async function release() {
    if (!spaceId || !ticket || releasing) return;
    releasing = true;
    actionError = null;
    try {
      await releaseTicket(spaceId, map.slug, ticket.num);
    } catch (e) {
      actionError = e instanceof ActionError ? e.message : (e as Error).message;
    } finally {
      releasing = false;
    }
  }

  // Persist the operator's agent pick the moment they make it, so it reads as a
  // saved setting rather than one that only sticks on the next spawn. The override
  // updates optimistically for instant feedback; the snapshot then echoes it back
  // as `lastAgent`. A failed persist surfaces on the same footer error line.
  async function rememberAgent(agent: string) {
    agentOverride = agent;
    if (!spaceId) return;
    actionError = null;
    try {
      await setSpaceAgent(spaceId, agent);
    } catch (e) {
      actionError = e instanceof ActionError ? e.message : (e as Error).message;
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
     card's radius and ring and takes a single border on the seam edge.

     Its surface is the card token over a backdrop blur, so the
     constellation stays faintly present behind the reading column rather than
     being cut away — the pane reads as laid over the map it came from. The camera
     still treats the footprint as taken (the parent's insets), so no star is left
     relying on that translucency to be seen. -->
<Card.Root
  role="complementary"
  aria-label={isMap ? "Map material" : "Ticket detail"}
  class={cn(
    "h-full min-h-0 flex-col gap-0 overflow-hidden rounded-none bg-card/80 py-0 ring-0 backdrop-blur-sm",
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
        {#if isMap}<Compass class="icon-size-md" />{:else if ticket}{pad(
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
      <!-- Only where a spawn is actually takeable. The payload is what a session
           on this ticket *would be told*, so on a ticket that offers no session to
           start it previews a briefing nobody can be given — a button whose answer
           is hypothetical. It rides the same condition as the footer's actions. -->
      {#if canSpawn}
        <Button
          variant="outline"
          size="sm"
          class="ml-auto shrink-0"
          title="Preview the payload a session on this ticket would be told"
          onclick={() => (showPreview = true)}
        >
          <Eye /> View Payload
        </Button>
      {/if}
    </div>

    <!-- The claim line: who holds this ticket, how long they have held it, and
         whether anything in this chartr is actually running it. A claim outlives
         the session that wrote it — the tab is in memory, the claim is a commit —
         so "no session here" is the ordinary reading after a restart, and the one
         that explains why an untouched ticket will not move. -->
    {#if !isMap && ticket?.status === "claimed"}
      <p class="flex items-center gap-1 text-[0.7rem] text-muted-foreground">
        <span class="truncate">
          claimed{claimAge ? ` ${claimAge}` : ""}{ticket.claimedBy
            ? ` · ${ticket.claimedBy}`
            : ""}
        </span>
        {#if !claimHeldLive}
          <span class="shrink-0 text-foreground">· no session here</span>
        {/if}
      </p>
    {/if}
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
                    class="min-w-0 items-center gap-1.5 p-2 text-xs hover:no-underline"
                  >
                    <span class="font-mono text-muted-foreground"
                      >#{pad(b.num)}</span
                    >
                    <span class="min-w-0 flex-1 truncate text-left font-medium"
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
      {/if}
    </Card.Content>
  </ScrollArea.Root>

  <!-- The action footer: every session this ticket can start, surfaced on one bar
       that the content scrolls under rather than buried at the end of the body.
       Two axes (ticket 02): the quiet selector on the left is *who* — the one
       agent every action spawns with — and the buttons on the right are *what*.
       The ticket's type picks the emphasised action; the rest live under "More".
       A claimed ticket has no spawn to offer — the claim is what holds it off the
       frontier — so the same bar carries the one action it does have: release. -->
  {#if canSpawn || canRelease}
    <div class="flex items-center gap-2 border-t border-border px-3 py-2">
      {#if canSpawn}
        <AgentSelector
          {agents}
          selected={agentOverride ?? lastAgent}
          onselect={rememberAgent}
          onregister={onRegisterAgent}
        />
      {/if}
      {#if actionError}
        <p
          class="flex min-w-0 items-start gap-1.5 text-[0.7rem] text-destructive"
          title={actionError}
        >
          <Warning class="mt-0.5 icon-size-md shrink-0" />
          <span class="truncate">{actionError}</span>
        </p>
      {/if}
      <span class="flex-1"></span>
      {#if canRelease}
        <Button
          variant="outline"
          size="sm"
          class="gap-1.5"
          disabled={releasing}
          title="Release — clear the claim so this ticket is takeable again"
          onclick={release}
        >
          <ArrowUUpLeft />
          {releasing ? "Releasing…" : "Release"}
        </Button>
      {/if}
      {#if preferredRole}
        <Button
          variant="default"
          size="sm"
          class="gap-1.5"
          disabled={!canAct || spawningRole !== null}
          title="Start a {preferredRole} session on #{ticket
            ? pad(ticket.num)
            : ''}"
          onclick={() => preferredRole && spawn(preferredRole, actAgent)}
        >
          <Rocket />
          {spawningRole === preferredRole
            ? "Starting…"
            : roleLabel(preferredRole)}
        </Button>
      {/if}
      {#if otherRoles.length}
        <DropdownMenu.Root>
          <DropdownMenu.Trigger
            class={cn(
              buttonVariants({ variant: "outline", size: "sm" }),
              "gap-1.5",
            )}
            disabled={!canAct || spawningRole !== null}
            title="Other sessions this ticket can start"
          >
            More <CaretDown class="icon-size-md" />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end" class="min-w-40">
            {#each otherRoles as r (r)}
              <DropdownMenu.Item onclick={() => spawn(r, actAgent)}>
                {roleLabel(r)}
              </DropdownMenu.Item>
            {/each}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      {/if}
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

<!-- The concurrency warning (ADR 0003 as amended). One session per space is the
     default because a space is one git working tree with no branch and no
     worktree behind it, so a second agent edits the same files as the first. That
     is a judgement only the operator can make — they know whether these two
     tickets touch the same code — so the gate warns and names the cost rather
     than refusing. Confirming re-sends the same spawn with `force`. -->
<Modal
  open={pendingSpawn !== null}
  title="This space already has a live session"
  onClose={() => (pendingSpawn = null)}
>
  <div class="space-y-4 text-sm">
    <p class="text-muted-foreground">
      Spawning a second session runs both agents in
      <strong class="font-medium text-foreground">the same working tree</strong
      >. There is no branch or worktree between them, so they can overwrite each
      other's uncommitted edits with no conflict to resolve.
    </p>
    <p class="text-muted-foreground">
      Safe when the two tickets touch different files. Risky when they don't.
    </p>
    <div class="flex justify-end gap-2">
      <Button variant="ghost" size="sm" onclick={() => (pendingSpawn = null)}
        >Cancel</Button
      >
      <Button
        variant="default"
        size="sm"
        disabled={spawningRole !== null}
        onclick={() => {
          const p = pendingSpawn;
          if (p) spawn(p.role, p.agent, true);
        }}
      >
        Spawn anyway
      </Button>
    </div>
  </div>
</Modal>
