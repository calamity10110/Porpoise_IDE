// Porpoise Bridge — Popup UI
(function() {
  const $ = (s) => document.querySelector(s);
  const statusDot = $("#status-dot");
  const statusText = $("#status-text");
  const pairSection = $("#pair-section");
  const connectedSection = $("#connected-section");
  const btnConnect = $("#btn-connect");
  const btnSend = $("#btn-send-page");
  const btnInspect = $("#btn-inspect");
  const linkOptions = $("#link-options");
  const linkReconnect = $("#link-reconnect");
  const agentSelect = $("#agent-select");
  const recentEvents = $("#recent-events");
  const pageUrl = $("#page-url");
  const pageTitle = $("#page-title");
  const pairHost = $("#pair-host");
  const pairPort = $("#pair-port");
  const pairToken = $("#pair-token");

  async function sendMessage(msg) {
    return chrome.runtime.sendMessage(msg);
  }

  async function refreshStatus() {
    const resp = await sendMessage({ type: "status" });
    if (!resp) return;
    const connected = resp.connected;
    statusDot.className = "dot " + (connected ? "connected" : "disconnected");
    statusText.textContent = connected ? `Connected to ${resp.config.host}:${resp.config.port}` : "Disconnected";
    pairSection.classList.toggle("hidden", connected);
    connectedSection.classList.toggle("hidden", !connected);

    if (connected) {
      loadPageInfo();
      loadAgentList();
    }
  }

  async function loadPageInfo() {
    try {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      if (tab) {
        pageUrl.textContent = tab.url || "";
        pageTitle.textContent = tab.title || "";
      }
    } catch (e) {
      pageUrl.textContent = "No active tab";
      pageTitle.textContent = "";
    }
  }

  async function loadAgentList() {
    try {
      const resp = await sendMessage({ type: "call", method: "agent/list", params: {} });
      if (resp && resp.result && Array.isArray(resp.result)) {
        agentSelect.innerHTML = '<option value="">Select agent...</option>';
        resp.result.forEach((agent) => {
          const opt = document.createElement("option");
          opt.value = agent.id || agent.name || agent;
          opt.textContent = agent.name || agent.id || agent;
          agentSelect.appendChild(opt);
        });
      }
    } catch (e) {
      console.warn("Porpoise: agent list failed", e);
    }
  }

  // Subscribe to events
  chrome.runtime.onMessage.addListener((msg) => {
    if (msg.type === "event" && msg.data && msg.data.event) {
      const div = document.createElement("div");
      div.className = "recent-event";
      div.textContent = msg.data.event + ": " + JSON.stringify(msg.data.data).slice(0, 60);
      recentEvents.prepend(div);
      while (recentEvents.children.length > 5) recentEvents.removeChild(recentEvents.lastChild);
    }
  });

  btnConnect.addEventListener("click", async () => {
    const host = pairHost.value.trim() || "localhost";
    const port = pairPort.value.trim() || "9876";
    const token = pairToken.value.trim();
    await chrome.storage.local.set({ host, port, token });
    const resp = await sendMessage({ type: "reconnect" });
    if (resp && resp.ok) refreshStatus();
  });

  btnSend.addEventListener("click", async () => {
    const agentId = agentSelect.value;
    if (!agentId) { alert("Select an agent first"); return; }
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab || !tab.url) return;

    try {
      await sendMessage({
        type: "call",
        method: "agent/spawn",
        params: {
          agent_id: agentId,
          prompt: `Examine this page and provide analysis:\nURL: ${tab.url}\nTitle: ${tab.title}`
        }
      });
    } catch (e) {
      console.error("Porpoise: send failed", e);
    }
  });

  btnInspect.addEventListener("click", async () => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab || !tab.id) return;
    try {
      await chrome.scripting.executeScript({
        target: { tabId: tab.id },
        files: ["content.js"]
      });
      chrome.tabs.sendMessage(tab.id, { type: "inspect_mode", enable: true }).catch(() => {});
    } catch (e) {
      console.error("Porpoise: inject failed", e);
    }
  });

  linkOptions.addEventListener("click", (e) => {
    e.preventDefault();
    chrome.runtime.openOptionsPage();
  });

  linkReconnect.addEventListener("click", (e) => {
    e.preventDefault();
    sendMessage({ type: "reconnect" });
    refreshStatus();
  });

  // Refresh status periodically
  refreshStatus();
  setInterval(refreshStatus, 2000);
})();
