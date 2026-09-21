import { test } from "node:test";
import assert from "node:assert/strict";
import { layout, makeStarfield, palette } from "../starmap.js";

test("constellations are deterministic and independent of statuses and input order", () => {
  const nodes = [
    { number: 1, blockers: [], state: "Ready" },
    { number: 2, blockers: [1], state: "Blocked" },
    { number: 3, blockers: [1, 2], state: "Blocked" },
  ];
  const expected = layout(nodes);
  assert.deepEqual(
    layout([...nodes].reverse().map((t) => ({ ...t, state: "Resolved" }))),
    expected,
  );
  assert.equal(expected.size, 3);
  assert.equal(Object.keys(palette).length, 5);
});
test("empty, cyclic, and large maps stay finite", () => {
  assert.equal(layout([]).size, 0);
  for (const nodes of [
    [
      { number: 1, blockers: [2] },
      { number: 2, blockers: [1] },
    ],
    Array.from({ length: 600 }, (_, i) => ({
      number: i + 1,
      blockers: i ? [i] : [],
    })),
  ]) {
    assert.ok(
      [...layout(nodes).values()].every(
        (p) => Number.isFinite(p.x) && Number.isFinite(p.y),
      ),
    );
  }
});
test("starfield keeps a deterministic layered depth profile", () => {
  const first = makeStarfield();
  const second = makeStarfield();
  assert.deepEqual(first, second);
  assert.deepEqual(
    first.map(({ depth, size, alpha, stars }) => ({
      depth,
      size,
      alpha,
      count: stars.length,
    })),
    [
      { depth: 0.06, size: 0.55, alpha: 0.32, count: 1000 },
      { depth: 0.24, size: 0.8, alpha: 0.47, count: 550 },
      { depth: 0.65, size: 1.05, alpha: 0.62, count: 350 },
      { depth: 1.2, size: 1.5, alpha: 0.8, count: 180 },
    ],
  );
});

const { decideDock, titleBudget, clipTitle } = await import("../starmap.js");
test("detail docking uses both shape and width, with a stable resize band", () => {
  assert.equal(decideDock("hybrid", 1200, 800, null, true), "right");
  assert.equal(decideDock("hybrid", 560, 680, "right", true), "bottom");
  assert.equal(decideDock("hybrid", 600, 680, "bottom", true), "bottom");
  assert.equal(decideDock("hybrid", 800, 1000, "right", true), "bottom");
  assert.equal(decideDock("hybrid", 900, 700, "bottom", true), "right");
});
test("zoom reveals titles progressively and clips at useful word boundaries", () => {
  assert.equal(titleBudget(0.12), 12);
  assert.equal(titleBudget(1.6), 60);
  assert.ok(titleBudget(0.9) > titleBudget(0.5));
  assert.equal(
    clipTitle("Find a smaller route through the unknown", 16),
    "Find a smaller…",
  );
});
test("blocker ordering, missing blockers and duplicate identities cannot reshuffle a map", () => {
  const tickets = [
    { number: 1, blockers: [] },
    { number: 2, blockers: [] },
    { number: 3, blockers: [1, 2] },
  ];
  assert.deepEqual(
    layout(tickets),
    layout([
      tickets[0],
      tickets[0],
      tickets[1],
      { number: 3, blockers: [2, 999, 1] },
    ]),
  );
});
