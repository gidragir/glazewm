# System Constraints: Iteration 8

## Logical Conflicts
- **Animation Mid-Flight Redirection vs. Frame Dropping**: A new focus target arriving mid-animation must not produce conflicting intermediate coordinates. The animation driver must dynamically recalculate the trajectory from the current interpolated `offset_x` rather than stacking competing timers.
- **Async Task Lifetime vs. Process Termination**: Background frame tasks spawned in Tokio must respect cancellation tokens (`CancellationToken`) so that config reloads or WM termination do not leave orphaned timers posting to destroyed `Dispatcher` instances.

## Edge Cases
- **Zero-Duration Animations**: If `animation_duration_ms == 0` or `animation_enabled == false`, bypass timer scheduling entirely and apply target `offset_x` in a single atomic pass.
- **Rapid Alternating Focus (Ping-Pong)**: Toggling focus between adjacent windows on opposite screen edges faster than animation duration must not induce resonant camera oscillations.
- **IPC Client Disconnection**: Sudden drop of client WebSocket connections during high-throughput event broadcast must not block the main daemon or leak receiver handles.

## Performance Risks
- **Channel Flooding**: Excessive frame ticks dispatched faster than the monitor refresh rate could overwhelm the Win32 message queue. Frame rate must be clamped to screen refresh interval ($\ge 16$ms / 60Hz or dynamically matching display frequency).
- **Pointer/Handle Staleness**: Windows closed mid-animation must be validated (`is_valid()`) before applying intermediate `DeferWindowPos` updates.
