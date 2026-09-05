## Required Codebase Context

- [ ] **`packages/wm-platform/src/platform_impl/windows/native_window.rs`**:
  - `NativeWindow::focus`: Integrate RAII `AttachThreadInput`, `AllowSetForegroundWindow`, and `LockSetForegroundWindow(LSFW_UNLOCK)`.
- [ ] **`packages/wm-platform/src/platform_impl/windows/window_listener.rs`**:
  - `window_event_proc`: Ensure `EVENT_SYSTEM_FOREGROUND` passes through for managed windows regardless of transient cloaking state.
- [ ] **`packages/wm/src/commands/window/cycle_column_preset.rs`**:
  - `find_column_ancestor`: Support column resolution for fullscreen windows; provide graceful fallback without error popups.
- [ ] **`packages/wm/src/commands/general/platform_sync.rs`**:
  - `calculate_physical_rect`: Remove width-squeezing boundary clamping; preserve 100% window width during displacement.
  - `windows_to_bring_to_front`: Add `WindowState::Fullscreen` to normal workspace z-order tracking.
- [ ] **`packages/wm/src/commands/container/focus_in_direction.rs`**:
  - `focus_in_direction`: Support bidirectional navigation between `WindowState::Fullscreen` and adjacent tiling columns.
- [ ] **`packages/wm/src/events/handle_window_focused.rs`**:
  - Anti-steal validation: Check cursor position and `needs_focus_update` to reject unsolicited background signals and multi-stage app popups.
- [ ] **`packages/wm/src/commands/window/manage_window.rs`**:
  - Remove unconditional `set_cloaked(true)`.
- [ ] **`packages/wm/src/commands/window/move_window_to_workspace.rs` & `update_window_state.rs`**:
  - Auto-wrap tiling windows into vertical `SplitContainer` columns when moving to an empty workspace.
