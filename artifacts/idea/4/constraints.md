## Win32 & DWM Constraints
1. **DWM Cloaking Timing Window**:
   - `DwmSetWindowAttribute(hwnd, DWMWA_CLOAKED)` must be called before `ShowWindow` or during `EVENT_OBJECT_CREATE` to prevent the OS from presenting the first frame at default coordinates.
2. **Legacy GDI & Win32 Applications**:
   - Some legacy applications do not support `DWMWA_CLOAKED` and must fallback to `SetWindowPlacement` / `SW_HIDE` before positioning.

## Structural Constraints
1. **Nested Split Containers vs Top-Level Columns**:
   - In `detach_container`, size redistribution must still occur for windows inside vertical split containers (where the sum of vertical heights must equal 100%), but must be skipped for top-level columns of a horizontal `Workspace`.
