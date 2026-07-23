<script lang="ts">
  import { onMount } from 'svelte'
  import { Terminal as Xterm } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import '@xterm/xterm/css/xterm.css'
  import type { Terminal, TerminalPrefs } from './model'
  import { openExternal } from './external'
  import {
    buildTerminalOptions,
    resolveRenderer,
    terminalKeyAction,
    TERMINAL_NEWLINE,
  } from './tokens'

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

  // The island is two elements, and the split is load-bearing rather than
  // decorative: `host` carries the operator's padding (and, with it, the theme's
  // background so the padded frame reads as part of the terminal), while `grid` is
  // the unpadded box xterm actually mounts into. The fit addon sizes the grid from
  // its parent's *computed* width, and a browser reports that as the border-box
  // width — so measuring a padded element would hand back the full pane and the
  // grid would overflow its own padding instead of reflowing inside it. Measuring
  // `grid`, which has no padding of its own, is what makes a padding change refit
  // to the columns the shell really has (spec, Scrollbar & padding).
  let host: HTMLDivElement
  let grid: HTMLDivElement

  onMount(() => {
    // The resolve seam owns the theme and options; the island just hands the
    // result to xterm at mount. Green/yellow/blue/magenta/cyan have no chrome
    // token (the theme is otherwise monochrome plus `--destructive`), so those
    // ANSI slots come from the seam's default preset rather than any token.
    const { options, css } = buildTerminalOptions(prefs)

    // The scrollbar and the padding have no xterm option, so they arrive as CSS
    // custom properties and land on the host — the chrome styling its own wrapper,
    // never the renderer inside it (ADR 0010). They are set *before* the terminal
    // opens and fits, so the very first fit already measures the padded box and the
    // shell reflows to the column count it really has (spec, Scrollbar & padding).
    for (const [prop, value] of Object.entries(css)) host.style.setProperty(prop, value)

    // The renderer/ligatures choice is decided once at the seam (spec, Renderer & the
    // ligatures conflict) and read here before the terminal is constructed, because
    // ligatures need a constructor option: the addon joins glyphs through xterm's
    // `registerCharacterJoiner`, which is proposed API gated behind `allowProposedApi`.
    // We only open that gate when ligatures are actually on, so the default terminal
    // keeps the stable API surface.
    const { renderer, ligatures } = resolveRenderer(prefs)

    const xterm = new Xterm({
      ...options,
      // The blink pref (default on) is gated by liveness: a dead shell never
      // blinks, so a frozen session reads as frozen regardless of the setting.
      cursorBlink: (options.cursorBlink ?? true) && term.alive,
      allowProposedApi: ligatures,
    })
    const fit = new FitAddon()
    xterm.loadAddon(fit)

    // Shift+Enter writes a literal newline instead of submitting (story 14). What a
    // keystroke *means* is decided at the resolve seam, off the operator's prefs;
    // the island only obeys it — `input()` puts the bytes through the same onData
    // path a typed key takes, and returning false stops xterm submitting the line.
    xterm.attachCustomKeyEventHandler((ev) => {
      if (terminalKeyAction(ev, prefs) !== 'newline') return true
      xterm.input(TERMINAL_NEWLINE)
      return false
    })

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

    // Clickable links. The web-links addon spots URLs in output and underlines
    // them; where a click *goes* is not the island's decision — `openExternal` is
    // the seam that prefers the native shell's system-browser hook and falls back
    // to a new browser tab (spec, Links). Bundled and lazily imported like every
    // other addon (CLAUDE.md — no CDN, no runtime fetch); it carries no pref, so
    // every terminal gets it.
    void import('@xterm/addon-web-links').then(({ WebLinksAddon }) => {
      xterm.loadAddon(new WebLinksAddon((_ev, uri) => openExternal(uri)))
    })

    xterm.open(grid)
    fit.fit()

    // Renderer selection, decided once at the seam (spec, Renderer & the ligatures
    // conflict). The WebGL (GPU) renderer is the default; enabling ligatures forces
    // this terminal onto the canvas renderer instead, because the ligatures addon and
    // WebGL cannot coexist. All three are bundled, lazily imported chunks (CLAUDE.md —
    // no CDN, no runtime fetch), and loaded *after* open() so they attach to a live
    // screen. The choice (`renderer`/`ligatures`) was resolved above, before the
    // terminal was constructed; because the island fully remounts on any prefs change,
    // it is made fresh at each mount with no hot-swap.
    if (renderer === 'canvas') {
      void import('@xterm/addon-canvas').then(({ CanvasAddon }) => {
        xterm.loadAddon(new CanvasAddon())
      })
      // Ligatures ride the canvas renderer. The addon reads its ligature data from the
      // local (bundled) font — it never fetches a font over the network — and the seam
      // only asks for it when the resolved family is a bundled one.
      if (ligatures) {
        void import('@xterm/addon-ligatures').then(({ LigaturesAddon }) => {
          xterm.loadAddon(new LigaturesAddon())
        })
      }
    } else {
      // WebGL by default. Wire the GPU context-loss event to dispose the addon, which
      // drops xterm back to its built-in DOM renderer — so a backgrounded tab or a
      // driver reset never leaves the terminal blank. Disposing the terminal disposes
      // the addon with it, so cleanup needs no separate handle.
      void import('@xterm/addon-webgl').then(({ WebglAddon }) => {
        const webgl = new WebglAddon()
        webgl.onContextLoss(() => webgl.dispose())
        xterm.loadAddon(webgl)
      })
    }

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

    // Copy-on-select is the one selection behaviour xterm has no option for, so the
    // island wires it off the pref: every selection change puts the text on the
    // clipboard. A denied or absent clipboard is silent — the selection still works,
    // it just is not copied.
    const selectionSub = prefs?.copyOnSelect
      ? xterm.onSelectionChange(() => {
          const text = xterm.getSelection()
          if (text) void navigator.clipboard?.writeText(text).catch(() => {})
        })
      : undefined

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
    ro.observe(grid)

    if (term.alive) xterm.focus()

    return () => {
      ro.disconnect()
      dataSub.dispose()
      resizeSub.dispose()
      selectionSub?.dispose()
      ws.close()
      xterm.dispose()
    }
  })
</script>

<div class="terminal-island" bind:this={host}>
  <div class="terminal-grid" bind:this={grid}></div>
</div>
