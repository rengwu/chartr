<script lang="ts">
  import type { Agent, ConfigLayer, Space, TerminalPrefs } from './model'
  import { settingsHash, type SettingsScope } from './route'
  import { createConfigLayer, openConfigLayer, openGlobalLayer } from './actions'
  import AgentLibrary from './AgentLibrary.svelte'
  import TerminalSettings from './TerminalSettings.svelte'
  import { Button } from './components/ui/button'
  import * as ScrollArea from './components/ui/scroll-area'
  import { ArrowSquareOut, FilePlus, Stack, User, Warning, X } from 'phosphor-svelte'

  // The settings surface (ticket 05): the agent library and the paths of the files
  // behind it, each openable in the operator's own editor. There is no committed
  // execution layer and nothing to explain about resolution any more — the library
  // is the whole of execution config (ADR 0009 as superseded, ADR 0014 retired).
  // Everything here traces back to a file; nothing is invented state.
  let {
    spaces,
    config,
    agents,
    detected,
    terminalPrefs,
    scope,
    onScope,
    onClose,
  }: {
    spaces: Space[]
    // The files the operator's config lives in, shared by every space — the agent
    // library, `terminal.toml`, and the two skill libraries that are not a space's
    // own.
    config: ConfigLayer[]
    // The operator's registered agent library — global, edited on the global scope.
    agents: Agent[]
    // The known agent CLIs found on this machine's PATH — the advisory hint the
    // agent library renders beneath the adapter input when registering one.
    detected: string[]
    // The operator's resolved terminal customization off the snapshot — read-only
    // here, rendered by the Terminal section on the global scope. Per-machine
    // cosmetic settings belong beside the user config, not under a space.
    terminalPrefs?: TerminalPrefs
    scope: SettingsScope
    onScope: (scope: SettingsScope) => void
    onClose: () => void
  } = $props()

  // `#/settings` with no sub-path falls back to the first space, so the screen is
  // never blank; an id that names no space does the same rather than 404-ing.
  const space = $derived<Space | null>(
    scope.kind === 'space' ? (spaces.find((s) => s.id === scope.spaceId) ?? spaces[0] ?? null)
    : scope.kind === 'user' ? null
    : (spaces[0] ?? null),
  )
  const onUser = $derived(scope.kind === 'user')

  // `terminal.toml` is a shared file like the rest, but it is shown by the Terminal
  // section rather than in the generic list — it comes with the settings it holds,
  // and listing it twice would be two rows for one file.
  const terminalLayer = $derived(config.find((l) => l.holds === 'terminal'))
  const files = $derived(config.filter((l) => l.holds !== 'terminal'))

  // The files a space carries in its own repository sit beside the shared ones.
  const layers = $derived<ConfigLayer[]>(space ? [...files, ...space.layers] : files)

  let busy = $state<string | null>(null)
  let note = $state<string | null>(null)

  // The layers that can be stamped from a defaults template rather than only
  // opened. A layer that does not exist yet has nothing for the editor to open, so
  // for these the row offers a Create action that writes the self-documenting
  // starter (all keys at their defaults) and then hands off to the same open. Kept
  // as a set the server is the real authority on — it refuses a name with no
  // template — so this only governs which rows show the button. Today that is the
  // per-machine terminal config alone; the agent library and skill roots grow by
  // their own edits, not from a stamped file.
  const creatable = new Set(['terminal-config'])

  // The escape hatch: the server resolves the *named* layer and launches the
  // operator's editor. Where it cannot, the path itself is the answer, surfaced
  // here. On the global scope there is no space to resolve through — and there may
  // be none registered at all — so it opens through the space-less endpoint.
  async function open(layerName: string) {
    busy = layerName
    try {
      const r = await (space ? openConfigLayer(space.id, layerName) : openGlobalLayer(layerName))
      note =
        r.opened === 'editor' || r.opened === 'os' ? null
        : r.exists ? `Nothing to open it with — it lives at ${r.path}`
        : `Nothing there yet — it would live at ${r.path}`
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }

  // Stamp a layer's file from its defaults template, for a layer that has nothing
  // on disk to open yet. The server writes the starter and rebuilds; the fresh
  // snapshot arrives over the control socket and flips the row to existing, so
  // there is no optimistic state to unwind — a refusal (an already-present file, a
  // name with no template) surfaces as the note like every other action.
  async function create(layerName: string) {
    busy = layerName
    try {
      const r = await createConfigLayer(layerName)
      note = `Created ${r.path} from defaults — open it to tweak.`
    } catch (e) {
      note = (e as Error).message
    } finally {
      busy = null
    }
  }
</script>

<!-- The settings route renders in place of the space cockpit: its own title bar
     on the same tier as the space header, a scope column, and the surface. Esc,
     the ⚙ button, or selecting a space leaves. -->
<div class="flex h-full min-h-0 flex-col">
  <header class="cockpit-bar justify-between">
    <div class="flex min-w-0 items-baseline gap-2">
      <span class="text-sm font-semibold">Settings</span>
    </div>
    <Button variant="ghost" size="icon-sm" aria-label="Close settings (Esc)" title="Close (Esc)" onclick={onClose}>
      <X />
    </Button>
  </header>

  <div class="flex min-h-0 flex-1">
    <!-- The scopes: every space, then the one global user file. -->
    <nav class="flex w-56 shrink-0 flex-col gap-1 overflow-y-auto border-r border-border p-2">
      <span class="px-1.5 py-1 text-[0.65rem] font-semibold tracking-wide text-muted-foreground uppercase">
        Global
      </span>
      <a
        href={settingsHash({ kind: 'user' })}
        class={[
          'flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-xs',
          onUser ? 'bg-accent text-accent-foreground font-medium' : 'hover:bg-accent/60',
        ]}
        onclick={(e) => {
          e.preventDefault()
          onScope({ kind: 'user' })
        }}
      >
        <User class="size-3.5 shrink-0" aria-hidden="true" />
        <span class="truncate">Your config</span>
      </a>

      <span class="mt-2 px-1.5 py-1 text-[0.65rem] font-semibold tracking-wide text-muted-foreground uppercase">
        Spaces
      </span>
      {#each spaces as s (s.id)}
        <a
          href={settingsHash({ kind: 'space', spaceId: s.id })}
          class={[
            'flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-xs',
            !onUser && space?.id === s.id ?
              'bg-accent text-accent-foreground font-medium'
            : 'hover:bg-accent/60',
          ]}
          onclick={(e) => {
            e.preventDefault()
            onScope({ kind: 'space', spaceId: s.id })
          }}
        >
          <Stack class="size-3.5 shrink-0" aria-hidden="true" />
          <span class="truncate">{s.name}</span>
        </a>
      {/each}
    </nav>

    <ScrollArea.Root class="min-h-0 min-w-0 flex-1">
      <div class="mx-auto flex w-full max-w-2xl flex-col gap-5 p-4">
        {#if note}
          <p class="rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs">{note}</p>
        {/if}

        {#if onUser}
          <!-- The global scope: the agent library — the only execution config there
               is — and the files it lives among, openable in the operator's editor. -->
          <AgentLibrary {agents} {detected} />

          <!-- Per-machine cosmetics, beside the user config rather than under a
               space: what terminal.toml has in force, and the file itself. -->
          <TerminalSettings prefs={terminalPrefs} layer={terminalLayer} {layerRow} />

          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Files on disk</h2>
            <p class="text-xs leading-relaxed text-muted-foreground">
              Where your config lives. All local, never committed, and per-machine — your agent
              library sits in one file, your skill forks in a second root.
            </p>
            {#each files as l (l.name)}
              {@render layerRow(l)}
            {/each}
          </section>
        {:else if !space}
          <p class="text-sm text-muted-foreground">No spaces registered.</p>
        {:else}
          <section class="flex flex-col gap-1.5">
            <h2 class="flex items-baseline gap-2 text-xs font-semibold">
              {space.name}
              <code class="truncate font-mono text-[0.7rem] font-normal text-muted-foreground">{space.path}</code>
            </h2>
          </section>

          {#if space.warnings?.length}
            <!-- Live problems are surfaced where config is read (stories 37, 38). -->
            <ul class="flex flex-col gap-1.5 rounded-md border border-border p-2.5">
              {#each space.warnings as w}
                <li class="flex items-start gap-1.5 text-xs leading-relaxed text-muted-foreground">
                  <Warning class="mt-0.5 size-3.5 shrink-0" aria-hidden="true" /> <span>{w}</span>
                </li>
              {/each}
            </ul>
          {/if}

          <section class="flex flex-col gap-2">
            <h2 class="text-xs font-semibold">Files on disk</h2>
            <p class="text-xs text-muted-foreground">
              Where each file lives — the ones this space carries in its own repository, and the
              ones it shares with every space.
            </p>
            {#each layers as l (l.name)}
              {@render layerRow(l)}
            {/each}
          </section>
        {/if}
      </div>
    </ScrollArea.Root>
  </div>
</div>

{#snippet layerRow(l: ConfigLayer)}
  <div class="flex items-center gap-2 rounded-md border border-border px-2.5 py-1.5">
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="text-xs font-medium">{l.holds}</span>
      <code class="truncate font-mono text-[0.65rem] text-muted-foreground" title={l.path}>{l.path}</code>
    </span>
    {#if !l.exists}
      <span class="shrink-0 text-[0.65rem] text-muted-foreground">not created yet</span>
    {/if}
    {#if !l.exists && creatable.has(l.name)}
      <!-- A missing templated layer has nothing to open, so it offers Create
           instead: the server stamps the self-documenting defaults file, and the
           row flips to openable on the next snapshot. -->
      <Button
        variant="outline"
        size="xs"
        class="shrink-0"
        title="Create {l.path} from its default values"
        disabled={busy !== null}
        onclick={() => create(l.name)}
      >
        <FilePlus data-icon="inline-start" />
        Create from defaults
      </Button>
    {:else}
      <Button
        variant="ghost"
        size="icon-xs"
        aria-label="Open {l.path}"
        title="Open in your editor ($VISUAL / $EDITOR)"
        disabled={busy !== null}
        onclick={() => open(l.name)}
      >
        <ArrowSquareOut />
      </Button>
    {/if}
  </div>
{/snippet}
