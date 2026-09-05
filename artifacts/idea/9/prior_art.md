## Standard Patterns
- **Niri Scrollable Tiling Model**: Manages workspaces as unbounded horizontal strips where columns are vertical stacks. Fullscreen windows maximize within the viewport without severing topology, enabling seamless horizontal panning across columns.
- **RAII Thread Input Binding**: Encapsulates `AttachThreadInput(source, target, TRUE)` inside a guard struct whose `Drop` implementation executes `AttachThreadInput(source, target, FALSE)` under all exit paths.
- **Guard Rail Event Filter**: Evaluates event causation through intent tokens: operations initiated by WM commands set an intent flag (`needs_focus_update`), distinguishing legitimate user commands from asynchronous OS signal interruptions.

## Alternative Approaches
- **Sub-classing HWNDs for Viewport Clipping**: Injecting a DirectX/DWM composition layer to clip windows without changing `HWND` geometry. Rejected: requires kernel injection or heavy DWM hooking violating zero-dependency safe Win32 architecture.
- **DWM Cloaking for Offscreen Windows**: Cloaking every displaced canvas window instead of virtual parking. Rejected: cloaking completely removes window thumbnails from the native Windows Alt+Tab switcher and taskbar live previews.
