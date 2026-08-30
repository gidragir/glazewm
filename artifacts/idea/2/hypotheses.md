## H1: Container Width Field Persistence
- **Description**: Adding a specific `assigned_width` field to `TilingWindow` or `SplitContainer` will allow the layout algorithm to use this value instead of dynamically recalculating equal divisions for all columns, thus preserving state when panning.
- **Validation Condition**: Resize a window, pan it entirely out of the viewport, pan it back in. The width should exactly match the resized dimensions, and neighboring windows should not have shifted coordinates unnecessarily.

## H2: Configurable Command Presets
- **Description**: Implementing a `ResizeColumnPreset` command that takes an array of percentages (e.g., `[25, 33, 50, 75]`) and cycles through them based on the current window's width.
- **Validation Condition**: Pressing `Alt+Shift+R` repeatedly successfully snaps the focused window width to 25%, 33%, 50%, 75%, and back to 25%.

## H3: Cyclic State Resolution
- **Description**: When cycling presets, if the current window width does not exactly match any preset (due to manual resize), the algorithm will snap to the next largest preset in the configured list.
- **Validation Condition**: Manually resize a window to 40% width. Triggering the preset cycle (with config `[25, 33, 50, 75]`) should immediately snap the window to 50%.
