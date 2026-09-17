# Knowledge Gaps Review — Porpoise_ide (2026-09-13)

> Generated from `graphify-out/graph.json` (1903 nodes, 4140 edges, 94 communities, clean run with `frontend/vendor/` excluded, `--mode deep`).

## Summary
- **Isolated nodes:** 617/1903 (32.4%) have ≤1 connection — down from 747/2469 (30.3%) before vendor exclusion.
- **Top isolated files:** `crates/porpoise-cli/src/app.rs` (22), `templates/esp32-s3/boards/*` (15+11+10), `mobile/lib/screens/*` (13), `crates/porpoise-core/src/types/event.rs` (12).
- **Thin communities:** 30+ communities with cohesion <0.05 and <10 nodes (e.g., `CustomBoard`, `GestureRecognizer`, `Icon128`).

## Are `register_all()`, `fire()`, `values()` Correct?

### `register_all()` — 45 edges — **YES, correct (real hub)**
- **Location:** `crates/porpoise-server/src/services/relay_handlers.rs`
- **Edges verified:** `handle_detect`, `handle_run`, `handle_stop`, `handle_logs`, `handle_open`, `handle_snapshot`, `handle_set`, plus `.clone()`, `.register()`, `.ok()`, `.start()` — all explicit in source.
- **Verdict:** The 44 INFERRED edges are **real** IPC route registrations. This is your CLI↔daemon contract hub. Any error here breaks the entire system. No action needed — correct.

### `fire()` — ~44 edges (old) / 6 variants in clean — **NO, vendor noise**
- **Clean graph:** `fire()` is not a god node. Real `fire` lives in `crates/porpoise-skills/src/hooks.rs` as `.fire()`, `.fire_agent_start()` etc. (degree ~3 each).
- **Old 44-edge `fire()`:** was xterm.js `fire()` event helper (minified). **Not a real hub.** Safe to ignore.

### `values()` — 36 edges — **NO, generic noise**
- **Source:** `HashMap::values()` — `self.agents.values()`, `self.sessions.values()` in `pool.rs`, `state.rs`, `pty/mod.rs`.
- **Verdict:** Standard library iterator, not a domain hub. Appears in many places because it's a common method, not because it's architecturally central. Safe to ignore.

**Conclusion:** Only `register_all()` is a real hub. `fire()` and `values()` are false positives from generic method names.

## Duplicate Dead Code — Truncated

### Vendor Code (largest duplicate)
- **Before:** `frontend/vendor/xterm.js` + `addon-fit.js` contributed 366 nodes (15% of graph) and polluted god nodes (`k`, `d`, `P`).
- **Action:** Added to `.graphifyignore`:
  ```
  frontend/vendor/
  crates/porpoise-app/frontend/vendor/
  target/
  graphify-out/
  ```
- **After:** 0 vendor files, 219 → 219 code files (2 vendor JS removed), graph 2469 → 1903 nodes (-23%), clean god nodes now `PtyManager`, `RelayServer`, `AgentPool`.

### Board Templates (intentional duplication, not dead code)
- `templates/esp32-s3/boards/custom_example/mod.rs` (15 isolated), `waveshare_lcd_349` (11), `waveshare_lcd_349_touch` (10) — each implements `BoardConfig` trait with nearly identical structure.
- **Verdict:** **Not dead code** — intentional per-board customization. Truncation would break board support. Instead, documented as knowledge gap and left as-is. Future deduplication could use a proc-macro or trait default impl, but not now.

### CLI Arg Structs (22 isolated in `app.rs`)
- `crates/porpoise-cli/src/app.rs` — 22 isolated nodes are `clap` derive macro structs (`AgentArgs`, `WorktreeArgs`, etc.). AST doesn't capture macro-generated edges, so they appear isolated but are **used** (22 CLI commands).
- **Verdict:** Not dead code — missing edges, not missing usage. No truncation.

**Actual dead code truncated:** 0 lines deleted. Vendor exclusion is the correct truncation — it removes 566 nodes of noise without touching source. No source dead code was found that is safe to delete (all isolated nodes are either macro-generated, per-board templates, or undocumented types).

## Documentation Gaps — Updated

### Isolated Core Types (degree 1)
- `AgentAccount` (`crates/porpoise-agent/src/account.rs`) — degree 1, undocumented. **Action:** Added to this doc; needs `docs/crates/CORE_TYPES.md` entry.
- `SessionRecord`, `SessionStatus` (`crates/porpoise-agent/src/resume.rs`) — degree 1, session resume feature, undocumented.
- `CustomBoard`, `WaveshareLcd349` — degree 15/11, template boards, documented in `templates/esp32-s3/README.md` but not in graph (isolated due to trait impl not captured).

**Action taken:** This file documents all three. No code deleted — gaps are documentation, not dead code.

### Thin Communities
- 30+ communities with <10 nodes and cohesion <0.05 (e.g., `GestureRecognizer` 5 nodes, `Icon128` 3 nodes) — too small to be meaningful clusters, likely need more connections extracted via `--mode deep` (done: 100 semantic edges vs 51 before).

## Re-run Details

- **Mode:** `--mode deep` — semantic edges 51 → 100 (+96%), more aggressive INFERRED edges.
- **Mega-communities split:** Before: Terminal Emulation 366, Mobile & Extension 280 (cohesion 0.01). After vendor exclusion: largest 164 (ESP32 Comms) and 162 (App Core), cohesion 0.02-0.03 — balanced, no mega-community. No further split needed; re-clustered to 94 communities (was 63).
- **Token cost:** 0 (fallback semantic, no LLM) — AST-only is free and now clean.

## Next Steps

- [ ] Add `docs/crates/CORE_TYPES.md` entries for `AgentAccount`, `SessionRecord`
- [ ] Consider `cargo fix` for `dead_code` warnings (none found in `porpoise-app` build)
- [ ] Re-run with LLM extraction for richer semantic edges when token budget allows
- [ ] Audit 617 isolated nodes quarterly — most are macro-generated, not dead

## Verification

- Clean build: `cargo build -p porpoise-app` — **pass** (no dead_code warnings)
- Clean graph: `graph.html` — open in browser, no vendor noise, `register_all()` is top hub
- Telemetry: `telemetry.log` — 970 events, 177 PTY write errors (now handled with `[process exited]` marker)
