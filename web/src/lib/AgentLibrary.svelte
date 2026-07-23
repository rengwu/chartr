<script lang="ts">
  import type { Agent } from './model'
  import { setAgent, deleteAgent } from './actions'
  import { formatArgs, parseArgs } from './args'
  import { Button } from './components/ui/button'
  import { Badge } from './components/ui/badge'
  import { Input } from './components/ui/input'
  import * as Select from './components/ui/select'
  import { CheckCircle, Plus, Trash, Warning, X } from 'phosphor-svelte'

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
  // flags are an opaque list the operator types. The command preview under each
  // agent is the honest substitute for a curated form — it is built server-side
  // by the same seam that builds the real argv, so what you read is what runs.
  let {
    agents,
    detected = [],
    assignmentsOf,
  }: {
    agents: Agent[]
    // The known agent CLIs found on this machine's PATH (ticket 04) — an advisory
    // hint, rendered as helper text beneath the adapter input rather than as a
    // placeholder, which would vanish on the first keystroke exactly when the list
    // is most useful. A suggestion, never a menu: any binary can be registered
    // whether or not it appears here (ADR 0002). With none detected the field
    // falls back to a single generic example instead.
    detected?: string[]
    // What each agent is currently assigned to, for the delete confirm — a
    // library edit should never silently strand a role.
    assignmentsOf: (name: string) => string[]
  } = $props()

  // The hint under the adapter field: the CLIs actually on PATH, or one generic
  // example when the probe found none. Kept alongside the input, so it survives
  // while the operator types (spec, Onboarding).
  const adapterHint = $derived(
    detected.length
      ? `on your PATH: ${detected.join(', ')} — or any other binary you run`
      : 'e.g. claude — or any other agent CLI on your PATH',
  )

  // One agent being edited, or a fresh registration. Only one at a time: this is
  // a library, not a form-heavy admin screen.
  type Draft = {
    // The original name, empty for a new registration. Renaming would strand
    // every assignment pointing at the old name, so a registered agent's name is
    // fixed — register another and reassign.
    original: string
    name: string
    adapter: string
    // 'default' leaves the adapter's own delivery in force; 'flag' reveals the
    // flag-name input beside it.
    delivery: 'default' | 'argv' | 'type' | 'flag'
    flag: string
    // The args as one editable line. It is parsed into the list the wire wants
    // only on save (args.ts), so what the operator typed stays theirs while they
    // are typing it — including the spacing.
    args: string
  }
  let draft = $state<Draft | null>(null)
  let busy = $state<string | null>(null)
  let note = $state<string | null>(null)
  let confirmingDelete = $state<string | null>(null)

  const deliveryLabels: Record<Draft['delivery'], string> = {
    default: "the adapter's default",
    argv: 'argv — a trailing argument',
    type: 'type — keystrokes into the TUI',
    flag: 'a named flag',
  }

  function blank(): Draft {
    return { original: '', name: '', adapter: '', delivery: 'default', flag: '', args: '' }
  }

  function toDraft(a: Agent): Draft {
    const p = a.prompt ?? ''
    return {
      original: a.name,
      name: a.name,
      adapter: a.adapter,
      delivery: p === '' ? 'default' : p === 'argv' ? 'argv' : p === 'type' ? 'type' : 'flag',
      flag: p.startsWith('-') ? p : '',
      args: formatArgs(a.args),
    }
  }

  // The `prompt` value a draft writes: empty means "the adapter's default", which
  // is what nearly every agent wants and what keeps the file free of a value that
  // only restates the built-in.
  function draftPrompt(d: Draft): string {
    return d.delivery === 'default' ? '' : d.delivery === 'flag' ? d.flag.trim() : d.delivery
  }

  async function save() {
    if (!draft) return
    const d = draft
    busy = 'save'
    note = null
    try {
      await setAgent(d.name.trim(), {
        adapter: d.adapter.trim(),
        args: parseArgs(d.args),
        prompt: draftPrompt(d),
      })
      draft = null
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }

  async function remove(name: string) {
    busy = name
    note = null
    try {
      const r = await deleteAgent(name)
      confirmingDelete = null
      if (r.assigned?.length) {
        note = `${name} is gone; ${r.assigned.join(', ')} fell back to their own fields.`
      }
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }
</script>

<section class="flex flex-col gap-2">
  <div class="flex items-baseline justify-between gap-2">
    <h2 class="text-xs font-semibold">Agents</h2>
    {#if !draft}
      <Button variant="ghost" size="xs" onclick={() => (draft = blank())}>
        <Plus /> register an agent
      </Button>
    {/if}
  </div>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Named ways to run a harness — the binary, whatever flags it takes, and how it receives its
    opening prompt. Flags are yours to type, model included: nothing here guesses what a given CLI
    calls anything. Registered on your machine and never committed, so a permission-skipping agent
    is something you grant yourself, not something a pull can hand you. Assign one to a role on any
    space.
  </p>

  {#if note}
    <p class="rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs">{note}</p>
  {/if}

  {#if draft}
    <div class="flex flex-col gap-2.5 rounded-md border border-ring p-2.5">
      <div class="flex items-center justify-between gap-2">
        <span class="text-xs font-semibold">
          {draft.original ? `Edit ${draft.original}` : 'Register an agent'}
        </span>
        <Button variant="ghost" size="icon-xs" aria-label="Cancel" onclick={() => (draft = null)}>
          <X />
        </Button>
      </div>

      <div class="flex flex-col gap-1.5">
        {@render textField(
          'name',
          draft.name,
          (v) => (draft!.name = v),
          'claude-yolo',
          draft.original !== '',
        )}
        {@render textField('adapter', draft.adapter, (v) => (draft!.adapter = v), 'the CLI to run')}
        <!-- The PATH probe's suggestions live here, beside the input, not in its
             placeholder — a placeholder disappears on the first keystroke, exactly
             when the list is most useful (spec, Onboarding). A hint, never a menu. -->
        <p class="-mt-0.5 pl-[3.875rem] text-[0.7rem] text-muted-foreground">{adapterHint}</p>

        <div class="flex items-center gap-1.5">
          <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">prompt</span>
          <Select.Root type="single" bind:value={draft.delivery}>
            <Select.Trigger class="h-7 min-w-0 flex-1 text-xs" aria-label="Prompt delivery">
              {deliveryLabels[draft.delivery]}
            </Select.Trigger>
            <Select.Content>
              {#each Object.entries(deliveryLabels) as [value, label] (value)}
                <Select.Item {value} class="text-xs">{label}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
          {#if draft.delivery === 'flag'}
            <Input
              class="h-7 w-32 font-mono text-xs"
              value={draft.flag}
              oninput={(e: Event) => (draft!.flag = (e.currentTarget as HTMLInputElement).value)}
              spellcheck="false"
              autocapitalize="off"
              autocomplete="off"
              aria-label="Prompt flag"
              placeholder="--prompt"
            />
          {/if}
        </div>

        {@render textField(
          'args',
          draft.args,
          (v) => (draft!.args = v),
          '--model sonnet --dangerously-skip-permissions',
        )}
      </div>

      <div class="flex items-center gap-1.5">
        <Button variant="default" size="xs" disabled={busy !== null} onclick={save}>save</Button>
        <Button variant="ghost" size="xs" onclick={() => (draft = null)}>cancel</Button>
        <span class="text-[0.7rem] text-muted-foreground">
          Args split on spaces; quote one that contains any.
        </span>
      </div>
    </div>
  {/if}

  {#if agents.length}
    <ul class="flex flex-col gap-1.5">
      {#each agents as a (a.name)}
        <li class="rounded-md border border-border p-2.5">
          <div class="flex items-center justify-between gap-2">
            <span class="min-w-0 truncate font-mono text-xs font-semibold">{a.name}</span>
            <span class="flex shrink-0 items-center gap-1.5">
              {#if a.present}
                <span class="flex items-center gap-1 text-[0.7rem] text-muted-foreground">
                  <CheckCircle class="size-3.5" /> on PATH
                </span>
              {:else}
                <Badge variant="destructive" class="gap-1"><Warning /> not found</Badge>
              {/if}
              <Button variant="ghost" size="xs" onclick={() => ((draft = toDraft(a)), (note = null))}>
                edit
              </Button>
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label="Delete {a.name}"
                disabled={busy !== null}
                onclick={() => (confirmingDelete = confirmingDelete === a.name ? null : a.name)}
              >
                <Trash />
              </Button>
            </span>
          </div>

          <!-- What will actually run. Built by the same seam as the real launch,
               so the preview cannot drift from the spawn. -->
          <p class="mt-1.5 truncate font-mono text-[0.7rem] text-muted-foreground" title={a.command.join(' ')}>
            {a.command.join(' ')}
          </p>
          <p class="mt-0.5 text-[0.7rem] text-muted-foreground">
            {#if a.delivery === 'type'}
              the opener is typed into its TUI
            {:else if a.delivery === 'argv'}
              the opener rides its command line, already submitted
            {:else}
              the opener rides <span class="font-mono">{a.delivery}</span>
            {/if}
          </p>
          {#if !a.present && a.missing}
            <p class="mt-1 text-[0.7rem] text-muted-foreground">{a.missing}</p>
          {/if}

          {#if confirmingDelete === a.name}
            {@const assigned = assignmentsOf(a.name)}
            <div class="mt-2 flex flex-col gap-1.5 rounded-md border border-border bg-muted/50 p-2">
              <p class="text-[0.7rem]">
                {#if assigned.length}
                  {assigned.join(', ')}
                  {assigned.length === 1 ? 'is' : 'are'} assigned to {a.name} and will fall back to
                  their own fields.
                {:else}
                  No role is assigned to {a.name}.
                {/if}
              </p>
              <div class="flex items-center gap-1.5">
                <Button
                  variant="destructive"
                  size="xs"
                  disabled={busy !== null}
                  onclick={() => remove(a.name)}
                >
                  delete
                </Button>
                <Button variant="ghost" size="xs" onclick={() => (confirmingDelete = null)}>
                  cancel
                </Button>
              </div>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {:else if !draft}
    <p class="text-xs text-muted-foreground">
      No agents registered yet. Register one to start spawning — every session, and ideate, picks an
      agent from this library, and there is no default.
    </p>
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
    <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">{label}</span>
    {#if locked}
      <span class="min-w-0 flex-1 truncate px-1 font-mono text-xs">{value}</span>
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
