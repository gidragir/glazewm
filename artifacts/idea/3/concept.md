## Definition
Enhancement of the infinite horizontal scrolling layout (iteration 3) focusing on smooth visual transitions (micro-animations) during viewport panning and focus changes, implemented with minimal architectural complexity.

## Value Proposition
Removes the jarring, disorienting instant snap when navigating left/right or spawning windows. Provides immediate spatial awareness of motion direction (slide-in / slide-out) across the horizontal canvas without introducing a heavy animation framework.

## Core Mechanics
1. **Lightweight Step-Interpolation (Micro-Slide Transition)**:
   - When viewport offset $\Delta X$ changes (via `auto_pan_viewport` or manual pan commands), smoothly transition from `current_offset_x` to `target_offset_x` over 4–6 fast steps (e.g. 15–20ms per step, ~80–120ms total duration).
   - Batch-apply window positions per step using existing `DeferWindowPos` (`SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER`).
2. **Animation Cancellation / Coalescing**:
   - If a new navigation command arrives while a transition is active, update the target offset immediately to avoid queue lag.
3. **Backlog: Strip Minimap / Canvas Indicator for Zebar**:
   - Expose canvas viewport bounds and column positions via IPC (`WorkspaceDto`) so Zebar/status bars can render a real-time horizontal minimap showing viewport location.
