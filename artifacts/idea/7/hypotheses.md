## H1: Global Virtual Desktop Safe Parking
- **Assumption**: Moving an off-viewport window to a global safe coordinate zone outside all physical monitor bounding boxes (`virtual_screen_union.right + 10000`, `virtual_screen_union.bottom + 10000`) while preserving its native visibility styles keeps the window accessible to Alt+Tab and the Windows taskbar while completely preventing visual rendering on any monitor.
- **Validation Condition**: On a dual-monitor setup, an offscreen window parked at safe virtual coordinates retains a clickable taskbar thumbnail and Alt+Tab entry, but renders 0 pixels across both physical screens.

## H2: Viewport Intersection Culling in Platform Sync
- **Assumption**: Evaluating spatial intersection between `window.to_rect()` and `monitor.working_area` during `platform_sync` allows determining exact visibility state without mutating the container tree model.
- **Validation Condition**: `reposition_window` accurately distinguishes fully visible, partially visible, and fully offscreen windows on a per-monitor basis.

## H3: Focus Direction Containment Isolation
- **Assumption**: Eliminating fallback to `workspace_focus_target` in `focus_in_direction` for infinite canvas workspaces isolates directional navigation to the current monitor's canvas columns and prevents unintended focus switching across displays.
- **Validation Condition**: Repeatedly triggering `focus right` from the rightmost column pans/clamps within the current workspace and never shifts OS focus to the adjacent monitor.

## H4: Dedicated Inter-Monitor Focus Command
- **Assumption**: Exposing `focus-monitor-in-direction` and `move-window-to-monitor-in-direction` provides explicit multi-monitor traversal without polluting intra-workspace tiling keybindings.
- **Validation Condition**: User can navigate between monitors using discrete monitor focus bindings without disrupting infinite canvas navigation.
