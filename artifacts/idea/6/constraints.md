## Logical Conflicts
1. **OS Authority vs. WM State**: Win32 natively treats foreground assignment as an OS-level global singleton. When an external app circumvents foreground lock, the WM must forcefully re-assert focus back to its internal focus tree without creating an infinite event ping-pong loop.
2. **Auto-Pan on Spawn vs. Signal Rejection**: Newly spawned active windows must auto-pan the viewport to keep new workflows visible, whereas existing off-screen windows receiving activity signals must be forbidden from triggering auto-pan.

## Edge Cases
1. **Multi-Monitor Displayed Workspace Sync**: In multi-monitor topologies, an inactive window may reside on a displayed workspace of a secondary monitor. If the user clicks on the secondary monitor, it must be recognized as valid manual focus across monitors.
2. **Application Startup Latency**: Applications that launch silently in the background and pop up windows seconds later after the user switched workspaces must not steal focus from the user's active workspace.
3. **Desktop Window Focus**: Clicks on the desktop background produce `WindowEvent::Focused` with unmanaged handles, which must not erroneously trigger rejection alerts.

## Performance Risks
1. **Event Channel Flooding**: Aggressive polling or focus loops from poorly behaved background processes could generate repeated `queue_focus_change` passes; mitigated by `is_focus_synced` checks and atomic pending sync passes.
2. **Win32 Hook Latency**: Synchronous checks in `handle_window_focused` must avoid blocking IPC or message loop threads during window hierarchy traversal.
