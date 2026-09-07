(() => {
  let next = 1;
  const document = Array.from(crypto.getRandomValues(new Uint32Array(4)), n => n.toString(16)).join("-");
  const pending = new Map();
  window.__chartrReply = response => {
    if (response.document !== document) return;
    const pair = pending.get(response.id);
    if (!pair) return;
    pending.delete(response.id);
    clearTimeout(pair[2]);
    response.ok ? pair[0](response.value) : pair[1](new Error(response.error));
  };
  const invoke = (action, options = {}) => new Promise((resolve, reject) => {
    if (typeof action !== "string") { reject(new Error("host action must be a string")); return; }
    const id = next++;
    const encoded = JSON.stringify({ ...options, id, document, action });
    if (new TextEncoder().encode(encoded).length > 1024 * 1024) {
      reject(new Error("host request exceeds 1 MiB"));
      return;
    }
    // Includes time waiting behind the bounded queue's one-shot operations.
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error("host request timed out"));
    }, 10 * 60 * 1000);
    pending.set(id, [resolve, reject, timer]);
    try { window.ipc.postMessage(encoded); }
    catch (error) { pending.delete(id); clearTimeout(timer); reject(error); }
  });
  window.addEventListener("pagehide", () => {
    for (const [, reject, timer] of pending.values()) {
      clearTimeout(timer);
      reject(new Error("plugin document closed"));
    }
    pending.clear();
  });
  window.addEventListener("pointerdown", () => {
    window.ipc.postMessage(JSON.stringify({ id: 0, action: "chartr.focus" }));
  }, true);
  Object.defineProperty(window, "chartr", { value: Object.freeze({ invoke }) });
})();
