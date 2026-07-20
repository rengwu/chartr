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
import { STAR, LABEL, visualState, hexA, type VisualState } from './theme'
import type { Ticket } from '../model'

export type SelectHandler = (num: number | null) => void

interface Node {
  num: number
  title: string
  type: string
  vstate: VisualState
  x: number
  y: number
  _x: number
  _y: number
  flare: number
}

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
  #clock = 0
  #last = 0
  #raf = 0
  #selected: number | null = null

  #starfield = makeStarfield()
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
  setModel(tickets: Ticket[]): void {
    const sig = structureSignature(tickets)
    this.#resolved = new Set(tickets.filter((t) => t.status === 'resolved').map((t) => t.num))

    if (sig === this.#sig && this.#nodes.length) {
      for (const t of tickets) {
        const n = this.#byNum.get(t.num)
        if (!n) continue
        n.title = t.title
        n.type = t.type
        n.vstate = visualState(t)
      }
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
  // camera eases the star into view, leaving room on the right for the pane.
  select(num: number | null): void {
    if (num !== null && !this.#byNum.has(num)) return
    this.#applySelection(num)
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
    if (num !== null) {
      const n = this.#byNum.get(num)!
      // Ease the star toward the left of centre, leaving room for a right pane.
      this.#goal.x = this.#w / 2 - 120 - n.x * this.#cam.s
      this.#goal.y = this.#h / 2 - n.y * this.#cam.s
    }
    this.#onSelect(num)
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
    const topInset = 44,
      botInset = 40
    const availH = Math.max(120, this.#h - topInset - botInset)
    const s = clamp(
      Math.min(this.#w / (maxx - minx || 1), availH / (maxy - miny || 1)),
      0.15,
      1.4,
    )
    const cx = (minx + maxx) / 2,
      cy = (miny + maxy) / 2
    this.#goal.s = s
    this.#goal.x = this.#w / 2 - cx * s
    this.#goal.y = topInset + availH / 2 - cy * s
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
    g.fillStyle = '#05070d'
    g.fillRect(0, 0, this.#w, this.#h)
    this.#drawStarfield(g)

    g.save()
    g.translate(this.#cam.x, this.#cam.y)
    g.scale(this.#cam.s, this.#cam.s)
    for (const e of this.#edges) this.#drawEdge(g, e)
    for (const n of this.#nodes) this.#drawStar(g, n, this.#clock)
    g.restore()
    this.#drawLabels(g)
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
    // steady ring — work landed, nothing circling (the session moons that will
    // ride a claim are a later, additive overlay).
    if (isC) {
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
    if (this.#selected === n.num) {
      g.strokeStyle = 'rgba(255,255,255,0.85)'
      g.lineWidth = 1.5
      g.beginPath()
      g.arc(x, y, c.r + 13, 0, TAU)
      g.stroke()
    }
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
