<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Agent, ResolvedSkill } from "./model";
  import { launchMenu, launchClick, launchPayload, contextHint, agentModel } from "./launchmenu";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Popover from "$lib/components/ui/popover";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import type { ButtonVariant, ButtonSize } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
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
  // there is no separate run button.
  //
  // The one step between a click and a launch is the **optional context box**
  // (ticket 03): a skill the library marks `needs-context` reads a subject —
  // `grill` wants to know *what* to grill — so its click opens a roomy multi-line
  // box hung off this control's own trigger instead of launching. Optional means
  // optional: an empty box launches the skill bare, exactly as a self-driving
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
  // it is also the popover's open state, so dismissing it clears the pending launch.
  let pending = $state<{ agent: string; skill: ResolvedSkill } | null>(null);
  let line = $state("");
  let field = $state<HTMLTextAreaElement | null>(null);
  // This control's own trigger, which the box anchors to: the dropdown has closed
  // by the time the box opens, so the popover needs the button as a custom anchor
  // to hang off the same spot the menu came from.
  let trigger = $state<HTMLElement | null>(null);

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
    bind:ref={trigger}
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
    align="end"
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

<!-- The optional context box (ticket 03). A popover rather than a dialog: this is
     a brief the operator drafts, not a task — it hangs off the trigger the menu
     came from (`customAnchor`, since the menu that opened it has closed),
     autofocuses its field, and dismisses on Esc or a click away without
     launching. A roomy multi-line box: the brief can run to a paragraph, so the
     field is a textarea and ⌘/Ctrl+Enter launches (plain Enter is a newline).
     Nothing here validates: an empty box opens the skill bare either way. -->
<Popover.Root open={pending !== null} onOpenChange={(open) => { if (!open) pending = null; }}>
  {#if pending}
    {@const skill = pending.skill}
    <Popover.Content
      customAnchor={trigger}
      align="end"
      class="w-96 gap-2 p-3"
      onOpenAutoFocus={(e) => { e.preventDefault(); field?.focus(); }}
    >
      <Popover.Header>
        <Popover.Title class="text-xs">
          {skill.name}
          <span class="font-normal text-muted-foreground">on {pending.agent}</span>
        </Popover.Title>
      </Popover.Header>
      <form class="flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); fire(); }}>
        <Textarea
          bind:ref={field}
          bind:value={line}
          rows={6}
          class="min-h-32 text-xs"
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
          <Popover.Description class="text-[0.65rem]">
            Optional — ⌘↵ launches, Esc cancels. An empty box opens {skill.name} bare.
          </Popover.Description>
          <Button type="submit" size="sm">Launch</Button>
        </div>
      </form>
    </Popover.Content>
  {/if}
</Popover.Root>
