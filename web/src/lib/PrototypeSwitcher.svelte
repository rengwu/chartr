<script lang="ts">
  // PROTOTYPE ONLY — throwaway. A floating bar that cycles a `?variant=` URL
  // param so several UI variations can be flipped through in the browser and
  // compared against the real app. Gated on import.meta.env.DEV by callers, and
  // it never ships: a stray merge can't show it to users. Delete this file when
  // the winning variant is folded in.
  import { CaretLeft, CaretRight } from 'phosphor-svelte'

  let {
    variants,
    current,
    onChange,
    label = '',
  }: {
    variants: string[]
    current: string
    onChange: (v: string) => void
    label?: string
  } = $props()

  function step(delta: number) {
    const i = variants.indexOf(current)
    const next = variants[(i + delta + variants.length) % variants.length]
    onChange(next)
  }

  function onKey(e: KeyboardEvent) {
    const el = document.activeElement
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || (el as HTMLElement).isContentEditable))
      return
    if (e.key === 'ArrowLeft') step(-1)
    else if (e.key === 'ArrowRight') step(1)
  }
</script>

<svelte:window onkeydown={onKey} />

<div
  class="fixed bottom-4 left-1/2 z-[9999] flex -translate-x-1/2 items-center gap-1 rounded-full border border-white/15 bg-black/85 px-1 py-1 text-white shadow-xl backdrop-blur"
>
  <button class="rounded-full p-1.5 hover:bg-white/10" aria-label="Previous variant" onclick={() => step(-1)}>
    <CaretLeft weight="bold" />
  </button>
  <span class="min-w-[9rem] px-2 text-center text-xs font-medium tabular-nums">
    Variant {current}{label ? ` — ${label}` : ''}
  </span>
  <button class="rounded-full p-1.5 hover:bg-white/10" aria-label="Next variant" onclick={() => step(1)}>
    <CaretRight weight="bold" />
  </button>
</div>
