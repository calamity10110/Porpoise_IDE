// Porpoise Bridge — Background Service Worker
// Manages WSS connection to daemon, routes messages between popup/content/daemon.

const CONN_STATES = { DISCONNECTED: 0, CONNECTING: 1, AUTHENTICATING: 2, CONNECTED: 3 };
let state = CONN_STATES.DISCONNECTED;
let ws = null;
let reconnectTimer = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_DELAY = 30;
const pending = new Map();
const eventListeners = new Set();
let config = { host: "localhost", port: "9876", token: "", useTls: true };

async function loadConfig() {
  const stored = await chrome.storage.local.get(["host", "port", "token", "useTls"]);
  if (stored.host !== undefined) config.host = stored.host;
  if (stored.port !== undefined) config.port = stored.port;
  if (stored.token !== undefined) config.token = stored.token;
  if (stored.useTls !== undefined) config.useTls = stored.useTls;
}

function saveConfig() {
  return chrome.storage.local.set(config);
}

function getWsUrl() {
  const scheme = config.useTls ? "wss" : "ws";
  return `${scheme}://${config.host}:${config.port}`;
}

function updateBadge() {
  const text = state === CONN_STATES.CONNECTED ? "●" : state === CONN_STATES.CONNECTING ? "○" : "!";
  const color = state === CONN_STATES.CONNECTED ? "#2ecc71" : state === CONN_STATES.CONNECTING ? "#f39c12" : "#e74c3c";
  chrome.action.setBadgeText({ text });
  chrome.action.setBadgeBackgroundColor({ color });
}

async function connect() {
  if (state === CONN_STATES.CONNECTING || state === CONN_STATES.AUTHENTICATING) return;
  if (ws) { ws.close(); ws = null; }

  state = CONN_STATES.CONNECTING;
  updateBadge();

  try {
    await loadConfig();
    const url = getWsUrl();
    ws = new WebSocket(url);

    ws.onopen = () => {
      state = CONN_STATES.AUTHENTICATING;
      updateBadge();
      if (config.token) {
        ws.send(JSON.stringify({ type: "auth", token: config.token }));
      } else {
        state = CONN_STATES.CONNECTED;
        reconnectAttempts = 0;
        updateBadge();
        notifyListeners({ type: "status", connected: true });
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.id && pending.has(msg.id)) {
          const { resolve, reject } = pending.get(msg.id);
          pending.delete(msg.id);
          if (msg.error) reject(new Error(msg.error));
          else resolve(msg);
        } else if (msg.event || (!msg.id && !msg.method)) {
          notifyListeners({ type: "event", data: msg });
        } else if (msg.type === "handshake" && state === CONN_STATES.AUTHENTICATING) {
          if (config.token) {
            ws.send(JSON.stringify({ type: "auth", token: config.token }));
          }
        } else if (msg.type === "auth_ok" || (msg.result && state === CONN_STATES.AUTHENTICATING)) {
          state = CONN_STATES.CONNECTED;
          reconnectAttempts = 0;
          updateBadge();
          notifyListeners({ type: "status", connected: true });
        }
      } catch (e) {
        console.warn("Porpoise: malformed message", e);
      }
    };

    ws.onclose = () => {
      ws = null;
      state = CONN_STATES.DISCONNECTED;
      updateBadge();
      notifyListeners({ type: "status", connected: false });
      for (const [id, { reject }] of pending) {
        reject(new Error("Connection lost"));
        pending.delete(id);
      }
      scheduleReconnect();
    };

    ws.onerror = () => {
      if (ws) ws.close();
    };
  } catch (e) {
    state = CONN_STATES.DISCONNECTED;
    updateBadge();
    scheduleReconnect();
  }
}

function scheduleReconnect() {
  if (reconnectTimer) return;
  const delay = Math.min(MAX_RECONNECT_DELAY, Math.pow(2, reconnectAttempts));
  reconnectAttempts++;
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connect();
  }, delay * 1000);
}

function disconnect() {
  if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
  reconnectAttempts = 0;
  if (ws) { ws.close(); ws = null; }
  state = CONN_STATES.DISCONNECTED;
  updateBadge();
  for (const [id, { reject }] of pending) {
    reject(new Error("Disconnected"));
    pending.delete(id);
  }
}

async function call(method, params = {}) {
  if (state !== CONN_STATES.CONNECTED || !ws) throw new Error("Not connected");
  const id = crypto.randomUUID();
  const request = { method, params, id };
  if (config.token) request.token = config.token;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify(request));
    setTimeout(() => {
      if (pending.has(id)) {
        pending.delete(id);
        reject(new Error("RPC timeout"));
      }
    }, 30000);
  });
}

function notifyListeners(msg) {
  for (const listener of eventListeners) {
    try { listener(msg); } catch (e) { console.warn("Porpoise: listener error", e); }
  }
}

// Message router
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  switch (msg.type) {
    case "connect":
      connect().then(() => sendResponse({ ok: true })).catch(e => sendResponse({ ok: false, error: e.message }));
      return true;
    case "disconnect":
      disconnect();
      sendResponse({ ok: true });
      break;
    case "status":
      sendResponse({ state, connected: state === CONN_STATES.CONNECTED, config });
      break;
    case "call":
      call(msg.method, msg.params || {}).then(r => sendResponse(r)).catch(e => sendResponse({ error: e.message }));
      return true;
    case "subscribe":
      eventListeners.add(sendResponse);
      sendResponse({ ok: true });
      return true;
    case "unsubscribe":
      eventListeners.delete(sendResponse);
      break;
    case "inject_tab":
      injectContentScript(msg.tabId, msg.mode).then(() => sendResponse({ ok: true })).catch(e => sendResponse({ error: e.message }));
      return true;
    case "reconnect":
      disconnect();
      connect().then(() => sendResponse({ ok: true })).catch(e => sendResponse({ error: e.message }));
      return true;
  }
});

// Context menu: send page to agent
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "porpoise-send-page",
    title: "Send page to Porpoise agent",
    contexts: ["page"]
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "porpoise-send-page" && tab) {
    chrome.tabs.sendMessage(tab.id, { type: "inspect_mode", enable: true }).catch(() => {
      injectContentScript(tab.id, "capture").then(() => {
        chrome.tabs.sendMessage(tab.id, { type: "capture_page" }).catch(() => {});
      });
    });
  }
});

// Inject content script helper
async function injectContentScript(tabId, mode) {
  try {
    await chrome.scripting.executeScript({
      target: { tabId },
      files: ["content.js"]
    });
  } catch (e) {
    if (!e.message.includes("already injected")) throw e;
  }
  if (mode === "inspect") {
    chrome.tabs.sendMessage(tabId, { type: "inspect_mode", enable: true }).catch(() => {});
  }
}

// Auto-connect on startup
chrome.runtime.onStartup.addListener(() => connect());
loadConfig().then(() => {
  if (config.host && config.port) connect();
});

export { connect, disconnect, call, loadConfig, saveConfig };
