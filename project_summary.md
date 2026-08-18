# GlazeWM Project Summary

## Overview
GlazeWM is a tiling window manager for Windows, inspired by i3wm. It allows users to organize windows and adjust layouts using keyboard-driven commands. The project is implemented in Rust and structured as a Cargo workspace with multiple dedicated packages handling different aspects of the window management lifecycle, from native OS abstractions to CLI interactions and IPC communication.

## System Architecture

The workspace follows a client-server architecture, where the main window manager daemon runs in the background and clients communicate with it via a WebSocket-based IPC protocol. 

The architecture is divided into the following layers:
- **Core Daemon (`wm`)**: The central window manager process that maintains the container tree and processes events.
- **Client Interfaces (`wm-cli`, `wm-watcher`)**: Auxiliary executables that interact with the main daemon.
- **Platform Abstraction (`wm-platform`)**: Cross-platform OS bindings (Win32 API for Windows, Cocoa/AX for macOS).
- **Shared Primitives (`wm-common`, `wm-ipc-client`, `wm-macros`)**: Common domain types, DTOs, macros, and IPC clients used across the workspace.

## Package Breakdown

### `wm` (Main Daemon)
The primary window manager executable (`glazewm.exe`). 
- **Role**: Manages container tree hierarchies (monitors, workspaces, split containers, windows), handles OS platform windowing events, processes user layout commands, serves IPC WebSocket requests, and manages system tray controls.
- **Concurrency**: Dual-threading model. The OS windowing event loop (`wm_platform::EventLoop`) runs on the main thread, while IPC operations and TCP connections execute on Tokio background tasks.

### `wm-cli` (Command-Line Interface)
The CLI executable (`glazewm-cli.exe`).
- **Role**: Boots the primary window manager daemon (`glazewm.exe`) or dispatches queries, commands, and event subscriptions over IPC via WebSocket. 
- **Execution**: `AppCommand::Start` launches the main daemon asynchronously via `cmd.exe /C start`. All other commands interact directly with the running daemon.

### `wm-watcher` (Watchdog Daemon)
An auxiliary watchdog daemon.
- **Role**: Monitors open window handles via IPC event streaming. If the main window manager process terminates unexpectedly, it restores native window states (visibility, taskbar presence, border color, and transparency) to prevent windows from being permanently lost or hidden.

### `wm-platform` (OS Abstraction Layer)
Cross-platform native windowing bindings.
- **Role**: Encapsulates OS-specific windowing subsystem calls (Win32 API on Windows, Cocoa and Accessibility AX APIs on macOS) into unified primitives for event loops, display enumeration, native window manipulation, input listeners, and thread-safe dispatching.

### `wm-ipc-client` (IPC Client Library)
Asynchronous WebSocket client library.
- **Role**: Provides the `IpcClient` struct for establishing IPC communication with the local GlazeWM server instance, transmitting commands, and polling for event subscriptions.

### `wm-common` (Domain & Utilities)
Core workspace utility package.
- **Role**: Provides shared domain data types, DTOs, IPC messaging formats, CLI argument parser contracts (`clap`), configuration schemas, window/display state primitives, and extension traits. Completely synchronous and runtime-agnostic.

### `wm-macros` (Procedural Macros)
Dedicated procedural macro crate.
- **Role**: Provides custom derive macros (`SubEnum`, `EnumFromInner`) for AST manipulation and boilerplate elimination across workspace enum types. Operates entirely at compile-time.

## Execution Context & Concurrency
- **Async Runtime**: The project heavily relies on the **Tokio** asynchronous runtime for non-blocking IPC communication (WebSocket over TCP) and event streaming.
- **Sync/Async Boundaries**: Platform UI operations and event loops strictly require execution on the main OS thread. Listeners (`WindowListener`, `DisplayListener`, etc.) stream native event notifications into `tokio::sync::mpsc::UnboundedReceiver` channels to bridge the synchronous UI world with the asynchronous Rust tasks.
- **State Mutability**: The central tree state (`WmState`) requires exclusive mutable access for layout reorganizations and pending sync side-effects, managed carefully without excessive cross-thread locking by using message passing.
