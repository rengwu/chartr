<script lang="ts">
  import type { Agent, Space, Terminal } from "./model";
  import SkillLauncher from "./SkillLauncher.svelte";
  import { Button } from "./components/ui/button";
  import { spaceAttention, spaceLiveness } from "./attention";
  import {
    X,
    Check,
    XCircle,
    CircleNotch,
    Rocket,
    Play,
    ArrowClockwise,
    ArrowUUpLeft,
    Warning,
    PauseCircle,
    GitBranchIcon,
    Plus,
    PlusIcon,
  } from "phosphor-svelte";

  // The three choices a dead session offers. The card names the choice; the
  // chrome above owns which action each one runs, so the halt actions stay
  // beside every other action call rather than being imported here.
  type HaltVerb = "resume" | "respawn" | "release";

  let {
    space,
    selected,
    activeTermId,
    agents,
    opening = false,
    onselect,
    onselectsession,
    onjumphalt,
    onforget,
    onendshell,
    onhalt,
    onopenshell,
    onlaunch,
    onregister,
  }: {
    space: Space;
    // Whether this card is the chrome's current selection. Passed rather than
    // derived: selection is the shell's business, and a card only reports the
    // click that would change it.
    selected: boolean;
    // The shell that is actually showing, already resolved by the chrome (which
    // falls back to a space's first terminal when the active id is stale). A row
    // reads active only within the selected card, so a background space never
    // paints one.
    activeTermId: string | null;
    // The registered agent library — global, the same list whatever space is in
    // view — handed straight to the skill launcher's agent picker.
    agents: Agent[];
    // A launch or shell-open is in flight somewhere in the chrome; the on-ramps
    // disable while it is, so one click cannot start two tabs.
    opening?: boolean;
    onselect: () => void;
    onselectsession: (t: Terminal) => void;
    // The halt flag doubles as the jump: the chrome deep-links the halted ticket.
    onjumphalt: () => void;
    onforget: () => void;
    onendshell: (t: Terminal) => void;
    onhalt: (t: Terminal, verb: HaltVerb) => void;
    onopenshell: () => void;
    // The launch the skill launcher settled: the chosen agent, the skill, and the
    // optional one line a `needs-context` skill asked for.
    onlaunch: (agent: string, skill: string, context?: string) => void;
    // Where an empty agent library sends the operator: the registration surface.
    onregister: () => void;
  } = $props();

  // Zero-pad a ticket number for a session row's label (#01), matching the detail
  // pane's ticket ids.
  function pad(n: number): string {
    return n < 10 ? "0" + n : String(n);
  }

  const attention = $derived(spaceAttention(space));
  const liveness = $derived(spaceLiveness(space));

  // Copy-to-clipboard for the branch chip. The card sets `select-none` — it is a
  // click target, and drag-selecting inside one is noise — so this button is the
  // *only* way the branch name leaves the UI, not a shortcut alongside manual
  // selection. That is why the failure is shown rather than swallowed the way the
  // terminal's copy-on-select swallows it: there, a denied clipboard still leaves
  // you a selection to ⌘C; here it would leave nothing at all.
  // `navigator.clipboard` needs a secure context, so a harness opened over plain
  // http on a LAN address lands in exactly that case.
  let copied = $state<{ ok: boolean } | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  async function copyToClipboard(text: string) {
    let ok = false;
    try {
      await navigator.clipboard.writeText(text);
      ok = true;
    } catch {
      ok = false;
    }
    copied = { ok };
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copied = null;
    }, 1200);
  }
</script>

<!-- One space, a bordered container on the sidebar surface (its own
     token family — not the bg-card content surface). The whole card is
     the selection target — clicking anywhere that isn't its own control
     selects the space — so the identity, its sessions and its actions
     all read as one object rather than a header you must aim at.
     Selected emphasis rides --primary, the one emphasis token; the
     chrome is monochrome. Because the whole card is a click target,
     it is `select-none` throughout — name, sessions, branch. Dragging
     a selection across a thing you click is noise, and the branch —
     the one string actually worth lifting — has its own copy button
     instead. The path stays the card's tooltip. -->
<div
  role="button"
  tabindex="0"
  aria-pressed={selected}
  aria-label="Select {space.name}"
  title={space.path}
  class={[
    "flex cursor-pointer flex-col gap-2 rounded-lg border p-2 transition-colors select-none",
    selected
      ? "border-primary/60 bg-sidebar-accent/30"
      : "border-sidebar-border hover:border-primary/30",
  ]}
  onclick={onselect}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onselect();
    }
  }}
>
  <!-- Identity: the space's name, with the forget action pinned
       top-right (the branch rides the action row below). Ambient
       cross-space attention (ticket 14, story 8) rides on the name
       line — a wants-you flag (a session halted) and a liveness dot,
       both echoing the same signals the queue pulls and the session
       cards below already carry in detail. Neither ever re-sorts the
       card; muscle memory over this list holds. -->
  <div class="flex items-start gap-1">
    <span
      class="flex min-w-0 flex-1 items-center gap-1.5 text-xs font-semibold"
    >
      {#if attention === "halt"}
        <!-- The flag is also the jump: one click selects the space
             and deep-links its halted ticket. Inside a card that is
             itself role="button", so the handler stops propagation
             the way the forget action does. -->
        <Button
          variant="ghost"
          size="icon-xs"
          class="-my-0.5 shrink-0 text-destructive hover:text-destructive"
          aria-label="a session halted — go to the halted ticket"
          title="A session halted, needs a decision — go to it"
          onclick={(e) => {
            e.stopPropagation();
            onjumphalt();
          }}
          onkeydown={(e) => {
            // The card handles Enter/Space itself and preventDefaults
            // it; let the button's own activation win instead.
            if (e.key === "Enter" || e.key === " ") e.stopPropagation();
          }}
        >
          <Warning />
        </Button>
      {/if}
      {#if liveness === "working"}
        <CircleNotch
          class="size-3 shrink-0 animate-spin text-primary"
          aria-label="a session is working"
        />
      {:else if liveness === "blocked"}
        <PauseCircle
          class="size-3 shrink-0 text-primary"
          aria-label="a session is blocked"
        />
      {/if}
      <span class="truncate">{space.name}</span>
    </span>
    <Button
      variant="ghost"
      size="icon-xs"
      class="-mt-0.5 -mr-0.5 hover:text-destructive"
      aria-label="Remove space"
      title="Remove from this list (your files stay put)"
      onclick={(e) => {
        e.stopPropagation();
        onforget();
      }}
    >
      <X />
    </Button>
  </div>

  <!-- Sessions: the space's open shells, each its own card inside the
       space's — identity over status, with its close action pinned the
       same way the space's is. Clicking one selects the space *and*
       switches to that session, the one click that does both. -->
  {#if space.terminals.length}
    <ul class="flex flex-col gap-1.5">
      {#each space.terminals as t (t.id)}
        {@const isActive = selected && activeTermId === t.id}
        <li>
          <div
            role="button"
            tabindex="0"
            aria-pressed={isActive}
            class={[
              "flex flex-col gap-0.5 rounded-md border px-2 py-1.5 transition-colors",
              isActive
                ? "border-primary/50 bg-sidebar-accent text-sidebar-accent-foreground"
                : "border-sidebar-border hover:bg-sidebar-accent/60",
            ]}
            onclick={(e) => {
              e.stopPropagation();
              onselectsession(t);
            }}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                e.stopPropagation();
                onselectsession(t);
              }
            }}
          >
            <div class="flex items-start gap-1">
              <span class="min-w-0 flex-1">
                {#if t.session}
                  <!-- A session: its identity is the ticket it is bound
                       to (role · #num) — told apart from an ad-hoc
                       shell, which shows its foreground process. -->
                  <span
                    class="flex min-w-0 items-center gap-1 text-xs font-medium"
                  >
                    <Rocket
                      class="size-3 shrink-0 text-primary"
                      aria-hidden="true"
                    />
                    <span class="truncate"
                      >{t.session.role} #{pad(t.session.ticketNum)}</span
                    >
                  </span>
                {:else}
                  <span class="block truncate font-mono text-xs">{t.proc}</span>
                {/if}
              </span>
              <Button
                variant="ghost"
                size="icon-xs"
                class="-mt-0.5 -mr-1 hover:text-destructive"
                aria-label="End {t.proc}"
                title={t.session ? "End this session" : "End this shell"}
                onclick={(e) => {
                  e.stopPropagation();
                  onendshell(t);
                }}
              >
                <X />
              </Button>
            </div>

            <div class="flex items-center gap-1.5">
              <!-- Status indicator. A tab with no known agent in front: a
                   spinner while working, a tick idle, an error mark once it
                   exits. A tab with a known agent reads the agent's own
                   broadcast state — the same spinner and tick, plus a held
                   pause mark when it is blocked waiting on its human. A dead
                   session freezes under a grey mark. -->
              {#if t.status === "working"}
                <CircleNotch
                  class="size-3.5 shrink-0 animate-spin text-primary"
                  aria-label="working"
                />
              {:else if t.status === "blocked"}
                <PauseCircle
                  class="size-3.5 shrink-0 text-primary"
                  aria-label="blocked"
                />
              {:else if t.status === "dead"}
                <XCircle
                  class="size-3.5 shrink-0 text-muted-foreground"
                  aria-label="dead"
                />
              {:else if t.status === "exited"}
                <XCircle
                  class="size-3.5 shrink-0 text-destructive"
                  aria-label="exited"
                />
              {:else}
                <Check
                  class="size-3.5 shrink-0 text-muted-foreground"
                  aria-label="idle"
                />
              {/if}
              <span
                class="min-w-0 flex-1 truncate text-[0.65rem] text-muted-foreground"
              >
                {#if t.session}{t.session.agent} · {t.status}{:else}{t.status}{/if}
              </span>

              {#if t.session && !t.alive}
                <!-- The death halt: a dead session is pinned to its ticket and
                     offers exactly three choices — resume it (crash recovery),
                     respawn a fresh session, or release the claim. chartr
                     takes none itself. -->
                <span class="-my-0.5 -mr-1 flex shrink-0 items-center">
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="hover:text-primary"
                    aria-label="Resume this session"
                    title="Resume — same-ticket crash recovery"
                    onclick={(e) => {
                      e.stopPropagation();
                      onhalt(t, "resume");
                    }}
                  >
                    <Play />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="hover:text-primary"
                    aria-label="Respawn a fresh session"
                    title="Respawn — a fresh session on the same ticket"
                    onclick={(e) => {
                      e.stopPropagation();
                      onhalt(t, "respawn");
                    }}
                  >
                    <ArrowClockwise />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="hover:text-destructive"
                    aria-label="Release the claim"
                    title="Release — clear the claim back to the frontier"
                    onclick={(e) => {
                      e.stopPropagation();
                      onhalt(t, "release");
                    }}
                  >
                    <ArrowUUpLeft />
                  </Button>
                </span>
              {/if}
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Actions: the two ticketless on-ramps — ideate and a plain
       shell — sharing their row with the branch, which doubles as
       the spacer that pushes them right. The branch is a real ghost
       Button rather than a bare span with a handler: it is a control
       inside a control, so it earns the same hover, focus ring and
       keyboard reach every other action in this row has. The size
       variants all set a fixed height and `shrink-0`, so this one
       overrides both — it still has to truncate and still has to be
       the spacer. -->
  <div class="flex items-center gap-1">
    <span
      class="flex min-w-0 flex-1 items-center gap-1.5 text-[0.6rem] text-muted-foreground"
    >
      {#if space.branch}
        <Button
          variant="ghost"
          class="-mx-1 h-auto min-w-0 shrink justify-start gap-1.5 rounded-sm px-1 py-0.5 text-[0.6rem] font-normal text-muted-foreground hover:text-foreground [&_svg:not([class*='size-'])]:size-3"
          aria-label="Copy branch name"
          title={copied
            ? copied.ok
              ? "Copied"
              : "Couldn’t copy — clipboard unavailable"
            : `Copy branch — ${space.branch}`}
          onclick={(e) => {
            e.stopPropagation();
            void copyToClipboard(space.branch ?? "");
          }}
        >
          {#if copied}
            {#if copied.ok}
              <Check class="text-primary" />
            {:else}
              <Warning class="text-destructive" />
            {/if}
          {:else}
            <GitBranchIcon />
          {/if}
          <span class="truncate font-mono">{space.branch}</span>
        </Button>
      {/if}
    </span>
    <!-- The skill launcher (skill-launcher map): one `Skills ▾` menu —
         the agent picker over the on-ramp skills. The row's own click
         just selects the space, which the launch does anyway, so this
         one deliberately does not stop propagation — the dropdown
         trigger has no click handler of its own to protect. The agent's
         model and the skill name live in the menu, not on this cramped
         label. -->
    <SkillLauncher
      {agents}
      lastAgent={space.lastAgent}
      skills={space.skills}
      label="Skills"
      disabled={opening}
      size="xs"
      ariaLabel="Launch a skill in {space.name}"
      title="Launch a self-driving skill in {space.name} — a live, ticketless agent tab. Nothing is claimed, nothing is committed, and it ends when you end it."
      onrun={onlaunch}
      {onregister}
    ></SkillLauncher>
    <Button
      variant="outline"
      size="xs"
      aria-label="Open a shell in {space.name}"
      title="Open a shell in {space.name}"
      disabled={opening}
      onclick={(e) => {
        e.stopPropagation();
        onopenshell();
      }}
    >
      <PlusIcon />
    </Button>
  </div>
</div>
