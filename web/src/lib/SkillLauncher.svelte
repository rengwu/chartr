<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Agent, ResolvedSkill } from "./model";
  import { launchMenu, launchClick, agentModel } from "./launchmenu";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import type { ButtonVariant, ButtonSize } from "$lib/components/ui/button";
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
  //   unchosen — agents exist but none is remembered. The agent section is checked
  //              at nothing and the skills sit disabled until the operator picks
  //              one — the actionable path is right there, so it is not a dead
  //              control (there is no automatic first choice, per the agent spec).
  //   ready    — the remembered agent opens checked; clicking any on-ramp skill
  //              launches it on that agent, and the server remembers the agent.
  //
  // Two sections: the agent selector (each row labelled by the model the agent
  // already carries — decision (b), no new model axis), then a divider, then the
  // on-ramp skills the resolved library offers. A skill click *is* the launch;
  // there is no separate run button. Context is 03's affordance — every skill
  // launches bare here, which is correct for the self-driving ones regardless.
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
    // remembered default) and clicked `skill`. Bare — no context (03).
    onrun: (agent: string, skill: string) => void;
    onregister?: () => void;
  } = $props();

  // The operator's in-menu agent pick this open, if any — a one-off override of the
  // remembered agent that a successful launch then persists server-side. Reset when
  // the menu closes, so it always reopens on the remembered agent.
  let picked = $state<string | undefined>();

  const menu = $derived(launchMenu(agents, lastAgent, skills, picked));
  // The agent row that renders checked: the effective choice when one is ready,
  // else nothing (the unchosen and empty states show no selection).
  const checked = $derived(menu.choice.kind === "ready" ? menu.choice.agent.name : "");

  function run(skill: ResolvedSkill) {
    const target = launchClick(menu, skill);
    if (target) onrun(target.agent, target.skill);
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

  <DropdownMenu.Content align="end" class="min-w-52 w-auto">
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
        {#each menu.skills as s (s.name)}
          <DropdownMenu.Item
            class="flex flex-col items-start gap-0.5"
            disabled={menu.choice.kind !== "ready"}
            onclick={() => run(s)}
          >
            <span class="font-medium">{s.name}</span>
            {#if s.description}
              <span class="max-w-64 text-[0.65rem] text-wrap text-muted-foreground">
                {s.description}
              </span>
            {/if}
          </DropdownMenu.Item>
        {/each}
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
