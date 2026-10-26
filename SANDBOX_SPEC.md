# Agent Linux VM Sandbox — Spec

Status: spec approved 2026-10-26. v1 implementation is in
`crates/porpoise-server/src/services/sandbox.rs` as `MicroVmBackend`,
exposed over the relay as the `sandbox_*` methods (provision / start /
stop / clean / delete / status / list / detect) — see "v1 scope" below.
Everything not marked v1 is planned, not built. Nothing in this spec is
faked: a backend that cannot run on the current host must report that
honestly instead of pretending to boot.

## Goal

A Linux VM sandbox for agents that can craft full software and test it, with
a shared base layer, one isolated environment per project, fast start/stop/
clean, and low latency + small disk footprint.

## Stack

- Rust backend: daemon, VM orchestration, isolation, resource limits, vsock
  API, PTY handling. This is `tamaporpoise-server` (axum, default
  `127.0.0.1:7410`).
- TypeScript frontend: dashboard, terminal, file and log viewers, controls.
  Thin shell only — no VM logic in the browser.
- Guest: minimal Linux (Alpine-based), BusyBox/static init, no systemd, no
  GUI, no sshd by default.
- Hypervisor: Firecracker first with KVM in the full design, QEMU `microvm`
  fallback. v1 execution uses the QEMU `microvm` path only (see v1 scope);
  Firecracker boot-source wiring is a follow-up, not v1.
- Container/process backends remain as dev fallbacks only, never as the
  sandbox for untrusted agent code.

## Shared base layer

- Golden base image, read-only, shared by all sandboxes: kernel, minimal OS,
  core tools.
- Toolchain layer, read-only and shared: Rust, C/C++, Go, Python, Node, git,
  build tools — mounted, not copied per VM.
- Agent runtime layer: agent runner, test harness, log collector, vsock client.
- Package cache layer, shared and read-only: apk/cargo/npm/pip caches, so
  builds do not re-download per VM.
- Project layer only: the agent's code, its overlay delta, its config.
  Everything else is shared.
- Base image versioning and rollback: pin a base version per project;
  upgrade the base without touching project deltas.

## Isolation and security

- Kernel-level isolation per sandbox: separate VM, separate kernel, separate
  memory — not just a process or chroot. The existing `ProcessBackend` is
  explicitly NOT a sandbox for untrusted code (isolated workspace + scrubbed
  env only).
- Per-sandbox vCPU, RAM, disk, and process limits, enforced by the host.
- Filesystem isolation: guest root is its overlay only; host workspace is
  mounted read-write only at `/workspace`, everything else read-only or
  invisible.
- Network isolation: default deny, no public internet; optional allowlisted
  registry proxy for package managers only.
- No host device passthrough by default, no shared host sockets, no Docker
  socket inside the guest.
- Resource exhaustion protection: timeout kill, memory cap, disk quota,
  fork/process cap.
- Auth carried over from tmp-server: admin (machine) vs user (assigned
  projects only), Argon2id passwords, session cookies, unassigned project =
  404, machine endpoint = 403 for users.

## Lifecycle functions (Rust backend, `IsolationBackend` trait)

- `create_sandbox(project, base_version, template, limits)` — overlay,
  `vm.toml`, workspace, `microvm.json`.
- `start()` — boots microVM from golden base + overlay, mounts workspace,
  starts agent runner.
- `stop()` — clean shutdown, preserves overlay and logs; kills the
  hypervisor process via its pid file when one exists.
- `kill()` — forced stop on timeout, hang, or resource breach.
- `clean()` — deletes overlay, temp files, and pid file; returns the sandbox
  to pristine base. Workspace is kept. Wired in v1 as
  `POST /api/projects/{id}/clean` (microvm projects only).
- `reset()` — clean + re-provision in one step; the standard "fresh test
  run" operation.
- `restart()` — stop + start, overlay preserved.
- `pause()` / `resume()` — suspend VM state when supported, for idle agents.
- `snapshot()` — saves overlay + VM metadata as a named checkpoint.
- `restore(snapshot)` — boots from a checkpoint instead of base.
- `clone()` — new sandbox from an existing overlay or snapshot, without
  copying the base.
- `delete()` — removes sandbox, overlay, snapshots, and workspace on
  confirmation (existing tmp-server delete route).
- `status()` — running, stopped, paused, cleaning, error, plus uptime and
  resource use.
- `list_sandboxes()` — all sandboxes visible to the caller, scoped by role
  (existing tmp-server project list).

## Build and test functions (agent-facing)

- `exec(command, cwd, env, timeout)` — run a command inside the sandbox,
  streamed output.
- `build()` — run the project's build command in its toolchain layer.
- `test()` — run the full test suite, capture exit code, per-test results,
  and duration.
- `run_script()` — run a named build/test/lint script from the project.
- `install_packages()` — install from the allowlisted cache/proxy only;
  changes land in the overlay only.
- `check()` — lint, format check, and type check as separate steps.
- `benchmark()` — run a benchmark and return timing, separate from
  correctness tests.
- `export_artifacts()` — copy build outputs and test reports out of
  `/workspace` to the host.
- `diff()` — show what changed in the overlay versus base: files
  added/modified/deleted.

## Workspace and file functions

- `mount_workspace()` / `unmount_workspace()` — virtio-fs/9p mount of the
  host project directory at `/workspace`.
- `read_file()` / `write_file()` / `list_files()` — agent and UI file access
  inside `/workspace`.
- `upload()` / `download()` — move files between host and sandbox without
  going through the guest network.
- `sync()` — flush workspace changes back to the host on stop, so nothing is
  lost on clean.
- Persistent vs ephemeral workspace modes: persistent keeps `/workspace`,
  ephemeral discards it with the overlay.

## Terminal and control functions (TypeScript frontend)

- Web terminal per sandbox: xterm.js over WebSocket/vsock, PTY-backed,
  scrollback and resize.
- Agent terminal alongside shell terminal: the cody `:` commands from
  tmp-server, plus a real shell.
- Send input, interrupt (Ctrl-C), and detach/reattach without killing the
  running process.
- Log viewer: live stdout/stderr, filtered by build/test/run, downloadable.
- Start, stop, clean, reset, snapshot, and clone controls per sandbox, with
  confirmation on destructive actions.
- Resource monitor: CPU, RAM, disk delta size, network state, boot time, per
  sandbox.
- File viewer and editor hooks: browse `/workspace`, open logs and test
  reports.
- Dashboard: all projects, their state, base version, and last test result
  at a glance.

## Speed and space optimisation

- Pre-warmed sandbox pool: idle microVMs booted from base, claimed in
  milliseconds when an agent starts work.
- Direct kernel boot, no BIOS, no bootloader, minimal init — target under 1s
  warm boot, a few seconds cold.
- Default 1 vCPU / 512 MB per sandbox (v1 `vm.toml` values), adjustable per
  project, ballooning when idle.
- qcow2 backing-file overlays: base stored once, per-sandbox delta only,
  compression + discard enabled.
- Shared page cache: the second sandbox from the same base is cheaper than
  the first.
- Lazy package fetch: only the packages a build actually uses are
  materialised.
- Fast clean: overlay deletion instead of disk wiping or re-imaging.
- Low-latency agent loop: vsock API instead of SSH, persistent connection,
  no per-command handshake.
- Parallel sandboxes: multiple agents, each isolated, limited only by host
  CPU/RAM, with a queue when the host is full.

## Agent API and automation

- HTTP/WebSocket API for the UI plus vsock API inside the guest, same
  command set.
- Job queue: `queue_build()`, `queue_test()`, cancel, retry, and job history
  per project.
- Webhook/event stream: sandbox started, stopped, test passed/failed,
  resource breach, cleaned.
- Scriptable CLI mirroring the web UI, for agents and for local use.
- Templates: Rust project, C project, Python project, full-stack project —
  each a preset base + toolchain + default build/test commands.
- Environment presets: env vars, resource limits, and network policy per
  template, overridable per project.

## Observability, limits, and recovery

- Per-sandbox logs, metrics, and test history, retained after stop and clean
  (unless explicitly deleted).
- Health checks: guest agent runner heartbeat, host hypervisor heartbeat.
- Auto-recovery: a crashed VM is reported, not silently restarted; restart
  is an explicit action.
- Disk quota alerts: overlay approaching its cap pauses new writes and warns
  before it fails a build.
- Audit log: who created, started, stopped, cleaned, or snapshotted each
  sandbox, and when.

## Admin functions

- User management and project assignment, as in tmp-server today.
- Base image management: publish, pin, retire, and garbage-collect unused
  bases and overlays.
- Host view: total CPU/RAM/disk, running sandboxes, pool size, hypervisor
  and KVM detection.
- Global limits: max sandboxes per user, max concurrent builds, default
  quotas.

## File layout (v1, per project under the server data dir)

Implemented under `<data_dir>/sandboxes/`:

- `sandboxes/<id>/workspace/` — host project workspace, mounted at
  `/workspace` in the guest.
- `sandboxes/<id>/vm.toml` — `backend = "microvm"`, `state`, `cpus = 1`,
  `memory_mb = 512`, `workspace` path.
- `sandboxes/<id>/microvm.json` — base image path, overlay path, kernel path,
  network policy (`deny`), toolchain mount (`ro`).
- `sandboxes/<id>/overlay.qcow2` — per-sandbox delta, created at start from
  the base via `qemu-img create -f qcow2 -b <base> -F qcow2 <overlay>`.
  Not created at provision time.
- `sandboxes/<id>/microvm.pid` — QEMU pid file while running, removed on stop.
- Shared: `sandboxes/images/agent-base.qcow2` (base image),
  `sandboxes/images/vmlinux` (guest kernel).

## v1 scope — what is actually built in this pass

- Spec (this file).
- `MicroVmBackend` in `crates/porpoise-server/src/services/sandbox.rs`:
  relay methods `sandbox_provision` / `sandbox_start` / `sandbox_stop` /
  `sandbox_clean` / `sandbox_delete` / `sandbox_status` / `sandbox_list` /
  `sandbox_detect`, each backed by the `MicroVmBackend` named `"microvm"`:
  - `provision()`: workspace + `vm.toml` (1 CPU / 512 MB) + `microvm.json`.
    No hypervisor required at provision time, so a microvm project can be
    created on any host.
  - `start()`: prerequisite gate — requires `/dev/kvm`, a QEMU binary, the
    base image, and the guest kernel. If any are missing it fails with the
    exact missing items listed. If all are present it creates the overlay
    (if absent) and spawns QEMU `microvm` daemonised with a pid file.
    This QEMU execution path is implemented but NOT verified on a KVM host
    (this host has no `/dev/kvm`, no QEMU, no base image, no kernel — see
    "Host limits" below).
  - `stop()`: kills the pid-file process when present (unix `kill`,
    Windows `taskkill`), removes the pid file, marks the project stopped.
  - `clean_microvm()`: deletes overlay + pid file, keeps workspace and
    config. Wired as `POST /api/projects/{id}/clean` for microvm projects
    only, scoped like start/stop (assigned user or admin).
  - `detect_backends()` gains a `microvm` entry: available only when KVM
    plus a hypervisor binary are present.
- Tests: provision writes the right `vm.toml`/`microvm.json`, start fails
  honestly when prerequisites are missing, clean removes the overlay and
  keeps the workspace, `backend_for("microvm")` resolves.

## Explicitly not in v1

- Firecracker boot-source wiring (detection is reported; execution is QEMU
  only in v1).
- Guest agent runner, vsock API, and build/test execution inside the guest
  (the guest side of `exec`/`build`/`test`).
- Snapshot/restore, clone, pause/resume, pre-warmed pool, templates, job
  queue, and the TypeScript sandbox UI (terminal, resource monitor, file
  viewer for microvm projects). The tmp-server embedded UI, auth, pens,
  agents, and battle screens are unchanged.
- Desktop/GUI in the guest, GPU passthrough, full public internet access.

## Host limits (checked 2026-10-26 on this host)

- No `/dev/kvm`, no `qemu-system-*`, no `firecracker`, no `docker`/`podman`
  on PATH; ~7.7 GB RAM total, ~1.1 GB free at check time. So the QEMU
  execution path cannot be verified here — TCG emulation is deliberately
  not used as a "fast" path. A KVM host with QEMU, a guest kernel, and a
  base image is required for the live boot smoke test.
