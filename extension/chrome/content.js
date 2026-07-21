(function() {
  let inspectMode = false;
  let hoveredEl = null;
  let overlay = null;

  function createOverlay() {
    overlay = document.createElement("div");
    overlay.style.cssText = "position:fixed;pointer-events:none;z-index:2147483646;border:2px solid #7aa0f7;background:rgba(122,160,247,0.12);transition:all 0.05s;display:none;";
    document.body.appendChild(overlay);
  }

  function getSelector(el) {
    if (!el || el === document.body) return "body";
    if (el.id) return "#" + el.id;
    let path = [];
    while (el && el !== document.body && el !== document.documentElement) {
      let selector = el.tagName.toLowerCase();
      if (el.id) { path.unshift("#" + el.id); break; }
      if (el.className && typeof el.className === "string") {
        selector += "." + el.className.trim().split(/\s+/).slice(0, 2).join(".");
      }
      let parent = el.parentElement;
      if (parent) {
        let idx = 1;
        for (let child of parent.children) {
          if (child === el) break;
          if (child.tagName === el.tagName) idx++;
        }
        if (idx > 1) selector += ":nth-of-type(" + idx + ")";
      }
      path.unshift(selector);
      el = el.parentElement;
    }
    return path.join(" > ");
  }

  function getComputedSnapshot(el) {
    const styles = window.getComputedStyle(el);
    const important = ["display", "position", "width", "height", "color", "background-color", "font-size", "font-family", "visibility", "opacity", "overflow", "z-index", "cursor"];
    const snapshot = {};
    for (const key of important) snapshot[key] = styles.getPropertyValue(key);
    return snapshot;
  }

  function onMouseMove(e) {
    if (!inspectMode || !overlay) return;
    const el = document.elementFromPoint(e.clientX, e.clientY);
    if (!el || el === hoveredEl || el === overlay || el === document.body) return;
    hoveredEl = el;
    const rect = el.getBoundingClientRect();
    overlay.style.display = "block";
    overlay.style.left = rect.left + "px";
    overlay.style.top = rect.top + "px";
    overlay.style.width = rect.width + "px";
    overlay.style.height = rect.height + "px";
  }

  function onMouseClick(e) {
    if (!inspectMode || !hoveredEl) return;
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();

    const el = hoveredEl;
    const selector = getSelector(el);
    const tagName = el.tagName.toLowerCase();
    const textContent = (el.textContent || "").trim().slice(0, 200);
    const innerHTML = el.outerHTML ? el.outerHTML.slice(0, 5000) : "";
    const computed = getComputedSnapshot(el);

    const result = { selector, tagName, textContent, innerHTML, computed };

    // Send back to background
    chrome.runtime.sendMessage({
      type: "call",
      method: "browser/inspect_element",
      params: result
    }).catch(() => {});

    // Also send to any listeners
    chrome.runtime.sendMessage({ type: "inspect_result", data: result }).catch(() => {});

    setInspectMode(false);
  }

  function onKeyDown(e) {
    if (e.key === "Escape" && inspectMode) {
      setInspectMode(false);
    }
  }

  function setInspectMode(enable) {
    inspectMode = enable;
    if (!overlay) createOverlay();
    overlay.style.display = enable && hoveredEl ? "block" : "none";
    document.body.style.cursor = enable ? "crosshair" : "";
    if (enable) {
      document.addEventListener("mousemove", onMouseMove, true);
      document.addEventListener("click", onMouseClick, true);
      document.addEventListener("keydown", onKeyDown, true);
    } else {
      document.removeEventListener("mousemove", onMouseMove, true);
      document.removeEventListener("click", onMouseClick, true);
      document.removeEventListener("keydown", onKeyDown, true);
      hoveredEl = null;
    }
  }

  function capturePage() {
    const title = document.title;
    const url = location.href;
    const html = document.documentElement.outerHTML.slice(0, 10000);
    const text = document.body.innerText.slice(0, 5000);
    return { title, url, html, text };
  }

  async function playSequence(steps) {
    const results = [];
    for (const step of steps || []) {
      try {
        if (step.type === "click") {
          const el = document.querySelector(step.selector);
          if (el) { el.click(); results.push({ step: step.type, ok: true }); }
          else results.push({ step: step.type, ok: false, error: "selector not found" });
        } else if (step.type === "type") {
          const el = document.querySelector(step.selector);
          if (el) {
            el.focus();
            if ("value" in el) el.value = step.value || "";
            el.dispatchEvent(new Event("input", { bubbles: true }));
            el.dispatchEvent(new Event("change", { bubbles: true }));
            results.push({ step: step.type, ok: true });
          } else results.push({ step: step.type, ok: false, error: "selector not found" });
        } else if (step.type === "wait") {
          await new Promise(r => setTimeout(r, step.ms || 1000));
          results.push({ step: step.type, ok: true });
        } else if (step.type === "extract") {
          const el = document.querySelector(step.selector);
          if (el) results.push({ step: step.type, ok: true, value: el.textContent?.slice(0, 500) || "" });
          else results.push({ step: step.type, ok: false, error: "selector not found" });
        } else if (step.type === "scroll") {
          window.scrollBy({ top: step.y || 0, left: step.x || 0, behavior: "smooth" });
          results.push({ step: step.type, ok: true });
        }
      } catch (e) {
        results.push({ step: step.type, ok: false, error: e.message });
      }
    }
    return results;
  }

  // Listen for messages from background
  chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
    switch (msg.type) {
      case "inspect_mode":
        if (msg.enable === false) setInspectMode(false);
        else setInspectMode(true);
        sendResponse({ ok: true });
        break;
      case "capture_page":
        sendResponse(capturePage());
        break;
      case "play_sequence":
        playSequence(msg.steps).then(sendResponse).catch(e => sendResponse({ error: e.message }));
        return true;
      case "ping":
        sendResponse({ url: location.href, title: document.title });
        break;
    }
  });
})();
