// Ported from chartr/web/src/lib/starmap: original camera, palette, label solver,
// and animation grammar. Plain JavaScript; no build or runtime dependencies.
export const TAU = 6.2831853;
export function mulberry32(seed) {
  let t = seed >>> 0;
  return function () {
    t += 0x6d2b79f5;
    let r = Math.imul(t ^ (t >>> 15), 1 | t);
    r ^= r + Math.imul(r ^ (r >>> 7), 61 | r);
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
}
export function rankOf(nodes) {
  const rank = {};
  for (const n of nodes) rank[n.num] = 0;
  const edges = edgesOf(nodes);
  for (let pass = 0; pass < Math.min(nodes.length, 32); pass++) {
    for (const e of edges) {
      if (rank[e.from] === undefined || rank[e.to] === undefined) continue;
      if (rank[e.to] < rank[e.from] + 1)
        rank[e.to] = Math.min(12, rank[e.from] + 1);
    }
  }
  return rank;
}
export function edgesOf(nodes) {
  const present = new Set(nodes.map((n) => n.num));
  const edges = [];
  for (const n of nodes) {
    for (const b of n.blockedBy ?? []) {
      if (present.has(b)) edges.push({ from: b, to: n.num });
    }
  }
  return edges;
}
function ringR(rank) {
  return 130 + rank * 165;
}
export function computeLayout(nodes) {
  const sorted = [...new Map(nodes.map((n) => [n.num, n])).values()].sort(
    (a, b) => a.num - b.num,
  );
  const rank = rankOf(sorted);
  const edges = edgesOf(sorted);
  const pts = {};
  const rnd = mulberry32(1337);
  for (const n of sorted) {
    const ang = rnd() * TAU;
    const jit = (rnd() - 0.5) * 70;
    const R = ringR(rank[n.num]) + jit;
    pts[n.num] = { x: Math.cos(ang) * R, y: Math.sin(ang) * R };
  }
  if (sorted.length > 500) return pts;
  const REP = 9000,
    SPRING = 0.02,
    REST = 150,
    RADIAL = 0.05;
  for (let it = 0; it < 420; it++) {
    for (let i = 0; i < sorted.length; i++) {
      const a = pts[sorted[i].num];
      for (let j = i + 1; j < sorted.length; j++) {
        const b = pts[sorted[j].num];
        const dx = a.x - b.x,
          dy = a.y - b.y,
          d2 = dx * dx + dy * dy || 0.01,
          d = Math.sqrt(d2),
          f = REP / d2,
          ux = dx / d,
          uy = dy / d;
        a.x += ux * f;
        a.y += uy * f;
        b.x -= ux * f;
        b.y -= uy * f;
      }
    }
    for (const e of edges) {
      const a = pts[e.from],
        b = pts[e.to];
      const dx = b.x - a.x,
        dy = b.y - a.y,
        d = Math.hypot(dx, dy) || 0.01,
        f = (d - REST) * SPRING,
        ux = dx / d,
        uy = dy / d;
      a.x += ux * f;
      a.y += uy * f;
      b.x -= ux * f;
      b.y -= uy * f;
    }
    for (const n of sorted) {
      const p = pts[n.num];
      const d = Math.hypot(p.x, p.y) || 0.01,
        f = (ringR(rank[n.num]) - d) * RADIAL;
      p.x += (p.x / d) * f;
      p.y += (p.y / d) * f;
    }
  }
  return pts;
}
export function structureSignature(nodes) {
  const nums = nodes
    .map((n) => n.num)
    .sort((a, b) => a - b)
    .join(",");
  const edges = edgesOf(nodes)
    .map((e) => `${e.from}>${e.to}`)
    .sort()
    .join(",");
  return `${nums}|${edges}`;
}

export const STAR = {
  resolved: { core: "#b9d6c4", glow: "#5b9077", r: 5.4, gr: 24 },
  frontier: { core: "#8ad8ff", glow: "#2f9be0", r: 8.1, gr: 49 },
  claimed: { core: "#ffd873", glow: "#ffb020", r: 7.2, gr: 36 },
  blocked: { core: "#e2c3c3", glow: "#9a6f6f", r: 4.5, gr: 20 },
  out_of_scope: { core: "#948da4", glow: "#6b6478", r: 4.5, gr: 18 },
};
export const LABEL = {
  resolved: "#a2c1ac",
  frontier: "#b3e5ff",
  claimed: "#ffe6a0",
  blocked: "#d0b3b3",
  out_of_scope: "#a89fb2",
};
export function visualState(t) {
  switch (t.status) {
    case "resolved":
      return "resolved";
    case "claimed":
      return "claimed";
    case "out_of_scope":
      return "out_of_scope";
    case "open":
    default:
      return t.frontier ? "frontier" : "blocked";
  }
}
export const SESSION_HUE = {
  session: "#ffd873",
  gold: "#ffe6a0",
  dead: "#6b7280",
};
export function hexA(hex, a) {
  const r = parseInt(hex.slice(1, 3), 16),
    g = parseInt(hex.slice(3, 5), 16),
    b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r},${g},${b},${a})`;
}

export const GRAMMAR = {
  implementing: {
    hue: SESSION_HUE.session,
    motion: "orbit",
    moon: "orbiting",
    marks: ["trail"],
  },
  blocked: {
    hue: SESSION_HUE.session,
    motion: "crawl",
    moon: "orbiting",
    marks: ["blink"],
  },
  dead: {
    hue: SESSION_HUE.dead,
    motion: "still",
    moon: "frozen",
    marks: ["halo"],
  },
};
export function nonColorSignature(s) {
  const g = GRAMMAR[s];
  return [g.motion, g.moon, ...g.marks].join("|");
}
export const WIDTH_THRESHOLD = 600;
export const ASPECT_RATIO = 1.1;
export const WIDTH_BAND = 32;
export const ASPECT_BAND = 0.12;
export function dockByWidth(w) {
  return w < WIDTH_THRESHOLD ? "bottom" : "right";
}
export function dockByAspect(w, h) {
  if (w <= 0) return "right";
  return h > w * ASPECT_RATIO ? "bottom" : "right";
}
export function dockHybrid(w, h) {
  if (w <= 0) return "right";
  return w < WIDTH_THRESHOLD || h > w * ASPECT_RATIO ? "bottom" : "right";
}
export function decideDock(method, w, h, prev, hysteresis) {
  if (w <= 0 || h <= 0) return prev ?? "right";
  if (!hysteresis || prev === null) {
    return method === "width"
      ? dockByWidth(w)
      : method === "aspect"
        ? dockByAspect(w, h)
        : dockHybrid(w, h);
  }
  const r = h / w;
  switch (method) {
    case "width":
      if (prev === "bottom")
        return w >= WIDTH_THRESHOLD + WIDTH_BAND ? "right" : "bottom";
      return w < WIDTH_THRESHOLD - WIDTH_BAND ? "bottom" : "right";
    case "aspect":
      if (prev === "bottom")
        return r <= ASPECT_RATIO - ASPECT_BAND ? "right" : "bottom";
      return r > ASPECT_RATIO + ASPECT_BAND ? "bottom" : "right";
    case "hybrid":
      if (prev === "bottom") {
        const stay =
          w < WIDTH_THRESHOLD + WIDTH_BAND || r > ASPECT_RATIO - ASPECT_BAND;
        return stay ? "bottom" : "right";
      }
      const go =
        w < WIDTH_THRESHOLD - WIDTH_BAND || r > ASPECT_RATIO + ASPECT_BAND;
      return go ? "bottom" : "right";
  }
}

const DEFAULT_BG = "#05070d";
function hits(a, b) {
  return a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1;
}
const LABEL_PRIORITY = {
  frontier: 0,
  claimed: 1,
  resolved: 2,
  blocked: 3,
  out_of_scope: 4,
};
const CULL_MARGIN = 140;
const SIDE_HYSTERESIS = 1;
const BELOW = 1;
const ABOVE = -1;
const TITLE_MIN_SCALE = 0.42;
const TITLE_FULL_SCALE = 1.6;
const TITLE_MIN_CHARS = 12;
const TITLE_MAX_CHARS = 60;
const TITLE_RAMP_EASE = 0.7;
export function titleBudget(scale) {
  const u = clamp(
    (scale - TITLE_MIN_SCALE) / (TITLE_FULL_SCALE - TITLE_MIN_SCALE),
    0,
    1,
  );
  return Math.round(
    TITLE_MIN_CHARS +
      Math.pow(u, TITLE_RAMP_EASE) * (TITLE_MAX_CHARS - TITLE_MIN_CHARS),
  );
}
export function clipTitle(title, budget) {
  if (title.length <= budget) return title;
  const cut = title.slice(0, budget);
  const sp = cut.lastIndexOf(" ");
  return (sp >= budget - 6 && sp > 0 ? cut.slice(0, sp) : cut.trimEnd()) + "…";
}
const TICKER_HOLD = 4.2;
const TICKER_FADE = 0.5;
const CAM_TAU = 0.12;
const CAM_EPS = 0.02;
const SCALE_EPS = 0.0005;
const MIN_SCALE = 0.12;
const MAX_SCALE = 3;
const LINE_PX = 16;
const WHEEL_GAIN = 0.0016;
const PINCH_GAIN = 0.011;
const MAX_WHEEL_PX = 140;
const MAX_PINCH_PX = 50;
function mod(a, b) {
  return ((a % b) + b) % b;
}
function clamp(v, a, b) {
  return v < a ? a : v > b ? b : v;
}
function makeStarfield() {
  const specs = [
    { f: 0.15, n: 140, sz: 0.7, a: 0.45 },
    { f: 0.3, n: 80, sz: 1.1, a: 0.65 },
    { f: 0.5, n: 34, sz: 1.7, a: 0.9 },
  ];
  let t = 9001 >>> 0;
  const rnd = () => {
    t += 0x6d2b79f5;
    let r = Math.imul(t ^ (t >>> 15), 1 | t);
    r ^= r + Math.imul(r ^ (r >>> 7), 61 | r);
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
  return specs.map((sp) => {
    const stars = [];
    for (let i = 0; i < sp.n; i++) stars.push({ x: rnd(), y: rnd(), t: rnd() });
    return { f: sp.f, sz: sp.sz, a: sp.a, stars };
  });
}
class MapRenderer {
  #host = null;
  #canvas = null;
  #ctx = null;
  #dpr = 1;
  #w = 0;
  #h = 0;
  #nodes = [];
  #byNum = new Map();
  #edges = [];
  #sig = "";
  #resolved = new Set();
  #cam = { x: 0, y: 0, s: 1 };
  #goal = { x: 0, y: 0, s: 1 };
  #insets = { top: 16, right: 16, bottom: 16, left: 16 };
  #clock = 0;
  #last = 0;
  #raf = 0;
  #motion = null;
  #active = true;
  #fog = [];
  #gesture = null;
  #selected = null;
  #tickerText = "";
  #tickerAt = -1e9;
  #tickerTimer = 0;
  #starfield = makeStarfield();
  #labelCache = null;
  #labelEpoch = 0;
  #labelSide = new Map();
  #bg = DEFAULT_BG;
  #onSelect = () => {};
  #ro = null;
  #detach = [];
  mount(host) {
    this.#host = host;
    const canvas = document.createElement("canvas");
    canvas.className = "starmap-canvas";
    canvas.tabIndex = 0;
    canvas.setAttribute(
      "aria-label",
      "Ticket star map. Arrow keys select tickets, F fits the map, Escape closes details.",
    );
    host.appendChild(canvas);
    this.#canvas = canvas;
    this.#ctx = canvas.getContext("2d");
    this.#dpr = Math.max(
      1,
      (typeof window !== "undefined" && window.devicePixelRatio) || 1,
    );
    this.#motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const resume = () => this.invalidate();
    document.addEventListener("visibilitychange", resume);
    this.#motion.addEventListener("change", resume);
    this.#detach.push(
      () => document.removeEventListener("visibilitychange", resume),
      () => this.#motion.removeEventListener("change", resume),
    );
    this.#measure();
    if (typeof ResizeObserver !== "undefined") {
      this.#ro = new ResizeObserver(() => this.#onResize());
      this.#ro.observe(host);
    }
    this.#bindPointer(canvas);
    this.#refit(true);
    if (this.#ctx) {
      this.#last = now();
      this.#raf = requestAnimationFrame(this.#render);
    }
  }
  setModel(tickets, sessions = {}) {
    this.invalidate();
    this.#labelEpoch++;
    const sig = structureSignature(tickets);
    this.#resolved = new Set(
      tickets.filter((t) => t.status === "resolved").map((t) => t.num),
    );
    if (sig === this.#sig) {
      const changed = [];
      for (const t of tickets) {
        const n = this.#byNum.get(t.num);
        if (!n) continue;
        n.title = t.title;
        n.type = t.type;
        const vstate = visualState(t);
        const sstate = sessions[t.num] ?? null;
        if (vstate !== n.vstate || sstate !== n.sstate) {
          n.flare = 1;
          changed.push(
            `#${t.num < 10 ? "0" : ""}${t.num} → ${sstate ?? vstate.replace("_", " ")}`,
          );
        }
        n.vstate = vstate;
        n.sstate = sstate;
      }
      if (changed.length) this.#tick(changed.join("   ·   "));
      this.#refreshEdges(tickets);
      return;
    }
    this.#labelSide.clear();
    this.#sig = sig;
    if (!tickets.length) this.#tickerText = "";
    const pts = computeLayout(tickets);
    this.#nodes = tickets.map((t) => {
      const p = pts[t.num];
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
      };
    });
    this.#byNum = new Map(this.#nodes.map((n) => [n.num, n]));
    this.#refreshEdges(tickets);
    if (this.#selected !== null && !this.#byNum.has(this.#selected))
      this.#selected = null;
    if (this.#selected !== null) this.#seat(this.#selected);
    else this.#refit(false);
  }
  #refreshEdges(tickets) {
    this.#edges = edgesOf(tickets).map((e) => ({
      from: e.from,
      to: e.to,
      satisfied: this.#resolved.has(e.from),
    }));
  }
  onSelect(cb) {
    this.#onSelect = cb;
  }
  select(num) {
    if (num !== null && !this.#byNum.has(num)) return;
    this.#applySelection(num);
  }
  fit() {
    this.invalidate();
    this.#labelEpoch++;
    this.#refit(false);
    this.#settleIfHeadless();
  }
  setInsets(insets) {
    const next = { ...this.#insets, ...insets };
    if (
      next.top === this.#insets.top &&
      next.right === this.#insets.right &&
      next.bottom === this.#insets.bottom &&
      next.left === this.#insets.left
    ) {
      return;
    }
    const claiming =
      next.top > this.#insets.top ||
      next.right > this.#insets.right ||
      next.bottom > this.#insets.bottom ||
      next.left > this.#insets.left;
    this.#insets = next;
    this.invalidate();
    if (this.#selected !== null && this.#byNum.has(this.#selected)) {
      this.#seat(this.#selected);
    } else if (claiming) {
      this.#refit(false);
      this.#settleIfHeadless();
    }
  }
  camera() {
    return { ...this.#goal };
  }
  restoreCamera(cam) {
    if (!isFinite(cam.x) || !isFinite(cam.y) || !(cam.s > 0)) return;
    const s = clamp(cam.s, MIN_SCALE, MAX_SCALE);
    this.invalidate();
    this.#goal = { x: cam.x, y: cam.y, s };
    this.#cam = { ...this.#goal };
    this.#labelEpoch++;
  }
  setBackground(color) {
    this.#bg = color || DEFAULT_BG;
  }
  destroy() {
    clearTimeout(this.#tickerTimer);
    if (this.#raf) cancelAnimationFrame(this.#raf);
    this.#raf = 0;
    this.#ro?.disconnect();
    for (const off of this.#detach) off();
    this.#detach = [];
    this.#canvas?.remove();
    this.#canvas = null;
    this.#ctx = null;
    this.#host = null;
  }
  positions() {
    const out = {};
    for (const n of this.#nodes) out[n.num] = { x: n.x, y: n.y };
    return out;
  }
  overlays() {
    const out = {};
    for (const n of this.#nodes) if (n.sstate) out[n.num] = n.sstate;
    return out;
  }
  ticker() {
    return this.#tickerAlpha() > 0 ? this.#tickerText : null;
  }
  screenOf(num) {
    const n = this.#byNum.get(num);
    if (!n) return null;
    return {
      x: n.x * this.#cam.s + this.#cam.x,
      y: n.y * this.#cam.s + this.#cam.y,
    };
  }
  selectAtScreen(sx, sy) {
    let hit = null;
    for (const n of this.#nodes) {
      const px = n._x * this.#cam.s + this.#cam.x;
      const py = n._y * this.#cam.s + this.#cam.y;
      const r = Math.max(14, STAR[n.vstate].r * this.#cam.s + 10);
      if (Math.hypot(sx - px, sy - py) < r) hit = n;
    }
    const num = hit ? hit.num : null;
    if (hit) hit.flare = Math.max(hit.flare, 0.6);
    this.#applySelection(num);
    this.#onSelect(num);
    return num;
  }
  #applySelection(num) {
    this.#selected = num;
    this.#labelEpoch++;
    this.invalidate();
    if (num !== null) this.#seat(num);
  }
  #freeRect() {
    const left = this.#insets.left,
      right = this.#w - this.#insets.right,
      top = this.#insets.top,
      bottom = this.#h - this.#insets.bottom;
    return {
      cx: (left + right) / 2,
      cy: (top + bottom) / 2,
      availW: Math.max(80, right - left),
      availH: Math.max(80, bottom - top),
    };
  }
  #tick(msg) {
    this.#tickerText = msg;
    this.#tickerAt = now();
    clearTimeout(this.#tickerTimer);
    this.#tickerTimer = setTimeout(
      () => this.invalidate(),
      (TICKER_HOLD + TICKER_FADE) * 1000 + 20,
    );
  }
  #tickerAlpha() {
    if (!this.#tickerText) return 0;
    const age = now() - this.#tickerAt;
    if (age < TICKER_HOLD) return 1;
    return clamp(1 - (age - TICKER_HOLD) / TICKER_FADE, 0, 1);
  }
  #seat(num) {
    const n = this.#byNum.get(num);
    if (!n) return;
    const { cx, cy } = this.#freeRect();
    this.#goal.x = cx - n.x * this.#goal.s;
    this.#goal.y = cy - n.y * this.#goal.s;
    this.#settleIfHeadless();
  }
  #settleIfHeadless() {
    if (this.#ctx) return;
    this.#cam.x = this.#goal.x;
    this.#cam.y = this.#goal.y;
    this.#cam.s = this.#goal.s;
  }
  #pan(dx, dy) {
    this.invalidate();
    this.#goal.x += dx;
    this.#goal.y += dy;
    this.#settleIfHeadless();
  }
  #zoomAt(sx, sy, f) {
    if (!(f > 0) || !isFinite(f)) return;
    this.invalidate();
    const ns = clamp(this.#goal.s * f, MIN_SCALE, MAX_SCALE);
    const k = ns / this.#goal.s;
    this.#goal.x = sx - (sx - this.#goal.x) * k;
    this.#goal.y = sy - (sy - this.#goal.y) * k;
    this.#goal.s = ns;
    this.#settleIfHeadless();
  }
  #easeCamera(dt) {
    const cam = this.#cam,
      goal = this.#goal;
    const near =
      Math.abs(goal.x - cam.x) < CAM_EPS &&
      Math.abs(goal.y - cam.y) < CAM_EPS &&
      Math.abs(goal.s - cam.s) < SCALE_EPS;
    if (near) {
      cam.x = goal.x;
      cam.y = goal.y;
      cam.s = goal.s;
      return;
    }
    // Ease in world-units per pixel so the zoom anchor stays pinned throughout the flight.
    const a = this.#motion?.matches ? 1 : 1 - Math.exp(-dt / CAM_TAU);
    const z = 1 / cam.s,
      zg = 1 / goal.s;
    const fx = -cam.x * z,
      fy = -cam.y * z;
    const nz = z + (zg - z) * a;
    const nfx = fx + (-goal.x * zg - fx) * a;
    const nfy = fy + (-goal.y * zg - fy) * a;
    cam.s = 1 / nz;
    cam.x = -nfx / nz;
    cam.y = -nfy / nz;
  }
  #measure() {
    this.#dpr = Math.min(2, window.devicePixelRatio || 1);
    const host = this.#host;
    const w =
      host?.clientWidth ||
      (typeof window !== "undefined" ? window.innerWidth : 800);
    const h =
      host?.clientHeight ||
      (typeof window !== "undefined" ? window.innerHeight : 600);
    this.#w = w;
    this.#h = h;
    if (this.#canvas) {
      const width = Math.max(1, Math.round(w * this.#dpr)),
        height = Math.max(1, Math.round(h * this.#dpr));
      if (this.#canvas.width !== width) this.#canvas.width = width;
      if (this.#canvas.height !== height) this.#canvas.height = height;
      this.#canvas.style.width = w + "px";
      this.#canvas.style.height = h + "px";
    }
  }
  #onResize() {
    const w = this.#w,
      h = this.#h;
    const before = this.#freeRect();
    this.#measure();
    if (this.#w === w && this.#h === h) {
      this.invalidate();
      return;
    }
    const after = this.#freeRect();
    const s = this.#goal.s;
    const wx = (before.cx - this.#goal.x) / s;
    const wy = (before.cy - this.#goal.y) / s;
    this.#goal.x = after.cx - wx * s;
    this.#goal.y = after.cy - wy * s;
    this.#cam = { ...this.#goal };
    this.invalidate();
    this.#draw();
  }
  #refit(snap) {
    if (!this.#nodes.length && !this.#fog.length) return;
    let minx = 1e9,
      miny = 1e9,
      maxx = -1e9,
      maxy = -1e9;
    for (const n of [...this.#nodes, ...this.#fog]) {
      minx = Math.min(minx, n.x);
      miny = Math.min(miny, n.y);
      maxx = Math.max(maxx, n.x);
      maxy = Math.max(maxy, n.y);
    }
    const pad = 90;
    minx -= pad;
    miny -= pad;
    maxx += pad;
    maxy += pad;
    const { cx: fcx, cy: fcy, availW, availH } = this.#freeRect();
    const s = clamp(
      Math.min(availW / (maxx - minx || 1), availH / (maxy - miny || 1)),
      0.15,
      1.4,
    );
    const cx = (minx + maxx) / 2,
      cy = (miny + maxy) / 2;
    this.#goal.s = s;
    this.#goal.x = fcx - cx * s;
    this.#goal.y = fcy - cy * s;
    if (snap) {
      this.#cam.s = this.#goal.s;
      this.#cam.x = this.#goal.x;
      this.#cam.y = this.#goal.y;
    }
  }
  #bindPointer(canvas) {
    const pointers = new Map();
    let drag = null,
      pinch = null;
    const xy = (e) => {
      const r = canvas.getBoundingClientRect();
      return { x: e.clientX - r.left, y: e.clientY - r.top };
    };
    const pair = () => {
      const [a, b] = [...pointers.values()];
      return {
        x: (a.x + b.x) / 2,
        y: (a.y + b.y) / 2,
        distance: Math.max(1, Math.hypot(a.x - b.x, a.y - b.y)),
      };
    };
    const down = (e) => {
      if (e.button !== 0) return;
      canvas.focus({ preventScroll: true });
      canvas.setPointerCapture(e.pointerId);
      pointers.set(e.pointerId, xy(e));
      this.#gesture = null;
      if (pointers.size === 1) drag = { ...xy(e), start: xy(e), moved: false };
      else {
        pinch = pair();
        drag = null;
      }
      canvas.classList.add("drag");
    };
    const move = (e) => {
      if (!pointers.has(e.pointerId)) {
        const p = xy(e);
        const hit = this.#nodes.find((n) => {
          const q = this.screenOf(n.num);
          return Math.hypot(p.x - q.x, p.y - q.y) < 18;
        });
        canvas.style.cursor = hit ? "pointer" : "";
        canvas.title = hit ? `${hit.title} · ${hit.type}` : "";
        return;
      }
      const p = xy(e);
      pointers.set(e.pointerId, p);
      if (pointers.size >= 2) {
        const next = pair();
        if (pinch) {
          this.#pan(next.x - pinch.x, next.y - pinch.y);
          this.#zoomAt(next.x, next.y, next.distance / pinch.distance);
        }
        pinch = next;
      } else if (drag) {
        drag.moved ||= Math.hypot(p.x - drag.start.x, p.y - drag.start.y) > 4;
        this.#pan(p.x - drag.x, p.y - drag.y);
        Object.assign(drag, p);
      }
    };
    const up = (e) => {
      if (!pointers.has(e.pointerId)) return;
      if (e.type === "pointerup" && drag && !drag.moved) {
        const p = xy(e);
        this.selectAtScreen(p.x, p.y);
      }
      pointers.delete(e.pointerId);
      pinch = null;
      const remaining = [...pointers.values()][0];
      drag = remaining ? { ...remaining, start: remaining, moved: true } : null;
      if (!pointers.size) canvas.classList.remove("drag");
    };
    const wheel = (e) => {
      e.preventDefault();
      if (this.#gesture && now() - this.#gesture.at < 0.5) return;
      this.#gesture = null;
      const p = xy(e);
      const unit =
        e.deltaMode === 1 ? LINE_PX : e.deltaMode === 2 ? this.#h : 1;
      const cap = e.ctrlKey ? MAX_PINCH_PX : MAX_WHEEL_PX;
      const delta = clamp(e.deltaY * unit, -cap, cap);
      this.#zoomAt(
        p.x,
        p.y,
        Math.exp(-delta * (e.ctrlKey ? PINCH_GAIN : WHEEL_GAIN)),
      );
    };
    const gestureStart = (e) => {
      e.preventDefault();
      this.#gesture = { scale: e.scale || 1, at: now() };
    };
    const gestureChange = (e) => {
      e.preventDefault();
      const g = this.#gesture;
      if (!g) return;
      const p = xy(e),
        scale = e.scale || 1;
      this.#zoomAt(p.x, p.y, scale / g.scale);
      Object.assign(g, { scale, at: now() });
    };
    const gestureEnd = (e) => {
      e.preventDefault();
      this.#gesture = null;
    };
    const key = (e) => {
      if (e.key.toLowerCase() === "f") {
        e.preventDefault();
        this.fit();
      }
      if (["ArrowRight", "ArrowLeft"].includes(e.key) && this.#nodes.length) {
        e.preventDefault();
        const nodes = [...this.#nodes].sort((a, b) => a.num - b.num);
        const i = nodes.findIndex((n) => n.num === this.#selected);
        const d = e.key === "ArrowRight" ? 1 : -1;
        const n =
          nodes[
            (i < 0 ? (d === 1 ? 0 : nodes.length - 1) : i + d + nodes.length) %
              nodes.length
          ];
        this.#applySelection(n.num);
        this.#onSelect(n.num);
      }
    };
    for (const [name, handler] of Object.entries({
      pointerdown: down,
      pointermove: move,
      pointerup: up,
      pointercancel: up,
      lostpointercapture: up,
      wheel,
      gesturestart: gestureStart,
      gesturechange: gestureChange,
      gestureend: gestureEnd,
      keydown: key,
    })) {
      canvas.addEventListener(name, handler, { passive: false });
      this.#detach.push(() => canvas.removeEventListener(name, handler));
    }
  }
  invalidate() {
    if (!this.#raf && this.#ctx && this.#active && !document.hidden)
      this.#raf = requestAnimationFrame(this.#render);
  }
  setActive(active) {
    this.#active = active;
    if (active) this.invalidate();
    else {
      cancelAnimationFrame(this.#raf);
      this.#raf = 0;
    }
  }
  setFog(fog) {
    const points = this.positions();
    this.#fog = fog.map((f, i) => {
      const anchor = points[f.clears_with];
      // Unlinked fog also lives in the world, never pinned to the viewport.
      let hash = 2166136261;
      for (const c of f.title)
        hash = Math.imul(hash ^ c.charCodeAt(0), 16777619);
      const angle = ((hash >>> 0) / 4294967296) * TAU;
      return {
        ...f,
        x: anchor ? anchor.x + 120 : Math.cos(angle) * 360,
        y: anchor ? anchor.y - 90 : Math.sin(angle) * 360,
        phase: i * 1.7,
      };
    });
    this.invalidate();
  }
  fogPositions() {
    return this.#fog.map(({ x, y }) => ({ x, y }));
  }
  liveCamera() {
    return { ...this.#cam };
  }
  #drawFog(g) {
    for (const fog of this.#fog) {
      const x = fog.x * this.#cam.s + this.#cam.x;
      const y = fog.y * this.#cam.s + this.#cam.y;
      const r = 120 * this.#cam.s;
      if (x + r < 0 || x - r > this.#w || y + r < 0 || y - r > this.#h)
        continue;
      const alpha = 0.1 + 0.018 * Math.sin(this.#clock * 0.5 + fog.phase);
      const gradient = g.createRadialGradient(x, y, 0, x, y, r);
      gradient.addColorStop(0, `rgba(126,145,97,${alpha})`);
      gradient.addColorStop(1, "rgba(126,145,97,0)");
      g.fillStyle = gradient;
      g.fillRect(x - r, y - r, r * 2, r * 2);
      g.font = "italic 10px system-ui,sans-serif";
      g.textAlign = "center";
      g.fillStyle = "#909b80";
      const maxWidth = Math.max(40, 210 * this.#cam.s);
      let budget = Math.min(45, fog.title.length);
      let title = clipTitle(fog.title, budget);
      // Canvas maxWidth condenses glyphs; shorten the text instead.
      while (budget > 0 && g.measureText(title).width > maxWidth) {
        title = clipTitle(fog.title, --budget);
      }
      g.fillText(title, x, y);
    }
  }
  #render = () => {
    this.#raf = 0;
    if (document.hidden || !this.#active) return;
    if (!this.#ctx) return;
    const t = now();
    let dt = t - this.#last;
    if (dt < 0 || dt > 0.1) dt = 0.016;
    this.#last = t;
    this.#clock = this.#motion?.matches ? 0 : t;
    for (const n of this.#nodes) {
      const ph = n.num * 1.7;
      n._x = n.x + Math.sin(this.#clock * 0.7 + ph) * 2.4;
      n._y = n.y + Math.cos(this.#clock * 0.55 + ph) * 2.4;
      if (this.#motion?.matches) n.flare = 0;
      else if (n.flare > 0) n.flare = Math.max(0, n.flare - dt / 1.1);
    }
    this.#easeCamera(dt);
    this.#draw();
    if (!this.#motion?.matches) this.invalidate();
  };
  #draw() {
    const g = this.#ctx;
    if (!g) return;
    g.setTransform(this.#dpr, 0, 0, this.#dpr, 0, 0);
    g.fillStyle = this.#bg;
    g.fillRect(0, 0, this.#w, this.#h);
    this.#drawStarfield(g);
    this.#drawFog(g);
    g.save();
    g.translate(this.#cam.x, this.#cam.y);
    g.scale(this.#cam.s, this.#cam.s);
    for (const e of this.#edges) this.#drawEdge(g, e);
    for (const n of this.#nodes) this.#drawStar(g, n, this.#clock);
    g.restore();
    this.#drawLabels(g);
    this.#drawTicker(g);
  }
  #drawStarfield(g) {
    const W = this.#w,
      H = this.#h;
    for (const L of this.#starfield) {
      for (const s of L.stars) {
        const x = mod(s.x * W + this.#cam.x * L.f, W);
        const y = mod(s.y * H + this.#cam.y * L.f, H);
        g.globalAlpha =
          L.a * (0.65 + 0.35 * Math.sin(s.t * TAU + this.#clock * 0.25));
        g.fillStyle = "rgba(255,255,255,1)";
        g.fillRect(x, y, L.sz, L.sz);
      }
    }
    g.globalAlpha = 1;
  }
  #drawEdge(g, e) {
    const a = this.#byNum.get(e.from),
      b = this.#byNum.get(e.to);
    if (!a || !b) return;
    const ax = a._x,
      ay = a._y,
      bx = b._x,
      by = b._y;
    const mx = (ax + bx) / 2,
      my = (ay + by) / 2,
      dx = bx - ax,
      dy = by - ay,
      len = Math.hypot(dx, dy) || 1;
    const nx = -dy / len,
      ny = dx / len,
      bow = Math.min(46, len * 0.13),
      cx = mx + nx * bow,
      cy = my + ny * bow;
    g.beginPath();
    g.moveTo(ax, ay);
    g.quadraticCurveTo(cx, cy, bx, by);
    if (e.satisfied) {
      g.strokeStyle = "rgba(160,192,166,0.62)";
      g.lineWidth = 1.8;
      g.setLineDash([]);
    } else {
      g.strokeStyle = "rgba(132,146,168,0.34)";
      g.lineWidth = 1.3;
      g.setLineDash([4, 6]);
    }
    g.stroke();
    g.setLineDash([]);
    if (e.satisfied) {
      for (let k = 0; k < 2; k++) {
        const u = mod(
            this.#clock * 0.1 + k / 2 + (e.from * 0.13 + e.to * 0.07),
            1,
          ),
          m = 1 - u;
        const fx = m * m * ax + 2 * m * u * cx + u * u * bx,
          fy = m * m * ay + 2 * m * u * cy + u * u * by;
        g.fillStyle =
          "rgba(190,225,200," + (0.16 + 0.44 * Math.sin(u * Math.PI)) + ")";
        g.beginPath();
        g.arc(fx, fy, 1.7, 0, TAU);
        g.fill();
      }
    }
    const midx = 0.25 * ax + 0.5 * cx + 0.25 * bx,
      midy = 0.25 * ay + 0.5 * cy + 0.25 * by;
    const al = Math.hypot(dx, dy) || 1,
      ux = dx / al,
      uy = dy / al;
    const ah = 7,
      aw = 3.8,
      px = -uy,
      py = ux,
      tipx = midx + ux * ah * 0.5,
      tipy = midy + uy * ah * 0.5;
    g.beginPath();
    g.moveTo(tipx, tipy);
    g.lineTo(tipx - ux * ah + px * aw, tipy - uy * ah + py * aw);
    g.lineTo(tipx - ux * ah - px * aw, tipy - uy * ah - py * aw);
    g.closePath();
    g.fillStyle = e.satisfied ? "#aecdb6" : "#6f7889";
    g.fill();
  }
  #drawStar(g, n, t) {
    const c = STAR[n.vstate];
    const x = n._x,
      y = n._y,
      fl = n.flare || 0;
    const isF = n.vstate === "frontier",
      isC = n.vstate === "claimed";
    const beat = 0.5 + 0.5 * Math.sin(t * 2.8);
    const pulse = isF ? 0.8 + 0.2 * beat : 1;
    const gr = (isF ? c.gr * (0.92 + 0.16 * beat) : c.gr) * (1 + fl * 0.5);
    const grd = g.createRadialGradient(x, y, 0, x, y, gr);
    grd.addColorStop(0, hexA(c.glow, Math.min(1, 0.85 * pulse + fl * 0.5)));
    grd.addColorStop(0.4, hexA(c.glow, 0.22 * pulse));
    grd.addColorStop(1, hexA(c.glow, 0));
    g.fillStyle = grd;
    g.beginPath();
    g.arc(x, y, gr, 0, TAU);
    g.fill();
    const cr = c.r;
    const cg = g.createRadialGradient(x, y, 0, x, y, cr * 1.35);
    cg.addColorStop(0, hexA(c.core, 1));
    cg.addColorStop(0.6, hexA(c.core, 0.92));
    cg.addColorStop(0.82, hexA(c.core, 0.45));
    cg.addColorStop(1, hexA(c.core, 0));
    g.fillStyle = cg;
    g.beginPath();
    g.arc(x, y, cr * 1.35, 0, TAU);
    g.fill();
    if (fl > 0) {
      g.strokeStyle = hexA(c.core, fl * 0.7);
      g.lineWidth = 1.5 + 2 * fl;
      g.beginPath();
      g.arc(x, y, c.r + (1 - fl) * 40, 0, TAU);
      g.stroke();
    }
    if (isC && !n.sstate) {
      g.strokeStyle = hexA(c.core, 0.45 + 0.25 * beat);
      g.lineWidth = 1.5;
      g.beginPath();
      g.arc(x, y, c.r + 5 + 1.2 * beat, 0, TAU);
      g.stroke();
      g.strokeStyle = hexA(c.core, 0.18 + 0.14 * beat);
      g.lineWidth = 1;
      g.beginPath();
      g.arc(x, y, c.r + 11 + 1.8 * beat, 0, TAU);
      g.stroke();
    }
    if (n.sstate) this.#drawSession(g, n, x, y, c.r, t);
    if (this.#selected === n.num) {
      g.strokeStyle = "rgba(255,255,255,0.85)";
      g.lineWidth = 1.5;
      g.beginPath();
      g.arc(x, y, c.r + 13, 0, TAU);
      g.stroke();
    }
  }
  #drawSession(g, n, x, y, r, t) {
    const s = n.sstate;
    if (!s) return;
    const gm = GRAMMAR[s];
    const orbR = r + 10;
    const frozen = gm.moon === "frozen";
    if (frozen) {
      g.strokeStyle = hexA(SESSION_HUE.dead, 0.3);
      g.setLineDash([3, 5]);
    } else {
      g.strokeStyle = hexA(SESSION_HUE.session, 0.16);
    }
    g.lineWidth = 1;
    g.beginPath();
    g.arc(x, y, orbR, 0, TAU);
    g.stroke();
    g.setLineDash([]);
    const speed =
      gm.motion === "orbit" ? 1.5 : gm.motion === "crawl" ? 0.18 : 0;
    const ang = frozen ? n.num * 1.3 : t * speed + n.num;
    const mx = x + Math.cos(ang) * orbR,
      my = y + Math.sin(ang) * orbR;
    if (gm.marks.includes("trail")) {
      for (let k = 1; k <= 3; k++) {
        const ta = ang - k * 0.22;
        g.fillStyle = hexA(SESSION_HUE.session, 0.3 - k * 0.09);
        g.beginPath();
        g.arc(x + Math.cos(ta) * orbR, y + Math.sin(ta) * orbR, 1.6, 0, TAU);
        g.fill();
      }
    }
    const moonCol = frozen ? SESSION_HUE.dead : SESSION_HUE.session;
    const moonA = gm.marks.includes("blink")
      ? 0.35 + 0.3 * Math.sin(t * 1.2)
      : frozen
        ? 0.9
        : 0.95;
    g.fillStyle = hexA(moonCol, moonA);
    g.beginPath();
    g.arc(mx, my, frozen ? 2.4 : 2.7, 0, TAU);
    g.fill();
    if (gm.marks.includes("halo")) {
      g.strokeStyle = hexA(SESSION_HUE.dead, 0.7);
      g.lineWidth = 1;
      g.beginPath();
      g.arc(mx, my, 4.6, 0, TAU);
      g.stroke();
    }
  }
  #drawTicker(g) {
    const a = this.#tickerAlpha();
    if (a <= 0) return;
    const { cx } = this.#freeRect();
    g.font = "11px ui-monospace,SFMono-Regular,Menlo,monospace";
    g.textAlign = "center";
    g.shadowColor = "rgba(0,0,0,0.85)";
    g.shadowBlur = 4;
    g.fillStyle = hexA(SESSION_HUE.gold, a);
    g.fillText("▸ " + this.#tickerText, cx, this.#insets.top + 20);
    g.shadowBlur = 0;
  }
  #drawLabels(g) {
    if (this.#cam.s < 0.22) return;
    const key =
      this.#cam.x.toFixed(2) +
      "|" +
      this.#cam.y.toFixed(2) +
      "|" +
      this.#cam.s.toFixed(4) +
      "|" +
      this.#w +
      "x" +
      this.#h +
      "|" +
      this.#labelEpoch;
    let cache = this.#labelCache;
    if (!cache || cache.key !== key) {
      cache = this.#solveLabels(g, key);
      this.#labelCache = cache;
    }
    g.textAlign = "center";
    g.font = cache.fs.toFixed(1) + "px ui-sans-serif,system-ui,sans-serif";
    g.shadowColor = "rgba(0,0,0,0.85)";
    g.shadowBlur = 4;
    for (const it of cache.items) {
      g.fillStyle = it.fill;
      g.fillText(it.text, it.x, it.y);
    }
    g.shadowBlur = 0;
  }
  #solveLabels(g, key) {
    const numOnly = this.#cam.s < TITLE_MIN_SCALE;
    const budget = titleBudget(this.#cam.s);
    const fs = clamp(11 * Math.pow(this.#cam.s, 0.3), 8, 13);
    g.font = fs.toFixed(1) + "px ui-sans-serif,system-ui,sans-serif";
    const s = this.#cam.s;
    const gap = 4;
    const step = fs + 4;
    const vis = [];
    for (const n of this.#nodes) {
      const sx = n.x * s + this.#cam.x;
      const sy = n.y * s + this.#cam.y;
      if (sx < -CULL_MARGIN || sx > this.#w + CULL_MARGIN) continue;
      if (sy < -CULL_MARGIN || sy > this.#h + CULL_MARGIN) continue;
      const c = STAR[n.vstate];
      let r = c.r + 2;
      if (n.sstate) r = c.r + 15;
      if (this.#selected === n.num) r = Math.max(r, c.r + 14);
      vis.push({ n, sx, sy, rad: r * s });
    }
    const obstacles = vis.map((v) => ({
      x0: v.sx - v.rad,
      y0: v.sy - v.rad,
      x1: v.sx + v.rad,
      y1: v.sy + v.rad,
    }));
    const order = [...vis].sort((a, b) => {
      const pa = this.#selected === a.n.num ? -1 : LABEL_PRIORITY[a.n.vstate];
      const pb = this.#selected === b.n.num ? -1 : LABEL_PRIORITY[b.n.vstate];
      return pa - pb || a.n.num - b.n.num;
    });
    const items = [];
    for (const v of order) {
      let text = (v.n.num < 10 ? "0" : "") + v.n.num;
      if (!numOnly) text += "  " + clipTitle(v.n.title, budget);
      const w = g.measureText(text).width;
      const slot = (side, k) =>
        side === BELOW
          ? v.sy + v.rad + gap + fs * 0.82 + k * step
          : v.sy - v.rad - gap - fs * 0.22 - k * step;
      const kept = this.#labelSide.get(v.n.num) ?? BELOW;
      const other = kept === BELOW ? ABOVE : BELOW;
      const cands = [];
      for (let k = 0; k <= 3 + SIDE_HYSTERESIS; k++) {
        if (k <= 3) cands.push({ y: slot(kept, k), side: kept });
        const j = k - SIDE_HYSTERESIS;
        if (j >= 0 && j <= 3) cands.push({ y: slot(other, j), side: other });
      }
      const left = v.sx - w / 2;
      for (const c of cands) {
        const box = {
          x0: left - 3,
          y0: c.y - fs * 0.82 - 2,
          x1: left + w + 3,
          y1: c.y + fs * 0.22 + 2,
        };
        let ok = true;
        for (const o of obstacles) {
          if (hits(box, o)) {
            ok = false;
            break;
          }
        }
        if (!ok) continue;
        obstacles.push(box);
        items.push({ text, x: v.sx, y: c.y, fill: LABEL[v.n.vstate] });
        this.#labelSide.set(v.n.num, c.side);
        break;
      }
    }
    return { key, fs, items };
  }
}
function now() {
  return (
    (typeof performance !== "undefined" ? performance.now() : Date.now()) / 1000
  );
}

// Host model adaptation and in-pane camera memory stay outside the renderer.
const ticketModel = (t) => ({
  num: t.number,
  title: t.title,
  type: t.kind,
  blockedBy: [...t.blockers].sort((a, b) => a - b),
  frontier: t.frontier || t.state === "Ready",
  status:
    {
      Resolved: "resolved",
      Claimed: "claimed",
      "Out of scope": "out_of_scope",
    }[t.state] || "open",
});
export const palette = Object.fromEntries(
  Object.entries({
    Ready: "frontier",
    Claimed: "claimed",
    Resolved: "resolved",
    Blocked: "blocked",
    "Out of scope": "out_of_scope",
  }).map(([key, state]) => {
    const s = STAR[state];
    return [key, [s.core, s.glow, s.r, s.gr]];
  }),
);
export function layout(tickets) {
  return new Map(
    Object.entries(computeLayout(tickets.map(ticketModel))).map(([n, p]) => [
      Number(n),
      p,
    ]),
  );
}
export class StarMap {
  constructor(host, onSelect) {
    this.host = host;
    this.renderer = new MapRenderer();
    this.renderer.mount(host);
    this.renderer.setBackground("#10110e");
    this.renderer.onSelect(onSelect);
    this.poses = new Map();
    this.selected = null;
  }
  setModel(map) {
    const key = map?.slug || "";
    const changed = key !== this.key;
    if (changed && this.key) this.poses.set(this.key, this.renderer.camera());
    this.renderer.setActive(!!map);
    if (!map) {
      this.key = "";
      this.selected = null;
      return;
    }
    if (changed) {
      this.renderer.select(null);
      this.renderer.setFog([]);
      this.renderer.setModel([]);
      this.selected = null;
    }
    this.renderer.setModel(map.tickets.map(ticketModel));
    this.renderer.setFog(map.fog || []);
    this.key = key;
    if (changed) {
      if (this.poses.has(key)) this.renderer.restoreCamera(this.poses.get(key));
      else {
        this.renderer.fit();
        this.renderer.restoreCamera(this.renderer.camera());
      }
    }
  }
  setSelection(number) {
    if (number === this.selected) return;
    this.selected = number;
    this.renderer.select(number);
  }
  setInsets(right, bottom) {
    this.renderer.setInsets({
      top: 52,
      left: 16,
      right: right + 16,
      bottom: bottom + 48,
    });
  }
  fit() {
    this.renderer.fit();
  }
  focus() {
    this.host.querySelector("canvas").focus();
  }
  destroy() {
    this.renderer.destroy();
  }
}
