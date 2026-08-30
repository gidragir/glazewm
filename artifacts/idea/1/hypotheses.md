## H1: Dynamic Viewport Shifting Performance via DeferWindowPos
- **Description**: Calling `DeferWindowPos` batch updates for all managed windows in a horizontal strip when panning viewports provides acceptable repaint performance (<30ms layout pass) across mixed Win32/Electron applications.
- **Validation Condition**: Benchmark `DeferWindowPos` latency with 10 active windows (5 off-screen, 5 visible) under heavy desktop load. Measure frame time drops and repainting artifacts.

## H2: WinEventHook Filtering Reliability
- **Description**: Win32 event hooks (`EVENT_OBJECT_CREATE`, `EVENT_SYSTEM_FOREGROUND`, `EVENT_OBJECT_DESTROY`) combined with style bitchecks (`WS_EX_TOPMOST`, `WS_POPUP`, `WS_THICKFRAME`) reliably discriminate between main application windows and transient tooltips/menus/dialogs.
- **Validation Condition**: Audit window creation stream across 50 standard Windows applications (VS Code, Chrome, Discord, Steam, Explorer) and confirm zero false-positive capture of transient popups into the layout strip.

## H3: Virtual Workspace Coordinates Abstraction
- **Description**: Maintaining a virtual $X$-coordinate space (e.g., $[-\infty, +\infty]$) decoupled from physical screen coordinates $[0, W_{\text{monitor}}]$ allows linear $O(1)$ viewport panning calculations without needing DWM desktop composition hooks.
- **Validation Condition**: Verify that moving windows off physical monitor bounds ($X < 0$ or $X > W_{\text{monitor}}$) does not trigger OS-level window minimization or automatic docking behavior by DWM.

## H4: UAC Privilege Elevation Boundary
- **Description**: Running the TWM binary elevated (`RequireAdministrator` manifest) or using `ChangeWindowMessageFilterEx` is strictly mandatory to intercept and reposition windows running with administrative privileges (UIPI restrictions).
- **Validation Condition**: Attempt `SetWindowPos` and event hooking on elevated Task Manager or PowerShell instances from a standard user process vs. an elevated process.
