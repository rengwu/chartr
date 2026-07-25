<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Agent, ResolvedSkill } from "./model";
  import { launchMenu, launchClick, launchPayload, contextHint, agentModel } from "./launchmenu";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import Modal from "./Modal.svelte";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import type { ButtonVariant, ButtonSize } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { CaretDown } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The skill launcher: the space card's on-ramp control, one dropdown that runs
  // any *self-driving* skill on a chosen agent (skill-launcher map). It is always a
  // dropdown — a single `skills ▾` trigger the operator opens and picks from every
  // time — not a split primary-action button, and it remembers no skill. It shares
  // the agent-choice logic (`chooseAgent`, via `launchMenu`) with the rest of the
  // codebase, so the empty and unchosen states behave exactly as the agent picker
  // does everywhere:
  //
  //   empty    — nothing is registered. The menu does not launch onto nothing: it
  //              names the wall and routes to registration via `onregister`.
  //   unchosen — agents exist but none of them is launchable (every one absent
  //              from PATH), or the remembered agent no longer resolves. The agent
  //              section is checked at nothing and the skills sit disabled until
  //              the operator picks one — the actionable path is right there, so it
  //              is not a dead control.
  //   ready    — the remembered agent opens checked, or, with nothing remembered,
  //              the first present one (`launchMenu`'s default — a menu of disabled
  //              skills reads as broken). Clicking any on-ramp skill launches it on
  //              that agent, and the server remembers the agent.
  //
  // Two sections: the agent selector (each row labelled by the model the agent
  // already carries — decision (b), no new model axis), then a divider, then the
  // on-ramp skills the resolved library offers. A skill click *is* the launch;
  // there is no separate run button.
  //
  // The one step between a click and a launch is the **optional context box**
  // (ticket 03): a skill the library marks `needs-context` reads a subject —
  // `grill` wants to know *what* to grill — so its click opens a roomy multi-line
  // box instead of launching. The box is a modal over the whole cockpit, not a
  // popover hung off this trigger: drafting a brief is a paragraph of typing, and
  // an anchored panel that vanishes on any click away is too easy to lose. Optional
  // means optional: an empty box launches the skill bare, exactly as a self-driving
  // one does, and Esc dismisses without launching. A skill without the flag never
  // sees the box.
  let {
    agents,
    lastAgent,
    skills,
    label = "Skills",
    disabled = false,
    variant = "outline",
    size = "sm",
    title,
    ariaLabel,
    icon,
    onrun,
    // Where the empty state sends the operator: the registration surface. Given by
    // every caller, so a launcher with an empty library is never a dead button.
    onregister,
  }: {
    agents: Agent[];
    lastAgent?: string;
    skills: ResolvedSkill[];
    label?: string;
    disabled?: boolean;
    variant?: ButtonVariant;
    size?: ButtonSize;
    title?: string;
    ariaLabel?: string;
    icon?: Snippet;
    // The launch: the operator picked `agent` (from the section above, or the
    // remembered default) and clicked `skill`. `context` is the optional one line
    // a `needs-context` skill offered — absent for every bare launch.
    onrun: (agent: string, skill: string, context?: string) => void;
    onregister?: () => void;
  } = $props();

  // The operator's in-menu agent pick this open, if any — a one-off override of the
  // remembered agent that a successful launch then persists server-side. Reset when
  // the menu closes, so it always reopens on the remembered agent.
  let picked = $state<string | undefined>();

  // The pending context box: the `needs-context` skill the operator clicked and the
  // agent it will run on, held while they type (or don't). `null` is the box shut —
  // it is also the modal's open state, so dismissing it clears the pending launch.
  let pending = $state<{ agent: string; skill: ResolvedSkill } | null>(null);
  let line = $state("");
  let field = $state<HTMLTextAreaElement | null>(null);

  const menu = $derived(launchMenu(agents, lastAgent, skills, picked));
  // The agent row that renders checked: the effective choice when one is ready,
  // else nothing (the unchosen and empty states show no selection).
  const checked = $derived(menu.choice.kind === "ready" ? menu.choice.agent.name : "");

  function run(skill: ResolvedSkill) {
    const step = launchClick(menu, skill);
    if (!step) return;
    if (step.kind === "context") {
      line = "";
      pending = { agent: step.agent, skill: step.skill };
      return;
    }
    onrun(step.agent, step.skill);
  }

  // Fire the pending launch with whatever is in the box — a blank line carries no
  // context and opens the skill bare, which is a real launch, not a refusal.
  function fire() {
    if (!pending) return;
    const { agent, skill, context } = launchPayload(pending.agent, pending.skill.name, line);
    pending = null;
    onrun(agent, skill, context);
  }
</script>

<DropdownMenu.Root onOpenChange={(open) => { if (!open) picked = undefined; }}>
  <DropdownMenu.Trigger
    class={cn(buttonVariants({ variant, size }), "gap-1")}
    {disabled}
    aria-label={ariaLabel}
    {title}
  >
    {#if icon}{@render icon()}{/if}
    {label}
    <CaretDown />
  </DropdownMenu.Trigger>

  <!-- When a `needs-context` skill hands off to the box, the closing menu must not
       pull focus back to the trigger — the box's field is where the operator is
       going. -->
  <DropdownMenu.Content
    align="start"
    class="min-w-52 w-auto"
    onCloseAutoFocus={(e) => { if (pending) e.preventDefault(); }}
  >
    {#if menu.choice.kind === "empty"}
      <DropdownMenu.Label
        class="max-w-64 text-[0.7rem] leading-relaxed font-normal text-wrap text-muted-foreground"
      >
        No agents registered yet.
      </DropdownMenu.Label>
      <DropdownMenu.Item onclick={() => onregister?.()}>Register an agent…</DropdownMenu.Item>
    {:else}
      <DropdownMenu.Label class="text-[0.7rem] font-normal text-muted-foreground">
        Agent
      </DropdownMenu.Label>
      <!-- Picking an agent must not dismiss the menu — the skill is still to be
           chosen below — so each row keeps the menu open on select. -->
      <DropdownMenu.RadioGroup value={checked} onValueChange={(v) => (picked = v)}>
        {#each menu.agents as a (a.name)}
          <DropdownMenu.RadioItem
            value={a.name}
            disabled={!a.present}
            closeOnSelect={false}
            class="flex flex-col items-start gap-0.5"
          >
            <span class="font-medium">{a.name}</span>
            {#if a.present}
              {@const model = agentModel(a)}
              {#if model}
                <span class="text-[0.65rem] text-muted-foreground">{model}</span>
              {/if}
            {:else if a.missing}
              <span class="text-[0.65rem] text-destructive">{a.missing}</span>
            {/if}
          </DropdownMenu.RadioItem>
        {/each}
      </DropdownMenu.RadioGroup>

      <DropdownMenu.Separator />

      <DropdownMenu.Label class="text-[0.7rem] font-normal text-muted-foreground">
        {#if menu.choice.kind === "ready"}Launch a skill{:else}Pick an agent to launch a skill{/if}
      </DropdownMenu.Label>
      {#if menu.skills.length}
        <!-- One line per skill: the name alone. A skill's description is a
             paragraph — three of them stacked turn the menu into a wall of prose
             to read past, so the description hangs off hover as a tooltip beside
             the row instead of sitting under it. -->
        <Tooltip.Provider delayDuration={250}>
          {#each menu.skills as s (s.name)}
            <Tooltip.Root>
              <Tooltip.Trigger>
                {#snippet child({ props })}
                  <DropdownMenu.Item
                    {...props}
                    class="font-medium"
                    disabled={menu.choice.kind !== "ready"}
                    onclick={() => run(s)}
                  >
                    {s.name}
                  </DropdownMenu.Item>
                {/snippet}
              </Tooltip.Trigger>
              {#if s.description}
                <Tooltip.Content side="right" sideOffset={6} class="max-w-64 text-wrap">
                  {s.description}
                </Tooltip.Content>
              {/if}
            </Tooltip.Root>
          {/each}
        </Tooltip.Provider>
      {:else}
        <DropdownMenu.Label
          class="max-w-64 text-[0.7rem] leading-relaxed font-normal text-wrap text-muted-foreground"
        >
          No on-ramp skills in this library.
        </DropdownMenu.Label>
      {/if}
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>

<!-- The optional context box (ticket 03). A modal on the shared `Modal` — the same
     dialog the payload preview uses — so it dims and sits over the whole cockpit
     with focus trapped in the field: a brief is a paragraph of typing, and an
     anchored panel that closes on any stray click is too easy to lose. It
     autofocuses the field, and Esc, the close button, or Cancel dismiss it without
     launching. ⌘/Ctrl+Enter launches (plain Enter is a newline). Nothing here
     validates: an empty box opens the skill bare either way. -->
{#if pending}
  {@const skill = pending.skill}
  <Modal
    open
    title="Context for {skill.name}"
    onClose={() => (pending = null)}
    onOpenAutoFocus={(e) => { e.preventDefault(); field?.focus(); }}
  >
    <form class="flex flex-col gap-3" onsubmit={(e) => { e.preventDefault(); fire(); }}>
      <p class="text-muted-foreground">
        What should <span class="font-medium text-foreground">{skill.name}</span> work on? It runs on
        <span class="font-medium text-foreground">{pending.agent}</span>. Optional — an empty box
        opens the skill bare.
      </p>
      <Textarea
        bind:ref={field}
        bind:value={line}
        rows={8}
        class="min-h-40 text-xs"
        placeholder={contextHint(skill)}
        aria-label="Optional context for {skill.name}"
        onkeydown={(e) => {
          if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            fire();
          }
        }}
      />
      <div class="flex items-center justify-between gap-2">
        <span class="text-[0.65rem] text-muted-foreground">⌘↵ launches, Esc cancels.</span>
        <div class="flex items-center gap-2">
          <Button type="button" variant="ghost" size="sm" onclick={() => (pending = null)}>
            Cancel
          </Button>
          <Button type="submit" size="sm">Launch</Button>
        </div>
      </div>
    </form>
  </Modal>
{/if}
