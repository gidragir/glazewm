# wm-common

## Component Role
Core workspace utility package providing shared domain data types, DTOs, IPC messaging formats, CLI argument parser contracts (`clap`), configuration schemas, window/display state primitives, and extension traits.

## Dependency Graph
- **Inherited (`workspace = true`)**:
  - `anyhow` (v1, workspace features: `["backtrace"]`)
  - `clap` (v4, workspace features: `["derive"]`)
  - `regex` (v1)
  - `serde` (v1, workspace features: `["derive"]`)
  - `tracing` (v0.1)
  - `uuid` (v1, workspace features: `["v4", "serde"]`)
- **Local Workspace Peers**:
  - `wm-platform` (`path = "../wm-platform"`)

## Public API

### Constants & Macros
- `pub const DEFAULT_IPC_PORT: u32 = 6123`
- `macro_rules! try_warn` — Macro logging operation errors via `tracing::warn!` and returning `Ok(())`.

### Extension Traits
- `pub trait UniqueExt: Iterator`
  - `fn unique_by<K, F>(self, key_fn: F) -> UniqueBy<Self, K, F> where Self: Sized, K: Hash + Eq, F: FnMut(&Self::Item) -> K`
- `pub trait VecDequeExt<T> where T: PartialEq`
  - `fn shift_to_index(&mut self, target_index: usize, item: T)`

### CLI & Application Commands (`app_command.rs`)
- `pub enum AppCommand` (`Start`, `Query`, `Command`, `Sub`, `Unsub`)
  - `pub fn parse_with_default(args: &Vec<String>) -> Self`
- `pub struct Verbosity` (`verbose: bool`, `quiet: bool`)
  - `pub fn level(&self) -> Level`
- `pub enum QueryCommand` (`AppMetadata`, `BindingModes`, `Focused`, `TilingDirection`, `Monitors`, `Windows`, `Workspaces`, `Paused`)
- `pub enum SubscribableEvent` (`All`, `ApplicationExiting`, `BindingModesChanged`, `FocusChanged`, `FocusedContainerMoved`, `MonitorAdded`, `MonitorUpdated`, `MonitorRemoved`, `TilingDirectionChanged`, `UserConfigChanged`, `WindowManaged`, `WindowUnmanaged`, `WorkspaceActivated`, `WorkspaceDeactivated`, `WorkspaceUpdated`, `PauseChanged`)
- `pub enum InvokeCommand` (CLI subcommand enum for WM operations: `AdjustBorders`, `Close`, `Focus`, `Ignore`, `Move`, `MoveWorkspace`, `Position`, `Resize`, `UpdateWorkspaceConfig`, `SetFloating`, `SetFullscreen`, `SetMinimized`, `SetTiling`, `SetTitleBarVisibility`, `SetTransparency`, `ShellExec`, `Size`, `ToggleFloating`, `ToggleFullscreen`, `ToggleMinimized`, `ToggleTiling`, `ToggleTilingDirection`, `SetTilingDirection`, `WmCycleFocus`, `WmDisableBindingMode`, `WmEnableBindingMode`, `WmExit`, `WmRedraw`, `WmReloadConfig`, `WmTogglePause`)

### Configuration Schema (`parsed_config.rs`)
- `pub struct ParsedConfig`
  - Fields: `binding_modes`, `gaps`, `general`, `keybindings`, `window_behavior`, `window_effects`, `window_rules`, `workspaces`.
- `pub struct WindowRuleConfig` (`commands`, `match_window`, `on`, `run_once`)
- `pub enum MatchType` (`Equals`, `Includes`, `Regex`, `NotEquals`, `NotRegex`)
  - `pub fn is_match(&self, value: &str) -> bool`
- Configuration section structs: `BindingModeConfig`, `GapsConfig`, `GeneralConfig`, `CursorJumpConfig`, `KeybindingConfig`, `WindowBehaviorConfig`, `WindowStateDefaultsConfig`, `FloatingStateConfig`, `FullscreenStateConfig`, `WindowEffectsConfig`, `WindowEffectConfig`, `BorderEffectConfig`, `HideTitleBarEffectConfig`, `CornerEffectConfig`, `TransparencyEffectConfig`, `WorkspaceConfig`.

### IPC Messages (`ipc.rs`)
- `pub enum ServerMessage` (`ClientResponse(ClientResponseMessage)`, `EventSubscription(EventSubscriptionMessage)`)
- `pub struct ClientResponseMessage` (`client_message: String`, `data: Option<ClientResponseData>`, `error: Option<String>`, `success: bool`)
- `pub enum ClientResponseData` (`AppMetadata`, `BindingModes`, `Command`, `EventSubscribe`, `EventUnsubscribe`, `Focused`, `Monitors`, `TilingDirection`, `Windows`, `Workspaces`, `Paused(bool)`)
- `pub struct EventSubscriptionMessage` (`data: Option<WmEvent>`, `error: Option<String>`, `subscription_id: Uuid`, `success: bool`)

### Data Transfer Objects (`dtos/`)
- `pub enum ContainerDto` (`Root(RootContainerDto)`, `Monitor(MonitorDto)`, `Workspace(WorkspaceDto)`, `Split(SplitContainerDto)`, `Window(WindowDto)`)
- Struct DTO definitions: `RootContainerDto`, `MonitorDto`, `WorkspaceDto`, `SplitContainerDto`, `WindowDto`.

### Window State & Events
- `pub struct ActiveDrag` (`operation: Option<ActiveDragOperation>`, `is_from_floating: bool`, `initial_position: Rect`)
- `pub enum DisplayState` (`Shown`, `Showing`, `Hidden`, `Hiding`)
- `pub enum HideCorner` (`BottomLeft`, `BottomRight`)
- `pub enum TilingDirection` (`Horizontal`, `Vertical`)
  - `pub fn inverse(&self) -> Self`
  - `pub fn from_direction(direction: &wm_platform::Direction) -> Self`
- `pub enum WindowState` (`Floating`, `Fullscreen`, `Minimized`, `Tiling`)
  - `pub fn default_from_config(config: &ParsedConfig) -> Self`
  - `pub fn is_same_state(&self, other: &Self) -> bool`
- `pub enum WmEvent` — Enum covering all broadcast window manager events.

## Architecture & Modules
- `wm-common` (`src/lib.rs`)
  - `active_drag` (`src/active_drag.rs`) — Active window drag operation models.
  - `app_command` (`src/app_command.rs`) — `clap` parser definitions for CLI and IPC commands.
  - `display_state` (`src/display_state.rs`) — Window display status enum (`Shown`, `Hidden`, etc.).
  - `dtos` (`src/dtos/mod.rs`) — User-friendly JSON-serializable container projections for IPC/logging.
    - `container_dto`, `monitor_dto`, `root_container_dto`, `split_container_dto`, `window_dto`, `workspace_dto`.
  - `hide_corner` (`src/hide_corner.rs`) — Screen corner targeting for off-screen hidden windows.
  - `ipc` (`src/ipc.rs`) — Server/Client IPC protocol message structures and constants.
  - `parsed_config` (`src/parsed_config.rs`) — Typed configuration schema and Serde custom deserializers.
  - `tiling_direction` (`src/tiling_direction.rs`) — Directional tiling split models and conversion logic.
  - `utils` (`src/utils/mod.rs`) — Common utility traits and macros.
    - `iterator_ext`, `try_warn`, `vec_deque_ext`.
  - `window_state` (`src/window_state.rs`) — Window state classification (`Tiling`, `Floating`, etc.).
  - `wm_event` (`src/wm_event.rs`) — Event payload taxonomy emitted by GlazeWM.

## Execution Context
- **Sync/Async Boundaries**: Purely synchronous domain layer; contains no async runtime code or dependencies.
- **Runtime Requirement**: Runtime-agnostic foundation dependency.
- **State Mutability**: Read-only DTOs and value objects; minimal mutation (`VecDequeExt::shift_to_index`).
- **Locking & Concurrency**: Unlocked, immutable data definitions safely sharable across threads (`Send + Sync`).
