## Required Codebase Context
1. **Coordinate Calculation & Resizable Trait**:
   - `packages/wm/src/traits/position_getters.rs`: Macro `impl_position_getters_as_resizable` calculating `parent_rect.x() - offset_x`. Needs inspection for virtual vs physical coordinate translation and monitor bounds intersection.
2. **Platform Sync & Window Repositioning**:
   - `packages/wm/src/commands/general/platform_sync.rs`: `reposition_window` and `platform_sync` handling `SetWindowPos`, `DisplayState`, and window visibility. Needs logic for safe offscreen coordinate parking and monitor clipping.
3. **Directional Focus Routing**:
   - `packages/wm/src/commands/container/focus_in_direction.rs`: `focus_in_direction` and `workspace_focus_target`. Needs decoupling to prevent automatic cross-monitor workspace transition on canvas boundary.
4. **Platform Display Geometry Abstraction**:
   - `packages/wm-platform/src/native_window.rs` and `packages/wm-platform/src/platform_impl/windows/`: Working area calculations, monitor bounding union (`GetSystemMetrics(SM_XVIRTUALSCREEN)`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, `SM_CYVIRTUALSCREEN`), and `SetWindowRgn` or `SetWindowPos` flags.
5. **Configuration & Commands Schema**:
   - `packages/wm-common/src/commands.rs` & `packages/wm-common/src/parsed_config.rs`: Registration of explicit `focus-monitor-in-direction` and multi-monitor movement commands.
