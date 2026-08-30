## Required Codebase Context
1. **Viewport Auto-Panning**:
   - [platform_sync.rs](file:///data/projects/glazewm/packages/wm/src/commands/general/platform_sync.rs): `auto_pan_viewport` function calculates `new_offset`.
2. **Manual Panning Commands**:
   - [wm.rs](file:///data/projects/glazewm/packages/wm/src/wm.rs): `InvokeCommand::PanViewportLeft` and `PanViewportRight`.
3. **Platform Dispatcher**:
   - [native_window.rs](file:///data/projects/glazewm/packages/wm-platform/src/platform_impl/windows/native_window.rs): `apply_window_positions` batch execution using `DeferWindowPos`.
4. **Configuration & Backlog**:
   - [parsed_config.rs](file:///data/projects/glazewm/packages/wm-common/src/parsed_config.rs): Optional animation settings (`animation_duration_ms`).
   - [workspace_dto.rs](file:///data/projects/glazewm/packages/wm-common/src/dtos/workspace_dto.rs): `offset_x` field for Zebar minimap integration.
