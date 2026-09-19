import { test } from "node:test";
import assert from "node:assert/strict";
import { layout, palette } from "../starmap.js";

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
