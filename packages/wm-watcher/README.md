## Component Role
`wm-watcher` serves as a Windows-exclusive auxiliary watchdog daemon for GlazeWM that monitors open window handles via IPC event streaming and restores native window states (visibility, taskbar presence, border color, and transparency) if the main window manager process terminates unexpectedly.

## Dependency Graph
- Inherited (`workspace = true`):
  - `anyhow` (`version = "1"`, features: `["backtrace"]`)
  - `serde_json` (`version = "1"`, features: `["raw_value"]`)
  - `tokio` (`version = "1"`, features: `["full"]`)
  - `tracing` (`version = "0.1"`)
  - `tracing-subscriber` (`version = "0.3"`, features: `["env-filter"]`)
  - `tauri-winres` [build-dependencies] (`version = "0.1"`)
- External:
  - N/A (all third-party crates are inherited from `[workspace.dependencies]`)
- Local workspace peers:
  - `wm-common` (`path = "../wm-common"`)
  - `wm-ipc-client` (`path = "../wm-ipc-client"`)
  - `wm-platform` (`path = "../wm-platform"`)

## Public API
- Binary Target (`glazewm-watcher`): Exposes no public library traits, structs, enums, or functions.
- Entry Points:
  - `#[tokio::main] async fn main() -> anyhow::Result<()>` (`src/main.rs`)
  - `fn main()` (`build.rs`)
- Internal Function Signatures (`src/main.rs`):
  - `async fn query_initial_windows(client: &mut wm_ipc_client::IpcClient) -> anyhow::Result<Vec<wm_common::WindowDto>>`
  - `async fn watch_managed_handles(client: &mut wm_ipc_client::IpcClient, handles: &mut Vec<isize>) -> anyhow::Result<()>`

## Architecture & Modules
```
Crate Root: wm-watcher (bin: "glazewm-watcher")
├── build.rs
│   └── fn main()
│       ├── Target Assertion: panic if cfg!(not(target_os = "windows"))
│       └── tauri_winres::WindowsResource
│           ├── set_icon("../../resources/assets/icon.ico")
│           ├── set_language(0x0409) [en-US]
│           └── set_version_info(FileVersion & ProductVersion from env!("VERSION_NUMBER"))
└── src/main.rs
    ├── Subsystem Directive: #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
    ├── async fn main() -> anyhow::Result<()>
    │   ├── tracing_subscriber::fmt().init()
    │   ├── IpcClient::connect().await -> IpcClient
    │   ├── query_initial_windows(&mut client).await -> Vec<WindowDto>
    │   ├── watch_managed_handles(&mut client, &mut managed_handles).await -> Result<()>
    │   └── match subscribe_res:
    │       ├── Ok(()) => log clean exit
    │       └── Err(err) => cleanup loop over managed_handles
    │           └── NativeWindow::from_handle(handle)
    │               ├── window.show() -> Result<()>
    │               ├── window.set_taskbar_visibility(true) -> Result<()>
    │               ├── window.set_border_color(None) -> Result<()>
    │               └── window.set_transparency(&OpacityValue::from_alpha(u8::MAX)) -> Result<()>
    ├── async fn query_initial_windows(client: &mut IpcClient) -> anyhow::Result<Vec<WindowDto>>
    │   ├── client.send("query windows").await
    │   ├── client.client_response("query windows").await
    │   └── Filter ClientResponseData::Windows -> Vec<WindowDto>
    └── async fn watch_managed_handles(client: &mut IpcClient, handles: &mut Vec<isize>) -> anyhow::Result<()>
        ├── client.send("sub -e window_managed window_unmanaged application_exiting").await
        ├── Extract subscription_id from ClientResponseData::EventSubscribe
        └── loop over client.event_subscription(&subscription_id).await:
            ├── WmEvent::WindowManaged { managed_window } => handles.push(window.handle)
            ├── WmEvent::WindowUnmanaged { unmanaged_handle } => handles.retain(|&h| h != unmanaged_handle)
            ├── WmEvent::ApplicationExiting => return Ok(())
            └── None => anyhow::bail!("IPC connection closed unexpectedly.")
```

## Execution Context
- Runtime Requirements: Tokio asynchronous runtime (`#[tokio::main]`), targeted exclusively to `target_os = "windows"`.
- Subsystem Boundaries: Spawning of win32 console windows is disabled in non-debug mode via `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`.
- Sync/Async Boundaries: Async I/O for IPC message delivery and event stream listening (`IpcClient`); synchronous Win32 window attribute recovery via native handles (`wm_platform::NativeWindow`) during abnormal termination cleanup.
- State Mutability: Exclusive single-threaded vector mutation of open window handles (`&mut Vec<isize>`) within the async event subscription loop.
- Locking Mechanisms: None; single-threaded async execution context with no cross-thread shared state (`Arc`, `Mutex`, `RwLock`) or synchronization channels.
