<script lang="ts">
  import type { Agent } from "./model";
  import { chooseAgent, type AgentChoice } from "./agentchoice";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import { Robot, CaretDown, Check } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The quiet "who" of the action footer: a single agent picker that every action
  // beside it spawns with. The agent is a per-space remembered choice, not a
  // per-role one, so it belongs on one control here rather than repeated behind a
  // caret on every action. This is the one picker; the actions themselves are
  // plain buttons that carry no agent choice of their own.
  //
  //   ready    — names the chosen agent; the menu lets the operator switch it.
  //   unchosen — nothing is remembered yet; the control names the choice to make
  //              ("Choose an agent") and the actions stay disabled until it is
  //              made, so no one-click path skips the initial choice.
  //   empty    — nothing is registered (ticket 04); the control routes to
  //              registration rather than opening a menu onto nothing.
  let {
    agents,
    // The name to reflect as selected — the space's remembered agent, or a local
    // override the operator has picked this session. `onselect` reports a change;
    // it does not persist on its own — a successful spawn is what the server
    // remembers, exactly as before.
    selected,
    onselect,
    onregister,
  }: {
    agents: Agent[];
    selected?: string;
    onselect: (agent: string) => void;
    onregister?: () => void;
  } = $props();

  const choice = $derived<AgentChoice>(chooseAgent(agents, selected));

  const label = $derived(
    choice.kind === "ready" ? choice.agent.name : "Choose an agent",
  );
</script>

{#if choice.kind === "empty"}
  <Button
    variant="ghost"
    size="sm"
    class="gap-1.5 text-muted-foreground"
    title="No agent registered — register one to start"
    onclick={() => onregister?.()}
  >
    <Robot class="size-4" /> Register an agent…
  </Button>
{:else}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class={cn(
        buttonVariants({ variant: "ghost", size: "sm" }),
        "gap-1.5 text-muted-foreground",
      )}
      title="Agent every action spawns with"
      aria-label="Choose agent"
    >
      <Robot class="size-4 shrink-0" />
      <span class="max-w-40 truncate">{label}</span>
      <CaretDown class="size-3 shrink-0 opacity-70" />
    </DropdownMenu.Trigger>
    <DropdownMenu.Content align="start" class="min-w-52 w-auto">
      {#each agents as a (a.name)}
        <DropdownMenu.Item
          class="flex items-start gap-2"
          disabled={!a.present}
          onclick={() => onselect(a.name)}
        >
          <Check
            class={cn(
              "mt-0.5 size-3.5 shrink-0",
              choice.kind === "ready" && choice.agent.name === a.name
                ? "opacity-100"
                : "opacity-0",
            )}
          />
          <span class="flex flex-col items-start gap-0.5">
            <span class="font-medium">{a.name}</span>
            {#if !a.present && a.missing}
              <span class="text-[0.65rem] text-destructive">{a.missing}</span>
            {/if}
          </span>
        </DropdownMenu.Item>
      {/each}
    </DropdownMenu.Content>
  </DropdownMenu.Root>
{/if}
