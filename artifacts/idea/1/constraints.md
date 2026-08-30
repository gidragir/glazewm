## Logical Conflicts
1. **Focus Feedback Loop**: Panning the viewport moves windows under the physical mouse cursor. If mouse hover triggers focus, and focus triggers panning, an infinite viewport oscillation loop occurs if cursor lands on a neighboring partially visible window post-shift.
2. **Min/Max Size Violations**: Horizontally tiling windows with fixed `MINMAXINFO` dimensions that exceed screen width creates viewport overflow clipping where no valid panning position can display the full window.

## Edge Cases
1. **DPI Scaling Boundaries**: Moving a workspace/window across monitors with different DPI scale factors (e.g., 100% to 175%) breaks virtual coordinate strip alignment unless geometry recalculation accounts for `WM_DPICHANGED` per window.
2. **Cloaked Windows**: Modern UWP/WinUI3 apps use cloaking (`DWMWA_CLOAKED`). Cloaked background windows emit `EVENT_OBJECT_CREATE` but are invisible, corrupting virtual strip spacing if managed as visible columns.
3. **Display Disconnect Race Condition**: Rapid monitor hot-unplugging during active viewport shift creates dangling HWND references and invalid monitor handle (`HMONITOR`) lookups.

## Performance Risks
1. **Synchronous WinAPI Lockups**: Calling `GetWindowLongPtrW` or `SendMessageW` synchronously inside the WinEventProc callback thread blocks the Windows OS global message queue if a managed app hangs.
2. **Repaint Thrashing**: High-frequency mouse hover events during rapid pointer movement trigger high-frequency `SetWindowPos` calls, leading to DWM queue saturation and severe UI stuttering.
