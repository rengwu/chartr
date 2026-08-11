<script lang="ts">
  import type { Agent } from "./model";
  import { setAgent, deleteAgent } from "./actions";
  import { formatArgs, parseArgs } from "./args";
  import { Button } from "./components/ui/button";
  import { toast } from "./components/ui/sonner";
  import { Input } from "./components/ui/input";
  import * as Select from "./components/ui/select";
  import * as Table from "./components/ui/table";
  import Modal from "./Modal.svelte";
  import { PencilSimple, Plus, Trash } from "phosphor-svelte";

  // The agent library (global scope): named ways to run a harness, registered
  // once and assigned to roles in any space. An agent is a *complete* launch
  // spec — the binary, whatever flags that harness wants, and how it takes its
  // opening prompt — because a role assignment names one thing, not
  // three-quarters of one.
  //
  // Nothing here knows anything about any particular CLI, and there is no model
  // field: a model is a flag, and it goes in the list with the rest. chartr
  // cannot know what `--model sonnet`, `--dangerously-skip-permissions` or
  // `--sandbox danger-full-access` mean to the harness that defines them, so
  // flags are an opaque list the operator types.
  let {
    agents,
    detected = [],
  }: {
    agents: Agent[];
    // The known agent CLIs found on this machine's PATH (ticket 04) — an advisory
    // hint, rendered as helper text beneath the adapter input rather than as a
    // placeholder, which would vanish on the first keystroke exactly when the list
    // is most useful. A suggestion, never a menu: any binary can be registered
    // whether or not it appears here (ADR 0002). With none detected the field
    // falls back to a single generic example instead.
    detected?: string[];
  } = $props();

  // The hint under the adapter field: the CLIs actually on PATH, or one generic
  // example when the probe found none. Kept alongside the input, so it survives
  // while the operator types (spec, Onboarding).
  const adapterHint = $derived(
    detected.length
      ? `detected adapters: ${detected.join(", ")}`
      : "e.g. claude, codex, opencode, pi",
  );

  // One agent being edited, or a fresh registration. Only one at a time: this is
  // a library, not a form-heavy admin screen.
  type Draft = {
    // The original name, empty for a new registration. Renaming would strand
    // every assignment pointing at the old name, so a registered agent's name is
    // fixed — register another and reassign.
    original: string;
    name: string;
    adapter: string;
    // 'default' leaves the adapter's own delivery in force; 'flag' reveals the
    // flag-name input beside it.
    delivery: "default" | "argv" | "type" | "flag";
    flag: string;
    // The args as one editable line. It is parsed into the list the wire wants
    // only on save (args.ts), so what the operator typed stays theirs while they
    // are typing it — including the spacing.
    args: string;
    // The environment as one editable line of `KEY=VALUE` entries, split the same
    // way args are. It is what a shell prefix expresses —
    // `CLAUDE_CONFIG_DIR=~/.claude2 claude` — and the field the operator already
    // has the muscle memory for.
    env: string;
  };
  let draft = $state<Draft | null>(null);
  let busy = $state<string | null>(null);
  let confirmingDelete = $state<string | null>(null);

  const deliveryLabels: Record<Draft["delivery"], string> = {
    default: "the adapter's default",
    argv: "argv — a trailing argument",
    type: "type — keystrokes into the TUI",
    flag: "a named flag",
  };

  function blank(): Draft {
    return {
      original: "",
      name: "",
      adapter: "",
      delivery: "default",
      flag: "",
      args: "",
      env: "",
    };
  }

  function toDraft(a: Agent): Draft {
    const p = a.prompt ?? "";
    return {
      original: a.name,
      name: a.name,
      adapter: a.adapter,
      delivery:
        p === ""
          ? "default"
          : p === "argv"
            ? "argv"
            : p === "type"
              ? "type"
              : "flag",
      flag: p.startsWith("-") ? p : "",
      args: formatArgs(a.args),
      // `env` arrives as typed, not as resolved, so editing an agent and saving it
      // cannot quietly replace the operator's `~/.claude2` with one machine's
      // absolute path. The expansion is visible in the command preview instead.
      env: formatArgs(a.env),
    };
  }

  // The `prompt` value a draft writes: empty means "the adapter's default", which
  // is what nearly every agent wants and what keeps the file free of a value that
  // only restates the built-in.
  function draftPrompt(d: Draft): string {
    return d.delivery === "default"
      ? ""
      : d.delivery === "flag"
        ? d.flag.trim()
        : d.delivery;
  }

  async function save() {
    if (!draft) return;
    const d = draft;
    busy = "save";
    try {
      await setAgent(d.name.trim(), {
        adapter: d.adapter.trim(),
        args: parseArgs(d.args),
        env: parseArgs(d.env),
        prompt: draftPrompt(d),
      });
      draft = null;
    } catch (e) {
      // A refusal surfaces as a toast — sonner's z-index clears the modal's own
      // backdrop (app.css), so it stands over the still-open register form rather
      // than behind it.
      toast.error((e as Error).message);
    } finally {
      busy = null;
    }
  }

  async function remove(name: string) {
    busy = name;
    try {
      await deleteAgent(name);
      confirmingDelete = null;
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      busy = null;
    }
  }

  function closeDraft() {
    draft = null;
  }
</script>

<section class="flex flex-col gap-2">
  <div class="flex items-baseline justify-between gap-2">
    <h2 class="text-xs font-semibold">Agents</h2>
    {#if !draft}
      <Button
        variant="outline"
        onclick={() => (draft = blank())}
      >
        <Plus /> New Agent
      </Button>
    {/if}
  </div>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Set up your agents with commands and flags.
  </p>

  {#if draft}
    <Modal
      open
      wide
      title={draft.original ? "Edit agent" : "Register new agent"}
      onClose={closeDraft}
    >
      <div class="flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          {@render textField(
            "name",
            draft.name,
            (v) => (draft!.name = v),
            "claude-opus-4-8",
            draft.original !== "",
          )}
          {@render textField(
            "adapter",
            draft.adapter,
            (v) => (draft!.adapter = v),
            "claude",
          )}
          <!-- The PATH probe's suggestions live here, beside the input, not in its
               placeholder — a placeholder disappears on the first keystroke, exactly
               when the list is most useful (spec, Onboarding). A hint, never a menu. -->
          <p class="-mt-0.5 pl-[3.875rem] text-[0.7rem] text-muted-foreground">
            {adapterHint}
          </p>

          <div class="flex items-center gap-1.5">
            <span
              class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
              >prompt</span
            >
            <Select.Root type="single" bind:value={draft.delivery}>
              <Select.Trigger
                class="h-7 min-w-0 flex-1 text-xs"
                aria-label="Prompt delivery"
              >
                {deliveryLabels[draft.delivery]}
              </Select.Trigger>
              <Select.Content>
                {#each Object.entries(deliveryLabels) as [value, label] (value)}
                  <Select.Item {value} class="text-xs">{label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
            {#if draft.delivery === "flag"}
              <Input
                class="h-7 w-32 font-mono text-xs"
                value={draft.flag}
                oninput={(e: Event) =>
                  (draft!.flag = (e.currentTarget as HTMLInputElement).value)}
                spellcheck="false"
                autocapitalize="off"
                autocomplete="off"
                aria-label="Prompt flag"
                placeholder="--prompt"
              />
            {/if}
          </div>

          {@render textField(
            "args",
            draft.args,
            (v) => (draft!.args = v),
            "--model claude-opus-4-8 --dangerously-skip-permissions",
          )}

          {@render textField(
            "env",
            draft.env,
            (v) => (draft!.env = v),
            "CLAUDE_CONFIG_DIR=~/.claude",
          )}
          <!-- The tilde note sits beside the field, like the adapter's PATH hint and
               for the same reason: nothing but a shell expands a `~`, and there is no
               shell here, so this one interpretation is worth saying where it is being
               typed rather than leaving it to be discovered as a stray directory. -->
          <p class="-mt-0.5 pl-[3.875rem] text-[0.7rem] text-muted-foreground">
            set before the binary runs.
          </p>
        </div>

        <div class="flex items-center justify-end gap-1.5">
          <Button
            variant="default"
            size="xs"
            disabled={busy !== null}
            onclick={save}>save</Button
          >
          <Button variant="ghost" size="xs" onclick={closeDraft}>cancel</Button>
        </div>
      </div>
    </Modal>
  {/if}

  {#if agents.length}
    <div class="overflow-hidden rounded-md border border-border">
      <Table.Root class="table-fixed">
        <Table.Header>
          <Table.Row class="hover:bg-transparent">
            <Table.Head>Name</Table.Head>
            <Table.Head>Adapter</Table.Head>
            <Table.Head class="w-20 text-center">Actions</Table.Head>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each agents as a (a.name)}
            <Table.Row>
              <Table.Cell class="truncate font-mono">
                {a.name}
              </Table.Cell>
              <Table.Cell class="truncate font-mono text-muted-foreground">
                {a.adapter}
              </Table.Cell>
              <Table.Cell>
                <div class="flex items-center justify-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    aria-label="Edit {a.name}"
                    onclick={() => (draft = toDraft(a))}
                  >
                    <PencilSimple />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    aria-label="Delete {a.name}"
                    disabled={busy !== null}
                    onclick={() =>
                      (confirmingDelete =
                        confirmingDelete === a.name ? null : a.name)}
                  >
                    <Trash />
                  </Button>
                </div>
              </Table.Cell>
            </Table.Row>
            {#if confirmingDelete === a.name}
              <Table.Row class="bg-muted/50 hover:bg-muted/50">
                <Table.Cell colspan={3} class="whitespace-normal">
                  <div
                    class="flex flex-wrap items-center justify-between gap-3"
                  >
                    <p class="text-[0.7rem]">
                      Delete {a.name}?
                    </p>
                    <div class="flex shrink-0 items-center gap-1.5">
                      <Button
                        variant="destructive"
                        size="xs"
                        disabled={busy !== null}
                        onclick={() => remove(a.name)}
                      >
                        delete
                      </Button>
                      <Button
                        variant="ghost"
                        size="xs"
                        onclick={() => (confirmingDelete = null)}
                      >
                        cancel
                      </Button>
                    </div>
                  </div>
                </Table.Cell>
              </Table.Row>
            {/if}
          {/each}
        </Table.Body>
      </Table.Root>
    </div>
  {:else if !draft}
    <div
      class="flex min-h-16 items-center justify-center rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-center text-xs text-muted-foreground"
    >
      No registered agents.
    </div>
  {/if}
</section>

{#snippet textField(
  label: string,
  value: string,
  set: (v: string) => void,
  placeholder: string,
  locked = false,
)}
  <div class="flex items-center gap-1.5">
    <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
      >{label}</span
    >
    {#if locked}
      <span class="min-w-0 flex-1 truncate px-1 font-mono text-xs">{value}</span
      >
      <span class="text-[0.7rem] text-muted-foreground">
        names are fixed — assignments point at them
      </span>
    {:else}
      <Input
        class="h-7 min-w-0 flex-1 font-mono text-xs"
        {value}
        oninput={(e: Event) => set((e.currentTarget as HTMLInputElement).value)}
        spellcheck="false"
        autocapitalize="off"
        autocomplete="off"
        aria-label={label}
        {placeholder}
      />
    {/if}
  </div>
{/snippet}
