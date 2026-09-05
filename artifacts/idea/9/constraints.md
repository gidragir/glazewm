## Logical Conflicts
- **Fullscreen vs Tiling Sibling Navigation**: In traditional BSP window managers (i3/bspwm), fullscreen windows exclusively occupy the display tree, suppressing directional navigation. In Niri, fullscreen is an attribute of a column/window along the horizontal strip; moving focus horizontally scrolls the viewport while retaining the fullscreen attribute on the originating node.
- **Offscreen Clamping vs Bleed Prevention**: Truncating column width to monitor bounds avoids cross-monitor bleeding in multi-monitor layouts but causes application layout collapse (`WM_SIZE`). Safely parking offscreen windows at `SAFE_PARK_X, SAFE_PARK_Y` solves both without modifying window dimensions.

## Edge Cases
- **Multi-Stage Window Creation (1C Enterprise / Teams / Splash Screens)**: Processes spawning splash screens followed by main windows emit multiple asynchronous `EVENT_SYSTEM_FOREGROUND` events. Focus guarding must not classify initial user-intended launches as unsolicited background signals.
- **Unmanaged Dialogs / Menus**: Context menus, dropdowns, and modal dialogs owned by a managed application should not be misidentified as rogue background focus thieves.
- **Single-Column Workspaces in Fullscreen**: Toggling fullscreen in a workspace with a single window must permit subsequent windows to spawn adjacent columns without breaking column hierarchy.

## Performance Risks
- **Thread Input Attachment Contention**: Indiscriminate or unreleased `AttachThreadInput` calls can cause thread message queue deadlocks if an attached process becomes unresponsive. A RAII guard pattern must guarantee deterministic detachment.
- **High-Frequency Viewport Sync Allocations**: Calculating displacement across large numbers of canvas columns must reuse layout buffers and avoid repeated heap reallocations.
