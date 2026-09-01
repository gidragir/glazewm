# Iteration 6 Implementation Plan: Strict User-Only Focus & Focus Stealing Prevention

## Overview
Eliminate focus stealing, unwanted workspace switching, and unsolicited horizontal canvas panning caused by background applications and system signals in GlazeWM, achieving full parity with standard Windows foreground lock and Niri Wayland compositor behavior.

## Changes by Component

### 1. `packages/wm-platform`
- **`native_window.rs`**: Import `LockSetForegroundWindow` and `LSFW_LOCK`; call after setting foreground in `NativeWindow::focus()`.
- **`event_loop.rs`**: Import `LockSetForegroundWindow` and `LSFW_LOCK`; invoke in `EventLoop::run()` on startup.

### 2. `packages/wm`
- **`events/handle_window_focused.rs`**:
  - Remove auto-switching of hidden workspaces (`focus_workspace`).
  - Validate incoming focus events against `window.display_state()` and `displayed_workspace()`.
  - Reject unsolicited off-screen/background focus events and immediately restore OS focus to `focused_container` via `state.pending_sync.queue_focus_change()`.
  - Preserve valid manual focus changes on the active workspace (e.g. mouse clicks on visible windows).
- **`commands/window/manage_window.rs`**:
  - Restrict focus promotion to windows on currently displayed workspaces.
  - If a newly managed window is routed to a hidden workspace, restore focus to the previous active container.

### 3. Workspace Root
- **`mise.toml`**:
  - Add unified verification task `[tasks.verify]`.
  - Configure `[tasks.test]` with MSVC cross-compilation and Wine runner.

## Verification
- Run `mise run verify` (type check, strict pedantic Clippy, unit tests).
