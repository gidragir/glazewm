# Agent Guidelines for GlazeWM

## Project Overview

GlazeWM is a high-performance tiling window manager for Windows written in Rust (Edition 2024). It provides dual layout capabilities:
1. **Traditional Tiling / BSP**: Directional splits, floating, and window grouping inspired by i3/bspwm.
2. **Infinite Horizontal Canvas (Niri-inspired)**: Horizontal scrolling workspace canvas with vertical column stacks, viewport synchronization (`offset_x`), customizable column width presets, and dedicated consume/expel mechanics.

The project follows a client-server architecture where the main daemon process manages the container tree and native OS windowing, exposing a non-blocking WebSocket IPC API for CLI interactions and third-party widgets/bars (e.g. Zebar).

---

## Workspace Crates Structure

```
glazewm/ (workspace root, Rust 2024)
├── packages/
│   ├── wm/             # Main daemon executable (glazewm.exe)
│   ├── wm-cli/         # CLI client binary (glazewm-cli.exe)
│   ├── wm-common/      # Shared domain types, DTOs, config parser, and commands
│   ├── wm-platform/    # Low-level OS abstractions (Win32 API bindings, hooks)
│   ├── wm-ipc-client/  # WebSocket client library for IPC
│   ├── wm-macros/      # Procedural derive macros (SubEnum, EnumFromInner)
│   └── wm-watcher/     # Windows watchdog process for crash cleanup
├── artifacts/          # Architecture design iterations and backlog items
│   ├── idea/           # Iteration designs 1-7 (Niri layout, presets, consume/expel, multi-monitor isolation)
│   └── backlog/        # Future feature proposals (e.g. Zebar minimap indicator)
└── resources/          # Packaging scripts, build support (winres), default configs, and icons
```

### Crate Breakdown & Responsibilities

- **`packages/wm`** (`bin: glazewm`):
  - Central daemon managing state (`WmState`), container hierarchy, and layout computations.
  - Houses the Win32 message loop on the main OS thread (`EventLoop`), IPC WebSocket server via Tokio, and system tray menu.
  - Dispatches commands (`commands/`) and handles OS window/display events (`events/`).

- **`packages/wm-cli`** (`bin: glazewm-cli`, `lib: glazewm_cli`):
  - Command-line interface for controlling GlazeWM from shell or keybindings.
  - Spawns daemon if not running or connects via `wm-ipc-client` to issue IPC commands and stream events.

- **`packages/wm-common`** (`lib`):
  - Pure domain models and runtime-agnostic primitives.
  - Defines `InvokeCommand`, `WmEvent`, `WindowState`, `DisplayState`, `ParsedConfig` (parsed from YAML via `serde_yml`), and all serialization DTOs (`WorkspaceDto`, `WindowDto`, `ContainerDto`).
  - Contains `offset_x` and layout metrics used across the workspace.

- **`packages/wm-platform`** (`lib`):
  - Win32 API abstraction layer; encapsulates Windows subsystem interactions (window handles, display enumeration, keyboard/mouse hooks, thread dispatching).
  - Isolates unsafe Win32 calls behind safe Rust interfaces (`NativeWindow`, `DisplayListener`, `WindowListener`, `KeybindingListener`, `Dispatcher`).

- **`packages/wm-ipc-client`** (`lib`):
  - Asynchronous WebSocket client (`IpcClient`) for communicating with the running daemon over local TCP port.

- **`packages/wm-macros`** (`proc-macro`):
  - Custom procedural macros: `#[derive(SubEnum)]` for enum subset mapping and `#[derive(EnumFromInner)]` for boilerplate-free enum conversions.

- **`packages/wm-watcher`** (`bin: glazewm-watcher`):
  - Independent watchdog process that listens to window states over IPC.
  - Restores hidden/modified native window properties (borders, taskbar presence, transparency) if the main WM daemon terminates unexpectedly.

---

## Key Modules & File Map

### `packages/wm/src/`
- `main.rs`: Process entry point, CLI arguments parsing, Tokio runtime setup, and thread spawning.
- `wm.rs` & `wm_state.rs`: Primary window manager orchestration, state transitions, and pending synchronization queue.
- `ipc_server.rs`: Tokio WebSocket server handling client IPC queries and event broadcasting.
- `models/`:
  - `container.rs` / `weak_container.rs`: `Container` enum wrapping `Rc<RefCell<...>>` node variants and weak parent pointers.
  - `workspace.rs`: `Workspace` model tracking child containers, focus order, tiling direction, and canvas `offset_x`.
  - `split_container.rs`: Intermediate branching node for vertical/horizontal splits.
  - `tiling_window.rs` & `non_tiling_window.rs`: Managed window leaf representations.
  - `monitor.rs` & `root_container.rs`: Display monitor roots.
- `commands/`:
  - `window/`: Window manipulation commands (`manage_window`, `unmanage_window`, `move_window_in_direction`, `move_window_to_workspace`, `cycle_column_preset`, `consume_or_expel_window`, `run_window_rules`, `resize_window`).
  - `container/`: Container operations (`attach_container`, `detach_container`, `flatten_split_container`, `focus_in_direction`, `move_container_within_tree`, `resize_tiling_container`).
  - `workspace/`: Workspace switching, activation, sorting (`activate_workspace`, `focus_workspace`, `move_workspace_in_direction`).
  - `monitor/`: Multi-monitor focus and configuration (`add_monitor`, `focus_monitor`, `sort_monitors`, `update_monitor`).
  - `general/`: General commands (`cycle_focus`, `reload_config`, `platform_sync`, `shell_exec`, `toggle_pause`).
- `events/`: Event handlers responding to OS callbacks (`handle_window_focused`, `handle_window_moved_or_resized`, `handle_window_shown`, `handle_window_hidden`, `handle_display_settings_changed`, `handle_mouse_move`).

### `packages/wm-common/src/`
- `parsed_config.rs`: YAML configuration schema, keybindings, window rules, gap settings, and column width presets.
- `app_command.rs`: Command line arguments definition (`clap`).
- `dtos/`: External JSON serialization representations for IPC state queries (`WorkspaceDto`, `WindowDto`, etc.).
- `ipc.rs`: Request/Response/Subscription messages protocol.

### `packages/wm-platform/src/`
- `event_loop.rs`: Platform native message pump.
- `native_window.rs`: Safe wrapper for Win32 `HWND` operations (dimensions, styles, cloaking, placement, batch `DeferWindowPos`).
- `dispatcher.rs`: Cross-thread action dispatcher to invoke operations on the main UI thread.
- `window_listener.rs`, `display_listener.rs`, `keybinding_listener.rs`, `mouse_listener.rs`: OS hook listeners.

---

## Core Architectural Concepts & Invariants

### 1. Dual Layout & Infinite Canvas Mechanics
- **Canvas Orientation**: Infinite horizontal canvas workspaces enforce a horizontal tiling direction at the workspace root, containing vertical `SplitContainer` columns.
- **Viewport Offset Synchronization**: As focus shifts between columns, `workspace.offset_x` automatically synchronizes to center or bring the focused column within the visible display viewport.
- **Column Width Presets**: Columns support dynamic width adjustments (`cycle-column-preset`) allowing toggling between predefined ratios (e.g., 1/3, 1/2, 2/3, full width) and custom fractional sizes.
- **Consume / Expel Operations**: Directional window movement (`Alt+Shift+Left/Right`) is decoupled from column merging to prevent unwanted nesting. Dedicated `consume-or-expel-window-left` and `consume-or-expel-window-right` commands pull neighboring windows into the focused vertical column or expel them back onto the horizontal canvas.
- **Multi-Monitor Canvas Isolation & Safe Offscreen Parking**: On multi-monitor setups, offscreen windows on infinite canvases are parked at safe virtual coordinates (`SAFE_PARK_X = 50_000, SAFE_PARK_Y = 50_000`) outside all physical screens. Partially visible columns are clamped to their monitor's working area boundaries. This guarantees 0-pixel bleeding onto adjacent monitors while keeping window thumbnails alive in the taskbar and Alt+Tab switcher.
- **Atomic Batch Positioning (`DeferWindowPos`)**: Window repositioning batches are pre-processed through `calculate_physical_rect` and applied atomically via `BeginDeferWindowPos`/`DeferWindowPos`/`EndDeferWindowPos` with individual fallback to eliminate DWM rendering stutter and cross-monitor tearing.

### 2. Concurrency & Concurrency Boundaries
- **UI Thread Safety**: Win32 window management APIs must execute on the OS thread running the `EventLoop`.
- **Channel Bridging**: Background Tokio tasks (handling IPC communication and timer intervals) communicate with the UI loop through non-blocking channels (`tokio::sync::mpsc`) and `wm_platform::Dispatcher`.
- **Reference Cycle Prevention**: Container tree nodes use `Rc<RefCell<...>>` for child references and `WeakContainer` (weak references) for parent pointers to prevent memory leaks during container detaching and tree flattening.

---

## Building, Linting & Testing

The repository uses **`mise`** for unified development tasks and **`cargo-xwin`** for cross-compiling from Linux to Windows MSVC targets.

### Mise Task Runners

```bash
mise run check          # Fast type checking (x86_64-pc-windows-gnu)
mise run check:msvc     # Type checking using MSVC target (x86_64-pc-windows-msvc)
mise run clippy         # Run strict clippy lints across all crates
mise run test           # Run unit tests (wm, wm-common, wm-ipc-client) under Wine
mise run test:fast      # Run unit tests with fast profile (incremental + optimized dependencies)
mise run test:perf      # Run tests with high-performance profile (opt-level 3, thin LTO)
mise run benchmark      # Run layout and event loop performance benchmarks
mise run verify         # Comprehensive verification (check + clippy + test)
mise run build:debug    # Build debug binary (glazewm.exe) with MSVC target
mise run build          # Build release binary (glazewm.exe) with MSVC target
mise run build:all      # Build all workspace packages in release mode
mise run deploy         # Copy release binaries to /srv/Shared/ for VM testing
mise run release        # Build all workspace binaries + deploy to /srv/Shared/
mise run sonar:status   # Check SonarCloud quality gate status
mise run sonar:summary  # Print issue summary grouped by rule & severity
mise run sonar:issues   # List open SonarCloud issues
mise run sonar:duplications # List and inspect code duplications
```

### Cargo Profiles & Compilation Optimization

The workspace defines tuned compilation profiles in [`Cargo.toml`](file:///Cargo.toml):
- **Development & Test Acceleration**: Both `[profile.dev]` and `[profile.test]` set `codegen-units = 256` for maximum parallel codegen. Furthermore, external dependencies are compiled with `opt-level = 2` via `[profile.dev.package."*"]` and `[profile.test.package."*"]`. This keeps workspace crate compilation fast (`opt-level = 0`) while executing CPU-heavy dependencies (parsing, regex, serialization, Tokio runtime) at near-native speed during test runs.
- **`test-fast` Profile**: Inherits from `test`, enables `incremental = true`, `debug = 1`, and `codegen-units = 256` for rapid test turnaround (`mise run test:fast`).
- **`test-perf` Profile**: Inherits from `release`, with `opt-level = 3`, `debug = 1` (line tables for profiling), `lto = "thin"`, `codegen-units = 16`, `panic = "unwind"`, and `strip = "none"`. Used for latency-sensitive benchmarks and throughput tests (`mise run benchmark`, `mise run test:perf`).
- **Windows Resource Compilation (`resources/build_support/winres.rs`)**: Windows resource embedding (`tauri-winres`) is centralized in a shared build helper across `packages/wm/build.rs`, `packages/wm-cli/build.rs`, and `packages/wm-watcher/build.rs` to configure application icons, manifests, and version metadata from `VERSION_NUMBER` without duplication.

### Direct Cargo & Cross-Compilation Commands

```bash
# Fast check using GNU target (Linux-friendly)
cargo check --target x86_64-pc-windows-gnu

# Check using MSVC target
cargo check --target x86_64-pc-windows-msvc

# Check specific crate
cargo check -p wm --target x86_64-pc-windows-gnu
cargo check -p wm-common --target x86_64-pc-windows-gnu

# Fast unit testing under Wine (using test-fast profile)
CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER=wine cargo xwin test --profile test-fast -p wm -p wm-common -p wm-ipc-client --target x86_64-pc-windows-msvc

# Performance benchmarks under Wine (using test-perf profile)
CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER=wine cargo xwin test --profile test-perf -p wm --target x86_64-pc-windows-msvc -- benchmark --nocapture

# Build MSVC debug binary
cargo xwin build -p wm --target x86_64-pc-windows-msvc

# Build MSVC release binary (main glazewm daemon)
cargo xwin build --release -p wm --target x86_64-pc-windows-msvc

# Build all workspace packages with MSVC target
cargo xwin build --release --target x86_64-pc-windows-msvc
```

### Binary Output Locations & VM Deployment
- Main WM daemon: `target/x86_64-pc-windows-msvc/release/glazewm.exe`
- CLI executable: `target/x86_64-pc-windows-msvc/release/glazewm-cli.exe`
- Watchdog executable: `target/x86_64-pc-windows-msvc/release/glazewm-watcher.exe`
- Deploy to shared Windows VM mount: `cp target/x86_64-pc-windows-msvc/release/glazewm*.exe /srv/Shared/`

---

## Coding Standards & Conventions

1. **Rust Edition 2024**: Codebase adheres to Rust 2024 edition idioms.
2. **Error Handling**:
   - `wm-platform`: Use `crate::Error` / `crate::Result` with `thiserror`.
   - `wm`, `wm-common`, `wm-cli`, `wm-watcher`: Use `anyhow::Result` enriched with context via `.with_context(|| ...)`.
   - Never use `.unwrap()` or `.expect()` in production code paths.
3. **Strict Clippy Compliance**:
   - The workspace enforces `#![warn(clippy::all, clippy::pedantic)]`.
   - Explicit float/integer casts must be annotated with `#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]` with clear rationale.
   - Use lossless conversions such as `f64::from(i32_val)` where possible.
4. **Logging & Diagnostics**:
   - Use structured `tracing` macros (`tracing::trace!`, `tracing::debug!`, `tracing::info!`, `tracing::warn!`, `tracing::error!`).
5. **Zero-Cost Abstractions & Allocations**:
   - Prefer references (`&T`, `&str`, `&[T]`) over unnecessary `.clone()` calls.
   - Avoid allocations inside high-frequency event passes (window moves, mouse tracking, layout recalculations).

---

## SonarQube & Static Analysis Workflow

GlazeWM is continuously monitored on SonarCloud for code quality, security vulnerabilities, and maintainability.

### Project Configuration
- **Platform**: SonarCloud (`https://sonarcloud.io`)
- **Project Key**: `gidragir_glazewm`
- **Organization**: `gidragir`
- **Config**: [`sonar-project.properties`](file:///sonar-project.properties)

### MCP Server & Skill Integration
- **Installed Skill**: [`.agents/skills/sonarqube-mcp/`](file:///.agents/skills/sonarqube-mcp/) provides end-to-end guidance and patterns for quality gates, issue triage, and pre-push analysis.
- **MCP Server**: `sonarqube` (lazy-loaded). When querying issues or quality gates, always supply `"projectKey": "gidragir_glazewm"` and `"resolved": false`.
- **AI Rule**: [`.agents/rules/sonarqube.md`](file:///.agents/rules/sonarqube.md) defines explicit protocols for AI pair-programming and refactoring.

### Project CLI Helper (`resources/scripts/sonar.py`)
Agents and developers should **never write ad-hoc inline Python scripts** to fetch issues or parse JSON outputs. Use the built-in zero-dependency utility:

```bash
# Quality Gate status
mise run sonar:status
python3 resources/scripts/sonar.py status

# High-level summary of open issues by rule & severity
mise run sonar:summary
python3 resources/scripts/sonar.py summary

# Filter issues by severity, type, rule, or file
mise run sonar:issues
python3 resources/scripts/sonar.py issues --severity CRITICAL --limit 10
python3 resources/scripts/sonar.py issues --rule rust:S3776
python3 resources/scripts/sonar.py issues --file packages/wm/src/wm.rs

# View detailed rule documentation & remediation guidance
python3 resources/scripts/sonar.py rule rust:S3776

# Inspect code duplications and exact matching blocks
mise run sonar:duplications
python3 resources/scripts/sonar.py duplications --details

# Parse raw MCP tool output JSON without writing custom parsing scripts
python3 resources/scripts/sonar.py parse-mcp path/to/mcp_output.json
```

