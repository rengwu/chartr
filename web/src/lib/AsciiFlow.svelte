<script lang="ts">
  import { onMount } from "svelte";
  import { readColor } from "./tokens";

  // Murky water, drawn as ASCII — the background behind the empty terminal pane.
  //
  // The important thing about this effect is what the cursor *doesn't* do: it
  // deposits nothing. The water is already there when the pane opens, a standing
  // field the pointer can only shove around. Painting ink under the cursor is
  // what made earlier versions read as a brush with a blob stuck to the pointer;
  // here the pointer contributes velocity alone, and what you see is the murk
  // being disturbed and slowly closing over again.
  //
  // This is an imperative island in the ADR 0010 sense: the chrome hosts it and
  // never reaches inside. Its one colour arrives resolved at the seam
  // (tokens.ts → readColor), never as a literal.
  //
  // ── How it stays cheap ───────────────────────────────────────────────────
  // The whole design is "pay for the disturbance, not for the pane":
  //
  //   · **The water at rest is a picture, not a simulation.** The resting field
  //     is rendered once into an offscreen canvas. A tick that touches nothing
  //     is a single `drawImage` of that picture — not thousands of glyph blits.
  //   · **Only the disturbed rectangle is simulated or redrawn.** An active box
  //     tracks where the water is moving; advection, diffusion and per-glyph
  //     drawing happen inside it and nowhere else. The rest of the pane is
  //     blitted from the resting picture in four strips around the box. Cost is
  //     proportional to the size of the stir, not the size of the window.
  //   · **Velocity is not self-advected.** That was two of the three bilinear
  //     samples per cell, and at this viscosity it is invisible — the field is
  //     smooth and damped hard. It just decays and diffuses.
  //   · **12.5Hz fixed tick**, decoupled from the frame rate. Slow water does
  //     not need 60 steps a second, and the sim behaves identically on a 120Hz
  //     panel and a 60Hz one. Note the physics constants below are per *tick*,
  //     so the tick rate also sets the pace: halving it halves how fast the
  //     water moves and heals, which is why this and the constants below are
  //     tuned together.
  //   · **The noise field is deterministic and anchored in pixel space**, so a
  //     resize reveals more of the same water instead of rolling a new field —
  //     and reallocation is debounced, so dragging the splitter does no work at
  //     all until it settles.
  //   · The loop stops dead once the water is still, leaving the resting picture
  //     on the canvas. A pane nobody is pointing at schedules nothing.
  //   · Reduced motion draws the resting field once and never animates.

  let { class: className = "" }: { class?: string } = $props();

  // Base glyph cell, in px. The live value coarsens on a very large pane so the
  // grid stays bounded: cost is per cell and cell count scales with pane *area*.
  const BASE_CELL = 12;
  const MAX_GRID = 12000;
  let cell = BASE_CELL;
  let fontPx = 11;

  const ALPHA_STEPS = 6; // quantization of ink → opacity; one atlas row each
  const INK_FLOOR = 0.05; // below this a cell is empty and is not drawn

  const TICK_MS = 80; // 12.5Hz — everything below is per tick, not per frame
  const RESIZE_SETTLE_MS = 160; // quiet period before a resize is acted on

  const ADVECT = 0.8; // fraction of stored velocity applied as displacement
  const VEL_DECAY = 0.9; // a stir coasts for a second or two, then is gone
  const DIFFUSE_INK = 0.26; // viscosity: what makes it cloud rather than streak
  const DIFFUSE_VEL = 0.22;
  const RELAX = 0.035; // pull back toward the resting field — the water healing
  const MAX_INK = 1.0;
  const STILL = 0.012; // below this the water counts as settled
  const HEALED = 0.02; // ink this close to the resting field counts as healed

  // The intensity ramp, by tier. Tiers 2–4 are directional: the glyph comes from
  // the cell's own velocity, so disturbed water reads as flow against the dashes
  // of the water at rest.
  const G_UNDER = 0;
  const G_DASH = 1;
  const G_RIGHT = 2;
  const G_LEFT = 3;
  const G_UP = 4;
  const G_DOWN = 5;
  const G_RIGHT2 = 6;
  const G_LEFT2 = 7;
  const G_O = 8;
  const G_ZERO = 9;
  const G_BIG_O = 10;
  const GLYPHS = ["_", "-", ">", "<", "^", "v", "»", "«", "o", "0", "O"];

  let host: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;

  let cols = 0;
  let rows = 0;
  let ambient = new Float32Array(0);
  let ink = new Float32Array(0);
  let scratch = new Float32Array(0);
  let vx = new Float32Array(0);
  let vy = new Float32Array(0);

  let atlas: HTMLCanvasElement | null = null;
  let resting: HTMLCanvasElement | null = null; // the water at rest, pre-rendered
  let tile = 0;
  let dpr = 1;
  let cssW = 0;
  let cssH = 0;

  // The disturbed rectangle, in cell coordinates, inclusive. Empty when x0 > x1.
  let x0 = 1;
  let x1 = 0;
  let y0 = 1;
  let y1 = 0;
  const boxEmpty = () => x0 > x1 || y0 > y1;

  let raf = 0;
  let acc = 0;
  let prev = 0;
  let tick = false; // alternates ticks, for the half-rate viscosity pass
  let inside = false;
  let haveLast = false;
  let lastX = 0;
  let lastY = 0;
  let toX = 0;
  let toY = 0;
  let pending = false;

  const calmer =
    typeof matchMedia === "function"
      ? matchMedia("(prefers-reduced-motion: reduce)")
      : null;

  function buildAtlas(color: string) {
    tile = Math.ceil(cell * dpr);
    const a = document.createElement("canvas");
    a.width = tile * GLYPHS.length;
    a.height = tile * ALPHA_STEPS;
    const g = a.getContext("2d");
    if (!g) return;
    g.scale(dpr, dpr);
    g.font = `${fontPx}px "IBM Plex Mono", ui-monospace, monospace`;
    g.textAlign = "center";
    g.textBaseline = "middle";
    g.fillStyle = color;
    for (let step = 0; step < ALPHA_STEPS; step++) {
      // Ink → opacity. The fringe stays a whisper and the cores come up near
      // full, so the water reads at a glance without the copy on top of it
      // losing the contrast fight.
      g.globalAlpha = 0.18 + (step / (ALPHA_STEPS - 1)) * 0.72;
      for (let i = 0; i < GLYPHS.length; i++) {
        g.fillText(GLYPHS[i], i * cell + cell / 2, step * cell + cell / 2);
      }
    }
    atlas = a;
  }

  // Deterministic value noise. Anchored to *pixel* coordinates with a fixed
  // seed, which is what makes a resize reveal more of the same water instead of
  // rolling an entirely new field — the bug that made dragging the splitter
  // look like static.
  function hash(ix: number, iy: number, seed: number): number {
    let h = (ix * 374761393 + iy * 668265263 + seed * 1274126177) | 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177);
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
  }

  function octave(
    px: number,
    py: number,
    spacing: number,
    seed: number,
  ): number {
    const gx = px / spacing;
    const gy = py / spacing;
    const i0 = Math.floor(gx);
    const j0 = Math.floor(gy);
    const fx = gx - i0;
    const fy = gy - j0;
    // Smoothstep the interpolants so the lattice doesn't show as a grid.
    const sx = fx * fx * (3 - 2 * fx);
    const sy = fy * fy * (3 - 2 * fy);
    const a = hash(i0, j0, seed);
    const b = hash(i0 + 1, j0, seed);
    const c = hash(i0, j0 + 1, seed);
    const d = hash(i0 + 1, j0 + 1, seed);
    const top = a + (b - a) * sx;
    const bot = c + (d - c) * sx;
    return top + (bot - top) * sy;
  }

  // The water at rest: two octaves, cut and squared so most of the pane is empty
  // and the remainder reads as a soft haze with a few denser cores. Sampled at
  // pixel positions, so it is stable across resizes and cell-size changes.
  function buildAmbient() {
    ambient = new Float32Array(cols * rows);
    const CUT = 0.47;
    for (let j = 0; j < rows; j++) {
      const py = j * cell;
      for (let i = 0; i < cols; i++) {
        const px = i * cell;
        const n = octave(px, py, 84, 1) * 0.34 + octave(px, py, 204, 2) * 0.22;
        const v = n / 0.56 - CUT;
        ambient[j * cols + i] =
          v > 0 ? ((v / (1 - CUT)) * 0.56) ** 2 * 1.15 : 0;
      }
    }
  }

  function chooseCell(w: number, h: number): boolean {
    const was = cell;
    let c = BASE_CELL;
    while (Math.ceil(w / c) * Math.ceil(h / c) > MAX_GRID) c++;
    cell = c;
    fontPx = c - 1;
    return cell !== was;
  }

  function allocate(w: number, h: number) {
    // Never below 2: the bilinear sample below reads i+1 / j+1.
    cols = Math.max(2, Math.ceil(w / cell));
    rows = Math.max(2, Math.ceil(h / cell));
    const n = cols * rows;
    ink = new Float32Array(n);
    scratch = new Float32Array(n);
    vx = new Float32Array(n);
    vy = new Float32Array(n);
    buildAmbient();
    ink.set(ambient); // the water is already there when the pane opens
    x0 = 1;
    x1 = 0;
    y0 = 1;
    y1 = 0;
  }

  // Pick the glyph and alpha step for one cell. Shared by the live path and the
  // resting picture so the two can never drift apart.
  function glyphFor(v: number, ax: number, ay: number): number {
    const t = v > MAX_INK ? MAX_INK : v;
    let tier = t >= 0.999 ? 7 : (t * 8) | 0;
    const spd = (ax < 0 ? -ax : ax) + (ay < 0 ? -ay : ay);
    // Moving water shows its direction: this is what makes a disturbance
    // legible as flow against the dashes of the water at rest.
    if (spd > 0.05 && tier < 2) tier = 2;
    if (tier <= 1) return tier === 0 ? G_UNDER : G_DASH;
    if (tier <= 4) {
      const horizontal = (ax < 0 ? -ax : ax) >= (ay < 0 ? -ay : ay);
      if (tier === 4 && horizontal) return ax >= 0 ? G_RIGHT2 : G_LEFT2;
      if (horizontal) return ax >= 0 ? G_RIGHT : G_LEFT;
      return ay >= 0 ? G_DOWN : G_UP;
    }
    return tier === 5 ? G_O : tier === 6 ? G_ZERO : G_BIG_O;
  }

  function alphaStep(v: number): number {
    const t = v > MAX_INK ? MAX_INK : v;
    const s = (t * ALPHA_STEPS) | 0;
    return s > ALPHA_STEPS - 1 ? ALPHA_STEPS - 1 : s;
  }

  // Render the resting field once into its own canvas. Every later tick that
  // touches nothing at all is a single blit of this.
  function buildResting() {
    if (!atlas) return;
    const c = document.createElement("canvas");
    c.width = Math.max(1, Math.round(cssW * dpr));
    c.height = Math.max(1, Math.round(cssH * dpr));
    const g = c.getContext("2d");
    if (!g) return;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    for (let j = 0; j < rows; j++) {
      for (let i = 0; i < cols; i++) {
        const v = ambient[j * cols + i];
        if (v < INK_FLOOR) continue;
        const gl = glyphFor(v, 0, 0);
        g.drawImage(
          atlas,
          gl * tile,
          alphaStep(v) * tile,
          tile,
          tile,
          i * cell,
          j * cell,
          cell,
          cell,
        );
      }
    }
    resting = c;
  }

  // Bilinear sample of a field at fractional cell coordinates, clamped at the
  // edges so ink pushed off-grid piles up rather than wrapping.
  function sample(f: Float32Array, x: number, y: number): number {
    if (x < 0) x = 0;
    else if (x > cols - 1.001) x = cols - 1.001;
    if (y < 0) y = 0;
    else if (y > rows - 1.001) y = rows - 1.001;
    const i = x | 0;
    const j = y | 0;
    const fx = x - i;
    const fy = y - j;
    const k = j * cols + i;
    const a = f[k];
    const b = f[k + 1];
    const c = f[k + cols];
    const d = f[k + cols + 1];
    return a + (b - a) * fx + (c - a + (d - c - b + a) * fx) * fy;
  }

  function grow(i0: number, j0: number, i1: number, j1: number) {
    if (boxEmpty()) {
      x0 = i0;
      x1 = i1;
      y0 = j0;
      y1 = j1;
      return;
    }
    if (i0 < x0) x0 = i0;
    if (i1 > x1) x1 = i1;
    if (j0 < y0) y0 = j0;
    if (j1 > y1) y1 = j1;
  }

  // The pointer's whole contribution: velocity, spread broadly and softly along
  // the segment it covered since the last tick. No ink is added — that is the
  // difference between disturbing water and painting on it.
  function stir(ax: number, ay: number, bx: number, by: number) {
    const dx = bx - ax;
    const dy = by - ay;
    const len = Math.hypot(dx, dy);
    const steps = Math.min(24, Math.max(1, Math.round(len / cell)));
    // A shove, not a velocity: small, hard-clamped, and accumulated by a field
    // that is already damping it. Handing the pointer's own speed to the water
    // is what makes an effect like this feel glued to the cursor.
    const ivx = Math.max(-0.5, Math.min(0.5, (dx / cell) * 0.05));
    const ivy = Math.max(-0.5, Math.min(0.5, (dy / cell) * 0.05));
    const R = 6; // wide and soft — a wake, not a dab
    for (let s = 0; s <= steps; s++) {
      const t = s / steps;
      const ci = (ax + dx * t) / cell;
      const cj = (ay + dy * t) / cell;
      const i0 = Math.max(0, Math.floor(ci) - R);
      const i1 = Math.min(cols - 1, Math.floor(ci) + R);
      const j0 = Math.max(0, Math.floor(cj) - R);
      const j1 = Math.min(rows - 1, Math.floor(cj) + R);
      for (let j = j0; j <= j1; j++) {
        for (let i = i0; i <= i1; i++) {
          const ex = i + 0.5 - ci;
          const ey = j + 0.5 - cj;
          const f = Math.exp(-(ex * ex + ey * ey) / 14);
          if (f < 0.02) continue;
          const k = j * cols + i;
          vx[k] += ivx * f;
          vy[k] += ivy * f;
        }
      }
    }
    grow(
      Math.max(0, Math.floor(Math.min(ax, bx) / cell) - R),
      Math.max(0, Math.floor(Math.min(ay, by) / cell) - R),
      Math.min(cols - 1, Math.ceil(Math.max(ax, bx) / cell) + R),
      Math.min(rows - 1, Math.ceil(Math.max(ay, by) / cell) + R),
    );
  }

  // Advect ink and relax it back toward the resting field — inside the active
  // box only. Velocity is not self-advected: it only decays here, and diffuses
  // below. Returns the motion left in the water.
  function advect(): number {
    // Copy the box (plus a margin the back-trace can reach) into scratch.
    const mi0 = Math.max(0, x0 - 2);
    const mi1 = Math.min(cols - 1, x1 + 2);
    const mj0 = Math.max(0, y0 - 2);
    const mj1 = Math.min(rows - 1, y1 + 2);
    for (let j = mj0; j <= mj1; j++) {
      const row = j * cols;
      scratch.set(ink.subarray(row + mi0, row + mi1 + 1), row + mi0);
    }
    let motion = 0;
    for (let j = y0; j <= y1; j++) {
      const row = j * cols;
      for (let i = x0; i <= x1; i++) {
        const k = row + i;
        const ux = vx[k];
        const uy = vy[k];
        const carried = sample(scratch, i - ux * ADVECT, j - uy * ADVECT);
        // Heal toward the resting field rather than decaying to nothing: the
        // murk is permanent, only the disturbance is temporary.
        ink[k] = carried + (ambient[k] - carried) * RELAX;
        vx[k] = ux * VEL_DECAY;
        vy[k] = uy * VEL_DECAY;
        motion += (ux < 0 ? -ux : ux) + (uy < 0 ? -uy : uy);
      }
    }
    return motion / Math.max(1, (x1 - x0 + 1) * (y1 - y0 + 1));
  }

  // Separable 3-tap diffusion over the active box, in place, using `scratch`.
  function diffuse(f: Float32Array, amount: number) {
    for (let j = y0; j <= y1; j++) {
      const row = j * cols;
      scratch.set(f.subarray(row + x0, row + x1 + 1), row + x0);
    }
    for (let j = y0; j <= y1; j++) {
      const row = j * cols;
      for (let i = x0; i <= x1; i++) {
        const k = row + i;
        const l = scratch[i > x0 ? k - 1 : k];
        const r = scratch[i < x1 ? k + 1 : k];
        f[k] = scratch[k] + ((l + r) * 0.5 - scratch[k]) * amount;
      }
    }
    for (let j = y0; j <= y1; j++) {
      const row = j * cols;
      scratch.set(f.subarray(row + x0, row + x1 + 1), row + x0);
    }
    for (let j = y0; j <= y1; j++) {
      const row = j * cols;
      for (let i = x0; i <= x1; i++) {
        const k = row + i;
        const u = scratch[j > y0 ? k - cols : k];
        const d = scratch[j < y1 ? k + cols : k];
        f[k] = scratch[k] + ((u + d) * 0.5 - scratch[k]) * amount;
      }
    }
  }

  // Blit the resting picture everywhere outside the active box, then draw the
  // box's cells individually. An untouched pane is one `drawImage`.
  function draw() {
    if (!ctx || !atlas || !resting) return;
    ctx.clearRect(0, 0, cssW, cssH);
    if (boxEmpty()) {
      ctx.drawImage(
        resting,
        0,
        0,
        resting.width,
        resting.height,
        0,
        0,
        cssW,
        cssH,
      );
      return;
    }
    const bx = x0 * cell;
    const by = y0 * cell;
    const bw = Math.min(cssW, (x1 + 1) * cell) - bx;
    const bh = Math.min(cssH, (y1 + 1) * cell) - by;
    const strip = (sx: number, sy: number, sw: number, sh: number) => {
      if (sw <= 0 || sh <= 0) return;
      ctx!.drawImage(
        resting!,
        sx * dpr,
        sy * dpr,
        sw * dpr,
        sh * dpr,
        sx,
        sy,
        sw,
        sh,
      );
    };
    strip(0, 0, cssW, by); // above
    strip(0, by + bh, cssW, cssH - by - bh); // below
    strip(0, by, bx, bh); // left
    strip(bx + bw, by, cssW - bx - bw, bh); // right
    for (let j = y0; j <= y1; j++) {
      const row = j * cols;
      for (let i = x0; i <= x1; i++) {
        const k = row + i;
        const v = ink[k];
        if (v < INK_FLOOR) continue;
        const gl = glyphFor(v, vx[k], vy[k]);
        ctx.drawImage(
          atlas,
          gl * tile,
          alphaStep(v) * tile,
          tile,
          tile,
          i * cell,
          j * cell,
          cell,
          cell,
        );
      }
    }
  }

  function step(): number {
    if (pending) {
      stir(lastX, lastY, toX, toY);
      lastX = toX;
      lastY = toY;
      pending = false;
    }
    if (boxEmpty()) return 0;
    // The disturbance spreads a cell a tick through diffusion; let the box
    // follow it rather than clipping the edges of the stir.
    grow(
      Math.max(0, x0 - 1),
      Math.max(0, y0 - 1),
      Math.min(cols - 1, x1 + 1),
      Math.min(rows - 1, y1 + 1),
    );
    const motion = advect();
    diffuse(ink, DIFFUSE_INK);
    // Velocity viscosity runs on alternate ticks at double strength: it is the
    // slowest-moving part of the sim and indistinguishable at half the rate.
    tick = !tick;
    if (tick) {
      diffuse(vx, DIFFUSE_VEL * 2);
      diffuse(vy, DIFFUSE_VEL * 2);
    }
    // Once the water has stopped moving *and* healed back to the resting field,
    // retire the box: the pane is a single blit again and the loop can stop.
    if (motion < STILL) {
      let healed = true;
      for (let j = y0; j <= y1 && healed; j++) {
        const row = j * cols;
        for (let i = x0; i <= x1; i++) {
          const d = ink[row + i] - ambient[row + i];
          if (d > HEALED || d < -HEALED) {
            healed = false;
            break;
          }
        }
      }
      if (healed) {
        for (let j = y0; j <= y1; j++) {
          const row = j * cols;
          ink.set(ambient.subarray(row + x0, row + x1 + 1), row + x0);
          vx.fill(0, row + x0, row + x1 + 1);
          vy.fill(0, row + x0, row + x1 + 1);
        }
        x0 = 1;
        x1 = 0;
        y0 = 1;
        y1 = 0;
      }
    }
    draw();
    return motion;
  }

  // rAF paces the loop, but the sim only advances on the fixed tick — the frame
  // rate is the display's, the physics is 12.5Hz.
  function loop(now: number) {
    raf = 0;
    const dt = prev ? now - prev : TICK_MS;
    prev = now;
    acc += dt > 250 ? TICK_MS : dt; // a backgrounded tab must not fast-forward
    let motion = 1;
    let stepped = false;
    while (acc >= TICK_MS) {
      acc -= TICK_MS;
      motion = step();
      stepped = true;
      if (acc > TICK_MS * 2) acc = 0; // never try to catch up more than a tick
    }
    if (!stepped || motion > STILL || !boxEmpty() || pending)
      raf = requestAnimationFrame(loop);
    else prev = 0;
  }

  function kick() {
    if (raf || !ctx) return;
    prev = 0;
    acc = TICK_MS; // step immediately rather than after a tick's delay
    raf = requestAnimationFrame(loop);
  }

  function onMove(e: PointerEvent) {
    const r = canvas.getBoundingClientRect();
    const x = e.clientX - r.left;
    const y = e.clientY - r.top;
    if (!haveLast) {
      // First move in: seat the stroke here so it doesn't drag a wake across the
      // whole pane from wherever the pointer was last seen.
      lastX = x;
      lastY = y;
      haveLast = true;
    }
    toX = x;
    toY = y;
    pending = true;
    inside = true;
    kick();
  }

  function onLeave() {
    haveLast = false;
    pending = false;
    inside = false;
    // The loop keeps running until the water settles, then stops on its own.
  }

  onMount(() => {
    ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;
    const color = readColor("--muted-foreground");

    // Everything expensive about a resize — reallocating the grid, rebuilding
    // the noise and re-rendering the resting picture — happens once, after the
    // drag has settled. During the drag the canvas simply stretches (it is
    // sized in CSS), so a splitter drag costs nothing at all.
    let settle: ReturnType<typeof setTimeout> | undefined;
    const apply = () => {
      settle = undefined;
      const r = host.getBoundingClientRect();
      const w = Math.round(r.width);
      const h = Math.round(r.height);
      const nextDpr = Math.min(2, window.devicePixelRatio || 1);
      if (w === cssW && h === cssH && nextDpr === dpr) return;
      if (w < 1 || h < 1) return;
      const dprChanged = nextDpr !== dpr;
      dpr = nextDpr;
      cssW = w;
      cssH = h;
      canvas.width = Math.max(1, Math.round(w * dpr));
      canvas.height = Math.max(1, Math.round(h * dpr));
      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
      const cellChanged = chooseCell(w, h);
      if (!atlas || dprChanged || cellChanged) buildAtlas(color);
      allocate(w, h);
      buildResting();
      draw();
    };
    apply();

    const ro = new ResizeObserver(() => {
      if (settle) clearTimeout(settle);
      settle = setTimeout(apply, RESIZE_SETTLE_MS);
    });
    ro.observe(host);

    // Reduced motion gets the water at rest and nothing else: no listeners, no
    // loop, no ticks.
    if (calmer?.matches) {
      return () => {
        ro.disconnect();
        if (settle) clearTimeout(settle);
      };
    }

    // The stir is read off the region this layer covers, not off the layer: the
    // canvas is pointer-events:none by design (it must never shadow the controls
    // that sit on it), so listening on itself would hear nothing, and giving it
    // hit-testing would leave dead patches wherever a button sits. Passive
    // listeners — this never calls preventDefault.
    const region = host.parentElement ?? host;
    region.addEventListener("pointermove", onMove, { passive: true });
    region.addEventListener("pointerleave", onLeave, { passive: true });

    return () => {
      ro.disconnect();
      if (settle) clearTimeout(settle);
      region.removeEventListener("pointermove", onMove);
      region.removeEventListener("pointerleave", onLeave);
      if (raf) cancelAnimationFrame(raf);
      raf = 0;
    };
  });
</script>

<div bind:this={host} class="ascii-flow {className}" aria-hidden="true">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  /* Decorative and inert: it never takes a pointer event away from the controls
     sitting on top of it, and carries no colour of its own — the glyph colour is
     handed in resolved at the token seam. The canvas is sized in CSS so a
     resize stretches it for free; the backing store is only re-cut once the
     drag settles. */
  .ascii-flow {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .ascii-flow canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
