// Dev-only smoke test. Uses an installed Playwright (or PLAYWRIGHT_MODULE).
// No application, source registry, paid agent, or user's tracker is modified.
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
const { chromium, webkit } = await import(
  process.env.PLAYWRIGHT_MODULE || "playwright"
);
const root = new URL("../", import.meta.url);
const allowed = new Set([
  "index.html",
  "styles.css",
  "app.js",
  "starmap.js",
  "icons/ui.svg",
]);
const server = createServer(async (req, res) => {
  const name = req.url === "/" ? "index.html" : req.url.slice(1);
  if (!allowed.has(name)) {
    res.writeHead(404);
    res.end();
    return;
  }
  res.setHeader(
    "Content-Type",
    name.endsWith(".js")
      ? "text/javascript"
      : name.endsWith(".css")
        ? "text/css"
        : name.endsWith(".svg")
          ? "image/svg+xml"
          : "text/html",
  );
  res.setHeader(
    "Content-Security-Policy",
    "default-src 'self' data: blob:; connect-src 'none'; frame-src 'none'; object-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'",
  );
  res.end(await readFile(new URL(name, root)));
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const browser = await (
  process.env.BROWSER === "webkit" ? webkit : chromium
).launch({ headless: true });
try {
  const page = await browser.newPage({
    viewport: { width: 1200, height: 800 },
    deviceScaleFactor: 1,
  });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.addInitScript(() => {
    // Observe painted fog, so a screen-fixed fallback cannot pass a world-layout test.
    window.fogScreen = {};
    const fillText = CanvasRenderingContext2D.prototype.fillText;
    CanvasRenderingContext2D.prototype.fillText = function (
      text,
      x,
      y,
      ...rest
    ) {
      if (
        [
          "Uncharted territory",
          "Missing anchor",
          "How does a journey end?",
        ].includes(text)
      )
        window.fogScreen[text] = { x, y };
      return fillText.call(this, text, x, y, ...rest);
    };
    const tickets = [
      {
        number: 1,
        title: "Who is this for?",
        kind: "grilling",
        state: "Resolved",
        blockers: [],
      },
      {
        number: 2,
        title: "Find the smallest useful map",
        kind: "prototype",
        state: "Resolved",
        blockers: [1],
      },
      {
        number: 3,
        title: "How should the launcher behave?",
        kind: "research",
        state: "Ready",
        blockers: [1],
      },
      {
        number: 4,
        title: "Make the frontier feel obvious",
        kind: "prototype",
        state: "Claimed",
        blockers: [2],
      },
      {
        number: 5,
        title: "Connect the agent and its methods",
        kind: "task",
        state: "Blocked",
        blockers: [3],
      },
      {
        number: 6,
        title: "Finish the first journey",
        kind: "task",
        state: "Blocked",
        blockers: [4, 5],
      },
      {
        number: 7,
        title: "Automatic background scheduling",
        kind: "research",
        state: "Out of scope",
        blockers: [],
      },
    ].map((t) => ({
      ...t,
      frontier: t.state === "Ready",
      html: `<h1>${t.title}</h1><h2>Question</h2><p>What is the smallest interaction that makes the next step clear?</p><h2>Done when</h2><ul><li>One deliberate action starts the right agent.</li><li>The operator can inspect the exact prompt.</li><li>The map remains useful without a configured provider.</li></ul>`,
      answer_html: "<p>Keep the next step visible.</p>",
      claimed_by: t.state === "Claimed" ? "test-session" : "",
      assets: [],
      warnings: [],
    }));
    window.fixture = {
      space: "Observatory",
      folder: "/fixture",
      maps: [
        {
          slug: "observatory",
          title: "A quieter way to find the next step",
          html: "<h1>A quieter way to find the next step</h1><h2>Destination</h2><p>A small, legible map from uncertainty to useful work.</p><h2>Notes</h2><p>Keep the implementation lean. Let the constellation carry the structure.</p>",
          destination: "A small, legible map from uncertainty to useful work.",
          finished: false,
          frontier: 1,
          warnings: [],
          fog: [
            { title: "How does a journey end?", clears_with: 6 },
            { title: "Uncharted territory", clears_with: null },
            { title: "Missing anchor", clears_with: 99 },
          ],
          tickets,
        },
      ],
      agents: ["Claude", "Codex"],
      skills: ["chartr-skills/wayfinder", "chartr-skills/research"],
      agent_error: null,
      skill_error: null,
      warnings: [],
    };
    window.fixture.maps.push({
      ...structuredClone(window.fixture.maps[0]),
      slug: "second-route",
      title: "A second route through the unknown",
    });
    window.calls = [];
    window.previewId = 0;
    window.chartr = {
      invoke: async (action, options = {}) => {
        window.calls.push({ action, options });
        if (action === "wayfinder.snapshot")
          return structuredClone(window.fixture);
        if (action === "wayfinder.preview")
          return {
            preview: ++window.previewId,
            text:
              "# Work one Wayfinder ticket\n\n## Method: chartr-skills/research\n\nInvestigate primary sources.\n\n" +
              options.note,
            sources: ["chartr-skills/wayfinder", "chartr-skills/research"],
          };
        if (action === "wayfinder.launch") {
          const t = window.fixture.maps[0].tickets[2];
          t.state = "Claimed";
          t.claimed_by = "new-session";
          t.frontier = false;
          return { session: "new-session" };
        }
        return true;
      },
    };
  });
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  await page.waitForSelector(".map-card");
  assert.equal(await page.locator("#detail").isVisible(), false);
  assert.equal(await page.locator("#tickets, #maps").count(), 0);
  if (process.env.SCREENSHOT_DIR)
    await page.screenshot({
      path: `${process.env.SCREENSHOT_DIR}/wayfinder-picker.png`,
    });
  await page.click(".map-card:first-child");
  await page.waitForSelector("#map canvas");
  await page.evaluate(async () => {
    window.island = (await import("/app.js")).stars.renderer;
  });
  const camera = () => page.evaluate(() => window.island.camera());
  const settled = () =>
    page.waitForFunction(() => {
      const a = window.island.liveCamera(),
        b = window.island.camera();
      return (
        Math.abs(a.x - b.x) < 0.05 &&
        Math.abs(a.y - b.y) < 0.05 &&
        Math.abs(a.s - b.s) < 0.001
      );
    });
  const clickStar = async (number) => {
    await settled();
    const point = await page.evaluate((n) => window.island.screenOf(n), number);
    await page.mouse.click(point.x, point.y);
  };
  assert.equal(
    await page.locator("#map-title").textContent(),
    "A quieter way to find the next step",
  );
  await settled();
  if (process.env.SCREENSHOT_DIR)
    await page.screenshot({
      path: `${process.env.SCREENSHOT_DIR}/wayfinder-map.png`,
    });
  // Every fog anchor is in world coordinates and follows the same camera.
  const fog = await page.evaluate(() => window.island.fogPositions());
  assert.equal(fog.length, 3);
  const fogBefore = await page.evaluate(() =>
    structuredClone(window.fogScreen),
  );
  const before = await camera();
  await page.mouse.move(150, 600);
  await page.mouse.down();
  await page.mouse.move(230, 640, { steps: 8 });
  await page.mouse.up();
  const dragged = await camera();
  assert.ok(Math.abs(dragged.x - before.x - 80) < 0.1);
  assert.ok(Math.abs(dragged.y - before.y - 40) < 0.1);
  assert.deepEqual(
    await page.evaluate(() => window.island.fogPositions()),
    fog,
  );
  const during = await page.evaluate(() => window.island.liveCamera());
  assert.ok(
    Math.abs(during.x - dragged.x) > 0.1,
    "Drag eases toward its target",
  );
  await settled();
  const fogAfter = await page.evaluate(() => window.fogScreen);
  for (const name of Object.keys(fogBefore)) {
    assert.ok(
      Math.abs(fogAfter[name].x - fogBefore[name].x - 80) < 0.1,
      `${name} pans horizontally`,
    );
    assert.ok(
      Math.abs(fogAfter[name].y - fogBefore[name].y - 40) < 0.1,
      `${name} pans vertically`,
    );
  }
  const anchor = { x: 320, y: 250 };
  const zoomBefore = await camera();
  await page.locator("canvas").dispatchEvent("wheel", {
    clientX: anchor.x,
    clientY: anchor.y,
    deltaY: -25,
    ctrlKey: true,
  });
  const zoomed = await camera();
  assert.ok(zoomed.s > zoomBefore.s);
  assert.ok(
    Math.abs(
      (anchor.x - zoomBefore.x) / zoomBefore.s -
        (anchor.x - zoomed.x) / zoomed.s,
    ) < 0.001,
  );
  await settled();
  // WKWebView cumulative pinch and duplicate-wheel suppression.
  await page.evaluate(() => {
    for (const [type, scale] of [
      ["gesturestart", 1],
      ["gesturechange", 1.2],
    ]) {
      const event = new Event(type, { cancelable: true });
      Object.assign(event, { scale, clientX: 320, clientY: 250 });
      document.querySelector("canvas").dispatchEvent(event);
    }
  });
  const gesture = await camera();
  assert.ok(Math.abs(gesture.s / zoomed.s - 1.2) < 0.001);
  await page.locator("canvas").dispatchEvent("wheel", {
    clientX: 320,
    clientY: 250,
    deltaY: -25,
    ctrlKey: true,
  });
  assert.deepEqual(await camera(), gesture);
  await page.locator("canvas").dispatchEvent("gestureend");
  if (process.env.BROWSER !== "webkit") {
    const client = await page.context().newCDPSession(page);
    const touchBefore = await camera();
    await client.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [
        { x: 200, y: 300, id: 1 },
        { x: 300, y: 300, id: 2 },
      ],
    });
    await client.send("Input.dispatchTouchEvent", {
      type: "touchMove",
      touchPoints: [
        { x: 180, y: 320, id: 1 },
        { x: 320, y: 320, id: 2 },
      ],
    });
    await client.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    assert.ok(
      (await camera()).s > touchBefore.s * 1.3,
      "Two touch points pinch the map",
    );
    assert.equal(
      await page.locator("#detail").isVisible(),
      false,
      "Pinch does not select a ticket",
    );
    await client.detach();
  }
  await page.click("#fit");
  await settled();
  await clickStar(3);
  await settled();
  const seated = await page.evaluate(() => ({
    star: window.island.screenOf(3),
    panel: document.querySelector("#detail").getBoundingClientRect().width,
  }));
  assert.ok(Math.abs(seated.star.x - (1200 - seated.panel) / 2) < 1);
  assert.ok(Math.abs(seated.star.y - 402) < 1);
  // Resize the shared seam; selected star follows its new free rectangle.
  const seam = await page.locator("#detail-resize").boundingBox();
  await page.mouse.move(seam.x + 4, seam.y + 100);
  await page.mouse.down();
  await page.mouse.move(seam.x - 80, seam.y + 100, { steps: 8 });
  await page.mouse.up();
  await settled();
  assert.ok(
    (await page.locator("#detail").boundingBox()).width > seated.panel + 70,
  );
  assert.equal(
    await page.locator("#detail-title").textContent(),
    "How should the launcher behave?",
  );
  assert.equal(await page.locator("#launcher").isVisible(), true);
  await page.fill("#note", "Keep it small.");
  if (process.env.SCREENSHOT_DIR)
    await page.screenshot({
      path: `${process.env.SCREENSHOT_DIR}/wayfinder-detail.png`,
    });
  await page.click("#preview");
  assert.ok(
    (await page.locator("#prompt-text").textContent()).includes(
      "Keep it small.",
    ),
  );
  await page.click("#launch");
  await page.waitForFunction(
    () => !document.querySelector("#prompt-dialog").open,
  );
  assert.equal(await page.locator("#claim").isVisible(), true);
  assert.equal(await page.locator("#launcher").isVisible(), false);
  assert.equal(await page.locator("#release, #focus-session").count(), 0);
  // A subsequent host snapshot can update a claim without detail-pane actions.
  await page.evaluate(() => {
    const t = window.fixture.maps[0].tickets[2];
    t.state = "Ready";
    t.claimed_by = "";
    t.frontier = true;
  });
  await page.waitForFunction(() => document.querySelector("#claim").hidden);
  await page.click("#agent-setup");
  assert.equal(
    await page.evaluate(() => window.calls.at(-1).options.provider),
    "agent",
  );
  // Camera/layout preserve star locations and operator input on status-only refreshes.
  const stable = await page.evaluate(() => ({
    points: window.island.positions(),
    camera: window.island.camera(),
    note: document.querySelector("#note").value,
  }));
  await page.evaluate(
    () => (window.fixture.maps[0].tickets[0].state = "Ready"),
  );
  await page.waitForTimeout(2100);
  assert.deepEqual(
    await page.evaluate(() => ({
      points: window.island.positions(),
      camera: window.island.camera(),
      note: document.querySelector("#note").value,
    })),
    stable,
  );
  for (const [width, height] of [
    [560, 680],
    [600, 680],
    [390, 700],
  ]) {
    await page.setViewportSize({ width, height });
    await page.waitForFunction(
      (w) =>
        document.querySelector("#app").dataset.dock === "bottom" &&
        document.querySelector("#detail").getBoundingClientRect().width === w,
      width,
    );
    await settled();
    const bounds = await page.evaluate(() => {
      const r = (id) => {
        const b = document.getElementById(id).getBoundingClientRect();
        return {
          x: b.x,
          y: b.y,
          right: b.right,
          bottom: b.bottom,
          width: b.width,
          height: b.height,
        };
      };
      return {
        panel: r("detail"),
        launch: r("preview"),
        body: document.body.scrollWidth,
        map: r("map"),
      };
    });
    assert.equal(bounds.panel.width, width);
    assert.ok(bounds.panel.y > 140);
    assert.ok(
      bounds.launch.bottom <= height && bounds.launch.right <= width,
      JSON.stringify(bounds),
    );
    assert.ok(bounds.body <= width, "No horizontal overflow");
    if (process.env.SCREENSHOT_DIR)
      await page.screenshot({
        path: `${process.env.SCREENSHOT_DIR}/wayfinder-${width}.png`,
      });
  }
  // The bottom seam supports both dragging and keyboard resizing.
  await page.click("#notice button");
  const bottomSize = (await page.locator("#detail").boundingBox()).height;
  const bottomSeam = await page.locator("#detail-resize").boundingBox();
  await page.mouse.move(180, bottomSeam.y + 4);
  await page.mouse.down();
  await page.mouse.move(180, bottomSeam.y - 40, { steps: 5 });
  await page.mouse.up();
  assert.ok(
    (await page.locator("#detail").boundingBox()).height > bottomSize + 35,
  );
  await page.locator("#detail-resize").focus();
  await page.keyboard.press("ArrowDown");
  assert.equal(
    await page.locator("#detail-resize").getAttribute("aria-orientation"),
    "horizontal",
  );
  await settled();
  const resizedCenter = await page.evaluate(() => ({
    point: window.island.screenOf(3),
    top: document.querySelector("#detail").getBoundingClientRect().top,
  }));
  assert.ok(Math.abs(resizedCenter.point.y - (resizedCenter.top + 4) / 2) < 1);
  await page.evaluate(() => {
    window.fixture.agents = [];
    window.fixture.agent_error = "Enable Agent and register an agent.";
  });
  await page.waitForFunction(() =>
    document.querySelector("#setup").textContent.includes("Enable Agent"),
  );
  assert.equal(await page.locator("#preview").isDisabled(), true);
  assert.equal(await page.locator("#map").isVisible(), true);
  // Closing preserves the operator's camera; Back returns to the picker.
  const closing = await camera();
  await page.click("#close-detail");
  assert.deepEqual(await camera(), closing);
  await page.click("#back");
  assert.equal(await page.locator("#picker").isVisible(), true);
  await page.locator(".map-card").nth(1).click();
  assert.equal(
    await page.locator("#map-title").textContent(),
    "A second route through the unknown",
  );
  await page.click("#back");
  await page.click(".map-card:first-child");
  assert.deepEqual(await camera(), closing);
  // Reduced motion renders on demand and still responds to camera input.
  await page.emulateMedia({ reducedMotion: "reduce" });
  await settled();
  const stillA = await page.locator("canvas").screenshot();
  await page.waitForTimeout(150);
  assert.deepEqual(await page.locator("canvas").screenshot(), stillA);
  const reducedBefore = await camera();
  await page.locator("canvas").dispatchEvent("wheel", {
    clientX: 160,
    clientY: 180,
    deltaY: 30,
    deltaMode: 1,
  });
  await settled();
  assert.ok((await camera()).s < reducedBefore.s);
  await page.click("#back");
  await page.evaluate(() => {
    window.fixture.maps = [];
  });
  await page.waitForFunction(() => !document.querySelector("#empty").hidden);
  assert.equal(await page.locator("#empty button").count(), 0);
  assert.equal(await page.locator("#detail").isVisible(), false);
  await page.evaluate(() => {
    window.fixture.folder = null;
  });
  await page.waitForFunction(() =>
    document
      .querySelector("#empty-copy")
      .textContent.includes("Open a folder space"),
  );
  assert.deepEqual(errors, []);
  console.log(
    "Web smoke passed: picker/back, camera easing and memory, anchored pinch (wheel/WebKit), world fog, selection centering, both resize seams, responsive layouts, reduced motion, preview/launch, claim updates, setup, empty space.",
  );
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
