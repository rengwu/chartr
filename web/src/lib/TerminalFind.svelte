<script lang="ts">
  import { Input } from './components/ui/input'
  import { Button } from './components/ui/button'
  import { MagnifyingGlass, CaretUp, CaretDown, TextAa, X } from 'phosphor-svelte'

  // The in-terminal find widget (ticket 07). It is chrome — tokens + vendored
  // primitives + Phosphor (ADR 0012) — hosted in the island wrapper and floating
  // over the terminal at top-right. It never reaches inside the renderer (ADR 0010):
  // it owns only its own input value and hands every action back to the island,
  // which drives the search addon. Its open/closed life is the island's transient
  // UI state, not config, so this component holds nothing across a remount.
  //
  // `count`/`index` come back from the addon's result event: `count` is the total
  // match count, `index` the active match (0-based, or -1 when the addon's highlight
  // threshold is exceeded and it stops counting a position). `caseSensitive` is the
  // toggle the island keeps, reflected here so the button reads pressed.
  let {
    count,
    index,
    caseSensitive,
    onquery,
    onnext,
    onprev,
    ontogglecase,
    onclose,
  }: {
    count: number
    index: number
    caseSensitive: boolean
    onquery: (q: string) => void
    onnext: () => void
    onprev: () => void
    ontogglecase: () => void
    onclose: () => void
  } = $props()

  let value = $state('')
  let inputRef = $state<HTMLInputElement | null>(null)

  // Cmd+F opens the widget and lands focus in the input straight away, so an
  // operator types their query without a second reach for the mouse.
  $effect(() => {
    inputRef?.focus()
  })

  // The match readout: nothing until something is typed, then a live count. `1/17`
  // while the addon is tracking a position; a bare count when it has passed its
  // highlight threshold (index -1) and stopped placing the cursor; `No results`
  // when the query matches nothing.
  const label = $derived(
    value === ''
      ? ''
      : count === 0
        ? 'No results'
        : index < 0
          ? `${count}`
          : `${index + 1}/${count}`,
  )

  function oninput(e: Event) {
    value = (e.currentTarget as HTMLInputElement).value
    onquery(value)
  }

  // Enter cycles forward, Shift+Enter back — bound to the input, where a keystroke
  // is a query edit, so a focused next/prev button's own Enter is never doubled.
  function oninputkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (e.shiftKey) onprev()
      else onnext()
    }
  }

  // Esc closes and Cmd+F re-selects, bound to the whole widget (keydown bubbles) so
  // they work wherever focus sits inside it — the input or any of the buttons — not
  // only while the input holds focus. Cmd+F here also keeps the browser's own find
  // from opening over the widget.
  function onwidgetkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      onclose()
    } else if ((e.metaKey || e.ctrlKey) && (e.key === 'f' || e.key === 'F')) {
      e.preventDefault()
      inputRef?.select()
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="absolute right-2 top-2 z-10 flex items-center gap-0.5 rounded-md border border-border bg-popover px-1.5 py-1 text-popover-foreground shadow-md"
  role="search"
  onkeydown={onwidgetkeydown}
>
  <MagnifyingGlass class="size-3.5 shrink-0 text-muted-foreground" />
  <Input
    bind:ref={inputRef}
    {value}
    {oninput}
    onkeydown={oninputkeydown}
    class="h-6 w-40 border-0 bg-transparent px-1 text-xs shadow-none focus-visible:ring-0"
    placeholder="Find"
    aria-label="Find in terminal"
    spellcheck="false"
    autocapitalize="off"
    autocomplete="off"
  />
  <span class="w-16 shrink-0 text-center text-[0.7rem] tabular-nums text-muted-foreground">
    {label}
  </span>
  <Button
    variant="ghost"
    size="icon-sm"
    disabled={count === 0}
    onclick={onprev}
    aria-label="Previous match"
  >
    <CaretUp />
  </Button>
  <Button
    variant="ghost"
    size="icon-sm"
    disabled={count === 0}
    onclick={onnext}
    aria-label="Next match"
  >
    <CaretDown />
  </Button>
  <Button
    variant={caseSensitive ? 'default' : 'ghost'}
    size="icon-sm"
    aria-pressed={caseSensitive}
    onclick={ontogglecase}
    aria-label="Match case"
  >
    <TextAa />
  </Button>
  <Button variant="ghost" size="icon-sm" onclick={onclose} aria-label="Close find">
    <X />
  </Button>
</div>
