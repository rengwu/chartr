<script lang="ts">
  import { onMount } from 'svelte'
  import { Terminal as Xterm, type ITheme } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import '@xterm/xterm/css/xterm.css'
  import type { Terminal } from './model'
  import { readColor } from './tokens'

  // The terminal is an imperative island: the Svelte chrome hosts it but never
  // reaches inside (ADR 0010). It owns one xterm.js instance and one binary
  // terminal socket — raw PTY bytes down are written straight into xterm,
  // keystrokes up go out as binary frames, and a resize goes up as a small text
  // control message. The server replays scrollback as the first frames, so a tab
  // reopened after a detach walks back into the running shell.
  let { term }: { term: Terminal } = $props()

  let host: HTMLDivElement

  // The xterm surface, resolved off the live design tokens at the seam
  // (tokens.ts) so the terminal reads as part of the reskinned chrome instead
  // of xterm's stock theme (ticket 04). The renderer itself is untouched — this
  // is the wrapper computing a plain colour object and handing it in (ADR
  // 0010). Green/yellow/blue/magenta/cyan have no chrome token to draw from
  // (the theme is otherwise monochrome plus `--destructive`), so those six
  // ANSI slots are literal, muted hues tuned to sit quietly on the token
  // surface rather than clash with it.
  function buildTheme(): ITheme {
    const background = readColor('--background')
    const foreground = readColor('--foreground')
    const dim = readColor('--muted-foreground')
    const red = readColor('--destructive')
    return {
      background,
      foreground,
      cursor: readColor('--ring'),
      cursorAccent: background,
      selectionBackground: readColor('--muted'),
      black: background,
      brightBlack: dim,
      white: foreground,
      brightWhite: foreground,
      red,
      brightRed: red,
      green: '#9cb68c',
      brightGreen: '#b3cba3',
      yellow: '#d1b374',
      brightYellow: '#e0c88f',
      blue: '#82a8c9',
      brightBlue: '#9dbdd9',
      magenta: '#b48cc2',
      brightMagenta: '#c7a5d3',
      cyan: '#7fb3ab',
      brightCyan: '#99c7c0',
    }
  }

  onMount(() => {
    const xterm = new Xterm({
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
      fontSize: 13,
      cursorBlink: term.alive,
      theme: buildTheme(),
    })
    const fit = new FitAddon()
    xterm.loadAddon(fit)
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
