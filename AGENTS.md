# Agent Guidelines for GlazeWM

## Project Overview

GlazeWM is a tiling window manager for Windows and macOS written in Rust, featuring both traditional tiling and a Niri-inspired infinite horizontal scrolling layout.

### Workspace Crates Structure

- **`packages/wm`** (`bin: glazewm`): Core window management logic, layout engine, event handlers, and command execution.
- **`packages/wm-cli`** (`bin, lib: glazewm`): CLI client for sending commands and querying state over IPC.
- **`packages/wm-common`** (`lib`): Shared data models, command definitions (`InvokeCommand`), config parser (`ParsedConfig`), and DTOs.
- **`packages/wm-platform`** (`lib`): Low-level platform API abstractions (Win32 API bindings, `NativeWindow`, display listeners, keyboard/mouse hooks).
- **`packages/wm-ipc-client`** (`lib`): WebSocket client for IPC communication.
- **`packages/wm-watcher`** (`bin: glazewm-watcher`): Windows watchdog process for cleanup upon exit.

---

## Building the Project

When developing on Linux for Windows targets, use **`cargo-xwin`** or the provided **`mise`** tasks.

### Quick Commands with Mise

```bash
mise run check       # Fast check (x86_64-pc-windows-gnu)
mise run clippy      # Run strict clippy lints
mise run test        # Run unit tests
mise run build       # Build release binary (glazewm.exe)
mise run build:all   # Build all workspace binaries
mise run deploy      # Copy release binaries to /srv/Shared/
mise run release     # Build all + deploy to /srv/Shared/ in one command
```

### Windows Targets (Direct Cargo Commands)

```bash
# Debug build for main application (glazewm.exe)
cargo xwin build -p wm --target x86_64-pc-windows-msvc

# Release build for main application
cargo xwin build --release -p wm --target x86_64-pc-windows-msvc

# Build all packages (wm, wm-cli, wm-watcher)
cargo xwin build --release --target x86_64-pc-windows-msvc
```

### Artifact Binary Locations

- Debug binary: `target/x86_64-pc-windows-msvc/debug/glazewm.exe`
- Release binary: `target/x86_64-pc-windows-msvc/release/glazewm.exe`
- CLI binary: `target/x86_64-pc-windows-msvc/release/glazewm-cli.exe`

### Deploying / Copying to Windows VM Share (`/srv/Shared`)

To copy compiled binaries to the shared directory accessible by the Windows VM:

```bash
# Copy all release binaries to VM shared folder
cp target/x86_64-pc-windows-msvc/release/glazewm*.exe /srv/Shared/

# Or copy specifically the main glazewm executable
cp target/x86_64-pc-windows-msvc/release/glazewm.exe /srv/Shared/
```

---

## Checking, Linting & Testing

### Fast Compilation & Type Checking

To quickly check for compiler errors and type correctness without linking:

```bash
# Fast check using x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu

# Fast check for specific crate (e.g. wm-common)
cargo check -p wm-common --target x86_64-pc-windows-gnu

# Check with MSVC target
cargo check --target x86_64-pc-windows-msvc
```

### Clippy & Code Quality Checks

GlazeWM enforces strict pedantic clippy lints (`#![warn(clippy::all, clippy::pedantic)]`):

```bash
# Run clippy on Windows GNU target
cargo clippy --target x86_64-pc-windows-gnu

# Run clippy with deny warnings
cargo clippy --target x86_64-pc-windows-gnu -- -D warnings
```

### Running Tests

```bash
# Run tests for platform-independent libraries
cargo test -p wm-common --target x86_64-pc-windows-gnu

# Check test compilation for wm
cargo test -p wm --target x86_64-pc-windows-gnu --no-run
```

---

## Coding Standards & Best Practices

1. **Error Handling**:
   - In `wm-platform`: Use `crate::Error` / `crate::Result` (`thiserror`).
   - In all other crates (`wm`, `wm-common`, `wm-cli`): Use `anyhow::Result` and add context with `.with_context(|| ...)`.
   - Never use `.unwrap()` or `.expect()` in production paths.

2. **Logging**:
   - Use `tracing` macros (`tracing::info!`, `tracing::debug!`, `tracing::warn!`, `tracing::error!`).

3. **Performance & Ownership**:
   - Prefer passing references (`&T`, `&str`, `&[T]`) over unnecessary `.clone()` calls.
   - For small `Copy` types (e.g. coordinates, dimensions), pass by value.
   - Avoid allocations and `.clone()` inside high-frequency loops (e.g. layout passes, window event handling).

4. **Numeric Casts**:
   - Annotate deliberate float/int casts with `#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]` and keep justifications clear.
   - Use `f64::from(i32_val)` instead of `i32_val as f64` for lossless conversions.
