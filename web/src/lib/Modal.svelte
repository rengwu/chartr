<script lang="ts">
  import type { Snippet } from 'svelte'

  // A modal dialog built on the native <dialog> element, so focus trapping,
  // Esc-to-dismiss, and the backdrop come from the platform rather than
  // hand-rolled. Driven by an `open` prop; it reports every dismissal — the
  // close button, Esc, or a backdrop click — through onClose.
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

  let dialog = $state<HTMLDialogElement | null>(null)

  // Keep the element's modal state in step with the `open` prop.
  $effect(() => {
    const d = dialog
    if (!d) return
    if (open && !d.open) d.showModal()
    else if (!open && d.open) d.close()
  })

  // The `close` event fires for Esc, form method="dialog", and our own
  // programmatic close. Only report the dismissal when we still think we're
  // open, so a programmatic close (open already false) does not re-enter.
  function onDialogClose() {
    if (open) onClose()
  }

  // showModal() centres a content card inside the full-viewport <dialog>, so a
  // click landing on the dialog element itself is a backdrop click.
  function onDialogClick(e: MouseEvent) {
    if (e.target === dialog) onClose()
  }
</script>

<dialog
  bind:this={dialog}
  class="modal"
  aria-labelledby="modal-title"
  onclose={onDialogClose}
  onclick={onDialogClick}
>
  <div class="modal-card" class:wide>
    <header class="modal-head">
      <h2 class="modal-title" id="modal-title">{title}</h2>
      <button class="icon-btn" type="button" aria-label="Close" onclick={onClose}>×</button>
    </header>
    <div class="modal-body">
      {@render children()}
    </div>
  </div>
</dialog>
