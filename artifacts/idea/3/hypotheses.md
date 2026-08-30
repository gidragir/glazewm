## H1: Lightweight Async Step-Transition (Micro-Animation)
- **Description**: Stepping `offset_x` across 4–6 discrete steps spaced by ~16ms (total duration ~80–100ms) provides a clear perception of motion direction (slide-in/slide-out) with minimal code complexity and zero physics engine overhead.
- **Validation Condition**: Navigating between columns off-screen visibly slides windows horizontally into view rather than teleporting instantly.

## H2: Event Loop Responsiveness Under Rapid Navigation
- **Description**: Storing the active target offset and updating it on consecutive navigation commands prevents motion stutter and eliminates latency during rapid hotkey repeat.
- **Validation Condition**: Holding navigation keys (e.g. `Alt+L` repeated 10 times) smoothly traverses the canvas without lag or frozen input.

## H3: Backlog - External Minimap Integration (Zebar)
- **Description**: Exposing current `offset_x`, workspace width, and window bounds over IPC allows status bars like Zebar to render a real-time minimap widget.
- **Validation Condition**: Verify that `WorkspaceDto` emits updated `offset_x` on every transition completion.
