# wm-cli

## Component Role
Command-line interface (CLI) executable (`glazewm-cli`) for GlazeWM. Boots the primary window manager daemon (`glazewm.exe`) or dispatches queries, commands, and event subscriptions over IPC via WebSocket.

## Dependency Graph
- **Build Dependencies**:
  - `tauri-winres` (v0.1) — Embeds Windows resources (`icon.ico`, manifest version metadata) into `glazewm-cli.exe`.
- **Inherited (`workspace = true`)**:
  - `anyhow` (v1, workspace features: `["backtrace"]`)
  - `futures-util` (v0.3)
  - `serde_json` (v1, workspace features: `["raw_value"]`)
  - `tokio` (v1, workspace features: `["full"]`)
  - `tokio-tungstenite` (v0.26)
  - `uuid` (v1, workspace features: `["v4", "serde"]`)
- **Local Workspace Peers**:
  - `wm-common` (`path = "../wm-common"`)
  - `wm-ipc-client` (`path = "../wm-ipc-client"`)

## Public API

### Library Exports (`src/lib.rs`)
- `pub async fn start(args: Vec<String>) -> anyhow::Result<()>`
  - Establishes `IpcClient` connection, formats arguments into an IPC request string, transmits command payload, and prints JSON-serialized response to stdout. Continuously streams event updates when handling `ClientResponseData::EventSubscribe`.

### Binary Entry Point (`src/main.rs`)
- `#[tokio::main] async fn main() -> anyhow::Result<()>`
  - Parses CLI flags using `wm_common::AppCommand::parse_with_default`.
  - On `AppCommand::Start`: Resolves `glazewm.exe` binary location and launches it asynchronously as an isolated shell process via `cmd.exe /C start`.
  - On all other commands (`Query`, `Command`, `Sub`, `Unsub`): Delegates execution to `wm_cli::start(args)`.

## Architecture & Modules
- `wm-cli` (`src/lib.rs`)
  - IPC payload dispatcher and live event loop handler.
- `glazewm-cli` (`src/main.rs`)
  - Executable entry point handling process launch vs IPC client subcommands.
- `build.rs`
  - Windows build script embedding executable resources, file versioning flags, and application icons.

## Execution Context
- **Runtime Requirement**: Tokio multi-threaded asynchronous runtime (`#[tokio::main]`).
- **Sync/Async Boundaries**: IPC interactions are non-blocking async operations (`wm_ipc_client::IpcClient`); process launching for `Start` uses synchronous `std::process::Command` execution.
- **State Mutability**: Mutable `IpcClient` connection managed locally within `start()` async frame.
- **Locking & Concurrency**: Unlocked, linear execution path; subscriptions block on an infinite async loop yielding JSON objects to `stdout`.
