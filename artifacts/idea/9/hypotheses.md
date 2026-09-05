## H1: Column Ancestor Resolution for Fullscreen Windows
- **Assumption**: Querying `find_column_ancestor` on a fullscreen window that retains its parent column (or resolves it via `insertion_target`) returns `Some(TilingContainer)` without runtime errors or popups.
- **Validation**: Invoke `cycle_column_preset` on a fullscreen container; verify operation returns `Ok(())` and preserves column width presets.

## H2: Single-Actuation Multi-Monitor Focus Switching
- **Assumption**: Attaching thread inputs via `AttachThreadInput` and calling `LockSetForegroundWindow(LSFW_UNLOCK)` + `AllowSetForegroundWindow(ASFW_ANY)` prior to `SetForegroundWindow` activates windows on other monitors on the first hotkey press.
- **Validation**: Unit/integration tests simulating cross-thread focus verify `SetForegroundWindow` succeeds immediately without triggering Windows taskbar flashing or requiring a secondary hotkey event.

## H3: Zero-Width Distortion Coordinate Displacement
- **Assumption**: Removing edge-clamping width truncation in `calculate_physical_rect` and parking displaced columns at `SAFE_PARK_X, SAFE_PARK_Y` allows columns to retain 100% logical width without generating `WM_SIZE` layout recalculation cascades in hosted applications.
- **Validation**: Calculate physical rect for a 1000px column displaced by `offset_x = 400`; verify resulting rect width equals 1000px, avoiding the 600px clamped squeeze.

## H4: Bidirectional Niri-Style Directional Focus Traversal
- **Assumption**: Expanding `focus_in_direction` to evaluate horizontal neighbors of fullscreen nodes enables seamless traversal between fullscreen and tiling stacks while maintaining normal workspace z-order.
- **Validation**: Execute `focus_in_direction(Right)` from a fullscreen column; verify focus transfers to the rightward tiling column, updates `focused_container`, and brings target window to front.

## H5: Deterministic Anti-Steal Focus Rejection
- **Assumption**: Validating incoming `EVENT_SYSTEM_FOREGROUND` against `state.pending_sync.needs_focus_update()` and physical cursor containment rejects unsolicited background signals (e.g. 1C, Teams, Discord) and redirects OS focus back to the user's active window.
- **Validation**: Trigger `handle_window_focused` from an unfocused window while cursor is outside its geometry and no WM command is pending; assert focus remains with active window and OS focus realignment is queued.

## H6: Structural Column Auto-Wrapping in Ingestion Pipeline
- **Assumption**: Routing windows through `move_window_to_workspace` or `set-tiling` on an empty horizontal workspace automatically initializes an intermediate vertical `SplitContainer` column, preventing direct leaf attachment to workspace roots.
- **Validation**: Move window to an empty workspace via `['move --workspace 3', 'set-tiling']`; verify workspace hierarchy contains `Workspace -> SplitContainer (Vertical) -> TilingWindow`.
