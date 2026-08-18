<script lang="ts">
  import type { Prompt, Terminal } from "./model";
  import {
    cancelPrompt,
    createPrompt,
    deletePrompt,
    sendPrompt,
    setSpacePrompts,
    updatePrompt,
  } from "./actions";
  import { promptTarget } from "./prompttarget";
  import { Button } from "./components/ui/button";
  import { Switch } from "./components/ui/switch";
  import { Input } from "./components/ui/input";
  import { Textarea } from "./components/ui/textarea";
  import { toast } from "./components/ui/sonner";
  import * as ScrollArea from "./components/ui/scroll-area";
  import * as Tooltip from "./components/ui/tooltip";
  import { PencilSimple, Plus, Trash } from "phosphor-svelte";

  // The Prompts pane (prompt-presets ticket 04): the operator's global catalog of
  // short behavioural presets, with the two things a space does with one — apply
  // it at launch, or hand it to the agent that is running right now.
  //
  // It holds no catalog of its own. Every control posts an action and the result
  // arrives as a fresh snapshot over the control socket, so what is rendered is
  // always what the server holds — including the `At launch` toggles, which is
  // why a refused write leaves the row exactly as it was rather than half-toggled.
  //
  // The target is always the space's active tab; picking another means selecting
  // it in the sidebar, which is the existing session navigation and the only
  // target picker there is.
  let {
    prompts,
    spaceId,
    selected,
    activeTerm,
  }: {
    // The catalog in creation order — global, the same list in every space.
    prompts: Prompt[];
    spaceId: string;
    // This space's `At launch` ids, already in catalog order (the server resolves
    // them; a selected id the catalog no longer holds is absent here and named in
    // the space's warnings instead).
    selected: string[];
    activeTerm: Terminal | null;
  } = $props();

  const target = $derived(promptTarget(activeTerm));
  // The one preset this tab is holding for its next observed idle, if any. Only
  // one is ever pending, so every other row's action stands down while it is:
  // the server refuses a second activation, and a button that always fails is
  // worse than one that says why it is inert.
  const pending = $derived(activeTerm?.pendingPrompt ?? null);

  // One preset being edited, or a fresh one. Only one at a time — this is a short
  // list of short texts, not a form-heavy surface. `id` null is a creation.
  type Draft = { id: string | null; name: string; body: string };
  let draft = $state<Draft | null>(null);
  // The id of the row with an action in flight (or `new` for the creation form),
  // so exactly the control that was clicked goes inert.
  let busy = $state<string | null>(null);
  let confirmingDelete = $state<string | null>(null);

  async function run(key: string, action: () => Promise<unknown>) {
    if (busy) return;
    busy = key;
    try {
      await action();
      return true;
    } catch (e) {
      toast.error((e as Error).message);
      return false;
    } finally {
      busy = null;
    }
  }

  // `At launch` is written as the whole list in catalog order, never as a
  // per-row toggle: one write is one selection, so two quick clicks cannot
  // interleave into a half-set list.
  function toggleLaunch(id: string, on: boolean) {
    const wanted = new Set(selected);
    if (on) wanted.add(id);
    else wanted.delete(id);
    const ids = prompts.filter((p) => wanted.has(p.id)).map((p) => p.id);
    run(id, () => setSpacePrompts(spaceId, ids));
  }

  async function saveDraft() {
    const d = draft;
    if (!d || !d.name.trim() || !d.body.trim()) return;
    const ok = await run(d.id ?? "new", () =>
      d.id
        ? updatePrompt(d.id, d.name.trim(), d.body.trim())
        : createPrompt(d.name.trim(), d.body.trim()),
    );
    if (ok) draft = null;
  }

  async function remove(id: string) {
    const ok = await run(id, () => deletePrompt(id));
    if (ok) confirmingDelete = null;
  }

  // Send and Queue are one action: the server types an idle agent at once and
  // holds it for a busy one, and says which it did. The pane only has to name
  // the outcome the operator is about to get.
  function deliver(id: string) {
    if (!activeTerm) return;
    run(id, () => sendPrompt(spaceId, activeTerm.id, id));
  }

  function cancel() {
    if (!activeTerm) return;
    run("cancel", () => cancelPrompt(spaceId, activeTerm.id));
  }
</script>

{#snippet form()}
  {#if draft}
    <div class="flex flex-col gap-2 border-b border-border bg-muted/30 p-3">
      <label class="flex flex-col gap-1 text-[0.7rem] text-muted-foreground">
        Name
        <Input
          bind:value={draft.name}
          placeholder="Brief answers"
          autocomplete="off"
        />
      </label>
      <label class="flex flex-col gap-1 text-[0.7rem] text-muted-foreground">
        Prompt
        <Textarea
          bind:value={draft.body}
          rows={4}
          placeholder="Keep answers short and skip the preamble."
        />
      </label>
      <div class="flex items-center justify-end gap-2">
        <Button variant="ghost" size="sm" onclick={() => (draft = null)}>
          Cancel
        </Button>
        <Button
          size="sm"
          disabled={busy !== null || !draft.name.trim() || !draft.body.trim()}
          onclick={saveDraft}
        >
          Save
        </Button>
      </div>
    </div>
  {/if}
{/snippet}

<div class="flex min-h-0 flex-1 flex-col">
  <!-- What this pane will act on, in one line. An ineligible tab gets the plain
       explanation here rather than a refusal repeated on every row. -->
  <p
    class="border-b border-border px-3 py-2 text-[0.7rem] text-muted-foreground"
  >
    {#if target.kind === "ineligible"}
      {target.reason}
    {:else}
      Targeting <span class="text-foreground">{activeTerm?.title}</span> — the
      active tab.
    {/if}
  </p>

  <ScrollArea.Root type="auto" class="min-h-0 flex-1">
    {#if draft && draft.id === null}
      {@render form()}
    {/if}

    {#if prompts.length}
      <ul>
        {#each prompts as p (p.id)}
          <li class="border-b border-border last:border-b-0">
            {#if draft && draft.id === p.id}
              {@render form()}
            {:else}
              <!-- One row, one line: the launch toggle, the name (its prompt
                   body lives in the hover tooltip rather than on the row), then
                   the delivery and edit controls. -->
              <div class="flex items-center gap-2 py-1.5 pr-2 pl-3">
                <Tooltip.Root>
                  <Tooltip.Trigger>
                    {#snippet child({ props })}
                      <Switch
                        {...props}
                        checked={selected.includes(p.id)}
                        disabled={busy !== null}
                        aria-label="Apply {p.name} at launch in this space"
                        onCheckedChange={(v: boolean) => toggleLaunch(p.id, v)}
                      />
                    {/snippet}
                  </Tooltip.Trigger>
                  <Tooltip.Content>Apply at launch in this space</Tooltip.Content>
                </Tooltip.Root>

                <Tooltip.Root>
                  <Tooltip.Trigger
                    class="min-w-0 flex-1 truncate text-left text-xs font-medium outline-none focus-visible:underline"
                  >
                    {p.name}
                  </Tooltip.Trigger>
                  <Tooltip.Content class="max-w-xs">
                    <span class="whitespace-pre-wrap">{p.body}</span>
                  </Tooltip.Content>
                </Tooltip.Root>

                {#if pending === p.id}
                  <span class="shrink-0 text-[0.7rem] text-muted-foreground">
                    Queued
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={busy !== null}
                    onclick={cancel}
                  >
                    Cancel
                  </Button>
                {:else if target.kind !== "ineligible"}
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={busy !== null || pending !== null}
                    title={pending
                      ? "Another preset is already queued for this tab"
                      : target.kind === "send"
                        ? "Send this preset to the active agent now"
                        : "Hold this preset for the agent's next idle"}
                    onclick={() => deliver(p.id)}
                  >
                    {target.kind === "send" ? "Send" : "Queue"}
                  </Button>
                {/if}

                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label="Edit {p.name}"
                  disabled={busy !== null}
                  onclick={() => (draft = { id: p.id, name: p.name, body: p.body })}
                >
                  <PencilSimple />
                </Button>
                {#if confirmingDelete === p.id}
                  <Button
                    variant="destructive"
                    size="sm"
                    disabled={busy !== null}
                    onclick={() => remove(p.id)}
                  >
                    Delete
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={() => (confirmingDelete = null)}
                  >
                    Keep
                  </Button>
                {:else}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label="Delete {p.name}"
                    disabled={busy !== null}
                    onclick={() => (confirmingDelete = p.id)}
                  >
                    <Trash />
                  </Button>
                {/if}
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {:else if !draft}
      <div class="grid h-full place-items-center p-6">
        <p class="max-w-xs text-center text-xs text-muted-foreground">
          No presets yet — a preset is a short standing instruction, applied at
          launch or sent to a running agent.
        </p>
      </div>
    {/if}
  </ScrollArea.Root>

  <div class="flex items-center border-t border-border px-3 py-2">
    <Button
      variant="ghost"
      size="sm"
      disabled={draft !== null || busy !== null}
      onclick={() => (draft = { id: null, name: "", body: "" })}
    >
      <Plus /> New preset
    </Button>
  </div>
</div>
