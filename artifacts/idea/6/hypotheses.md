## H1: Win32 Foreground Lock Enforcement
- **Description**: Calling `LockSetForegroundWindow(LSFW_LOCK)` in the platform event loop and after WM focus operations prevents external Win32 processes from successfully promoting their HWND to foreground while user input is active.
- **Validation**: Background processes invoking `SetForegroundWindow` fail at the OS level or trigger taskbar flashing without raising native `EVENT_SYSTEM_FOREGROUND` preemption over the user's active window.

## H2: Off-Screen Focus Event Rejection
- **Description**: Inspecting `window.display_state()` and matching `workspace.monitor().displayed_workspace()` within `handle_window_focused` allows the WM to deterministically distinguish between intentional user focus changes and background preemption attempts.
- **Validation**: Incoming focus notifications from windows where `display_state != Shown` or where workspace is inactive do not alter `focused_container`, do not call `focus_workspace`, and trigger an immediate `queue_focus_change` restore.

## H3: Preservation of User Mouse-Focus Integrity
- **Description**: User mouse clicks on visible windows on currently displayed workspaces produce `EVENT_SYSTEM_FOREGROUND` with `display_state == Shown` and `is_on_displayed_workspace == true`, allowing smooth manual focus transitions without interference.
- **Validation**: Clicking on adjacent tiling or floating windows on the active workspace immediately shifts focus and updates focus visual effects without triggering rejection logic.

## H4: Canvas Viewport Stability
- **Description**: Suppressing `animate_pan_workspace` on rejected focus events guarantees that the horizontal canvas offset (`offset_x`) remains completely immobile when background applications emit events.
- **Validation**: Off-screen columns receiving events do not cause viewport panning animations or screen flickering.
