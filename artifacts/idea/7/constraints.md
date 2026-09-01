## Logical Conflicts
1. **Continuous Canvas vs Discrete Displays**: A conceptual infinite canvas has continuous 1D horizontal coordinates $[-\infty, +\infty]$, whereas the OS virtual desktop is a non-continuous multi-rect 2D space. Translating continuous 1D canvas offsets directly into OS $(X, Y)$ without bounding box segmentation inherently creates physical monitor collision.
2. **Taskbar Presence vs Offscreen Hiding**: Standard Win32 mechanisms to hide windows (`ShowWindow(SW_HIDE)` or DWM cloaking) hide taskbar icons, creating a direct conflict with requirement to keep offscreen windows present in taskbar/Alt+Tab.

## Edge Cases
1. **Negative Virtual Coordinates**: Secondary monitor positioned to the left of primary monitor has negative $X$ coordinates ($[-1920, 0]$); safe parking must compute the global bounding union across all monitors to avoid placing parked windows on left/top monitors.
2. **Mixed DPI & Multi-Monitor Scaling**: Different monitors having distinct scaling factors ($100\%, 125\%, 150\%$) can distort coordinate math when a window boundary touches the monitor transition line.
3. **Window Partial Straddling**: A column positioned partially offscreen ($50\%$ visible, $50\%$ offscreen) may cross the border onto a neighboring monitor if physical coordinates are used directly without region clipping (`SetWindowRgn`) or virtual clipping frames.
4. **OS Foreground Stealing on Focus**: When an offscreen window is activated via Alt+Tab or taskbar, GlazeWM must intercept the activation, update `workspace.offset_x` to bring the column into viewport, and reposition the window before rendering.

## Performance Risks
1. **Frequent SetWindowPos Calls**: Repositioning multiple offscreen windows between candidate canvas coordinates and safe parking coordinates must use batch window operations (`DeferWindowPos` / `SWP_NOREDRAW` / `SWP_NOACTIVATE`) to avoid DWM rendering lag during fast scrolling.
