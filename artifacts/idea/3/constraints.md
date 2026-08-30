## Logical Conflicts & Synchronization
1. **Event Loop Non-Blocking**: Transition steps (`sleep(16ms)`) must not block the main WM event loop or delay incoming WinEvent / keyboard hotkey events.
2. **Rapid Keypress Spam**: When a user rapidly presses navigation hotkeys (e.g. holding `Alt+L`), the transition must immediately snap or update to the newest target without accumulating a backlog of steps.

## Edge Cases
1. **Window Destruction During Transition**: If a window on the strip closes while a transition is active, the next step must gracefully ignore invalid HWND handles without crashing `DeferWindowPos`.
2. **Workspace Switching**: If the user switches workspaces during a transition, the animation for the previous workspace must immediately terminate.

## Performance Risks
1. **DWM Message Flooding**: High-frequency stepping on 10+ windows could flood DWM if steps are spaced under 10ms. A balanced 16ms step interval (60Hz) with 4–6 total steps (~80–100ms) minimizes overhead while ensuring visual smoothness.
