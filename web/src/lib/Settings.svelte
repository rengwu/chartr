<script lang="ts">
  import type {
    Agent,
    AutoTitlePrefs,
    ConfigLayer,
    NotifyPrefs,
    RoleBinding,
    Source,
  } from './model'
  import { createConfigLayer, openGlobalLayer } from './actions'
  import AgentLibrary from './AgentLibrary.svelte'
  import SourcesSettings from './SourcesSettings.svelte'
  import TerminalSettings from './TerminalSettings.svelte'
  import NotifySettings from './NotifySettings.svelte'
  import AutoTitleSettings from './AutoTitleSettings.svelte'
  import SystemPromptsSettings from './SystemPromptsSettings.svelte'
  import { Button } from './components/ui/button'
  import * as ScrollArea from './components/ui/scroll-area'
  import { ArrowSquareOut, FilePlus, X } from 'phosphor-svelte'

  // The settings surface (ticket 05): the operator's global config — the agent
  // library and the paths of the files behind it, each openable in the operator's
  // own editor. There is no committed execution layer and nothing to explain about
  // resolution any more — the library is the whole of execution config. Everything
  // here traces back to a file; nothing is invented state.
  let {
    config,
    agents,
    detected,
    sources,
    roles,
    gitAvailable = false,
    nativePicker = false,
    notifyPrefs,
    autoTitlePrefs,
    onClose,
  }: {
    // The files the operator's config lives in — the agent library, `terminal.toml`,
    // and the two skill libraries.
    config: ConfigLayer[]
    // The operator's registered agent library — global, edited here.
    agents: Agent[]
    // The known agent CLIs found on this machine's PATH — the advisory hint the
    // agent library renders beneath the adapter input when registering one.
    detected: string[]
    // The operator's ordered skill-source list and the four role bindings that
    // resolve through it — the sources section. In resolution order off the
    // snapshot; nothing here re-sorts them.
    sources: Source[]
    roles: RoleBinding[]
    // Whether a git source can be registered on this machine at all.
    gitAvailable?: boolean
    // Whether this machine has an OS folder chooser — threaded down to the
    // sources form so a dir source's path can be picked, not only typed.
    nativePicker?: boolean
    // The resolved notify.toml values are read-only here, on the same terms as
    // terminal customization: show what is in force and open the owning file.
    notifyPrefs?: NotifyPrefs
    // The resolved autotitle.toml settings, read-only here on the same terms as
    // the notification rule.
    autoTitlePrefs?: AutoTitlePrefs
    onClose: () => void
  } = $props()

  // `terminal.toml` is a config file like the rest, but it is shown by the Terminal
  // section rather than in the generic list — it comes with the settings it holds.
  const terminalLayer = $derived(config.find((l) => l.holds === 'terminal'))
  const notifyLayer = $derived(config.find((l) => l.holds === 'notifications'))
  const autoTitleLayer = $derived(config.find((l) => l.holds === 'autotitle'))

  // The three prompts the System Prompts section surfaces, in a fixed display
  // order by their server-side names. Any an older server did not send drop out,
  // so the section renders only what actually has a file behind it.
  const systemPromptLayers = $derived(
    ['core-ticket', 'core-space', 'preferences']
      .map((name) => config.find((l) => l.name === name))
      .filter((l): l is ConfigLayer => l !== undefined),
  )

  let busy = $state<string | null>(null)
  let note = $state<string | null>(null)

  // The layers that can be stamped from a defaults template rather than only
  // opened. A layer that does not exist yet has nothing for the editor to open, so
  // for these the row offers a Create action that writes the self-documenting
  // starter (all keys at their defaults) and then hands off to the same open. Kept
  // as a set the server is the real authority on — it refuses a name with no
  // template — so this only governs which rows show the button. The two
  // self-documenting per-machine prefs files have starters; the agent library and
  // skill roots grow by their own edits, not from a stamped file.
  const creatable = new Set([
    'terminal-config',
    'notify-config',
    'autotitle-config',
    // The core overrides scaffold from chartr's shipped bytes: a missing one
    // offers Create from defaults, stamping the embedded core for the operator
    // to edit. Preferences scaffolds a blank file on the same terms — its default
    // is empty, so Create recovers a deleted or moved preferences.md.
    'core-ticket',
    'core-space',
    'preferences',
  ])

  // The escape hatch: the server resolves the *named* layer and launches the
  // operator's editor. Where it cannot, the path itself is the answer, surfaced
  // here.
  async function open(layerName: string) {
    busy = layerName
    try {
      const r = await openGlobalLayer(layerName)
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
     on the same tier as the space header, and the global config surface. Esc,
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
    <ScrollArea.Root class="min-h-0 min-w-0 flex-1">
      <div class="mx-auto flex w-full max-w-2xl flex-col gap-5 p-4">
        {#if note}
          <p class="rounded-md border border-border bg-muted/50 px-2.5 py-1.5 text-xs">{note}</p>
        {/if}

        <!-- The agent library — the only execution config there is. -->
        <AgentLibrary {agents} {detected} />

        <!-- Where skills come from, and which skill each role is spawned with.
             The section is load-bearing: it explains an orphaned checkout, it
             is the only recovery for a deleted role binding, and it is what
             makes the silent first-run migration discoverable at all. -->
        <SourcesSettings {sources} {roles} {gitAvailable} {nativePicker} />

        <!-- Per-machine cosmetics: what terminal.toml has in force, and the file
             itself. -->
        <TerminalSettings layer={terminalLayer} {layerRow} />
        <NotifySettings prefs={notifyPrefs} layer={notifyLayer} {layerRow} />
        <AutoTitleSettings
          prefs={autoTitlePrefs}
          layer={autoTitleLayer}
          {layerRow}
        />

        <!-- The prompts chartr composes into every session: the two overridable
             cores and the operator's own preferences, each openable and
             creatable-from-defaults through the same shared row. -->
        <SystemPromptsSettings layers={systemPromptLayers} {layerRow} />
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
