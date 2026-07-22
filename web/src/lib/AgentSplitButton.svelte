<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Agent } from "./model";
  import { chooseAgent, type AgentChoice } from "./agentchoice";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import type { ButtonVariant, ButtonSize } from "$lib/components/ui/button";
  import { CaretDown } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The one control that starts something on an agent, and the one agent picker
  // in the codebase (agent-selection spec, Spawn surfaces). Every surface that
  // launches — each role on the action bar, and the ideate on-ramp — is this
  // control, so the rule is the same everywhere it appears:
  //
  //   ready    — the primary action runs the space's remembered agent and names
  //              it on the label; the caret opens the list for a one-off
  //              override, which a successful run then remembers server-side.
  //   unchosen — the primary action opens the list instead of running. There is
  //              no automatic first choice, and no one-click path bypasses it.
  //   empty    — ticket 04's surface. Until then it falls through to the server's
  //              own refusal, which is what the caller renders.
  //
  // The list is every registered agent; one whose binary is absent is disabled
  // with its reason on the row, not hidden behind a hover title.
  let {
    agents,
    lastAgent,
    label,
    // The label when nothing is remembered — the button is about to open a
    // picker, not to run, and "Start Implement" says that better than "Implement".
    unchosenLabel,
    // Compact placements (a sidebar's icon-sized row) carry the agent's name in
    // the menu and the tooltip rather than on the label, which has no room for it.
    nameOnLabel = true,
    busy = false,
    busyLabel = "Starting…",
    disabled = false,
    variant = "outline",
    size = "sm",
    title,
    ariaLabel,
    // What this control opens, said in the interface rather than left to a
    // source comment (spec story 29). Heads the menu, above the agent list.
    note,
    icon,
    onrun,
  }: {
    agents: Agent[];
    lastAgent?: string;
    label: string;
    unchosenLabel?: string;
    nameOnLabel?: boolean;
    busy?: boolean;
    busyLabel?: string;
    disabled?: boolean;
    variant?: ButtonVariant;
    size?: ButtonSize;
    title?: string;
    ariaLabel?: string;
    note?: string;
    icon?: Snippet;
    onrun: (agent: string) => void;
  } = $props();

  const choice = $derived<AgentChoice>(chooseAgent(agents, lastAgent));
  let open = $state(false);

  const buttonLabel = $derived.by(() => {
    if (busy) return busyLabel;
    if (choice.kind === "ready" && nameOnLabel) {
      return `${label} with ${choice.agent.name}`;
    }
    return choice.kind === "ready" ? label : (unchosenLabel ?? label);
  });

  const buttonTitle = $derived(
    choice.kind === "ready" && title ? `${title} — with ${choice.agent.name}` : title,
  );

  function primary() {
    if (choice.kind === "ready") {
      onrun(choice.agent.name);
    } else if (choice.kind === "unchosen") {
      open = true;
    } else {
      // Empty library: ticket 04 owns this surface. Until then, run with no name
      // and let the server's refusal be the thing the operator reads.
      onrun("");
    }
  }
</script>

<DropdownMenu.Root bind:open>
  <div class="inline-flex">
    <Button
      {variant}
      {size}
      {disabled}
      class="rounded-r-none"
      title={buttonTitle}
      aria-label={ariaLabel}
      onclick={primary}
    >
      {#if icon}{@render icon()}{/if}
      {buttonLabel}
    </Button>
    <DropdownMenu.Trigger
      class={cn(buttonVariants({ variant, size }), "rounded-l-none border-l-0 px-1.5")}
      {disabled}
      aria-label="Choose agent"
    >
      <CaretDown />
    </DropdownMenu.Trigger>
  </div>
  <DropdownMenu.Content align="end" class="min-w-52 w-auto">
    {#if note}
      <DropdownMenu.Label class="max-w-64 text-[0.7rem] leading-relaxed font-normal text-wrap text-muted-foreground">
        {note}
      </DropdownMenu.Label>
      <DropdownMenu.Separator />
    {/if}
    {#each agents as a (a.name)}
      <DropdownMenu.Item
        class="flex flex-col items-start gap-0.5"
        disabled={!a.present}
        onclick={() => onrun(a.name)}
      >
        <span class="font-medium">{a.name}</span>
        {#if !a.present && a.missing}
          <span class="text-[0.65rem] text-destructive">{a.missing}</span>
        {/if}
      </DropdownMenu.Item>
    {/each}
    {#if !agents.length}
      <DropdownMenu.Label class="max-w-64 text-[0.7rem] leading-relaxed font-normal text-wrap text-muted-foreground">
        No agents registered.
      </DropdownMenu.Label>
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>
