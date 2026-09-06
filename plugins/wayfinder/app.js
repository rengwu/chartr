import { StarMap, decideDock } from "./starmap.js";

const $ = (id) => document.getElementById(id);
const invoke = (action, options) =>
  window.chartr.invoke(`wayfinder.${action}`, options);
const state = {
  data: null,
  slug: "",
  mode: null,
  busy: false,
  refreshing: false,
  preview: null,
  dock: null,
  pane: { right: 0, bottom: 0 },
};
const map = () => state.data?.maps.find((m) => m.slug === state.slug);
const ticket = () =>
  typeof state.mode === "number"
    ? map()?.tickets.find((t) => t.number === state.mode)
    : null;
export const stars = new StarMap($("map"), (n) => select(n));

function notice(message) {
  $("notice").hidden = !message;
  $("notice").querySelector("span").textContent = message || "";
}
function options(select, entries, placeholder, preserveMissing = false) {
  const selected = select.value;
  if (
    preserveMissing &&
    selected &&
    !entries.some(([value]) => value === selected)
  )
    entries.push([selected, `${selected} (unavailable)`]);
  const values = placeholder ? [["", placeholder], ...entries] : entries;
  const signature = JSON.stringify(values);
  if (select.dataset.options === signature) return;
  select.dataset.options = signature;
  select.replaceChildren(
    ...values.map(([value, label]) => new Option(label, value)),
  );
  if (values.some(([value]) => value === selected)) select.value = selected;
}
function selection() {
  return {
    slug: state.slug,
    ticket: ticket()?.number ?? null,
    method: $("methods").value || null,
    note: $("note").value,
  };
}
function select(mode) {
  state.mode = mode;
  state.preview = null;
  $("note").value = "";
  $("reading").scrollTop = 0;
  render();
}
const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
function insets() {
  const panel = $("detail"),
    box = panel.getBoundingClientRect();
  stars.setInsets(
    panel.hidden || state.dock === "bottom" ? 0 : box.width,
    panel.hidden || state.dock === "right" ? 0 : box.height,
  );
}
function paneLimits() {
  return state.dock === "bottom"
    ? {
        min: Math.min(240, Math.max(120, innerHeight - 192)),
        max: Math.max(120, innerHeight - 192),
        fallback: innerHeight * 0.5,
      }
    : {
        min: 260,
        max: Math.max(260, innerWidth - 220),
        fallback: Math.min(400, innerWidth * 0.58),
      };
}
function sizePane() {
  state.dock = decideDock("hybrid", innerWidth, innerHeight, state.dock, true);
  $("app").dataset.dock = state.dock;
  const { min, max, fallback } = paneLimits();
  const size = clamp(state.pane[state.dock] || fallback, min, max);
  $("app").style.setProperty("--pane-size", `${size}px`);
  const seam = $("detail-resize");
  seam.setAttribute(
    "aria-orientation",
    state.dock === "bottom" ? "horizontal" : "vertical",
  );
  seam.setAttribute("aria-valuemin", Math.round(min));
  seam.setAttribute("aria-valuemax", Math.round(max));
  seam.setAttribute("aria-valuenow", Math.round(size));
  insets();
}
new ResizeObserver(insets).observe($("detail"));
window.addEventListener("resize", sizePane);
let resizing = null;
$("detail-resize").addEventListener("pointerdown", (e) => {
  if (e.button !== 0) return;
  e.preventDefault();
  resizing = { id: e.pointerId, dock: state.dock };
  e.currentTarget.setPointerCapture(e.pointerId);
  $("app").classList.add("resizing");
});
$("detail-resize").addEventListener("pointermove", (e) => {
  if (!resizing || resizing.id !== e.pointerId) return;
  const { min, max } = paneLimits();
  state.pane[state.dock] = clamp(
    state.dock === "bottom" ? innerHeight - e.clientY : innerWidth - e.clientX,
    min,
    max,
  );
  sizePane();
});
for (const name of ["pointerup", "pointercancel", "lostpointercapture"]) {
  $("detail-resize").addEventListener(name, () => {
    resizing = null;
    $("app").classList.remove("resizing");
  });
}
$("detail-resize").addEventListener("dblclick", () => {
  state.pane[state.dock] = 0;
  sizePane();
});
$("detail-resize").addEventListener("keydown", (e) => {
  const keys =
    state.dock === "bottom"
      ? ["ArrowUp", "ArrowDown"]
      : ["ArrowLeft", "ArrowRight"];
  if (![...keys, "Home", "End"].includes(e.key)) return;
  e.preventDefault();
  const { min, max, fallback } = paneLimits();
  state.pane[state.dock] =
    e.key === "Home"
      ? min
      : e.key === "End"
        ? max
        : clamp(
            (state.pane[state.dock] || fallback) +
              (e.key === keys[0] ? 24 : -24),
            min,
            max,
          );
  sizePane();
});
function openMap(slug) {
  state.slug = slug;
  select(null);
  stars.focus();
}
function back() {
  state.slug = "";
  select(null);
  $("map-grid").querySelector("button")?.focus();
}
function renderPicker(data) {
  const signature = JSON.stringify(
    data.maps.map((m) => [
      m.slug,
      m.title,
      m.tickets.map((t) => t.state),
      m.destination,
    ]),
  );
  if ($("map-grid").dataset.signature === signature) return;
  $("map-grid").dataset.signature = signature;
  $("map-grid").replaceChildren(
    ...data.maps.map((m) => {
      const card = document.createElement("button");
      card.className = "map-card";
      card.setAttribute("aria-label", `Open ${m.title}`);
      card.onclick = () => openMap(m.slug);
      const title = document.createElement("span");
      title.className = "map-card-title";
      title.textContent = m.title;
      const total = m.tickets.length,
        resolved = m.tickets.filter((t) => t.state === "Resolved").length;
      const progress = document.createElement("progress");
      progress.max = total || 1;
      progress.value = resolved;
      progress.setAttribute("aria-label", "Resolution progress");
      const count = document.createElement("span");
      count.className = "map-card-count";
      count.textContent = `${resolved} / ${total} resolved`;
      card.append(title, progress, count);
      return card;
    }),
  );
}

function ticketLink(t, number) {
  const link = document.createElement(t ? "a" : "span");
  link.className = "ticket-link ticket-reference";
  const num = document.createElement("span");
  num.className = "num";
  num.textContent = String(number).padStart(2, "0");
  const name = document.createElement("span");
  name.className = "name";
  name.textContent = t?.title || "Missing ticket";
  const dot = document.createElement("span");
  dot.className = "dot";
  dot.dataset.state = t?.state || "Blocked";
  link.append(num, name, dot);
  link.title = t ? `Open ticket ${number}: ${t.title} · ${t.state}` : "Missing ticket";
  if (t) {
    link.href = `#ticket-${number}`;
    link.onclick = (event) => {
      event.preventDefault();
      select(number);
      $("close-detail").focus({ preventScroll: true });
    };
  } else link.setAttribute("aria-disabled", "true");
  return link;
}

function render() {
  const data = state.data;
  if (!data) return;
  if (!map()) {
    state.slug = "";
    state.mode = null;
  }
  if (typeof state.mode === "number" && !ticket()) state.mode = null;
  const m = map(),
    t = ticket(),
    open = state.mode !== null;
  renderPicker(data);
  $("picker").hidden = !!m || !data.maps.length;
  $("map").hidden = !m;
  document.querySelector(".toolbar").hidden = !m;
  document.querySelector(".map-footer").hidden = !m;
  $("map-title").textContent = m?.title || "";
  $("overview").setAttribute("aria-pressed", state.mode === "overview");
  $("overview").title = m
    ? `${m.title} — open map material`
    : "Open map material";
  $("empty").hidden = data.maps.length > 0;
  $("empty-copy").textContent = data.folder
    ? "Use the wayfinder skill to chart a map in .plan/maps/ following this plugin's TRACKER-CONVENTION.md."
    : "Open a folder space to discover its maps.";
  $("detail").hidden = !open;
  $("app").classList.toggle("has-detail", open);
  stars.setModel(m);
  stars.setSelection(t?.number ?? null);
  sizePane();
  $("map-count").textContent = m
    ? `${m.tickets.length} tickets · ${m.frontier} ready`
    : "";
  if (!open) return;
  $("detail-title").textContent = t?.title || m?.title || "Wayfinder";
  $("detail-title").title = $("detail-title").textContent;
  $("ticket-number").textContent = t ? String(t.number).padStart(2, "0") : "✧";
  $("ticket-kind").textContent = t?.kind || "Map material";
  $("ticket-state").hidden = !t;
  if (t) {
    $("ticket-state").textContent = t.state;
    $("ticket-state").dataset.state = t.state;
  }
  const html = t?.html || m?.html || "";
  // Only sanitized host Markdown enters this surface. Names and diagnostics use textContent.
  if ($("body").dataset.source !== html) {
    $("body").dataset.source = html;
    $("body").innerHTML = html;
    if ($("body").firstElementChild?.matches("h1"))
      $("body").firstElementChild.remove();
  }
  $("blockers-section").hidden = !t || !t.blockers.length;
  $("blockers").replaceChildren(
    ...(t?.blockers || []).map((n) =>
      ticketLink(
        m.tickets.find((t) => t.number === n),
        n,
      ),
    ),
  );
  $("assets-section").hidden = !t?.assets.length;
  $("assets").replaceChildren(
    ...(t?.assets || []).map((path) => {
      const button = document.createElement("button");
      button.className = "ticket-link";
      button.textContent = `${path} ↗`;
      button.onclick = () => openFile(path);
      return button;
    }),
  );
  $("frontier-section").hidden = !!t || !m?.frontier;
  $("frontier").replaceChildren(
    ...(m?.tickets || [])
      .filter((t) => t.frontier)
      .map((t) => ticketLink(t, t.number)),
  );
  $("warnings").replaceChildren(
    ...[...(m?.warnings || []), ...(t?.warnings || []), ...data.warnings].map(
      (w) => {
        const p = document.createElement("p");
        p.textContent = w;
        return p;
      },
    ),
  );
  $("launcher").hidden = !t?.frontier;
  options(
    $("agents"),
    data.agents.map((name) => [name, name]),
    data.agents.length ? null : "No registered agents",
  );
  options(
    $("methods"),
    data.skills.map((name) => [name, name]),
    "Automatic",
    true,
  );
  $("setup").textContent = [
    data.agent_error ||
      (!data.agents.length ? "Register an agent to launch." : ""),
    data.skill_error ||
      (!data.skills.length
        ? "Add an enabled source with the wayfinder skill."
        : ""),
  ]
    .filter(Boolean)
    .join(" ");
  $("preview").disabled =
    state.busy ||
    !data.folder ||
    !data.agents.length ||
    !data.skills.length ||
    !!data.agent_error ||
    !!data.skill_error;
  $("claim").hidden = !t?.claimed_by;
  $("claim-label").textContent = t?.claimed_by
    ? `Claimed by ${t.claimed_by}`
    : "";
}

async function refresh() {
  if (state.refreshing || state.busy) return;
  state.refreshing = true;
  try {
    const data = await invoke("snapshot");
    const changed = JSON.stringify(data) !== JSON.stringify(state.data);
    state.data = data;
    if (changed) render();
  } catch (error) {
    notice(error.message);
  } finally {
    state.refreshing = false;
  }
}
async function openFile(target = null) {
  try {
    await invoke("open", {
      slug: state.slug,
      ticket: ticket()?.number ?? null,
      target,
    });
  } catch (error) {
    notice(error.message);
  }
}
async function setup(provider) {
  try {
    await invoke("settings", { provider });
  } catch (error) {
    notice(error.message);
  }
}

$("back").onclick = back;
$("overview").onclick = () => select("overview");
$("close-detail").onclick = () => {
  select(null);
  stars.focus();
};
$("fit").onclick = () => stars.fit();
$("notice").querySelector("button").onclick = () => notice(null);
$("open-file").onclick = () => openFile();
$("agent-setup").onclick = () => setup("agent");
$("skill-setup").onclick = () => setup("skills");
$("body").addEventListener("click", (e) => {
  const link = e.target.closest("a");
  if (!link) return;
  e.preventDefault();
  const target = link.getAttribute("href");
  if (target && !target.startsWith("#")) openFile(target);
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && !$("prompt-dialog").open) {
    if (state.mode !== null) {
      select(null);
      stars.focus();
    } else if (map() && !e.target.matches("input,textarea,select")) back();
  }
});
$("launcher").onsubmit = async (e) => {
  e.preventDefault();
  if (state.busy) return;
  state.busy = true;
  render();
  notice(null);
  try {
    const preview = await invoke("preview", selection());
    state.preview = { ...preview, agent: $("agents").value };
    $("prompt-text").textContent = preview.text;
    $("prompt-sources").textContent =
      `${state.preview.agent} · ${preview.sources.join(" + ")}`;
    $("preview-status").textContent = "";
    $("launch").disabled = false;
    $("prompt-dialog").showModal();
  } catch (error) {
    notice(error.message);
  } finally {
    state.busy = false;
    render();
  }
};
function closePreview() {
  if (!state.busy) {
    $("prompt-dialog").close();
    state.preview = null;
  }
}
$("close-preview").onclick = $("cancel-preview").onclick = closePreview;
$("prompt-dialog").addEventListener("cancel", (e) => {
  if (state.busy) e.preventDefault();
  else state.preview = null;
});
$("launch").onclick = async () => {
  if (state.busy || !state.preview) return;
  state.busy = true;
  $("launch").disabled = true;
  $("preview-status").textContent = "Preparing session…";
  try {
    await invoke("launch", {
      preview: state.preview.preview,
      agent: state.preview.agent,
    });
    $("prompt-dialog").close();
    notice("Agent launched. The ticket is claimed by its session.");
  } catch (error) {
    $("preview-status").textContent = error.message;
  } finally {
    state.busy = false;
    state.preview = null;
    await refresh();
    render();
  }
};

window.addEventListener("pagehide", () => stars.destroy(), { once: true });

if (window.chartr) {
  refresh();
  setInterval(() => {
    if (!document.hidden) refresh();
  }, 2000);
} else notice("Open this plugin in Chartr to connect to the current space.");
