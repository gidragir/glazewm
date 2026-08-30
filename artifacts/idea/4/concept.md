## Definition
Iteration 4 focuses on bug fixes and lifecycle refinement for the infinite horizontal canvas: eliminating sibling size inflation on window close and suppressing the centered-window flash on new window spawn via pre-show placement/cloaking.

## Value Proposition
Restores full structural stability and visual elegance to the infinite canvas. Windows off-screen no longer corrupt their dimensions when other windows close, and newly opened applications (e.g. Windows Explorer, Chrome) smoothly enter the canvas without flashing in the center of the monitor first.

## Core Mechanics
1. **Isolated Detach Logic (`detach_container.rs`)**:
   - If the container's parent is a horizontal `Workspace`, bypass sibling proportion inflation (`size_delta` redistribution) entirely. Existing columns retain their exact `tiling_size`.
2. **Pre-Show Placement & Cloaking on Window Spawn (`manage_window.rs` / `window_listener.rs`)**:
   - Cloak new windows before initial layout positioning to suppress default OS-centered rendering.
   - Compute initial strip coordinates $(X_{\text{canvas}}, Y)$ in the background, set the frame, and then uncloak the window.
3. **Coordinated Viewport Auto-Pan**:
   - Seamlessly pan viewport to the newly spawned window without visual flicker.
