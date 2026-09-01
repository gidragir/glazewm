## Definition
Multi-monitor viewport containment and canvas isolation architecture for infinite horizontal scrolling workspaces, preventing off-canvas window bleeding into adjacent monitor bounding boxes while retaining taskbar presence, Alt+Tab accessibility, and decoupling intra-workspace directional focus navigation from cross-monitor transitions.

## Value Proposition
Eliminates cross-monitor visual pollution, accidental window bleeding, and unintended focus jumps across physical displays on multi-monitor setups running infinite horizontal canvas tiling, ensuring each monitor operates as a strictly isolated viewport.

## Core Mechanics
1. **Viewport Intersection & Safe Coordinate Parking**:
   - For every window container residing on a monitor's active workspace, compute its candidate virtual desktop position: `rect = (parent_rect.x - offset_x + col_x, parent_rect.y, width, height)`.
   - Test spatial intersection against the parent monitor's physical bounding box (`monitor.working_area`).
   - If the window rect falls completely outside the parent monitor's working area, re-route its physical placement to a designated safe offscreen coordinate zone (e.g. outside the union of all active monitor bounding boxes in Windows virtual desktop space) or apply non-destructive containment attributes to prevent rendering inside adjacent monitor rectangles while preserving `WS_VISIBLE`, taskbar entry, and Alt+Tab enumeration.
   - If the window rect partially intersects the monitor edge, clamp or clip the render frame (`SetWindowRgn` or partial boundary clamping) to prevent overlapping neighboring monitor coordinates.

2. **Decoupled Intra-Workspace vs Inter-Monitor Focus Navigation**:
   - `focus_in_direction` (`Left` / `Right` / `Up` / `Down`) is strictly bound to the active workspace container hierarchy and infinite canvas columns.
   - Reaching the leftmost or rightmost column on the infinite canvas clamps or scrolls the canvas without invoking cross-monitor fallback (`workspace_focus_target`).
   - Cross-monitor focus switching is strictly delegated to dedicated commands (`focus-monitor-in-direction` / `focus-monitor`).

3. **Explicit Cross-Monitor Migration**:
   - Windows move across monitor boundaries exclusively through explicit drag-and-drop mouse events or dedicated IPC commands (`move-window-to-monitor` / `move-window-to-workspace`).
