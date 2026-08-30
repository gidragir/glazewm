## Required Codebase Context
1. **WinAPI Crate Dependencies**: Verify availability and integration of `windows-sys` or `windows` Rust crate modules:
   - `Win32::UI::WindowsAndMessaging` (`SetWindowPos`, `SetWinEventHook`, `GetWindowLongPtrW`, `DeferWindowPos`)
   - `Win32::Graphics::Gdi` (`EnumDisplayMonitors`, `GetMonitorInfoW`)
   - `Win32::UI::Accessibility` (`HWINEVENTHOOK`)
2. **Event Loop Architecture**: Evaluate channel abstraction (`tokio::sync::mpsc` or `crossbeam`) for non-blocking message passing between WinEventProc thread and layout engine thread.
3. **Window Rule Engine & Matcher**: Pattern matching module (Regex/Glob) for executable name (`process_name`), window class (`window_class`), and window title (`window_title`) to match against user configuration.
4. **Layout Tree Data Structure**: Data structures to store virtual columns and window sequences per workspace (`Vec<Column>` or custom doubly-linked list representation).
5. **DPI & Multi-Monitor Topology Provider**: Per-monitor DPI detection (`GetDpiForMonitor`) and display bounds mapping (`MONITORINFOEXW`).
