## Standard Patterns
1. **Niri's Column Widths**: Niri treats columns as the primary horizontal building block. Users can set column widths as percentages of the monitor width. Niri provides built-in actions to set widths or cycle through presets.
2. **PaperWM (GNOME)**: Similar infinite panning. Allows dynamic resizing but also snaps to specific proportions to maintain a clean reading experience.

## Alternative Approaches
1. **Absolute vs Relative**: Storing absolute pixels can lead to issues across DPIs. Storing relative percentages (e.g., `0.33` for 33%) is more robust for a responsive canvas.
2. **Cyclic vs Explicit Commands**: Instead of one command that cycles, having distinct commands for each preset (e.g., `set-width 50`, `set-width 25`). A cyclic command reduces hotkey bloat, which is preferable.
