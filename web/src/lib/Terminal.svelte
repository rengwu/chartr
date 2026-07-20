<script lang="ts">
  import { onMount } from 'svelte'
  import { Terminal as Xterm } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import '@xterm/xterm/css/xterm.css'
  import type { Terminal } from './model'

  // The terminal is an imperative island: the Svelte chrome hosts it but never
  // reaches inside (ADR 0010). It owns one xterm.js instance and one binary
  // terminal socket — raw PTY bytes down are written straight into xterm,
  // keystrokes up go out as binary frames, and a resize goes up as a small text
  // control message. The server replays scrollback as the first frames, so a tab
  // reopened after a detach walks back into the running shell.
  let { term }: { term: Terminal } = $props()

  let host: HTMLDivElement

  onMount(() => {
    const xterm = new Xterm({
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
      fontSize: 13,
      cursorBlink: term.alive,
      theme: { background: '#0b0e14', foreground: '#c9d1d9' },
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
