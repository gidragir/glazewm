# wm

## Component Role
Main window manager daemon executable (`glazewm`) for GlazeWM. Manages container tree hierarchies (monitors, workspaces, split containers, windows), handles OS platform windowing events, processes user layout commands, serves IPC WebSocket requests, and manages system tray controls.

## Dependency Graph
- **Build Dependencies**:
  - `tauri-winres` (workspace = true) — Compiles Windows binary resources (icons, manifests, and version info).
- **External Dependencies**:
  - `ambassador` (v0.4)
  - `auto-launch` (v0.5)
  - `enum-as-inner` (v0.6)
  - `image` (v0.25)
  - `serde_yaml` (v0.9)
  - `shell-util` (v0.0)
  - `tracing-appender` (v0.2)
  - `tray-icon` (v0.21)
- **Inherited (`workspace = true`)**:
  - `anyhow` (v1, workspace features: `["backtrace"]`)
  - `clap` (v4, workspace features: `["derive"]`)
  - `futures-util` (v0.3)
  - `home` (v0.5)
  - `serde` (v1, workspace features: `["derive"]`)
  - `serde_json` (v1, workspace features: `["raw_value"]`)
  - `tokio` (v1, workspace features: `["full"]`)
  - `tokio-tungstenite` (v0.26)
  - `tracing` (v0.1)
  - `tracing-subscriber` (v0.3, workspace features: `["env-filter"]`)
  - `uuid` (v1, workspace features: `["v4", "serde"]`)
  - `wm-macros` (workspace = true)
- **Local Workspace Peers**:
  - `wm-cli` (`path = "../wm-cli"`)
  - `wm-common` (`path = "../wm-common"`)
  - `wm-ipc-client` (`path = "../wm-ipc-client"`)
  - `wm-platform` (`path = "../wm-platform"`)
- **Dev Dependencies**:
  - `bon` (v3)
  - `wm-platform` (`path = "../wm-platform"`, features: `["test_utils"]`)

## Public API

### Binary Entry Point (`src/main.rs`)
- `fn main() -> anyhow::Result<()>`
  - Main entry point; parses CLI arguments. Initializes Tokio runtime and OS `EventLoop` for `AppCommand::Start`, or forwards subcommands to `wm_cli::start(args)`.

### Core Application State & Orchestration (`src/`)
- `pub struct WindowManager`
  - Encapsulates `event_rx`, `exit_rx`, and `state: WmState`.
  - `pub fn new(config: &mut UserConfig, dispatcher: Dispatcher) -> anyhow::Result<Self>`
  - `pub fn process_event(&mut self, event: PlatformEvent, config: &mut UserConfig) -> anyhow::Result<()>`
  - `pub fn process_commands(&mut self, commands: &[InvokeCommand], subject_container_id: Option<Uuid>, config: &mut UserConfig) -> anyhow::Result<()>`
- `pub struct WmState`
  - Central mutable tree state, platform dispatcher handle, binding mode configuration, and pending UI sync operations.
  - `pub fn new(dispatcher: Dispatcher, event_tx: mpsc::UnboundedSender<WmEvent>, exit_tx: mpsc::UnboundedSender<()>) -> Self`
  - `pub fn populate(&mut self, config: &mut UserConfig) -> anyhow::Result<()>`
- `pub struct IpcServer`
  - `pub async fn start() -> anyhow::Result<Self>` — Binds TCP WebSocket server on `DEFAULT_IPC_PORT`.
- `pub struct UserConfig`
  - `pub fn new(config_path: Option<PathBuf>) -> anyhow::Result<Self>`
  - `pub fn read(path: &Path) -> anyhow::Result<(ParsedConfig, String)>`
- `pub struct SystemTray`
  - `pub fn new(dispatcher: Dispatcher) -> anyhow::Result<Self>`
- `pub struct PendingSync`
  - `pub fn has_changes(&self) -> bool`
  - `pub fn clear(&mut self) -> &mut Self`

### Domain Container Models (`src/models/`)
- `pub enum Container` (`Root`, `Monitor`, `Workspace`, `Split`, `Window`)
- Struct models: `RootContainer`, `Monitor`, `Workspace`, `SplitContainer`, `WindowContainer`, `TilingWindow`, `NonTilingWindow`, `NativeMonitorProperties`, `NativeWindowProperties`, `InsertionTarget`, `WorkspaceTarget`.

### Trait Extensions (`src/traits/`)
- `pub trait CommonGetters`
- `pub trait PositionGetters`
- `pub trait TilingDirectionGetters`
- `pub trait TilingSizeGetters`
- `pub trait WindowGetters`

## Architecture & Modules
- `wm` (`src/main.rs`)
  - `commands` (`src/commands/mod.rs`) — Layout manipulation subcommands (`container`, `general`, `monitor`, `window`, `workspace`).
  - `events` (`src/events/mod.rs`) — OS windowing platform event handlers (display metrics, window creation/destruction, drag/move/resize).
  - `ipc_server` (`src/ipc_server.rs`) — Tokio-based WebSocket IPC listener and JSON message processor.
  - `models` (`src/models/mod.rs`) — Tree structure node definitions representing monitors, workspaces, containers, and windows.
  - `pending_sync` (`src/pending_sync.rs`) — Deferred visual update queue (z-order, borders, focus state, redraws).
  - `sys_tray` (`src/sys_tray.rs`) — Native system tray icon and context menu integration.
  - `traits` (`src/traits/mod.rs`) — Tree navigation and spatial geometry getter extension traits.
  - `user_config` (`src/user_config.rs`) — Configuration loading, YAML parsing, and event rule matching.
  - `wm` (`src/wm.rs`) — Top-level `WindowManager` loop logic and command execution pipeline.
  - `wm_state` (`src/wm_state.rs`) — State store for container graph, active binding modes, and platform dispatcher operations.

## Execution Context
- **Sync/Async Boundaries**: Dual-threading runtime model. OS windowing event loop (`wm_platform::EventLoop`) runs on the main thread, while IPC operations and TCP connections execute on Tokio background tasks.
- **Runtime Requirement**: Requires Tokio asynchronous runtime alongside OS native event loop dispatchers.
- **State Mutability**: Tree mutations, layout reorganizations, and pending sync side-effects require exclusive mutable access (`&mut WmState`).
- **Locking & Concurrency**: Unbounded channels (`mpsc::UnboundedChannel`) and broadcast channels (`broadcast::channel`) pass messages asynchronously between Tokio tasks and the main event loop thread without shared mutexes.
