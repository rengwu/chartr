// Deterministic star-map layout. Positions are seeded from the ticket data
// itself and relaxed a fixed number of steps, so the same map lays out the same
// way every load — a ticket stays where you learned it (starmap-design.md,
// decision 3). This is a clean TypeScript reimplementation behind the island
// seam (ADR 0010), with the tuned constants cribbed directly from the
// wayfinder-maps renderer to prevent feel-drift.
//
// The function is pure and depends only on ticket numbers and their blocked_by
// edges — never on status. That is the load-bearing property the seam tests
// pin: a model push that changes only a ticket's status can never move a star,
// because status is not an input to layout at all.

export const TAU = 6.2831853

export interface LayoutNode {
  num: number
  blockedBy?: number[]
}

export interface Point {
  x: number
  y: number
}

// A small, fast, seedable PRNG (mulberry32) — the same generator the shipped
// viewer seeds its layout with, so a given map relaxes into the same shape.
export function mulberry32(seed: number): () => number {
  let t = seed >>> 0
  return function () {
    t += 0x6d2b79f5
    let r = Math.imul(t ^ (t >>> 15), 1 | t)
    r ^= r + Math.imul(r ^ (r >>> 7), 61 | r)
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296
  }
}

// Dependency depth per node: a ticket's rank is one past its deepest blocker, so
// roots sit at rank 0 and the layout radiates outward (the `Layers()` rank the
// old renderer read from the model, recomputed here from the edges the snapshot
// already carries).
export function rankOf(nodes: LayoutNode[]): Record<number, number> {
  const rank: Record<number, number> = {}
  for (const n of nodes) rank[n.num] = 0
  const edges = edgesOf(nodes)
  for (let pass = 0; pass < nodes.length; pass++) {
    for (const e of edges) {
      if (rank[e.from] === undefined || rank[e.to] === undefined) continue
      if (rank[e.to] < rank[e.from] + 1) rank[e.to] = rank[e.from] + 1
    }
  }
  return rank
}

export interface Edge {
  from: number
  to: number
}

// The blocked_by edges as blocker→dependent, dropping any that dangle onto a
// ticket the map doesn't contain (a malformed map still lays out — ADR spec:
// adoption is never gated on lint).
export function edgesOf(nodes: LayoutNode[]): Edge[] {
  const present = new Set(nodes.map((n) => n.num))
  const edges: Edge[] = []
  for (const n of nodes) {
    for (const b of n.blockedBy ?? []) {
      if (present.has(b)) edges.push({ from: b, to: n.num })
    }
  }
  return edges
}

function ringR(rank: number): number {
  return 130 + rank * 165
}

// Compute the deterministic layout: a physics relaxation that spreads nodes into
// an organic constellation, each soft-pulled toward a radius set by its
// dependency depth. Nodes are seeded in ascending `num` order so the single RNG
// stream — and therefore the whole layout — is independent of the order tickets
// happen to arrive in the snapshot.
export function computeLayout(nodes: LayoutNode[]): Record<number, Point> {
  const sorted = [...nodes].sort((a, b) => a.num - b.num)
  const rank = rankOf(sorted)
  const edges = edgesOf(sorted)

  const pts: Record<number, Point> = {}
  const rnd = mulberry32(1337)
  for (const n of sorted) {
    const ang = rnd() * TAU
    const jit = (rnd() - 0.5) * 70
    const R = ringR(rank[n.num]) + jit
    pts[n.num] = { x: Math.cos(ang) * R, y: Math.sin(ang) * R }
  }

  const REP = 9000,
    SPRING = 0.02,
    REST = 150,
    RADIAL = 0.05
  for (let it = 0; it < 420; it++) {
    // Pairwise repulsion: stars push apart so the constellation doesn't clump.
    for (let i = 0; i < sorted.length; i++) {
      const a = pts[sorted[i].num]
      for (let j = i + 1; j < sorted.length; j++) {
        const b = pts[sorted[j].num]
        const dx = a.x - b.x,
          dy = a.y - b.y,
          d2 = dx * dx + dy * dy || 0.01,
          d = Math.sqrt(d2),
          f = REP / d2,
          ux = dx / d,
          uy = dy / d
        a.x += ux * f
        a.y += uy * f
        b.x -= ux * f
        b.y -= uy * f
      }
    }
    // Edge springs: a blocker and its dependent settle toward a rest length, so
    // "what unblocks what" stays mostly monotonic and readable.
    for (const e of edges) {
      const a = pts[e.from],
        b = pts[e.to]
      const dx = b.x - a.x,
        dy = b.y - a.y,
        d = Math.hypot(dx, dy) || 0.01,
        f = (d - REST) * SPRING,
        ux = dx / d,
        uy = dy / d
      a.x += ux * f
      a.y += uy * f
      b.x -= ux * f
      b.y -= uy * f
    }
    // Radial pull toward the depth ring: roots drift inward, deeper tickets rim.
    for (const n of sorted) {
      const p = pts[n.num]
      const d = Math.hypot(p.x, p.y) || 0.01,
        f = (ringR(rank[n.num]) - d) * RADIAL
      p.x += (p.x / d) * f
      p.y += (p.y / d) * f
    }
  }
  return pts
}

// A stable signature of a map's *structure* — its ticket numbers and edges, not
// their statuses. The renderer recomputes layout only when this changes, so a
// pure status push keeps every star exactly where it was.
export function structureSignature(nodes: LayoutNode[]): string {
  const nums = nodes
    .map((n) => n.num)
    .sort((a, b) => a - b)
    .join(',')
  const edges = edgesOf(nodes)
    .map((e) => `${e.from}>${e.to}`)
    .sort()
    .join(',')
  return `${nums}|${edges}`
}
