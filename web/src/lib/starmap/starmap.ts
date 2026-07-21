// The star-map renderer: one coherent imperative canvas island behind a narrow
// seam — mount, receive the pushed model, emit selection — never decomposed into
// components (ADR 0010). The Svelte chrome hosts it but never reaches inside.
//
// Two guarantees the seam tests pin (spec, Testing Decisions):
//   1. Deterministic layout — the same ticket data always lays out the same way.
//   2. Zero star movement across the lifecycle — a model push changes a star's
//      appearance (its derived status) but never its position. Layout is
//      recomputed only when the map's *structure* (its tickets and edges)
//      changes; a pure status push keeps every position byte-for-byte.
//
// Rendering (the starfield, glow, pulse, easing) is the part that can only be
// judged by eye (starmap-design.md, Open risk), and it degrades gracefully: when
// no 2D context is available (a headless test), the island still ingests the
// model, holds deterministic positions, and emits selection — it simply draws
// nothing. That is what lets the seam be tested without a browser canvas.

import { computeLayout, structureSignature, edgesOf, TAU } from './layout'
import { STAR, LABEL, SESSION_HUE, visualState, hexA, type VisualState } from './theme'
import { GRAMMAR, type SessionState } from './session'
import type { Ticket } from '../model'

export type SelectHandler = (num: number | null) => void

// Fallback only — used until the wrapper's first setBackground() call. Ticket
// 04 moves the real value to the token-derived `--card` colour, resolved by
// StarMap.svelte through tokens.ts and handed in at the seam; the renderer
// itself never reads CSS (ADR 0010).
const DEFAULT_BG = '#05070d'

interface Node {
  num: number
  title: string
  type: string
  vstate: VisualState
  // The session overlay riding this star, or null when no session speaks for it
  // (ticket 13). Strictly additive: it never touches layout or the base star.
  sstate: SessionState | null
  x: number
  y: number
  _x: number
  _y: number
  flare: number
}

// How long one ticker line lingers before it fades out, and how long the fade
// takes — the live map's answer to "what just changed under me?" is one glance,
// then calm again.
const TICKER_HOLD = 4.2
const TICKER_FADE = 0.5

interface Cam {
  x: number
  y: number
  s: number
}

function mod(a: number, b: number): number {
  return ((a % b) + b) % b
}
function clamp(v: number, a: number, b: number): number {
  return v < a ? a : v > b ? b : v
}

// A parallax starfield for depth, seeded once so it never reshuffles.
interface StarLayer {
  f: number
  sz: number
  a: number
  stars: { x: number; y: number; t: number }[]
}
function makeStarfield(): StarLayer[] {
  const specs = [
    { f: 0.15, n: 140, sz: 0.7, a: 0.45 },
    { f: 0.3, n: 80, sz: 1.1, a: 0.65 },
    { f: 0.5, n: 34, sz: 1.7, a: 0.9 },
  ]
  // A tiny inline PRNG so the field is stable without importing layout's stream.
  let t = 9001 >>> 0
  const rnd = () => {
    t += 0x6d2b79f5
    let r = Math.imul(t ^ (t >>> 15), 1 | t)
    r ^= r + Math.imul(r ^ (r >>> 7), 61 | r)
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296
  }
  return specs.map((sp) => {
    const stars = []
    for (let i = 0; i < sp.n; i++) stars.push({ x: rnd(), y: rnd(), t: rnd() })
    return { f: sp.f, sz: sp.sz, a: sp.a, stars }
  })
}

export class StarMap {
  #host: HTMLElement | null = null
  #canvas: HTMLCanvasElement | null = null
  #ctx: CanvasRenderingContext2D | null = null
  #dpr = 1
  #w = 0
  #h = 0

  #nodes: Node[] = []
  #byNum = new Map<number, Node>()
  #edges: { from: number; to: number; satisfied: boolean }[] = []
  #sig = ''
  #resolved = new Set<number>()

  #cam: Cam = { x: 0, y: 0, s: 1 }
  #goal: Cam = { x: 0, y: 0, s: 1 }
  // The detail pane's footprint. The camera fits the constellation into — and
  // seats a selected star at the centre of — the viewport *minus* these insets,
  // so a right- or bottom-docked pane never covers the star being read (ticket
  // 07). The pane reports its measured size, so the camera eases to the space it
  // actually leaves free, in either docking.
  #insets = { top: 16, right: 16, bottom: 16, left: 16 }
  #clock = 0
  #last = 0
  #raf = 0
  #selected: number | null = null
  // One fading ticker line, drawn by the island itself (never a chrome
  // component — ADR 0010): what just changed under you, then calm again.
  #tickerText = ''
  #tickerAt = -1e9

  #starfield = makeStarfield()
  #bg = DEFAULT_BG
  #onSelect: SelectHandler = () => {}
  #ro: ResizeObserver | null = null
  #detach: (() => void)[] = []

  // --- seam: mount ----------------------------------------------------------
  mount(host: HTMLElement): void {
    this.#host = host
    const canvas = document.createElement('canvas')
    canvas.className = 'starmap-canvas'
    host.appendChild(canvas)
    this.#canvas = canvas
    // getContext returns null in a headless test env; the island runs on without
    // it (model + layout + selection all work, nothing is drawn).
    this.#ctx = canvas.getContext('2d')
    this.#dpr = Math.max(1, (typeof window !== 'undefined' && window.devicePixelRatio) || 1)

    this.#measure()
    if (typeof ResizeObserver !== 'undefined') {
      this.#ro = new ResizeObserver(() => this.#onResize())
      this.#ro.observe(host)
    }
    this.#bindPointer(canvas)
    this.#refit(true)

    if (this.#ctx) {
      this.#last = now()
      this.#raf = requestAnimationFrame(this.#render)
    }
  }

  // --- seam: receive the pushed model --------------------------------------
  // Ingest the map's tickets. When the structure is unchanged, only per-star
  // appearance is refreshed and positions are untouched — the no-movement
  // guarantee. When tickets or edges change, the layout is recomputed and the
  // camera re-fit, which is the only path that ever moves a star.
  // `sessions` is the session overlay keyed by ticket number (ticket 13),
  // derived by the wrapper from the same snapshot. It rides alongside the
  // tickets rather than through a second seam call, so one push is one visual
  // beat: a state change flares its star once and writes one fading ticker line.
  setModel(tickets: Ticket[], sessions: Record<number, SessionState> = {}): void {
    const sig = structureSignature(tickets)
    this.#resolved = new Set(tickets.filter((t) => t.status === 'resolved').map((t) => t.num))

    if (sig === this.#sig && this.#nodes.length) {
      const changed: string[] = []
      for (const t of tickets) {
        const n = this.#byNum.get(t.num)
        if (!n) continue
        n.title = t.title
        n.type = t.type
        const vstate = visualState(t)
        const sstate = sessions[t.num] ?? null
        if (vstate !== n.vstate || sstate !== n.sstate) {
          n.flare = 1
          changed.push(`#${t.num < 10 ? '0' : ''}${t.num} → ${sstate ?? vstate.replace('_', ' ')}`)
        }
        n.vstate = vstate
        n.sstate = sstate
      }
      if (changed.length) this.#tick(changed.join('   ·   '))
      this.#refreshEdges(tickets)
      return
    }

    this.#sig = sig
    const pts = computeLayout(tickets)
    this.#nodes = tickets.map((t) => {
      const p = pts[t.num]
      return {
        num: t.num,
        title: t.title,
        type: t.type,
        vstate: visualState(t),
        sstate: sessions[t.num] ?? null,
        x: p.x,
        y: p.y,
        _x: p.x,
        _y: p.y,
        flare: 0,
      }
    })
    this.#byNum = new Map(this.#nodes.map((n) => [n.num, n]))
    this.#refreshEdges(tickets)
    if (this.#selected !== null && !this.#byNum.has(this.#selected)) this.#selected = null
    this.#refit(true)
  }

  #refreshEdges(tickets: Ticket[]): void {
    this.#edges = edgesOf(tickets).map((e) => ({
      from: e.from,
      to: e.to,
      satisfied: this.#resolved.has(e.from),
    }))
  }

  // --- seam: emit selection -------------------------------------------------
  onSelect(cb: SelectHandler): void {
    this.#onSelect = cb
  }

  // Programmatic selection (a deep-link naming a star, or ticket 07's pane): the
  // camera eases the star into the space the pane leaves free.
  select(num: number | null): void {
    if (num !== null && !this.#byNum.has(num)) return
    this.#applySelection(num)
  }

  // The detail pane's measured footprint. Updating it re-eases the camera: a
  // selected star re-seats into the new free rect (so a responsive right→bottom
  // re-dock re-seats it), and with nothing selected the whole map eases to fit
  // the free area — which is how the map-material pane clears room for itself.
  setInsets(insets: Partial<{ top: number; right: number; bottom: number; left: number }>): void {
    this.#insets = { ...this.#insets, ...insets }
    if (this.#selected !== null && this.#byNum.has(this.#selected)) {
      this.#seat(this.#selected)
    } else {
      this.#refit(false)
      this.#settleIfHeadless()
    }
  }

  // --- seam: palette ---------------------------------------------------------
  // The card surface colour, resolved from the shared design tokens by the
  // wrapper and handed in here (ticket 04) — the renderer never reads CSS
  // itself (ADR 0010). A no-op on the next-drawn frame's positions; it only
  // changes what colour the field paints on.
  setBackground(color: string): void {
    this.#bg = color || DEFAULT_BG
  }

  destroy(): void {
    if (this.#raf) cancelAnimationFrame(this.#raf)
    this.#raf = 0
    this.#ro?.disconnect()
    for (const off of this.#detach) off()
    this.#detach = []
    this.#canvas?.remove()
    this.#canvas = null
    this.#ctx = null
    this.#host = null
  }

  // --- test seam ------------------------------------------------------------
  // World-space positions of every star, keyed by ticket number. The seam tests
  // read this to assert both determinism (same data → same positions) and the
  // no-movement guarantee (a status push leaves it identical).
  positions(): Record<number, { x: number; y: number }> {
    const out: Record<number, { x: number; y: number }> = {}
    for (const n of this.#nodes) out[n.num] = { x: n.x, y: n.y }
    return out
  }

  // The session overlay each star is currently carrying, keyed by ticket number
  // (absent for a star no session speaks for). The seam tests read this to assert
  // the grammar each state renders in, alongside GRAMMAR's channels.
  overlays(): Record<number, SessionState> {
    const out: Record<number, SessionState> = {}
    for (const n of this.#nodes) if (n.sstate) out[n.num] = n.sstate
    return out
  }

  // The live ticker line, or null once it has faded — one line per pushed change.
  ticker(): string | null {
    return this.#tickerAlpha() > 0 ? this.#tickerText : null
  }

  // The ticket numbers currently calling from offscreen: a human-review star
  // outside the free rect leaves a gold chevron at that edge, because a call to
  // action may not depend on where the camera happens to be. Exactly the set the
  // renderer draws chevrons for.
  beckoning(): number[] {
    const r = this.#beckonRect()
    const out: number[] = []
    for (const n of this.#nodes) {
      if (n.sstate !== 'human-review') continue
      const sx = n._x * this.#cam.s + this.#cam.x
      const sy = n._y * this.#cam.s + this.#cam.y
      if (sx < r.x0 || sx > r.x1 || sy < r.y0 || sy > r.y1) out.push(n.num)
    }
    return out
  }

  // Current screen position of a star under the live camera.
  screenOf(num: number): { x: number; y: number } | null {
    const n = this.#byNum.get(num)
    if (!n) return null
    return { x: n.x * this.#cam.s + this.#cam.x, y: n.y * this.#cam.s + this.#cam.y }
  }

  // Hit-test a screen point and, if it lands on a star, select and emit it — the
  // exact path a click drives. Returns the selected ticket number, or null for a
  // click on empty space (which deselects).
  selectAtScreen(sx: number, sy: number): number | null {
    let hit: Node | null = null
    for (const n of this.#nodes) {
      const px = n._x * this.#cam.s + this.#cam.x
      const py = n._y * this.#cam.s + this.#cam.y
      const r = Math.max(14, STAR[n.vstate].r * this.#cam.s + 10)
      if (Math.hypot(sx - px, sy - py) < r) hit = n
    }
    const num = hit ? hit.num : null
    if (hit) hit.flare = Math.max(hit.flare, 0.6)
    this.#applySelection(num)
    return num
  }

  #applySelection(num: number | null): void {
    this.#selected = num
    if (num !== null) this.#seat(num)
    this.#onSelect(num)
  }

  // The viewport minus the pane's insets — the free area the camera works in.
  #freeRect(): { cx: number; cy: number; availW: number; availH: number } {
    const left = this.#insets.left,
      right = this.#w - this.#insets.right,
      top = this.#insets.top,
      bottom = this.#h - this.#insets.bottom
    return {
      cx: (left + right) / 2,
      cy: (top + bottom) / 2,
      availW: Math.max(80, right - left),
      availH: Math.max(80, bottom - top),
    }
  }

  // The rect a beckoning star counts as visible inside: the free area the pane
  // leaves, less a margin wide enough to seat a chevron and its caption. A star
  // hidden behind the detail pane is offscreen for this purpose — the chevron is
  // a call to action, so it errs toward calling.
  #beckonRect(): { x0: number; x1: number; y0: number; y1: number } {
    const M = 48
    const left = this.#insets.left + M,
      right = this.#w - this.#insets.right - M,
      top = this.#insets.top + M,
      bottom = this.#h - this.#insets.bottom - M
    return { x0: Math.min(left, right), x1: Math.max(left, right), y0: Math.min(top, bottom), y1: Math.max(top, bottom) }
  }

  #tick(msg: string): void {
    this.#tickerText = msg
    this.#tickerAt = now()
  }

  #tickerAlpha(): number {
    if (!this.#tickerText) return 0
    const age = now() - this.#tickerAt
    if (age < TICKER_HOLD) return 1
    return clamp(1 - (age - TICKER_HOLD) / TICKER_FADE, 0, 1)
  }

  // Ease a star to the centre of the free rect at the current zoom.
  #seat(num: number): void {
    const n = this.#byNum.get(num)
    if (!n) return
    const { cx, cy } = this.#freeRect()
    this.#goal.x = cx - n.x * this.#cam.s
    this.#goal.y = cy - n.y * this.#cam.s
    this.#settleIfHeadless()
  }

  // With a live 2D context the render loop eases the camera toward its goal; with
  // none (a headless test) there is no loop, so the camera settles immediately —
  // which also makes screenOf meaningful for the seam tests.
  #settleIfHeadless(): void {
    if (this.#ctx) return
    this.#cam.x = this.#goal.x
    this.#cam.y = this.#goal.y
    this.#cam.s = this.#goal.s
  }

  // --- sizing + camera ------------------------------------------------------
  #measure(): void {
    const host = this.#host
    const w = host?.clientWidth || (typeof window !== 'undefined' ? window.innerWidth : 800)
    const h = host?.clientHeight || (typeof window !== 'undefined' ? window.innerHeight : 600)
    this.#w = w
    this.#h = h
    if (this.#canvas) {
      this.#canvas.width = Math.max(1, Math.round(w * this.#dpr))
      this.#canvas.height = Math.max(1, Math.round(h * this.#dpr))
      this.#canvas.style.width = w + 'px'
      this.#canvas.style.height = h + 'px'
    }
  }

  #onResize(): void {
    this.#measure()
    this.#refit(false)
  }

  // Fit the whole constellation into the viewport. `snap` sets the camera
  // immediately (a fresh map, a resize); otherwise the render loop eases to it.
  #refit(snap: boolean): void {
    if (!this.#nodes.length) return
    let minx = 1e9,
      miny = 1e9,
      maxx = -1e9,
      maxy = -1e9
    for (const n of this.#nodes) {
      minx = Math.min(minx, n.x)
      miny = Math.min(miny, n.y)
      maxx = Math.max(maxx, n.x)
      maxy = Math.max(maxy, n.y)
    }
    const pad = 90
    minx -= pad
    miny -= pad
    maxx += pad
    maxy += pad
    const { cx: fcx, cy: fcy, availW, availH } = this.#freeRect()
    const s = clamp(
      Math.min(availW / (maxx - minx || 1), availH / (maxy - miny || 1)),
      0.15,
      1.4,
    )
    const cx = (minx + maxx) / 2,
      cy = (miny + maxy) / 2
    this.#goal.s = s
    this.#goal.x = fcx - cx * s
    this.#goal.y = fcy - cy * s
    if (snap) {
      this.#cam.s = this.#goal.s
      this.#cam.x = this.#goal.x
      this.#cam.y = this.#goal.y
    }
  }

  // --- input: pan, zoom, click ---------------------------------------------
  #bindPointer(canvas: HTMLCanvasElement): void {
    let drag: { x: number; y: number; moved: boolean } | null = null
    const rectXY = (e: MouseEvent) => {
      const r = canvas.getBoundingClientRect()
      return { x: e.clientX - r.left, y: e.clientY - r.top }
    }
    const onDown = (e: MouseEvent) => {
      drag = { x: e.clientX, y: e.clientY, moved: false }
      canvas.classList.add('drag')
    }
    const onMove = (e: MouseEvent) => {
      if (!drag) return
      const dx = e.clientX - drag.x,
        dy = e.clientY - drag.y
      if (Math.abs(dx) + Math.abs(dy) > 3) drag.moved = true
      this.#goal.x += dx
      this.#goal.y += dy
      this.#cam.x += dx
      this.#cam.y += dy
      drag.x = e.clientX
      drag.y = e.clientY
    }
    const onUp = (e: MouseEvent) => {
      canvas.classList.remove('drag')
      if (drag && !drag.moved) {
        const { x, y } = rectXY(e)
        this.selectAtScreen(x, y)
      }
      drag = null
    }
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      const { x, y } = rectXY(e)
      const f = Math.exp(-e.deltaY * 0.0016)
      const ns = clamp(this.#goal.s * f, 0.12, 3)
      const k = ns / this.#goal.s
      this.#goal.x = x - (x - this.#goal.x) * k
      this.#goal.y = y - (y - this.#goal.y) * k
      this.#goal.s = ns
    }
    canvas.addEventListener('mousedown', onDown)
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    canvas.addEventListener('wheel', onWheel, { passive: false })
    this.#detach.push(
      () => canvas.removeEventListener('mousedown', onDown),
      () => window.removeEventListener('mousemove', onMove),
      () => window.removeEventListener('mouseup', onUp),
      () => canvas.removeEventListener('wheel', onWheel),
    )
  }

  // --- render loop ----------------------------------------------------------
  #render = (): void => {
    this.#raf = requestAnimationFrame(this.#render)
    const g = this.#ctx
    if (!g) return
    const t = now()
    let dt = t - this.#last
    if (dt < 0 || dt > 0.1) dt = 0.016
    this.#last = t
    this.#clock = t

    for (const n of this.#nodes) {
      const ph = n.num * 1.7
      n._x = n.x + Math.sin(this.#clock * 0.7 + ph) * 2.4
      n._y = n.y + Math.cos(this.#clock * 0.55 + ph) * 2.4
      if (n.flare > 0) n.flare = Math.max(0, n.flare - dt / 1.1)
    }
    this.#cam.x += (this.#goal.x - this.#cam.x) * 0.28
    this.#cam.y += (this.#goal.y - this.#cam.y) * 0.28
    this.#cam.s += (this.#goal.s - this.#cam.s) * 0.28

    g.setTransform(this.#dpr, 0, 0, this.#dpr, 0, 0)
    g.fillStyle = this.#bg
    g.fillRect(0, 0, this.#w, this.#h)
    this.#drawStarfield(g)

    g.save()
    g.translate(this.#cam.x, this.#cam.y)
    g.scale(this.#cam.s, this.#cam.s)
    for (const e of this.#edges) this.#drawEdge(g, e)
    for (const n of this.#nodes) this.#drawStar(g, n, this.#clock)
    g.restore()
    this.#drawLabels(g)
    this.#drawChevrons(g)
    this.#drawTicker(g)
  }

  #drawStarfield(g: CanvasRenderingContext2D): void {
    const W = this.#w,
      H = this.#h
    for (const L of this.#starfield) {
      for (const s of L.stars) {
        const x = mod(s.x * W + this.#cam.x * L.f, W)
        const y = mod(s.y * H + this.#cam.y * L.f, H)
        g.globalAlpha = L.a * (0.65 + 0.35 * Math.sin(s.t * TAU))
        g.fillStyle = 'rgba(255,255,255,1)'
        g.fillRect(x, y, L.sz, L.sz)
      }
    }
    g.globalAlpha = 1
  }

  #drawEdge(g: CanvasRenderingContext2D, e: { from: number; to: number; satisfied: boolean }): void {
    const a = this.#byNum.get(e.from),
      b = this.#byNum.get(e.to)
    if (!a || !b) return
    const ax = a._x,
      ay = a._y,
      bx = b._x,
      by = b._y
    const mx = (ax + bx) / 2,
      my = (ay + by) / 2,
      dx = bx - ax,
      dy = by - ay,
      len = Math.hypot(dx, dy) || 1
    const nx = -dy / len,
      ny = dx / len,
      bow = Math.min(46, len * 0.13),
      cx = mx + nx * bow,
      cy = my + ny * bow
    g.beginPath()
    g.moveTo(ax, ay)
    g.quadraticCurveTo(cx, cy, bx, by)
    if (e.satisfied) {
      g.strokeStyle = 'rgba(160,192,166,0.62)'
      g.lineWidth = 1.8
      g.setLineDash([])
    } else {
      g.strokeStyle = 'rgba(132,146,168,0.34)'
      g.lineWidth = 1.3
      g.setLineDash([4, 6])
    }
    g.stroke()
    g.setLineDash([])
    // A satisfied edge (blocker resolved) flows particles blocker→dependent, so
    // the frontier visibly ignites as paths clear (starmap-design.md dec. 5).
    if (e.satisfied) {
      for (let k = 0; k < 2; k++) {
        const u = mod(this.#clock * 0.1 + k / 2 + (e.from * 0.13 + e.to * 0.07), 1),
          m = 1 - u
        const fx = m * m * ax + 2 * m * u * cx + u * u * bx,
          fy = m * m * ay + 2 * m * u * cy + u * u * by
        g.fillStyle = 'rgba(190,225,200,' + (0.16 + 0.44 * Math.sin(u * Math.PI)) + ')'
        g.beginPath()
        g.arc(fx, fy, 1.7, 0, TAU)
        g.fill()
      }
    }
    // Direction arrowhead at the curve's midpoint.
    const midx = 0.25 * ax + 0.5 * cx + 0.25 * bx,
      midy = 0.25 * ay + 0.5 * cy + 0.25 * by
    const al = Math.hypot(dx, dy) || 1,
      ux = dx / al,
      uy = dy / al
    const ah = 7,
      aw = 3.8,
      px = -uy,
      py = ux,
      tipx = midx + ux * ah * 0.5,
      tipy = midy + uy * ah * 0.5
    g.beginPath()
    g.moveTo(tipx, tipy)
    g.lineTo(tipx - ux * ah + px * aw, tipy - uy * ah + py * aw)
    g.lineTo(tipx - ux * ah - px * aw, tipy - uy * ah - py * aw)
    g.closePath()
    g.fillStyle = e.satisfied ? '#aecdb6' : '#6f7889'
    g.fill()
  }

  #drawStar(g: CanvasRenderingContext2D, n: Node, t: number): void {
    const c = STAR[n.vstate]
    const x = n._x,
      y = n._y,
      fl = n.flare || 0
    const isF = n.vstate === 'frontier',
      isC = n.vstate === 'claimed',
      isP = n.vstate === 'proposed'
    const beat = 0.5 + 0.5 * Math.sin(t * 2.8)
    const pulse = isF ? 0.8 + 0.2 * beat : 1
    const gr = (isF ? c.gr * (0.92 + 0.16 * beat) : c.gr) * (1 + fl * 0.5)

    const grd = g.createRadialGradient(x, y, 0, x, y, gr)
    grd.addColorStop(0, hexA(c.glow, Math.min(1, 0.85 * pulse + fl * 0.5)))
    grd.addColorStop(0.4, hexA(c.glow, 0.22 * pulse))
    grd.addColorStop(1, hexA(c.glow, 0))
    g.fillStyle = grd
    g.beginPath()
    g.arc(x, y, gr, 0, TAU)
    g.fill()

    const cr = c.r
    const cg = g.createRadialGradient(x, y, 0, x, y, cr * 1.35)
    cg.addColorStop(0, hexA(c.core, 1))
    cg.addColorStop(0.6, hexA(c.core, 0.92))
    cg.addColorStop(0.82, hexA(c.core, 0.45))
    cg.addColorStop(1, hexA(c.core, 0))
    g.fillStyle = cg
    g.beginPath()
    g.arc(x, y, cr * 1.35, 0, TAU)
    g.fill()

    if (fl > 0) {
      g.strokeStyle = hexA(c.core, fl * 0.7)
      g.lineWidth = 1.5 + 2 * fl
      g.beginPath()
      g.arc(x, y, c.r + (1 - fl) * 40, 0, TAU)
      g.stroke()
    }
    // A live claim breathes with two soft rings; a proposed star seals to one
    // steady ring — work landed, nothing circling. A session overlay speaks for
    // the claim when there is one, so the vanilla claimed rings stand down rather
    // than competing with the moon's orbit.
    if (isC && !n.sstate) {
      g.strokeStyle = hexA(c.core, 0.45 + 0.25 * beat)
      g.lineWidth = 1.5
      g.beginPath()
      g.arc(x, y, c.r + 5 + 1.2 * beat, 0, TAU)
      g.stroke()
      g.strokeStyle = hexA(c.core, 0.18 + 0.14 * beat)
      g.lineWidth = 1
      g.beginPath()
      g.arc(x, y, c.r + 11 + 1.8 * beat, 0, TAU)
      g.stroke()
    } else if (isP) {
      g.strokeStyle = hexA(c.core, 0.7)
      g.lineWidth = 1.6
      g.beginPath()
      g.arc(x, y, c.r + 6, 0, TAU)
      g.stroke()
    }
    if (n.sstate) this.#drawSession(g, n, x, y, c.r, c.gr, t)
    if (this.#selected === n.num) {
      g.strokeStyle = 'rgba(255,255,255,0.85)'
      g.lineWidth = 1.5
      g.beginPath()
      g.arc(x, y, c.r + 13, 0, TAU)
      g.stroke()
    }
  }

  // The session overlay (ticket 13), drawn straight from the grammar in
  // session.ts: a session is a body — an amber moon orbiting the star it holds.
  // `motion` is the liveness (orbits / crawls / frozen), `moon` is the pipeline
  // stage (circling / docked at the rim / frozen mid-orbit), and `marks` are the
  // shapes that ride along. Because motion and shape carry every state, the
  // overlay still reads with the colour taken away.
  #drawSession(
    g: CanvasRenderingContext2D,
    n: Node,
    x: number,
    y: number,
    r: number,
    gr: number,
    t: number,
  ): void {
    const s = n.sstate
    if (!s) return
    const gm = GRAMMAR[s]
    const orbR = r + 10
    const docked = gm.moon === 'docked'
    const frozen = gm.moon === 'frozen'

    // Human review warms the star itself toward gold before anything orbital —
    // the deliberate break: this one is a call to action, not a status.
    if (gm.marks.includes('ping-rings')) {
      const wash = g.createRadialGradient(x, y, 0, x, y, gr * 1.2)
      wash.addColorStop(0, hexA(SESSION_HUE.gold, 0.36))
      wash.addColorStop(1, hexA(SESSION_HUE.gold, 0))
      g.fillStyle = wash
      g.beginPath()
      g.arc(x, y, gr * 1.2, 0, TAU)
      g.fill()
    }

    // The orbital apparatus. A dead session greys the whole thing, not just the
    // moon — the orbit itself is defunct — and breaks it into a dashed line.
    if (frozen) {
      g.strokeStyle = hexA(SESSION_HUE.dead, 0.3)
      g.setLineDash([3, 5])
    } else {
      g.strokeStyle = hexA(SESSION_HUE.session, docked ? 0.12 : 0.16)
    }
    g.lineWidth = 1
    g.beginPath()
    g.arc(x, y, orbR, 0, TAU)
    g.stroke()
    g.setLineDash([])

    // Where the moon sits: docked at the rim once the work has landed, otherwise
    // somewhere on its orbit — sweeping, crawling, or stopped where it died.
    const speed = gm.motion === 'orbit' ? 1.5 : gm.motion === 'crawl' ? 0.18 : 0
    const ang = docked ? -TAU / 4 : frozen ? n.num * 1.3 : t * speed + n.num
    const mx = x + Math.cos(ang) * orbR,
      my = y + Math.sin(ang) * orbR

    if (gm.marks.includes('trail')) {
      for (let k = 1; k <= 3; k++) {
        const ta = ang - k * 0.22
        g.fillStyle = hexA(SESSION_HUE.session, 0.3 - k * 0.09)
        g.beginPath()
        g.arc(x + Math.cos(ta) * orbR, y + Math.sin(ta) * orbR, 1.6, 0, TAU)
        g.fill()
      }
    }

    const moonCol = frozen
      ? SESSION_HUE.dead
      : docked
        ? s === 'human-review'
          ? SESSION_HUE.beacon
          : SESSION_HUE.human
        : SESSION_HUE.session
    // Quiet blinks: the moon fades in and out as it crawls, so the hint reads
    // even where the crawl is too slow to see.
    const moonA = gm.marks.includes('blink') ? 0.35 + 0.3 * Math.sin(t * 1.2) : frozen ? 0.9 : 0.95
    g.fillStyle = hexA(moonCol, moonA)
    g.beginPath()
    g.arc(mx, my, docked ? 3.1 : frozen ? 2.4 : 2.7, 0, TAU)
    g.fill()

    // A dead moon wears a grey halo — a body stopped, ringed like a marker.
    if (gm.marks.includes('halo')) {
      g.strokeStyle = hexA(SESSION_HUE.dead, 0.7)
      g.lineWidth = 1
      g.beginPath()
      g.arc(mx, my, 4.6, 0, TAU)
      g.stroke()
    }

    // Agent review: a smaller violet body counter-orbiting the docked proposal —
    // an adversary circling the work, going the other way.
    if (gm.marks.includes('counter-orbit')) {
      const ra = -t * 2.8 + n.num,
        rr = orbR + 5
      for (let k = 1; k <= 3; k++) {
        const tb = ra + k * 0.3
        g.fillStyle = hexA(SESSION_HUE.violet, 0.32 - k * 0.09)
        g.beginPath()
        g.arc(x + Math.cos(tb) * rr, y + Math.sin(tb) * rr, 1.5, 0, TAU)
        g.fill()
      }
      g.fillStyle = hexA(SESSION_HUE.violet, 0.95)
      g.beginPath()
      g.arc(x + Math.cos(ra) * rr, y + Math.sin(ra) * rr, 2.3, 0, TAU)
      g.fill()
    }

    // …and the pings that emanate from a star that wants you.
    if (gm.marks.includes('ping-rings')) {
      for (let k = 0; k < 2; k++) {
        const u = mod(t / 1.7 + k * 0.5, 1)
        g.strokeStyle = hexA(SESSION_HUE.beacon, (1 - u) * 0.55)
        g.lineWidth = 1.6 * (1 - u) + 0.4
        g.beginPath()
        g.arc(x, y, r + 7 + u * 30, 0, TAU)
        g.stroke()
      }
    }
  }

  // A human-review star that has scrolled out of the free rect keeps calling
  // through a gold chevron pinned at the edge it left by, captioned with its
  // number — the call to action must not depend on where the camera is.
  #drawChevrons(g: CanvasRenderingContext2D): void {
    const r = this.#beckonRect()
    const pulse = 0.6 + 0.4 * Math.sin(this.#clock * 4)
    for (const num of this.beckoning()) {
      const n = this.#byNum.get(num)
      if (!n) continue
      const sx = n._x * this.#cam.s + this.#cam.x
      const sy = n._y * this.#cam.s + this.#cam.y
      const bx = clamp(sx, r.x0, r.x1),
        by = clamp(sy, r.y0, r.y1)
      const dx = sx - bx,
        dy = sy - by,
        d = Math.hypot(dx, dy) || 1
      g.save()
      g.translate(bx, by)
      g.rotate(Math.atan2(dy / d, dx / d))
      g.fillStyle = hexA(SESSION_HUE.gold, pulse)
      g.beginPath()
      g.moveTo(10, 0)
      g.lineTo(-4, -7)
      g.lineTo(-4, 7)
      g.closePath()
      g.fill()
      g.restore()
      g.font = '10px ui-monospace,SFMono-Regular,Menlo,monospace'
      g.textAlign = 'center'
      g.fillStyle = hexA(SESSION_HUE.gold, 0.9)
      g.fillText(`#${num < 10 ? '0' : ''}${num} wants you`, bx, by + (dy < 0 ? 22 : -14))
    }
  }

  // One fading line naming what just changed — drawn by the island, in the
  // island's idiom, so the chrome hosts no state of its own (ADR 0010).
  #drawTicker(g: CanvasRenderingContext2D): void {
    const a = this.#tickerAlpha()
    if (a <= 0) return
    const { cx } = this.#freeRect()
    g.font = '11px ui-monospace,SFMono-Regular,Menlo,monospace'
    g.textAlign = 'center'
    g.shadowColor = 'rgba(0,0,0,0.85)'
    g.shadowBlur = 4
    g.fillStyle = hexA(SESSION_HUE.gold, a)
    g.fillText('▸ ' + this.#tickerText, cx, this.#insets.top + 20)
    g.shadowBlur = 0
  }

  #drawLabels(g: CanvasRenderingContext2D): void {
    if (this.#cam.s < 0.22) return
    const numOnly = this.#cam.s < 0.42
    const fs = clamp(11 * Math.pow(this.#cam.s, 0.3), 8, 13)
    g.textAlign = 'center'
    g.font = fs.toFixed(1) + 'px ui-sans-serif,system-ui,sans-serif'
    g.shadowColor = 'rgba(0,0,0,0.85)'
    g.shadowBlur = 4
    const placed: { x: number; y: number; w: number }[] = []
    const h = fs + 2,
      step = h + 2,
      tries = [0, step, -step, 2 * step, -2 * step, 3 * step]
    for (const n of this.#nodes) {
      const sx = n._x * this.#cam.s + this.#cam.x
      const sy = n._y * this.#cam.s + this.#cam.y
      const c = STAR[n.vstate]
      let label = (n.num < 10 ? '0' : '') + n.num
      if (!numOnly) {
        const t2 = n.title.length > 30 ? n.title.slice(0, 29) + '…' : n.title
        label += '  ' + t2
      }
      const w = g.measureText(label).width
      const cy = sy + c.r * this.#cam.s + fs + 3
      let fy = cy
      for (const off of tries) {
        const ty = cy + off
        let ok = true
        for (const p of placed) {
          if (Math.abs(sx - p.x) < (w + p.w) / 2 + 4 && Math.abs(ty - p.y) < h + 2) {
            ok = false
            break
          }
        }
        if (ok) {
          fy = ty
          break
        }
      }
      placed.push({ x: sx, y: fy, w })
      g.fillStyle = LABEL[n.vstate]
      g.fillText(label, sx, fy)
    }
    g.shadowBlur = 0
  }
}

function now(): number {
  return (typeof performance !== 'undefined' ? performance.now() : Date.now()) / 1000
}
