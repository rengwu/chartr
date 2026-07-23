<script lang="ts">
  import { onMount } from 'svelte'
  import { Terminal as Xterm } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import '@xterm/xterm/css/xterm.css'
  import type { Terminal, TerminalPrefs } from './model'
  import { buildTerminalOptions } from './tokens'

  // The terminal is an imperative island: the Svelte chrome hosts it but never
  // reaches inside (ADR 0010). It owns one xterm.js instance and one binary
  // terminal socket — raw PTY bytes down are written straight into xterm,
  // keystrokes up go out as binary frames, and a resize goes up as a small text
  // control message. The server replays scrollback as the first frames, so a tab
  // reopened after a detach walks back into the running shell.
  //
  // `prefs` is the operator's resolved terminal customization off the model
  // snapshot (the per-machine `terminal.toml`). It is resolved into concrete xterm
  // options at the token seam (tokens.ts), never read inside the renderer, and the
  // island fully remounts when it changes: a keyed `{#key}` wrapper in the chrome
  // tears this component down and mounts a fresh one, so each mount reads the
  // current prefs once and the terminal socket replays scrollback on re-attach —
  // nothing is lost (spec, Island reactivity — remount on change).
  let { term, prefs }: { term: Terminal; prefs?: TerminalPrefs } = $props()

  let host: HTMLDivElement

  onMount(() => {
    // The resolve seam owns the theme and options; the island just hands the
    // result to xterm at mount. Green/yellow/blue/magenta/cyan have no chrome
    // token (the theme is otherwise monochrome plus `--destructive`), so those
    // ANSI slots come from the seam's default preset rather than any token.
    const { options } = buildTerminalOptions(prefs)
    const xterm = new Xterm({
      ...options,
      // The blink pref (default on) is gated by liveness: a dead shell never
      // blinks, so a frozen session reads as frozen regardless of the setting.
      cursorBlink: (options.cursorBlink ?? true) && term.alive,
    })
    const fit = new FitAddon()
    xterm.loadAddon(fit)

    // The unicode11 addon (wide-glyph/emoji widths) is an optional, pref-gated
    // addon — lazily imported and activated only when the file asks for it, so a
    // machine that never enables it never pays for the chunk. It is bundled, not
    // fetched (CLAUDE.md). Because the island fully remounts on any prefs change,
    // this mount imports exactly what the current prefs want with no hot-swap.
    if (prefs?.unicode11) {
      void import('@xterm/addon-unicode11').then(({ Unicode11Addon }) => {
        xterm.loadAddon(new Unicode11Addon())
        xterm.unicode.activeVersion = '11'
      })
    }

    xterm.open(host)
    fit.fit()

    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const ws = new WebSocket(
      `${proto}//${location.host}/ws/terminal/${encodeURIComponent(term.id)}`,
    )
    ws.binaryType = 'arraybuffer'

    const enc = new TextEncoder()

    function sendResize() {
      if (ws.readyState !== WebSocket.OPEN) return
      ws.send(JSON.stringify({ resize: { cols: xterm.cols, rows: xterm.rows } }))
    }

    ws.onopen = () => sendResize()
    ws.onmessage = (ev: MessageEvent) => {
      xterm.write(new Uint8Array(ev.data as ArrayBuffer))
    }

    const dataSub = xterm.onData((d) => {
      if (ws.readyState === WebSocket.OPEN) ws.send(enc.encode(d))
    })
    const resizeSub = xterm.onResize(() => sendResize())

    // Refit the PTY whenever the pane changes size, so the shell reflows to the
    // column rather than the geometry it happened to mount at.
    const ro = new ResizeObserver(() => {
      try {
        fit.fit()
      } catch {
        // fit throws if the host is momentarily detached during a layout change;
        // the next observation refits.
      }
    })
    ro.observe(host)

    if (term.alive) xterm.focus()

    return () => {
      ro.disconnect()
      dataSub.dispose()
      resizeSub.dispose()
      ws.close()
      xterm.dispose()
    }
  })
</script>

<div class="terminal-island" bind:this={host}></div>
