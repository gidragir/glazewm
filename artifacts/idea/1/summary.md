## Initial Proposition
Development of a Niri-like tiling window manager for Windows using Rust and Windows API, focusing on an infinite horizontal canvas and per-monitor independent workspaces.

## Clarifications
1. **Viewport Shift Logic**: Focusing an off-screen or partially visible window (via hover or hotkey) triggers horizontal canvas panning to pull the window entirely into the visible screen bounds without exceeding screen margins. Zooming out/scaling is deferred post-MVP.
2. **Window Management Boundaries**: Automatic capture of all windows by default, with configuration file overrides for window rules (ignore lists, floating mode forcing).
3. **Monitor Hot-Plugging & Workspace Migration**: Unbound workspaces from disconnected monitors relocate to active monitors with the fewest workspaces (prioritizing rightward neighbors). Workspaces themselves cannot be manually moved; individual windows moved across monitors become new columns in the target workspace.
4. **Unmanageable Windows**: Non-standard windows, problematic WinAPI structures, or windows resisting geometry mutations must be gracefully ignored/exempted from tiling.
5. **MVP Criteria**: Core functionality over rendering perfection. Hotkey/hover viewport panning on a single monitor workspace without requiring smooth composition animations (acknowledging Electron/WinAPI repainting artifacts).

## Perceived Pitfalls
1. Electron/Chromium-based application stuttering or frame dropping during frequent `SetWindowPos` geometry updates.
2. Non-standard WinAPI window styles (tooltips, popups, elevated UAC windows) corrupting the horizontal canvas ordering.
3. Asynchronous mouse event race conditions between hover detection and viewport shift calculations.
