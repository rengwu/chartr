<script lang="ts">
  import type { Agent } from "./model";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import type { ButtonSize } from "$lib/components/ui/button";
  import { CaretDown } from "phosphor-svelte";
  import { cn } from "$lib/utils";

  // The new-shell control (skill-sources ticket 08): the space card's one
  // ticketless on-ramp, a split button that replaced both the skill launcher and
  // the bare `+` shell button.
  //
  // The **body** is `empty shell` — the zero-decision action, and what keeps the
  // static label honest: a body that launched the last-used agent would make "new
  // shell" lie about what a click does. Empty means empty — nothing is injected
  // into it, no conventions pointer and no payload of any kind.
  //
  // The **caret** opens the menu: `empty shell` again, a divider, then the
  // registered agents *in registration order* — no ranking, no remembered
  // default, no model sublabels. The spawn picker's ranking stays where it is; a
  // free session has no "confirm this and go" flow to optimise, so the list is
  // static and the operator picks every time. An agent whose binary is absent
  // from PATH sits disabled under its own reason, exactly as the spawn picker
  // shows it.
  //
  // With an empty library the divider stays and the message routes to
  // registration, so the menu is never a dead control.
  let {
    agents,
    disabled = false,
    size = "xs",
    label = "new shell",
    ariaLabel,
    title,
    // The zero-decision action: a plain shell, nothing injected.
    onshell,
    // A free session on the agent the operator clicked: a live, ticketless tab
    // told the free payload, titled by this name.
    onfree,
    // Where an empty library sends the operator: the registration surface.
    onregister,
  }: {
    agents: Agent[];
    disabled?: boolean;
    size?: ButtonSize;
    label?: string;
    ariaLabel?: string;
    title?: string;
    onshell: () => void;
    onfree: (agent: string) => void;
    onregister?: () => void;
  } = $props();
</script>

<!-- One control, two hit targets: the body and the caret share an outline and
     join at the seam, so it reads as a single button with a menu rather than two
     buttons that happen to be adjacent. -->
<div class="flex items-center">
  <Button
    variant="outline"
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
    {label}
  </Button>
  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class={cn(buttonVariants({ variant: "outline", size }), "rounded-l-none px-1")}
      {disabled}
      aria-label="Open a shell or start a free session"
    >
      <CaretDown />
    </DropdownMenu.Trigger>

    <DropdownMenu.Content align="end" class="min-w-48 w-auto">
      <DropdownMenu.Item onclick={() => onshell()}>{label}</DropdownMenu.Item>

      <DropdownMenu.Separator />

      {#if agents.length === 0}
        <DropdownMenu.Label
          class="max-w-64 text-[0.7rem] leading-relaxed font-normal text-wrap text-muted-foreground"
        >
          No agents registered yet.
        </DropdownMenu.Label>
        <DropdownMenu.Item onclick={() => onregister?.()}>Register an agent…</DropdownMenu.Item>
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
