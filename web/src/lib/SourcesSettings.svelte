<script lang="ts">
  import type { RoleBinding, Source } from "./model";
  import {
    ActionError,
    openSkill,
    refreshSource,
    registerSource,
    removeSource,
    reorderSources,
    setRoleBinding,
    setSourceEnabled,
  } from "./actions";
  import { openExternal } from "./external";
  import { Button } from "./components/ui/button";
  import { toast } from "./components/ui/sonner";
  import { Badge } from "./components/ui/badge";
  import { Input } from "./components/ui/input";
  import { Checkbox } from "./components/ui/checkbox";
  import * as Select from "./components/ui/select";
  import * as Table from "./components/ui/table";
  import Modal from "./Modal.svelte";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import {
    ArrowsClockwise,
    DotsSixVertical,
    LinkSimple,
    Plus,
    Trash,
  } from "phosphor-svelte";

  // The sources section (ticket 10): the operator's ordered list of places skills
  // come from, and the five actions the planning tickets designed for it —
  // register, remove, toggle, reorder, and refresh a git source.
  //
  // It is load-bearing rather than a convenience. It is where an orphaned git
  // checkout is explained, where an unbound role is shown refusing to spawn, and
  // the thing the silent first-run migration is discoverable *in* — a source that
  // was registered before the operator ever opened this screen appears here as an
  // ordinary row, which is the whole of what "quiet, not hidden" means.
  //
  // The editable/open-the-file line is ADR 0014's: the five actions are inline,
  // and everything else is the operator's own editor on the file itself. There is
  // never a second config store.
  let {
    sources,
    roles,
    gitAvailable,
  }: {
    // In resolution order, exactly as the server holds it — never sorted here.
    sources: Source[];
    roles: RoleBinding[];
    // Whether `git` is on PATH. A git registration is refused at the gate when it
    // is not, so the form says so up front rather than after a failed clone.
    gitAvailable: boolean;
  } = $props();

  let busy = $state<string | null>(null);
  // The row awaiting a second click before it is removed. Registering is cheap to
  // undo, removing a git source is not — the checkout stays on disk but the URL
  // and pin do not, so it asks twice.
  let confirming = $state<string | null>(null);

  type Draft = {
    name: string;
    kind: "dir" | "git";
    path: string;
    url: string;
    ref: string;
  };
  let draft = $state<Draft | null>(null);

  // The registration form opens blank — chartr's own skills repo is pre-registered
  // on a fresh install now (it lands as an ordinary row), so there is no default to
  // ride in the `url` field here.
  const kindLabels: Record<Draft["kind"], string> = {
    dir: "folder on this machine",
    git: "git repository",
  };

  // Every source is the operator's own now — chartr ships none (ADR 0017) — so
  // the whole list reorders. Position is resolution order; a drag sends it whole.
  //
  // The reorder gesture is svelte-dnd-action, the same library the sidebar's
  // space cards use: each row is a drag item keyed by name, the library reorders
  // `dndItems` in place and hands it back on `consider` (live, during the drag)
  // and `finalize` (the drop). Only the drop commits. `dndItems` mirrors the
  // `sources` prop except while a drag is in flight or a committed order is still
  // awaiting the snapshot that confirms it (`pendingOrder`), so the drop holds on
  // screen instead of bouncing back to the old order for a frame.
  const reduceMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;
  const flipDurationMs = reduceMotion ? 0 : 120;

  // One column template shared by the header and every row, so the two grids line
  // up: drag-handle · enable · name · type · skills · actions. Only `name` flexes.
  const gridCols =
    "grid grid-cols-[1.25rem_1.25rem_minmax(0,1fr)_4rem_3.5rem_4.25rem] items-center gap-2";

  // svelte-dnd-action keys each item by an `id` property (that is how the sidebar's
  // space cards work — a Space carries one). A Source is keyed by its name, so the
  // drag items carry a synthetic `id` mirroring it; without it the zone never
  // initializes and the rows cannot be picked up at all.
  type DndSource = Source & { id: string };
  let dndItems = $state<DndSource[]>([]);
  let dndDragging = $state(false);
  let pendingOrder = $state<string[] | null>(null);

  // Order a source list by a name sequence — used to project a committed order
  // onto whatever snapshot is in hand, old or new.
  function orderByNames(list: Source[], order: string[]): Source[] {
    const at = new Map(order.map((n, i) => [n, i]));
    return [...list].sort(
      (a, b) => (at.get(a.name) ?? 0) - (at.get(b.name) ?? 0),
    );
  }

  $effect(() => {
    const current = sources;
    // The library owns the list while the pointer is down.
    if (dndDragging) return;
    const base = pendingOrder ? orderByNames(current, pendingOrder) : current;
    dndItems = base.map((s) => ({ ...s, id: s.name }));
    // Cleared once the server's own order matches what we committed.
    if (
      pendingOrder &&
      current.map((s) => s.name).join("\n") === pendingOrder.join("\n")
    )
      pendingOrder = null;
  });

  function handleDndConsider(e: CustomEvent<DndEvent<DndSource>>) {
    dndDragging = true;
    dndItems = e.detail.items;
  }
  function handleDndFinalize(e: CustomEvent<DndEvent<DndSource>>) {
    dndItems = e.detail.items;
    dndDragging = false;
    const names = e.detail.items.map((s) => s.name);
    // A drop back where it started writes nothing.
    if (names.join("\n") === sources.map((s) => s.name).join("\n")) return;
    pendingOrder = names;
    void commitOrder(names);
  }

  async function commitOrder(names: string[]) {
    try {
      await reorderSources(names);
    } catch (e) {
      // The write failed, so the held order is a fiction — drop it and let the
      // next snapshot restore the server's truth.
      pendingOrder = null;
      toast.error(e instanceof ActionError ? e.message : (e as Error).message);
    }
  }

  // Every source action's outcome surfaces as a toast now (App.svelte's Toaster),
  // centered over the main panel — the refusal that used to sit in an inline note
  // line, and the "Refreshed X." confirmation alike.
  async function run(key: string, fn: () => Promise<unknown>, ok?: string) {
    busy = key;
    try {
      await fn();
      if (ok) toast.success(ok);
    } catch (e) {
      toast.error(e instanceof ActionError ? e.message : (e as Error).message);
    } finally {
      busy = null;
      confirming = null;
    }
  }

  // The sentinel a role carries for "no preference" — resolve by source
  // precedence rather than pinning one source's skill. Mirrors config.RoleBindingAuto.
  const AUTO = "auto";

  // What the picker's trigger shows for a role's current binding: "no preference"
  // for the auto sentinel, the qualified ref when one is pinned, and a plain
  // prompt when the role is unbound and refusing to spawn.
  function roleLabel(b: RoleBinding): string {
    if (b.ref === AUTO) return "no preference";
    if (!b.ref) return "choose a skill…";
    return b.ref;
  }

  // The value the Select is controlled by. An unbound role matches no option, so
  // the trigger falls back to its placeholder until a choice is made.
  function roleValue(b: RoleBinding): string | undefined {
    return b.ref || undefined;
  }

  function chooseRole(role: string, ref: string) {
    void run(`role:${role}`, () => setRoleBinding(role, ref));
  }

  // The Resolved column's link button: a dir source's skill is revealed in
  // Finder/Explorer server-side, nothing further to do here; a git source's
  // skill has no local file worth revealing (it lives in a checkout chartr
  // resets on refresh), so the server hands back the repository's own URL and
  // this is what actually opens it — the same outbound-link seam every other
  // link in the cockpit goes through.
  function openSkillLink(ref: string) {
    void run(`open:${ref}`, async () => {
      const r = await openSkill(ref);
      if (r.kind === "remote") {
        openExternal(r.url);
      } else if (r.opened === "none") {
        throw new Error(`Nothing to reveal it with — it lives at ${r.path}`);
      }
    });
  }

  function submit(e: Event) {
    e.preventDefault();
    if (!draft) return;
    const d = draft;
    void run("register", async () => {
      await registerSource({
        name: d.name,
        kind: d.kind,
        path: d.path,
        url: d.url,
        ref: d.ref,
      });
      draft = null;
    });
  }
</script>

<section class="flex flex-col gap-2">
  <div class="flex items-baseline justify-between gap-2">
    <h2 class="text-xs font-semibold">Skill sources</h2>
    <div class="flex items-center gap-1">
      {#if !draft}
        <Button
          variant="outline"
          onclick={() =>
            (draft = {
              name: "",
              kind: "dir",
              path: "",
              url: "",
              ref: "",
            })}
        >
          <Plus /> New Source
        </Button>
      {/if}
    </div>
  </div>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Registered skill sources, exposed to agents via <code>CHARTR.md</code>.
    Precedence matters for skill names that exists in more than one source.
  </p>

  {#if draft}
    <Modal open wide title="Register a source" onClose={() => (draft = null)}>
      <form class="flex flex-col gap-2.5" onsubmit={submit}>
        <div class="flex items-center gap-1.5">
          <span
            class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
            >name</span
          >
          <Input
            class="h-7 min-w-0 flex-1 text-xs"
            value={draft.name}
            placeholder="my skills"
            oninput={(e: Event) =>
              (draft!.name = (e.currentTarget as HTMLInputElement).value)}
          />
        </div>

        <div class="flex items-center gap-1.5">
          <span
            class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
            >kind</span
          >
          <Select.Root type="single" bind:value={draft.kind}>
            <Select.Trigger
              class="h-7 min-w-0 flex-1 text-xs"
              aria-label="Source kind"
            >
              {kindLabels[draft.kind]}
            </Select.Trigger>
            <Select.Content>
              {#each Object.entries(kindLabels) as [value, label] (value)}
                <Select.Item {value} class="text-xs">{label}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>

        {#if draft.kind === "dir"}
          <div class="flex items-center gap-1.5">
            <span
              class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
              >path</span
            >
            <Input
              class="h-7 min-w-0 flex-1 font-mono text-xs"
              value={draft.path}
              placeholder="~/skills"
              oninput={(e: Event) =>
                (draft!.path = (e.currentTarget as HTMLInputElement).value)}
            />
          </div>
          <p class="pl-[3.875rem] text-[0.7rem] text-muted-foreground">
            Your folder, edited by you — chartr only reads it. Absolute, or
            <code class="font-mono">~/</code> for your home directory.
          </p>
        {:else}
          <div class="flex items-center gap-1.5">
            <span
              class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
              >url</span
            >
            <Input
              class="h-7 min-w-0 flex-1 font-mono text-xs"
              value={draft.url}
              placeholder="https://github.com/someone/skills.git"
              oninput={(e: Event) =>
                (draft!.url = (e.currentTarget as HTMLInputElement).value)}
            />
          </div>
          <div class="flex items-center gap-1.5">
            <span
              class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground"
              >ref</span
            >
            <Input
              class="h-7 min-w-0 flex-1 font-mono text-xs"
              value={draft.ref}
              placeholder="the default branch"
              oninput={(e: Event) =>
                (draft!.ref = (e.currentTarget as HTMLInputElement).value)}
            />
          </div>
          <!-- The only assertion of trust in a git source's whole lifetime is the
               moment the URL is typed (ADR 0017); there is no confirm gate after
               it, so the consequence is said here, before the clone. -->
          <p
            class="pl-[3.875rem] text-[0.7rem] leading-relaxed text-muted-foreground"
          >
            chartr clones this into its own checkout under <code
              class="font-mono">sources/</code
            >
            and runs whatever skills it holds. Pasting the URL is the trust decision
            — nothing asks again.
            {#if !gitAvailable}
              <strong class="font-medium text-foreground">
                `git` is not on this machine's PATH, so this registration will
                be refused.
              </strong>
            {/if}
          </p>
        {/if}

        <div class="flex items-center justify-end gap-1.5">
          <Button type="submit" size="xs" disabled={busy !== null}>
            {busy === "register" ? "Registering…" : "Register"}
          </Button>
          <Button variant="ghost" size="xs" onclick={() => (draft = null)}>
            Cancel
          </Button>
        </div>
      </form>
    </Modal>
  {/if}

  <!-- The source list is a table now, in the shape of the agent library's: a
       header row of columns, and one plate per source. Position *is* resolution
       order, so the rows are drag-sortable with svelte-dnd-action — the same
       library the sidebar's space cards use. There are no up/down controls: the
       drag is the whole of the reorder, and the number that used to label a row's
       rank is simply its place in the list. -->
  {#if dndItems.length}
    <div class="overflow-hidden rounded-md border border-border">
      <div
        class="{gridCols} h-8 border-b border-border px-2.5 text-[0.65rem] font-medium text-muted-foreground"
      >
        <span></span>
        <span></span>
        <span>Name</span>
        <span>Type</span>
        <span class="text-right">Skills</span>
        <span class="text-right">Actions</span>
      </div>

      <div
        class="flex flex-col"
        use:dndzone={{
          items: dndItems,
          flipDurationMs,
          dropTargetStyle: {},
          useCursorForDetection: true,
          // A private type, so a row cannot be dragged out into another zone (the
          // sidebar's space list is one) and vanish — with no other zone of this
          // type, a drop anywhere off the list snaps the row back into it.
          type: "skill-sources",
        }}
        onconsider={handleDndConsider}
        onfinalize={handleDndFinalize}
      >
        <!-- Keyed by `id`, never `name`: svelte-dnd-action's drag placeholder is a
             spread of the dragged item that keeps every field *except* `id`, which
             it swaps for a private sentinel. Keying on that `id` is what lets Svelte
             mount a distinct node for the placeholder — key on any preserved field
             (like `name`) and the node is reused instead of swapped, which kills the
             live reflow and orphans the row on drop. -->
        {#each dndItems as s (s.id)}
          <div
            animate:flip={{ duration: flipDurationMs }}
            class="flex cursor-grab flex-col gap-1.5 border-b border-border px-2.5 py-2 select-none last:border-b-0"
          >
            <div class={gridCols}>
              <!-- The whole row is the drag item; the handle is only the
                   affordance that says so, the way the sidebar cards read as
                   grabbable without one. -->
              <span
                class="flex justify-center text-muted-foreground"
                aria-hidden="true"
              >
                <DotsSixVertical class="size-4" />
              </span>

              <Checkbox
                checked={s.enabled}
                aria-label="{s.enabled ? 'Disable' : 'Enable'} {s.name}"
                disabled={busy !== null}
                onCheckedChange={(v: boolean) =>
                  run(s.name, () => setSourceEnabled(s.name, v))}
              />

              <span class="flex min-w-0 flex-col">
                <span class="flex min-w-0 items-baseline gap-1.5">
                  <span
                    class="truncate text-xs font-medium"
                    class:text-muted-foreground={!s.enabled}
                  >
                    {s.name}
                  </span>
                  {#if s.status !== "ok"}
                    <Badge variant="outline">{s.status}</Badge>
                  {/if}
                  {#if s.shadowed?.length && s.shadowed.length === s.skills.length && s.skills.length > 0}
                    <Badge variant="outline">shadowed</Badge>
                  {/if}
                </span>
                <!-- A remote is identified by the URL the operator trusted, not
                     by the opaque checkout path chartr happens to clone it into;
                     a local folder is identified by its path, which is the folder
                     itself. -->
                <code
                  class="truncate font-mono text-[0.65rem] text-muted-foreground"
                  title={s.kind === "git" ? (s.url ?? s.path) : s.path}
                >
                  {s.kind === "git" ? (s.url ?? s.path) : s.path}
                </code>
              </span>

              <!-- Type: a git source is a *remote* the operator trusted a URL
                   into; a dir source is a *local* folder chartr only reads. -->
              <span class="text-[0.65rem] text-muted-foreground">
                {s.kind === "git" ? "remote" : "local"}
              </span>

              <span class="text-right text-[0.65rem] text-muted-foreground">
                {s.skills.length}
              </span>

              <div class="flex items-center justify-end gap-1">
                {#if s.kind === "git"}
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    aria-label="Refresh {s.name}"
                    title="Refetch the skill repository. Discards local changes, if any."
                    disabled={busy !== null}
                    onclick={() =>
                      run(
                        s.name,
                        () => refreshSource(s.name),
                        `Refreshed ${s.name}.`,
                      )}
                  >
                    <ArrowsClockwise />
                  </Button>
                {/if}
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="hover:text-destructive"
                  aria-label="Remove {s.name}"
                  title="Remove this source from the list"
                  disabled={busy !== null}
                  onclick={() =>
                    (confirming = confirming === s.name ? null : s.name)}
                >
                  <Trash />
                </Button>
              </div>
            </div>

            <!-- Removing a git source is not cheap to undo — the checkout stays
                 on disk but the URL and pin do not — so it asks once, in a
                 modal, rather than risking a stray click on a full-width row
                 among draggable siblings. -->
            {#if confirming === s.name}
              <Modal
                open
                title="Remove source"
                onClose={() => (confirming = null)}
              >
                <div class="flex flex-col gap-3">
                  <p class="text-xs">Remove {s.name} from the list?</p>
                  <div class="flex items-center justify-end gap-1.5">
                    <Button
                      variant="destructive"
                      disabled={busy !== null}
                      onclick={() => run(s.name, () => removeSource(s.name))}
                    >
                      Remove
                    </Button>
                    <Button variant="ghost" onclick={() => (confirming = null)}>
                      Cancel
                    </Button>
                  </div>
                </div>
              </Modal>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div
      class="flex min-h-16 items-center justify-center rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-center text-xs text-muted-foreground"
    >
      No registered skill sources.
    </div>
  {/if}
</section>

<section class="flex flex-col gap-2">
  <h2 class="text-xs font-semibold">Role bindings</h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Which skill is used as the prompt when a role is spawned.
  </p>
  {#if roles.length}
    <div class="overflow-hidden rounded-md border border-border">
      <Table.Root class="table-fixed">
        <Table.Header>
          <Table.Row class="hover:bg-transparent">
            <Table.Head class="w-24">Role</Table.Head>
            <Table.Head>Resolved</Table.Head>
            <Table.Head class="w-52">Skill</Table.Head>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each roles as b (b.role)}
            <Table.Row>
              <Table.Cell class="truncate text-xs font-medium capitalize">
                {b.role}
              </Table.Cell>
              <!-- What a spawn would actually run: the resolved `Source/skill` —
                   the precedence winner when "no preference" is chosen — or, when
                   nothing resolves, why. This is the role's status; the picker to
                   its right is only the choice. -->
              <Table.Cell class="truncate">
                {#if b.resolved}
                  <div class="flex items-center gap-1">
                    <code
                      class="truncate font-mono text-[0.7rem] text-muted-foreground"
                      >{b.resolved}</code
                    >
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      class="shrink-0 text-muted-foreground hover:text-foreground"
                      title="Reveal this skill on disk, or open its source repository"
                      aria-label="Reveal the skill {b.resolved} used by the {b.role} role"
                      disabled={busy !== null}
                      onclick={() => openSkillLink(b.resolved)}
                    >
                      <LinkSimple />
                    </Button>
                  </div>
                {:else}
                  <span class="text-[0.7rem] text-destructive"
                    >no enabled source holds this skill</span
                  >
                {/if}
              </Table.Cell>
              <Table.Cell>
                <Select.Root
                  type="single"
                  value={roleValue(b)}
                  onValueChange={(v) => v && chooseRole(b.role, v)}
                >
                  <Select.Trigger
                    class="h-7 w-full font-mono text-xs"
                    aria-label="Skill for the {b.role} role"
                    disabled={busy !== null}
                  >
                    <span class="truncate text-muted-foreground"
                      >{roleLabel(b)}</span
                    >
                  </Select.Trigger>
                  <Select.Content>
                    <Select.Item value={AUTO} class="text-xs"
                      >no preference</Select.Item
                    >
                    {#each b.candidates as ref (ref)}
                      <Select.Item value={ref} class="font-mono text-xs"
                        >{ref}</Select.Item
                      >
                    {/each}
                  </Select.Content>
                </Select.Root>
              </Table.Cell>
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>
  {/if}
</section>
