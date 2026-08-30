# Niri-like Infinite Horizontal Scrolling Canvas

This plan outlines the architecture and implementation steps to introduce an infinite horizontal scrolling tiling layout (inspired by Niri) to GlazeWM.

## Goal Description

The objective is to replace or supplement the traditional bounded tiling grid with an infinite horizontal column-based canvas per workspace. Windows will be organized linearly along an unconstrained X-axis. When an off-screen or partially visible window receives focus (via hotkeys or hover), the workspace's viewport automatically pans horizontally to pull the window entirely into the visible screen bounds. On monitor disconnect, unbound workspaces are migrated to active monitors with the fewest workspaces.

## User Review Required

> [!WARNING]
> **Layout Strategy**: Should this horizontal strip behavior *replace* the existing grid layout entirely, or should it be an alternative layout mode configurable via `config.yaml`? The plan assumes it will be a new layout mode or replace the core tiling algorithm, but confirmation is needed.
>
> **Focus Feedback Loop**: Panning the viewport can move windows underneath the physical mouse cursor. If "focus follows cursor" is enabled, this could trigger an infinite loop of focus changes and panning. How should we mitigate this? (e.g., disabling hover focus during active pan, or ignoring hover events if the mouse hasn't physically moved).
> 
> **Cloaked Windows**: Windows 11 UWP apps often have "cloaked" background windows. We need to ensure our WinEvent hooks properly filter these out using `DWMWA_CLOAKED` checks.
>
> **Performance**: The plan relies on `DeferWindowPos` to move all windows in the workspace synchronously. Depending on the number of apps, this might cause DWM stuttering. Are we okay with this for the MVP?

## Proposed Changes

### 1. `wm-platform` (Platform API)
To achieve acceptable repaint performance and minimize DWM stuttering when panning the entire viewport, we must batch window positioning operations.
- **[MODIFY]** `packages/wm-platform/src/lib.rs` and `packages/wm-platform/src/platform_impl/windows/native_window.rs`
  - Introduce a batch positioning API utilizing `DeferWindowPos`.
  - Add a function like `pub fn apply_window_positions(positions: &[(NativeWindow, Rect)]) -> crate::Result<()>` to the `Dispatcher` or as a standalone Windows platform utility.
- **[MODIFY]** `packages/wm-platform/src/platform_impl/windows/window_listener.rs`
  - Ensure window event filtering checks for `DWMWA_CLOAKED` to avoid adding invisible UWP background apps to the horizontal strip.

### 2. `wm-common` (Domain Models & Configuration)
State definitions need to reflect virtual coordinates and panning offsets.
- **[MODIFY]** `packages/wm-common/src/parsed_config.rs`
  - Potentially add new configuration fields for horizontal padding, column widths, or animation speeds (if applicable later).
- **[MODIFY]** `packages/wm-common/src/dtos/workspace_dto.rs`
  - Add `offset_x: f64` to `WorkspaceDto` to communicate the current pan offset to external IPC clients (like Zebar) so they can reflect scroll state.
- **[MODIFY]** `packages/wm-common/src/app_command.rs`
  - Add new `InvokeCommand` variants for manual panning: `PanViewportLeft`, `PanViewportRight`.

### 3. `wm` (Core Application & Layout Engine)
The core tree models and layout algorithms must be updated to use virtual X coordinates.
- **[MODIFY]** `packages/wm/src/models/workspace.rs`
  - Add an `offset_x: f64` property to the `Workspace` state to track the viewport's physical pan offset.
- **[MODIFY]** `packages/wm/src/wm_state.rs` & `packages/wm/src/wm.rs`
  - Update the workspace layout algorithm. Instead of recursively subdividing physical monitor bounds, arrange `TilingWindow` nodes sequentially in an infinite horizontal line.
  - Physical rect calculation for window $i$: $X_{\text{screen}} = (\sum_{j=0}^{i-1} W_j + \text{gaps}) - \text{offset\_x}$.
- **[MODIFY]** `packages/wm/src/events/focus_changed.rs` (or equivalent focus handlers)
  - Whenever a window receives focus, check its logical frame. If $X_{\text{screen}} < 0$ or $X_{\text{screen}} + W > W_{\text{monitor}}$, calculate the required $\Delta X$ to bring it into view and update `offset_x`.
  - Enqueue a full workspace redraw.
- **[MODIFY]** `packages/wm/src/pending_sync.rs`
  - When committing layout changes, group window frame updates and call the new `apply_window_positions` batch function from `wm-platform` to perform atomic updates via `DeferWindowPos`.
- **[MODIFY]** `packages/wm/src/events/monitor_updated.rs` (or equivalent)
  - Implement workspace migration on monitor disconnect. Identify unmapped workspaces, find the active monitor with the lowest workspace count, and reassign the disconnected workspaces to it.

## Verification Plan

### Automated Tests
- Unit tests for the layout algorithm to verify virtual coordinate calculation and proper bounds checking.
- Unit tests for focus offset adjustments (verifying `offset_x` calculation accurately brings a window into the monitor bounds).
- Unit tests for workspace migration logic ensuring workspaces distribute correctly on monitor removal.

### Manual Verification
- Compile and run `glazewm.exe`.
- Open 5-6 windows and confirm they tile horizontally out of the screen bounds without compressing.
- Focus off-screen windows via keyboard shortcuts and verify the viewport pans instantly.
- Disconnect a monitor and verify its workspaces appear on the remaining monitor.
- Test UWP apps (e.g., Windows Settings, Calculator) to verify cloaked window handles do not disrupt the timeline.
