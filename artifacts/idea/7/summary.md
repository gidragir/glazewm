## Initial Proposition
On multi-monitor setups with infinite horizontal canvas tiling, windows that scroll outside the visible bounds of Monitor 1's viewport physically appear on Monitor 2 because Windows coordinates form a unified virtual screen. Furthermore, moving focus via directional hotkeys across columns causes unwanted cross-monitor window jumps.

## Clarifications
1. **Single vs Multi-Monitor Parity**: On a single monitor, off-screen windows remain uncloaked/visible to taskbar and Alt+Tab; multi-monitor behavior must preserve this exact parity per monitor without bleeding onto adjacent screens.
2. **Focus Navigation Isolation**: Navigating with directional focus (`focus left/right`) must navigate columns within the active workspace and never automatically transition to an adjacent monitor upon reaching canvas boundaries.
3. **Dedicated Monitor Navigation**: Cross-monitor focus shifts must occur solely through dedicated cross-monitor commands or explicit user actions.
4. **Window Migration Boundary**: A window must never automatically migrate or render across monitors during focus scrolling; window transfer across monitors is restricted to explicit mouse drag-and-drop or explicit move commands.
5. **Animation Priority**: Panning animation is secondary; deterministic coordinate containment and strict multi-monitor boundary isolation are primary.

## Perceived Pitfalls
1. **Virtual Desktop Overlap**: Windows OS positions HWNDs in global desktop space; naive offset calculations (`parent_rect.x - offset_x`) inherently place overflowing windows into neighboring monitor rects.
2. **Taskbar & Alt+Tab Eviction**: If offscreen containment incorrectly applies `ShowWindow(SW_HIDE)` or DWM cloaking, the OS removes window thumbnails from taskbar and Alt+Tab, violating single-monitor parity.
3. **Partial Column Bleeding**: Columns partially visible at the viewport edge may project partial pixel strips onto adjacent monitors if not clipped or bound.
