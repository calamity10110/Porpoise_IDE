// Porpoise Bridge — Options Page
(function() {
  const host = document.getElementById("opt-host");
  const port = document.getElementById("opt-port");
  const token = document.getElementById("opt-token");
  const tls = document.getElementById("opt-tls");
  const btnSave = document.getElementById("btn-save");
  const toast = document.getElementById("toast");

  async function loadSettings() {
    const stored = await chrome.storage.local.get(["host", "port", "token", "useTls"]);
    host.value = stored.host || "localhost";
    port.value = stored.port || "9876";
    token.value = stored.token || "";
    tls.checked = stored.useTls !== false;
  }

  function showToast(msg, type) {
    toast.textContent = msg;
    toast.className = "toast " + type;
    setTimeout(() => { toast.className = "toast"; }, 3000);
  }

  btnSave.addEventListener("click", async () => {
    const h = host.value.trim() || "localhost";
    const p = port.value.trim() || "9876";
    const t = token.value.trim();
    const useTls = tls.checked;

    await chrome.storage.local.set({ host: h, port: p, token: t, useTls });

    try {
      const resp = await chrome.runtime.sendMessage({ type: "reconnect" });
      if (resp && resp.ok) {
        showToast("Connected!", "success");
      } else {
        showToast("Settings saved, but connection failed: " + (resp && resp.error), "error");
      }
    } catch (e) {
      showToast("Settings saved. Reconnect triggered.", "success");
    }
  });

  loadSettings();
})();
