(() => {
  "use strict";

  const STORAGE_PATH = "agents.json";
  const state = {
    agents: [],
    selected: "",
    editing: null,
    deleting: null,
    busy: false,
    toastTimer: null,
  };

  const $ = (id) => document.getElementById(id);
  const launcherPage = $("launcher-page");
  const managementPage = $("management-page");
  const emptyHero = $("empty-hero");
  const readyHero = $("ready-hero");
  const promptInput = $("prompt");
  const agentPicker = $("agent-picker");
  const launchButton = $("launch-session");
  const agentsEmpty = $("agents-empty");
  const agentsTableWrap = $("agents-table-wrap");
  const agentsTableBody = $("agents-table-body");
  const agentDialog = $("agent-dialog");
  const deleteDialog = $("delete-dialog");
  const agentForm = $("agent-form");
  const formError = $("agent-form-error");
  const deliveryInput = $("agent-delivery");
  const promptFlagRow = $("prompt-flag-row");
  const paneMenu = $("pane-menu");
  const paneMenuButton = $("pane-menu-button");

  async function invoke(action, options = {}) {
    if (!window.chartr || typeof window.chartr.invoke !== "function") {
      throw new Error("This plugin must be opened inside Chartr.");
    }
    return window.chartr.invoke(action, options);
  }

  async function readAgents() {
    try {
      const encoded = await invoke("data.read", { path: STORAGE_PATH });
      const parsed = JSON.parse(encoded);
      const values = Array.isArray(parsed) ? parsed : parsed.agents;
      state.agents = Array.isArray(values) ? values.filter(validStoredAgent) : [];
    } catch (error) {
      // A missing private data file is the supported first-run state. Corrupt
      // data is surfaced because silently replacing it on the next save would
      // lose registrations the user may be able to repair.
      if (!String(error).toLowerCase().includes("no such file")) {
        showToast(`Could not read registered agents: ${messageOf(error)}`, true);
      }
      state.agents = [];
    }
    render();
  }

  async function writeAgents() {
    const data = JSON.stringify({ version: 1, agents: state.agents }, null, 2) + "\n";
    await invoke("data.write", { path: STORAGE_PATH, data });
  }

  function validStoredAgent(agent) {
    return Boolean(
      agent &&
        typeof agent.name === "string" &&
        typeof agent.adapter === "string" &&
        Array.isArray(agent.args) &&
        Array.isArray(agent.env || []) &&
        typeof (agent.delivery || "default") === "string",
    );
  }

  function render() {
    const hasAgents = state.agents.length > 0;
    emptyHero.hidden = hasAgents;
    readyHero.hidden = !hasAgents;
    promptInput.disabled = !hasAgents || state.busy;
    launchButton.disabled = !hasAgents || state.busy;
    agentPicker.disabled = !hasAgents || state.busy;

    if (!state.agents.some((agent) => agent.name === state.selected)) {
      state.selected = state.agents[0]?.name || "";
    }

    agentPicker.replaceChildren();
    if (!hasAgents) {
      agentPicker.append(new Option("No registered agents", ""));
    } else {
      for (const agent of state.agents) {
        const option = new Option(agent.name, agent.name, false, agent.name === state.selected);
        option.title = agent.adapter;
        agentPicker.append(option);
      }
    }

    agentsEmpty.hidden = hasAgents;
    agentsTableWrap.hidden = !hasAgents;
    agentsTableBody.replaceChildren(...state.agents.map(agentRow));
  }

  function agentRow(agent) {
    const row = document.createElement("tr");
    const name = document.createElement("td");
    name.textContent = agent.name;
    name.title = agent.name;
    const adapter = document.createElement("td");
    adapter.textContent = agent.adapter;
    adapter.title = agent.adapter;
    const actions = document.createElement("td");
    actions.className = "actions-cell";
    const buttons = document.createElement("div");
    buttons.className = "action-buttons";
    buttons.append(
      actionButton("Edit", pencilIcon(), () => openAgentDialog(agent)),
      actionButton("Delete", trashIcon(), () => openDeleteDialog(agent)),
    );
    actions.append(buttons);
    row.append(name, adapter, actions);
    return row;
  }

  function actionButton(label, icon, action) {
    const button = document.createElement("button");
    button.className = "table-action";
    button.type = "button";
    button.title = label;
    button.setAttribute("aria-label", label);
    button.innerHTML = icon;
    button.addEventListener("click", action);
    return button;
  }

  function pencilIcon() {
    return '<svg viewBox="0 0 20 20" aria-hidden="true"><path d="m4 16 3.2-.7 8.1-8.1a1.6 1.6 0 0 0-2.3-2.3L4.9 13Z"/><path d="m11.8 6.1 2.2 2.2"/></svg>';
  }

  function trashIcon() {
    return '<svg viewBox="0 0 20 20" aria-hidden="true"><path d="M4 6h12M8 3h4l1 3H7ZM6 6l.7 11h6.6L14 6M8.5 9v5M11.5 9v5"/></svg>';
  }

  function showManagement(openNew = false) {
    closePaneMenu();
    launcherPage.hidden = true;
    managementPage.hidden = false;
    if (openNew) openAgentDialog();
  }

  function showLauncher() {
    managementPage.hidden = true;
    launcherPage.hidden = false;
    render();
  }

  function togglePaneMenu() {
    const opening = paneMenu.hidden;
    paneMenu.hidden = !opening;
    paneMenuButton.setAttribute("aria-expanded", String(opening));
    if (opening) paneMenu.querySelector("button").focus();
  }

  function closePaneMenu() {
    paneMenu.hidden = true;
    paneMenuButton.setAttribute("aria-expanded", "false");
  }

  function openAgentDialog(agent = null) {
    state.editing = agent?.name || null;
    $("agent-dialog-title").textContent = agent ? "Edit agent" : "Register new agent";
    $("agent-name").value = agent?.name || "";
    $("agent-adapter").value = agent?.adapter || "";
    $("agent-args").value = formatArgs(agent?.args || []);
    $("agent-env").value = formatArgs(agent?.env || []);

    const savedDelivery = agent?.delivery || "default";
    if (savedDelivery.startsWith("-")) {
      deliveryInput.value = "flag";
      $("agent-prompt-flag").value = savedDelivery;
    } else {
      deliveryInput.value = ["default", "argv", "type"].includes(savedDelivery)
        ? savedDelivery
        : "default";
      $("agent-prompt-flag").value = "";
    }
    updatePromptFlag();
    clearFormError();
    agentDialog.hidden = false;
    $("agent-name").focus();
  }

  function closeAgentDialog() {
    if (state.busy) return;
    agentDialog.hidden = true;
    state.editing = null;
    clearFormError();
  }

  function updatePromptFlag() {
    promptFlagRow.hidden = deliveryInput.value !== "flag";
    $("agent-prompt-flag").required = deliveryInput.value === "flag";
  }

  async function saveAgent(event) {
    event.preventDefault();
    if (state.busy) return;
    const name = $("agent-name").value.trim();
    const adapter = $("agent-adapter").value.trim();
    const argsText = $("agent-args").value.trim();
    const envText = $("agent-env").value.trim();
    const delivery =
      deliveryInput.value === "flag"
        ? $("agent-prompt-flag").value.trim()
        : deliveryInput.value;

    if (!/^[A-Za-z0-9_-]{1,64}$/.test(name)) {
      return setFormError("Name may contain only letters, numbers, hyphens, and underscores.");
    }
    if (!adapter) return setFormError("Adapter is required.");
    const args = parseArgs(argsText);
    if (!argsText || args.length === 0) return setFormError("Args is required.");
    const env = parseArgs(envText);
    const invalidEnv = env.find((entry) => !/^[A-Za-z_][A-Za-z0-9_]*=/.test(entry));
    if (invalidEnv) return setFormError(`Environment entry “${invalidEnv}” must be KEY=VALUE.`);
    if (deliveryInput.value === "flag" && !/^-[^\s]+$/.test(delivery)) {
      return setFormError("Prompt flag must start with a hyphen and contain no spaces.");
    }
    if (state.agents.some((agent) => agent.name === name && agent.name !== state.editing)) {
      return setFormError(`An agent named “${name}” is already registered.`);
    }

    const next = { name, adapter, args, env, delivery };
    const before = state.agents.slice();
    const index = state.agents.findIndex((agent) => agent.name === state.editing);
    if (index === -1) state.agents.push(next);
    else state.agents.splice(index, 1, next);
    state.agents.sort((left, right) => left.name.localeCompare(right.name));
    state.selected = name;
    state.busy = true;
    try {
      await writeAgents();
      agentDialog.hidden = true;
      state.editing = null;
      showToast(index === -1 ? "Agent registered." : "Agent updated.");
    } catch (error) {
      state.agents = before;
      setFormError(`Could not save the agent: ${messageOf(error)}`);
    } finally {
      state.busy = false;
      render();
    }
  }

  function openDeleteDialog(agent) {
    state.deleting = agent.name;
    $("delete-message").textContent = `Delete “${agent.name}”? This does not close sessions already running with it.`;
    deleteDialog.hidden = false;
    $("confirm-delete").focus();
  }

  function closeDeleteDialog() {
    if (state.busy) return;
    deleteDialog.hidden = true;
    state.deleting = null;
  }

  async function deleteAgent() {
    if (!state.deleting || state.busy) return;
    const before = state.agents.slice();
    const deleted = state.deleting;
    state.agents = state.agents.filter((agent) => agent.name !== deleted);
    state.busy = true;
    try {
      await writeAgents();
      deleteDialog.hidden = true;
      state.deleting = null;
      showToast("Agent deleted.");
    } catch (error) {
      state.agents = before;
      showToast(`Could not delete the agent: ${messageOf(error)}`, true);
    } finally {
      state.busy = false;
      render();
    }
  }

  async function launchSession(event) {
    event.preventDefault();
    if (state.busy) return;
    const agent = state.agents.find((candidate) => candidate.name === state.selected);
    if (!agent) return;
    state.busy = true;
    launchButton.textContent = "Launching…";
    render();
    try {
      await invoke("terminal.launch", {
        command: agent.adapter,
        args: agent.args,
        env: agent.env,
        prompt: promptInput.value,
        delivery: agent.delivery,
      });
      promptInput.value = "";
    } catch (error) {
      showToast(`Could not launch the session: ${messageOf(error)}`, true);
    } finally {
      state.busy = false;
      launchButton.textContent = "Launch session";
      render();
    }
  }

  async function loadMetadata() {
    try {
      const metadata = await invoke("space.metadata");
      $("space-name").textContent = metadata.name || "Current space";
      $("git-branch").textContent = metadata.branch || "No Git branch";
    } catch (error) {
      $("space-name").textContent = "Current space";
      $("git-branch").textContent = "Git unavailable";
    }
  }

  // Shell-like field editing without shell interpretation: whitespace splits,
  // quotes group, and only quote/backslash escapes inside double quotes.
  function parseArgs(text) {
    const output = [];
    let current = "";
    let started = false;
    let quote = null;
    for (let index = 0; index < text.length; index += 1) {
      const character = text[index];
      if (quote) {
        if (
          quote === '"' &&
          character === "\\" &&
          (text[index + 1] === '"' || text[index + 1] === "\\")
        ) {
          current += text[(index += 1)];
        } else if (character === quote) {
          quote = null;
        } else {
          current += character;
        }
      } else if (character === '"' || character === "'") {
        quote = character;
        started = true;
      } else if (/\s/.test(character)) {
        if (started) output.push(current);
        current = "";
        started = false;
      } else {
        current += character;
        started = true;
      }
    }
    if (started) output.push(current);
    return output;
  }

  function formatArgs(args) {
    return args.map((argument) => {
      if (argument !== "" && !/[\s"'\\]/.test(argument)) return argument;
      return `"${argument.replace(/([\\"])/g, "\\$1")}"`;
    }).join(" ");
  }

  function setFormError(message) {
    formError.textContent = message;
    formError.hidden = false;
  }

  function clearFormError() {
    formError.textContent = "";
    formError.hidden = true;
  }

  function messageOf(error) {
    return error instanceof Error ? error.message : String(error);
  }

  function showToast(message, error = false) {
    const toast = $("toast");
    window.clearTimeout(state.toastTimer);
    toast.textContent = message;
    toast.classList.toggle("error", error);
    toast.hidden = false;
    state.toastTimer = window.setTimeout(() => {
      toast.hidden = true;
    }, 4200);
  }

  paneMenuButton.addEventListener("click", togglePaneMenu);
  $("manage-agents").addEventListener("click", () => showManagement(false));
  $("register-first-agent").addEventListener("click", () => showManagement(true));
  $("back-to-launcher").addEventListener("click", showLauncher);
  $("new-agent").addEventListener("click", () => openAgentDialog());
  $("close-agent-dialog").addEventListener("click", closeAgentDialog);
  $("cancel-agent-dialog").addEventListener("click", closeAgentDialog);
  agentDialog.querySelector(".dialog-scrim").addEventListener("click", closeAgentDialog);
  deliveryInput.addEventListener("change", updatePromptFlag);
  agentForm.addEventListener("submit", saveAgent);
  $("close-delete-dialog").addEventListener("click", closeDeleteDialog);
  $("cancel-delete").addEventListener("click", closeDeleteDialog);
  deleteDialog.querySelector(".dialog-scrim").addEventListener("click", closeDeleteDialog);
  $("confirm-delete").addEventListener("click", deleteAgent);
  $("launcher-form").addEventListener("submit", launchSession);
  agentPicker.addEventListener("change", () => {
    state.selected = agentPicker.value;
  });
  document.addEventListener("pointerdown", (event) => {
    if (!event.target.closest(".pane-menu")) closePaneMenu();
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    if (!deleteDialog.hidden) closeDeleteDialog();
    else if (!agentDialog.hidden) closeAgentDialog();
    else closePaneMenu();
  });

  render();
  loadMetadata();
  readAgents();
})();
