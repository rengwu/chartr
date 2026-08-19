<script lang="ts">
  import type { Agent } from "./model";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import type { ButtonSize, ButtonVariant } from "$lib/components/ui/button";
  import { DotsThree } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The new-shell control (skill-sources ticket 08): the space card's one
  // ticketless on-ramp, a split button that replaced both the skill launcher and
  // the bare `+` shell button.
  //
  // The **body is fixed to the plain shell** — the zero-decision action, nothing
  // injected — and the **caret is a launcher, not a selector**. The menu is a
  // plain list of the registered agents *in registration order* (no ranking, no
  // model sublabels): clicking a row *launches a free session on that agent right
  // away*, rather than re-pointing the body. Nothing is selected, nothing is
  // remembered — the body always says `new shell` and always opens one.
  //
  // An agent whose binary is absent from PATH sits disabled under its own reason,
  // exactly as the spawn picker shows it. With an empty library the message
  // routes to registration, so the menu is never a dead control.
  let {
    agents,
    disabled = false,
    size = "sm",
    // Both halves take this variant. `outline` (the default) gives the joined,
    // outlined split button the space pane carries; `ghost` drops the border for
    // two plain buttons grouped together, as the space card's compact `+` wants.
    variant = "outline",
    label = "New Shell",
    ariaLabel = "Open a plain shell",
    title = "Open a plain shell — nothing is injected.",
    // The body's content. Defaults to the `label` text; a caller can render an
    // icon instead (the space card's compact `+`), keeping the caret and its
    // agent list unchanged.
    children,
    // The zero-decision action: a plain shell, nothing injected.
    onshell,
    // A free session on the agent the operator clicked: a live, ticketless tab
    // launched bare and titled by this name.
    onfree,
    // Where an empty library sends the operator: the registration surface.
    onregister,
  }: {
    agents: Agent[];
    disabled?: boolean;
    size?: ButtonSize;
    variant?: ButtonVariant;
    label?: string;
    ariaLabel?: string;
    title?: string;
    children?: import("svelte").Snippet;
    onshell: () => void;
    onfree: (agent: string) => void;
    onregister?: () => void;
  } = $props();
</script>

<!-- One control, two hit targets: the body and the caret join at the seam (no
     rounding, no doubled border between them), so an outlined pair reads as a
     single button with a menu, and a ghost pair as two buttons grouped together
     rather than two that merely happen to be adjacent. -->
<div class="flex items-center">
  <Button
    {variant}
    {size}
    class="rounded-r-none border-r-0"
    {disabled}
    aria-label={ariaLabel}
    {title}
    onclick={(e) => {
      e.stopPropagation();
      onshell();
    }}
  >
    {#if children}{@render children()}{:else}{label}{/if}
  </Button>
  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class={cn(buttonVariants({ variant, size }), "rounded-l-none px-1")}
      {disabled}
      aria-label="Launch a free session on a registered agent"
    >
      <DotsThree />
    </DropdownMenu.Trigger>

    <DropdownMenu.Content align="end" class="min-w-48 w-auto">
      {#if agents.length === 0}
        <DropdownMenu.Label
          class="max-w-64 text-[0.7rem] leading-relaxed font-normal text-wrap text-muted-foreground"
        >
          No agents registered yet.
        </DropdownMenu.Label>
        <DropdownMenu.Item onclick={() => onregister?.()}
          >Register an agent</DropdownMenu.Item
        >
      {:else}
        {#each agents as a (a.name)}
          <DropdownMenu.Item
            disabled={!a.present}
            class="flex flex-col items-start gap-0.5"
            onclick={() => onfree(a.name)}
          >
            <span class="font-medium">{a.name}</span>
            {#if !a.present && a.missing}
              <span class="text-[0.65rem] text-destructive">{a.missing}</span>
            {/if}
          </DropdownMenu.Item>
        {/each}
      {/if}
    </DropdownMenu.Content>
  </DropdownMenu.Root>
</div>
