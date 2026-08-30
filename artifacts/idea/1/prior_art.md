## Standard Patterns
1. **Virtual Canvas Coordinate Mapping**: Decouple logical window position $(X_{\text{virtual}}, Y_{\text{virtual}})$ from physical viewport screen position $(X_{\text{screen}}, Y_{\text{screen}})$. Panning alters global Offset $O_x$, computing $X_{\text{screen}} = X_{\text{virtual}} - O_x$.
2. **Reactor / Event-Driven State Machine**: WinEvent Hooks push window lifecycle events into an asynchronous mpsc (multi-producer, single-consumer) Rust channel (`tokio::sync::mpsc` or `crossbeam-channel`). A central event-loop updates state and dispatches position commands.
3. **Command-Query Responsibility Segregation (CQRS) for Geometry**: Maintain layout state in memory (Command phase) and asynchronously sync to WinAPI `SetWindowPos` (Query/Apply phase) to prevent GUI thread locks.

## Alternative Approaches
1. **Wayland Compositor Native (Niri Baseline)**: Niri operates as a Wayland compositor directly controlling rendering passes.
2. **Windows Desktop Window Manager (DWM) Hooking / DirectComposition**: Intercept DWM composition targets via undocumented APIs or DirectComposition layers to animate window movement smoothly on GPU. (High complexity, fragile across Windows builds).
3. **Standard WinAPI Rect Adjustment (Selected Approach)**: Pure user-land window positioning via standard WinAPI (`SetWindowPos` / `DeferWindowPos`), sacrificing compositor-level smooth animations for maximum stability, simplicity, and low complexity.
