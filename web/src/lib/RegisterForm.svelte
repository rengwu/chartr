<script lang="ts">
  import { registerSpace, ActionError } from './actions'
  import { Input } from '$lib/components/ui/input'
  import { Button } from '$lib/components/ui/button'
  import { cn } from '$lib/utils'

  // The register affordance, used two ways: as the first-run screen's headline
  // action, and inline in the sidebar for adding a further space.
  let {
    variant = 'inline',
    onRegistered,
  }: {
    variant?: 'first-run' | 'inline'
    onRegistered?: (id: string) => void
  } = $props()

  let path = $state('')
  let busy = $state(false)
  let error = $state<string | null>(null)
  let notice = $state<string | null>(null)

  async function submit(e: Event) {
    e.preventDefault()
    const p = path.trim()
    if (!p || busy) return
    busy = true
    error = null
    notice = null
    try {
      const res = await registerSpace(p)
      // The git-init announcement lives here, on the action's own response —
      // announced, never silent (story 2).
      notice = res.gitInited
        ? `Registered — ${p} wasn’t a git repository, so a new one was initialized there.`
        : `Registered ${p}.`
      path = ''
      onRegistered?.(res.id)
    } catch (err) {
      error = err instanceof ActionError ? err.message : String(err)
    } finally {
      busy = false
    }
  }
</script>

<form class={cn('flex flex-col gap-3', variant === 'first-run' && 'w-full max-w-sm')} onsubmit={submit}>
  {#if variant === 'first-run'}
    <h1 class="text-lg font-semibold">Register your first space</h1>
    <p class="text-sm text-muted-foreground">
      Point the chartr at a project folder. If it isn’t a git repository yet, one is
      initialized there — announced, never silent.
    </p>
  {/if}

  <div class="flex gap-2">
    <Input
      class="flex-1"
      type="text"
      placeholder="/path/to/your/project"
      bind:value={path}
      spellcheck="false"
      autocapitalize="off"
      autocomplete="off"
      aria-label="Project folder to register"
    />
    <Button type="submit" disabled={busy || path.trim() === ''}>
      {busy ? 'Registering…' : 'Register'}
    </Button>
  </div>

  {#if error}<p class="text-xs text-destructive" role="alert">{error}</p>{/if}
  {#if notice}<p class="text-xs text-muted-foreground">{notice}</p>{/if}
</form>
