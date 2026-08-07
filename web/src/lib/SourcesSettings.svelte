<script lang="ts">
  import type { RoleBinding, Source } from './model'
  import {
    ActionError,
    refreshSource,
    registerSource,
    removeSource,
    reorderSources,
    restoreRoleBinding,
    setSourceEnabled,
  } from './actions'
  import { reorder } from './reorder'
  import { Button } from './components/ui/button'
  import { Badge } from './components/ui/badge'
  import { Input } from './components/ui/input'
  import { Checkbox } from './components/ui/checkbox'
  import * as Select from './components/ui/select'
  import {
    ArrowDown,
    ArrowUp,
    ArrowsClockwise,
    Eye,
    Plus,
    Trash,
    Warning,
    X,
  } from 'phosphor-svelte'

  // The sources section (ticket 10): the operator's ordered list of places skills
  // come from, and the six actions the planning tickets designed for it —
  // register, remove, toggle, reorder, refresh, and restore a role binding.
  //
  // It is load-bearing rather than a convenience. It is where an orphaned git
  // checkout is explained, the only recovery for a deleted role binding, and the
  // thing the silent first-run migration is discoverable *in* — a source that was
  // registered before the operator ever opened this screen appears here as an
  // ordinary row, which is the whole of what "quiet, not hidden" means.
  //
  // The editable/open-the-file line is ADR 0014's: the six actions are inline,
  // and everything else is the operator's own editor on the file itself. There is
  // never a second config store.
  let {
    sources,
    roles,
    gitAvailable,
    onPreviewFree,
  }: {
    // In resolution order, exactly as the server holds it — never sorted here.
    sources: Source[]
    roles: RoleBinding[]
    // Whether `git` is on PATH. A git registration is refused at the gate when it
    // is not, so the form says so up front rather than after a failed clone.
    gitAvailable: boolean
    onPreviewFree: () => void
  } = $props()

  let note = $state<string | null>(null)
  let busy = $state<string | null>(null)
  // The row awaiting a second click before it is removed. Registering is cheap to
  // undo, removing a git source is not — the checkout stays on disk but the URL
  // and pin do not, so it asks twice.
  let confirming = $state<string | null>(null)

  type Draft = { name: string; kind: 'dir' | 'git'; path: string; url: string; ref: string }
  let draft = $state<Draft | null>(null)

  const kindLabels: Record<Draft['kind'], string> = {
    dir: 'folder on this machine',
    git: 'git repository',
  }

  // The operator's own rows: everything but the synthetic default, which is
  // always last and never moves. Reorder sends the whole list, so the default is
  // appended back before the call.
  const movable = $derived(sources.filter((s) => !s.default))

  async function run(key: string, fn: () => Promise<unknown>, ok?: string) {
    busy = key
    note = null
    try {
      await fn()
      if (ok) note = ok
    } catch (e) {
      note = e instanceof ActionError ? e.message : (e as Error).message
    } finally {
      busy = null
      confirming = null
    }
  }

  // Move a row one place. Position *is* resolution order, so a move is the whole
  // of what "which skill wins" means and there is nothing else to edit.
  function move(name: string, delta: number) {
    const names = movable.map((s) => s.name)
    const from = names.indexOf(name)
    const to = from + delta
    if (from < 0 || to < 0 || to >= names.length) return
    void run(name, () => reorderSources(reorder(names, from, to)))
  }

  function submit(e: Event) {
    e.preventDefault()
    if (!draft) return
    const d = draft
    void run('register', async () => {
      await registerSource({ name: d.name, kind: d.kind, path: d.path, url: d.url, ref: d.ref })
      draft = null
    })
  }

  // "fetched 12 Mar 2026" — a date, not a timestamp: the pin is the sha, and the
  // time only answers "how old is this".
  function fetchedOn(iso?: string): string {
    if (!iso) return ''
    const d = new Date(iso)
    return Number.isNaN(d.getTime()) ? iso : (
        d.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' })
      )
  }

  function shortSha(commit?: string): string {
    return commit ? commit.slice(0, 7) : ''
  }

  // What the default row says about itself: chartr's shipped bytes until the
  // first refresh clones the upstream over them, a pin from then on.
  function defaultLine(s: Source): string {
    if (s.seeded) return 'shipped with this build'
    return `fetched ${fetchedOn(s.fetched)} — ${shortSha(s.commit)}`
  }

  function statusNote(s: Source): string {
    if (s.status === 'unavailable') return 'the path is gone — the row stays until you remove it'
    if (s.status === 'empty') return 'nothing here holds a SKILL.md'
    return ''
  }
</script>

<section class="flex flex-col gap-2">
  <div class="flex items-baseline justify-between gap-2">
    <h2 class="text-xs font-semibold">Skill sources</h2>
    <div class="flex items-center gap-1">
      <Button variant="ghost" size="xs" onclick={onPreviewFree}>
        <Eye /> free session payload
      </Button>
      {#if !draft}
        <Button variant="ghost" size="xs" onclick={() => (draft = { name: '', kind: 'dir', path: '', url: '', ref: '' })}>
          <Plus /> register a source
        </Button>
      {/if}
    </div>
  </div>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Where skills come from, in the order they resolve — the first enabled source to hold a name
    wins, and the loser stays reachable as <code class="font-mono">Source/skill</code>. chartr ships
    none of them itself. Registered on this machine and never committed.
  </p>

  {#if note}
    <p class="rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs">{note}</p>
  {/if}

  {#if draft}
    <form class="flex flex-col gap-2.5 rounded-md border border-ring p-2.5" onsubmit={submit}>
      <div class="flex items-center justify-between gap-2">
        <span class="text-xs font-semibold">Register a source</span>
        <Button variant="ghost" size="icon-xs" aria-label="Cancel" onclick={() => (draft = null)}>
          <X />
        </Button>
      </div>

      <div class="flex items-center gap-1.5">
        <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">name</span>
        <Input
          class="h-7 min-w-0 flex-1 text-xs"
          value={draft.name}
          placeholder="my skills"
          oninput={(e: Event) => (draft!.name = (e.currentTarget as HTMLInputElement).value)}
        />
      </div>

      <div class="flex items-center gap-1.5">
        <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">kind</span>
        <Select.Root type="single" bind:value={draft.kind}>
          <Select.Trigger class="h-7 min-w-0 flex-1 text-xs" aria-label="Source kind">
            {kindLabels[draft.kind]}
          </Select.Trigger>
          <Select.Content>
            {#each Object.entries(kindLabels) as [value, label] (value)}
              <Select.Item {value} class="text-xs">{label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      {#if draft.kind === 'dir'}
        <div class="flex items-center gap-1.5">
          <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">path</span>
          <Input
            class="h-7 min-w-0 flex-1 font-mono text-xs"
            value={draft.path}
            placeholder="~/skills"
            oninput={(e: Event) => (draft!.path = (e.currentTarget as HTMLInputElement).value)}
          />
        </div>
        <p class="pl-[3.875rem] text-[0.7rem] text-muted-foreground">
          Your folder, edited by you. chartr only reads it.
        </p>
      {:else}
        <div class="flex items-center gap-1.5">
          <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">url</span>
          <Input
            class="h-7 min-w-0 flex-1 font-mono text-xs"
            value={draft.url}
            placeholder="https://github.com/someone/skills.git"
            oninput={(e: Event) => (draft!.url = (e.currentTarget as HTMLInputElement).value)}
          />
        </div>
        <div class="flex items-center gap-1.5">
          <span class="w-14 shrink-0 font-mono text-[0.65rem] text-muted-foreground">ref</span>
          <Input
            class="h-7 min-w-0 flex-1 font-mono text-xs"
            value={draft.ref}
            placeholder="the default branch"
            oninput={(e: Event) => (draft!.ref = (e.currentTarget as HTMLInputElement).value)}
          />
        </div>
        <!-- The only assertion of trust in a git source's whole lifetime is the
             moment the URL is typed (ADR 0017); there is no confirm gate after
             it, so the consequence is said here, before the clone. -->
        <p class="pl-[3.875rem] text-[0.7rem] leading-relaxed text-muted-foreground">
          chartr clones this into its own checkout under <code class="font-mono">sources/</code> and
          runs whatever skills it holds. Pasting the URL is the trust decision — nothing asks again.
          {#if !gitAvailable}
            <strong class="font-medium text-foreground">
              `git` is not on this machine's PATH, so this registration will be refused.
            </strong>
          {/if}
        </p>
      {/if}

      <div class="flex justify-end">
        <Button type="submit" size="xs" disabled={busy !== null}>
          {busy === 'register' ? 'Registering…' : 'Register'}
        </Button>
      </div>
    </form>
  {/if}

  <ol class="flex flex-col gap-1.5">
    {#each sources as s, i (s.name)}
      <li class="flex flex-col gap-1 rounded-md border border-border px-2.5 py-2">
        <div class="flex items-center gap-2">
          <Checkbox
            checked={s.enabled}
            aria-label="{s.enabled ? 'Disable' : 'Enable'} {s.name}"
            disabled={busy !== null}
            onCheckedChange={(v: boolean) => run(s.name, () => setSourceEnabled(s.name, v))}
          />
          <span class="flex min-w-0 flex-1 flex-col">
            <span class="flex min-w-0 items-baseline gap-1.5">
              <span class="truncate text-xs font-medium" class:text-muted-foreground={!s.enabled}>
                {s.name}
              </span>
              <span class="shrink-0 font-mono text-[0.65rem] text-muted-foreground">{s.kind}</span>
              {#if s.status !== 'ok'}
                <Badge variant="outline">{s.status}</Badge>
              {/if}
              {#if s.shadowed?.length && s.shadowed.length === s.skills.length && s.skills.length > 0}
                <Badge variant="outline">shadowed</Badge>
              {/if}
            </span>
            <code class="truncate font-mono text-[0.65rem] text-muted-foreground" title={s.path}>
              {s.path}
            </code>
          </span>

          <span class="shrink-0 text-[0.65rem] text-muted-foreground">
            {s.skills.length}
            {s.skills.length === 1 ? 'skill' : 'skills'}
          </span>

          <!-- Position is resolution order, so it is shown as the number it is
               and moved with the two controls beside it. The default row is
               always last and neither moves nor is removed. -->
          {#if s.default}
            <span class="shrink-0 text-[0.65rem] text-muted-foreground">last</span>
          {:else}
            <span class="shrink-0 font-mono text-[0.65rem] text-muted-foreground">{i + 1}</span>
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Move {s.name} earlier"
              title="Resolve earlier"
              disabled={busy !== null || i === 0}
              onclick={() => move(s.name, -1)}
            >
              <ArrowUp />
            </Button>
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Move {s.name} later"
              title="Resolve later"
              disabled={busy !== null || i >= movable.length - 1}
              onclick={() => move(s.name, 1)}
            >
              <ArrowDown />
            </Button>
          {/if}

          {#if s.kind === 'git'}
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Refresh {s.name}"
              title="Fetch the tip of this ref — discards local edits inside chartr's checkout"
              disabled={busy !== null}
              onclick={() => run(s.name, () => refreshSource(s.name), `Refreshed ${s.name}.`)}
            >
              <ArrowsClockwise />
            </Button>
          {/if}

          {#if !s.default}
            <Button
              variant={confirming === s.name ? 'destructive' : 'ghost'}
              size={confirming === s.name ? 'xs' : 'icon-xs'}
              aria-label="Remove {s.name}"
              title="Remove this source from the list"
              disabled={busy !== null}
              onclick={() => {
                if (confirming !== s.name) {
                  confirming = s.name
                  return
                }
                void run(s.name, () => removeSource(s.name))
              }}
            >
              {#if confirming === s.name}Remove?{:else}<Trash />{/if}
            </Button>
          {/if}
        </div>

        {#if s.default}
          <p class="text-[0.7rem] text-muted-foreground">{defaultLine(s)}</p>
        {/if}

        {#if s.kind === 'git'}
          <!-- The line that will otherwise bite someone: this directory is
               chartr's, and a refresh resets it. The answer to "I want to edit
               this" is a `dir` source. -->
          <p class="text-[0.7rem] leading-relaxed text-muted-foreground">
            <code class="font-mono">{s.url}</code>
            {#if s.ref}<span class="font-mono"> · {s.ref}</span>{/if}
            {#if s.commit}<span class="font-mono"> · {shortSha(s.commit)}</span>{/if}
            {#if s.fetched}<span> · fetched {fetchedOn(s.fetched)}</span>{/if}
            <br />
            This checkout is chartr's, not a workspace — a refresh discards anything you edit inside
            it. To edit these skills, fork them into a folder and register that as a
            <code class="font-mono">dir</code> source.
          </p>
        {/if}

        {#if statusNote(s)}
          <p class="flex items-start gap-1.5 text-[0.7rem] leading-relaxed text-muted-foreground">
            <Warning class="mt-0.5 size-3 shrink-0" aria-hidden="true" />
            <span>{statusNote(s)}</span>
          </p>
        {/if}

        {#each s.warnings ?? [] as w}
          <p class="flex items-start gap-1.5 text-[0.7rem] leading-relaxed text-muted-foreground">
            <Warning class="mt-0.5 size-3 shrink-0" aria-hidden="true" /> <span>{w}</span>
          </p>
        {/each}
      </li>
    {/each}
  </ol>

  <!-- Where the file is named, the orphaning is named with it (ticket 01). -->
  <p class="text-[0.7rem] leading-relaxed text-muted-foreground">
    This list lives in <code class="font-mono">sources.toml</code> under your config root, openable
    below. Git checkouts sit beside it under <code class="font-mono">sources/</code>; if that file is
    lost they are orphaned, and chartr does not collect them — deleting them is yours to do.
  </p>
</section>

<section class="flex flex-col gap-2">
  <h2 class="text-xs font-semibold">Role bindings</h2>
  <p class="text-xs leading-relaxed text-muted-foreground">
    Which skill each role is spawned with, resolved through the list above. Bind a role to something
    else by editing <code class="font-mono">user.toml</code>; a row you delete makes that role refuse
    to spawn until it is rebound, and this is where you put it back.
  </p>
  {#each roles as b (b.role)}
    <div class="flex items-center gap-2 rounded-md border border-border px-2.5 py-1.5">
      <span class="w-20 shrink-0 text-xs font-medium capitalize">{b.role}</span>
      <span class="min-w-0 flex-1">
        {#if b.ref}
          <code
            class="truncate font-mono text-[0.7rem]"
            class:text-muted-foreground={b.resolves}
            class:text-destructive={!b.resolves}>{b.ref}</code
          >
          {#if !b.resolves}
            <span class="text-[0.7rem] text-muted-foreground">
              — no enabled source holds this skill
            </span>
          {/if}
        {:else}
          <span class="text-[0.7rem] text-muted-foreground">
            not bound — this role refuses to spawn
          </span>
        {/if}
      </span>
      {#if b.ref !== b.default}
        <Button
          variant="outline"
          size="xs"
          class="shrink-0"
          title="Bind {b.role} back to {b.default}"
          disabled={busy !== null}
          onclick={() => run(b.role, () => restoreRoleBinding(b.role), `${b.role} → ${b.default}`)}
        >
          Restore default
        </Button>
      {/if}
    </div>
  {/each}
</section>
