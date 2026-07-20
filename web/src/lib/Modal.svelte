<script lang="ts">
  import type { Snippet } from 'svelte'
  import * as Dialog from '$lib/components/ui/dialog'
  import { cn } from '$lib/utils'

  // A modal dialog on the shadcn-svelte Dialog (Bits UI underneath), so focus
  // trapping, Esc-to-dismiss, and the backdrop come from the primitive rather
  // than hand-rolled. Driven by an `open` prop; it reports every dismissal —
  // the close button, Esc, or a backdrop click — through onClose.
  let {
    open,
    title,
    onClose,
    wide = false,
    children,
  }: {
    open: boolean
    title: string
    onClose: () => void
    // A wider card with a scrollable body, for content-heavy modals like the
    // payload preview; the default stays the compact form the forms use.
    wide?: boolean
    children: Snippet
  } = $props()

  function onOpenChange(next: boolean) {
    if (!next) onClose()
  }
</script>

<Dialog.Root {open} {onOpenChange}>
  <Dialog.Content class={cn('flex max-h-[85vh] flex-col overflow-hidden', wide ? 'sm:max-w-2xl' : 'sm:max-w-sm')}>
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
    </Dialog.Header>
    <div class="min-h-0 flex-1">
      {@render children()}
    </div>
  </Dialog.Content>
</Dialog.Root>
