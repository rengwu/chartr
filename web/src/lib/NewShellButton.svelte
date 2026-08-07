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
  // The **caret is a selector, not a launcher**, and the **body launches what is
  // selected**. The menu is a radio group: `new shell` (the plain shell, nothing
  // injected), a divider, then the registered agents *in registration order* — no
  // ranking, no model sublabels. Picking a row only *chooses*; the body then
  // carries that choice as its label, so what a click does is always written on
  // the button. An agent whose binary is absent from PATH sits disabled under its
  // own reason, exactly as the spawn picker shows it.
  //
  // The selection defaults to the plain shell — the zero-decision action — and is
  // this control's own state, not the space's: nothing is remembered across a
  // reload, and the server's `lastAgent` deliberately does not seed it (a free
  // session has no claim on that memory). It sticks for as long as the card is
  // mounted, so a second session on the same agent is one click.
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

  // Which row the body will run: the agent's name, or "" for the plain shell.
  // Held here rather than lifted, because it decides nothing outside this button.
  let picked = $state("");

  // The effective choice. An agent that has since been deregistered, or whose
  // binary has left PATH, silently falls back to the plain shell rather than
  // leaving the body pointed at something that cannot run.
  const chosen = $derived(agents.find((a) => a.name === picked && a.present));
  const runs = $derived(chosen?.name ?? "");
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
    aria-label={runs ? `Start a session on ${runs}` : ariaLabel}
    title={runs
      ? `Start a free session on ${runs} — a live, ticketless agent tab. Nothing is claimed, nothing is committed, and it ends when you end it.`
      : title}
    onclick={(e) => {
      e.stopPropagation();
      if (runs) onfree(runs);
      else onshell();
    }}
  >
    {runs || label}
  </Button>
  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class={cn(buttonVariants({ variant: "outline", size }), "rounded-l-none px-1")}
      {disabled}
      aria-label="Choose what the button opens"
    >
      <CaretDown />
    </DropdownMenu.Trigger>

    <DropdownMenu.Content align="end" class="min-w-48 w-auto">
      <!-- One radio group across both halves of the menu: the plain shell and the
           agents are the same kind of choice, so exactly one of them is checked
           and the divider is a grouping, not a boundary between a command and a
           setting. -->
      <DropdownMenu.RadioGroup value={runs} onValueChange={(v) => (picked = v)}>
        <DropdownMenu.RadioItem value="">{label}</DropdownMenu.RadioItem>

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
            <DropdownMenu.RadioItem
              value={a.name}
              disabled={!a.present}
              class="flex flex-col items-start gap-0.5"
            >
              <span class="font-medium">{a.name}</span>
              {#if !a.present && a.missing}
                <span class="text-[0.65rem] text-destructive">{a.missing}</span>
              {/if}
            </DropdownMenu.RadioItem>
          {/each}
        {/if}
      </DropdownMenu.RadioGroup>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
</div>
