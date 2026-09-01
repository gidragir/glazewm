## Definition
A strict user-only focus transition and focus-stealing prevention architecture for tiling window managers, enforcing deterministic input isolation by blocking unsolicited OS-level foreground activations, preventing background workspace switching, suppressing canvas viewport panning, and anchoring window/workspace focus transitions exclusively to explicit user interactions.

## Value Proposition
Prevents background processes, toast notifications, and unmanaged system signals from hijacking keyboard/mouse focus, disrupting active user typing workflows, or causing disorienting workspace jumps and horizontal canvas animations.

## Core Mechanics
1. **Foreground Lock Enforcement**: Platform layer initializes and asserts Win32 `LockSetForegroundWindow(LSFW_LOCK)`, instructing the OS kernel to reject background application requests to invoke `SetForegroundWindow`.
2. **Focus Event Validation**: Incoming OS-level focus events (`EVENT_SYSTEM_FOREGROUND` / `WindowEvent::Focused`) undergo legitimacy filtering against the window manager's container tree:
   - If the window matches the WM's currently targeted focus container (`focused_container == window`), mark focus as synchronized (`is_focus_synced = true`) and reorder workspace.
   - If the window resides on a hidden/inactive workspace (`DisplayState != Shown` or workspace not active on any monitor), reject the focus transition, suppress workspace switching (`focus_workspace`), suppress canvas panning (`animate_pan_workspace`), and immediately re-assert OS focus back to the user's active container (`queue_focus_change`).
   - If the window is visible on the active workspace and received direct user interaction (e.g. mouse click), update `set_focused_descendant` and synchronize effects.
3. **Spawn-Time Focus Isolation**: New window registration (`manage_window`) assigns focus and triggers viewport panning only if the window belongs to a currently displayed workspace; windows assigned to hidden workspaces by rules are detached from immediate focus queues, retaining active container focus.
