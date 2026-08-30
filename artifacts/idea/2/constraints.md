## Logical Conflicts
1. **Minimum/Maximum Size Constraints**: Presets (e.g., 25%) might calculate to a width smaller than a window's `MINMAXINFO` minimum width. The layout engine must gracefully handle constraints without breaking the canvas alignment.
2. **Viewport Overflow**: Setting a column width larger than the monitor's physical width (e.g., >100% or absolute pixels > monitor width) requires clamping or ensuring panning can still reach all parts of the window.

## Edge Cases
1. **Preset Cycling State**: If a user manually resizes a window, where does the preset cycle resume? The cycle state needs to either reset or find the closest preset.
2. **Multi-Monitor DPI**: Percentage-based presets scale naturally, but if absolute pixel presets are introduced, moving the window to a monitor with a different DPI scale factor could result in inappropriate physical sizing.

## Performance Risks
1. **Redundant Layout Passes**: Cycling through presets rapidly (holding the shortcut) might trigger continuous layout recalculations and `DeferWindowPos` updates. Throttling or debouncing might be necessary if DWM struggles.
