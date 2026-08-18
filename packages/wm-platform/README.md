## Component Role
`wm-platform` provides a cross-platform window management abstraction layer for GlazeWM. It encapsulates OS-specific windowing subsystem calls (Win32 API on Windows, Cocoa and Accessibility AX APIs on macOS) into unified primitives for event loops, display enumeration, native window manipulation, input listeners, and thread-safe dispatching.

## Dependency Graph
- Inherited (`workspace = true`):
  - `home` (`version = "0.5"`)
  - `regex` (`version = "1"`)
  - `serde` (`version = "1"`, features: `["derive"]`)
  - `thiserror` (`version = "2"`)
  - `tokio` (`version = "1"`, features: `["full"]`)
  - `tracing` (`version = "0.1"`)
- External (specify enabled features):
  - Target `cfg(target_os = "windows")`:
    - `windows` (`version = "0.52"`, features: `["implement", "Win32_Devices_HumanInterfaceDevice", "Win32_Foundation", "Win32_Graphics_Dwm", "Win32_Graphics_Gdi", "Win32_Security", "Win32_System_Com", "Win32_System_Environment", "Win32_System_LibraryLoader", "Win32_System_Registry", "Win32_System_RemoteDesktop", "Win32_System_SystemServices", "Win32_System_Threading", "Win32_UI_Accessibility", "Win32_UI_HiDpi", "Win32_UI_Input_Ime", "Win32_UI_Input_KeyboardAndMouse", "Win32_UI_Shell_Common", "Win32_UI_TextServices", "Win32_UI_WindowsAndMessaging"]`)
    - `windows-interface` (`version = "0.52"`)
  - Target `cfg(target_os = "macos")`:
    - `objc2` (`version = "0.6.4"`)
    - `objc2-app-kit` (`version = "0.3.2"`, default-features: `false`, features: `["NSAlert", "NSApplication", "NSEvent", "NSGraphics", "NSRunningApplication", "NSResponder", "NSScreen", "NSWindow", "NSWorkspace", "libc", "objc2-core-foundation"]`)
    - `objc2-application-services` (`version = "0.3.2"`)
    - `objc2-core-foundation` (`version = "0.3.2"`, features: `["CFCGTypes", "CFUUID"]`)
    - `objc2-core-graphics` (`version = "0.3.2"`)
    - `objc2-foundation` (`version = "0.3.2"`, default-features: `false`, features: `["NSArray", "NSEnumerator", "NSNotification", "NSKeyValueCoding", "NSString", "NSThread", "NSValue"]`)
  - Dev Dependencies:
    - `libtest-mimic-collect` (`version = "0.3.2"`)
  - Package Features:
    - `test_utils` (enables `pub mod test_utils`)
- Local workspace peers:
  - None

## Public API
- **Type Aliases**:
  - `pub type Result<T> = std::result::Result<T, Error>`
  - `pub type DispatchFn = dyn FnOnce() + Send + 'static`
  - `pub type WndProcCallback = dyn Fn(isize, u32, usize, isize) -> Option<isize> + Send + 'static`
- **Core Types & Structs**:
  - `pub struct EventLoop`
    - `pub fn new() -> crate::Result<(Self, Dispatcher)>`
    - `pub fn run(self) -> crate::Result<()>`
  - `pub struct Dispatcher` (implements `Clone`, `Send`, `Sync`)
    - `pub fn stop_event_loop(&self) -> crate::Result<()>`
    - `pub fn dispatch_async<F>(&self, dispatch_fn: F) -> crate::Result<()>`
    - `pub fn dispatch_sync<F, R>(&self, dispatch_fn: F) -> crate::Result<R>`
    - `pub fn thread_id(&self) -> ThreadId`
    - `pub fn displays(&self) -> crate::Result<Vec<Display>>`
    - `pub fn sorted_displays(&self) -> crate::Result<Vec<Display>>`
    - `pub fn display_devices(&self) -> crate::Result<Vec<DisplayDevice>>`
    - `pub fn display_from_point(&self, point: &Point) -> crate::Result<Display>`
    - `pub fn primary_display(&self) -> crate::Result<Display>`
    - `pub fn nearest_display(&self, native_window: &NativeWindow) -> crate::Result<Display>`
    - `pub fn visible_windows(&self) -> crate::Result<Vec<NativeWindow>>`
    - `pub fn focused_window(&self) -> crate::Result<NativeWindow>`
    - `pub fn cursor_position(&self) -> crate::Result<Point>`
    - `pub fn is_mouse_down(&self, button: &MouseButton) -> bool`
    - `pub fn window_from_point(&self, point: &Point) -> crate::Result<Option<NativeWindow>>`
    - `pub fn set_cursor_position(&self, point: &Point) -> crate::Result<()>`
    - `pub fn reset_focus(&self) -> crate::Result<()>`
    - `pub fn open_file_explorer(&self, path: &Path) -> crate::Result<()>`
    - `pub fn show_error_dialog(&self, title: &str, message: &str)`
  - `pub struct ThreadBound<T>` (implements `Send`, `Sync`, `Clone`, `Drop`)
    - `pub fn new(inner: T, dispatcher: Dispatcher) -> Self`
    - `pub fn get_ref(&self) -> crate::Result<&T>`
    - `pub fn get_mut(&mut self) -> crate::Result<&mut T>`
    - `pub fn into_inner(self) -> crate::Result<T>`
    - `pub fn with<F, R>(&self, f: F) -> crate::Result<R>`
    - `pub fn with_mut<F, R>(&mut self, f: F) -> crate::Result<R>`
    - `pub fn is_event_loop_thread(&self) -> bool`
  - `pub struct NativeWindow`
    - `pub fn id(&self) -> WindowId`
    - `pub fn title(&self) -> crate::Result<String>`
    - `pub fn process_name(&self) -> crate::Result<String>`
    - `pub fn frame(&self) -> crate::Result<Rect>`
    - `pub fn position(&self) -> crate::Result<(f64, f64)>`
    - `pub fn size(&self) -> crate::Result<(f64, f64)>`
    - `pub fn is_valid(&self) -> bool`
    - `pub fn is_visible(&self) -> crate::Result<bool>`
    - `pub fn is_minimized(&self) -> crate::Result<bool>`
    - `pub fn is_maximized(&self) -> crate::Result<bool>`
    - `pub fn is_resizable(&self) -> crate::Result<bool>`
    - `pub fn is_desktop_window(&self) -> crate::Result<bool>`
    - `pub fn set_frame(&self, rect: &Rect) -> crate::Result<()>`
    - `pub fn resize(&self, width: i32, height: i32) -> crate::Result<()>`
    - `pub fn reposition(&self, x: i32, y: i32) -> crate::Result<()>`
    - `pub fn minimize(&self) -> crate::Result<()>`
    - `pub fn maximize(&self) -> crate::Result<()>`
    - `pub fn focus(&self) -> crate::Result<()>`
    - `pub fn close(&self) -> crate::Result<()>`
  - `pub struct Display`, `pub struct DisplayDevice`, `pub struct WindowId`, `pub struct DisplayId`, `pub struct DisplayDeviceId`
  - `pub struct WindowListener`, `pub struct DisplayListener`, `pub struct MouseListener`, `pub struct KeybindingListener`
  - `pub struct Keybinding`, `pub struct SingleInstance`, `pub struct Color`, `pub struct OpacityValue`
  - `pub struct Point`, `pub struct Rect`, `pub struct RectDelta`, `pub struct Delta<T>`, `pub struct PressedButtons`
- **Enums**:
  - `pub enum Error`, `pub enum ParseError`, `pub enum PlatformEvent`, `pub enum WindowEvent`, `pub enum MouseEvent`
  - `pub enum MouseEventKind`, `pub enum MouseButton`, `pub enum ConnectionState`, `pub enum MirroringState`
  - `pub enum OutputTechnology`, `pub enum WindowZOrder`, `pub enum CornerStyle`, `pub enum Direction`
  - `pub enum LengthValue`, `pub enum Key`, `pub enum KeyCode`
- **Platform Extension Traits**:
  - `pub trait DispatcherExtWindows` (`cfg(target_os = "windows")`)
  - `pub trait DispatcherExtMacOs` (`cfg(target_os = "macos")`)
  - `pub trait NativeWindowWindowsExt` (`cfg(target_os = "windows")`)
  - `pub trait NativeWindowExtMacOs` (`cfg(target_os = "macos")`)
  - `pub trait DisplayExtWindows` (`cfg(target_os = "windows")`)
  - `pub trait DisplayExtMacOs` (`cfg(target_os = "macos")`)
  - `pub trait DisplayDeviceExtWindows` (`cfg(target_os = "windows")`)
  - `pub trait DisplayDeviceExtMacOs` (`cfg(target_os = "macos")`)

## Architecture & Modules
```
wm-platform (src/lib.rs)
├── models (src/models/mod.rs)
│   ├── color.rs
│   ├── corner_style.rs
│   ├── delta.rs
│   ├── direction.rs
│   ├── key.rs
│   ├── key_code.rs
│   ├── length_value.rs
│   ├── opacity_value.rs
│   ├── point.rs
│   ├── rect.rs
│   └── rect_delta.rs
├── dispatcher.rs
├── display.rs
├── display_listener.rs
├── error.rs
├── event_loop.rs
├── keybinding_listener.rs
├── mouse_listener.rs
├── native_window.rs
├── platform_event.rs
├── single_instance.rs
├── thread_bound.rs
├── window_listener.rs
├── test_utils.rs [cfg(feature = "test_utils")]
└── platform_impl (src/platform_impl/mod.rs)
    ├── windows/ (Win32 / DWM integration)
    │   ├── com.rs
    │   ├── display.rs
    │   ├── display_listener.rs
    │   ├── event_loop.rs
    │   ├── keyboard_hook.rs
    │   ├── mouse_listener.rs
    │   ├── native_window.rs
    │   ├── single_instance.rs
    │   └── window_listener.rs
    └── macos/ (Cocoa / Core Graphics / Accessibility AX integration)
        ├── application.rs
        ├── application_observer.rs
        ├── ax_ui_element.rs
        ├── ax_value.rs
        ├── display.rs
        ├── display_listener.rs
        ├── event_loop.rs
        ├── ffi.rs
        ├── keyboard_hook.rs
        ├── mouse_listener.rs
        ├── native_window.rs
        ├── notification_center.rs
        ├── single_instance.rs
        └── window_listener.rs
```

## Execution Context
- **Sync/Async Boundaries**:
  - `EventLoop::run` blocks the calling thread to execute the native OS message pumping loop (`CFRunLoopRun` on macOS, Win32 message loop on Windows).
  - Listeners (`WindowListener`, `DisplayListener`, `MouseListener`, `KeybindingListener`) stream native event notifications into `tokio::sync::mpsc::UnboundedReceiver` channels for consumption by async runtimes.
  - Cross-thread closure dispatch is supported synchronously via `Dispatcher::dispatch_sync` (blocks caller via `std::sync::mpsc` with a 5-second timeout) or asynchronously via `Dispatcher::dispatch_async`.
- **Runtime Requirements**: Tokio async channels (`tokio::sync::mpsc`) for event propagation; platform thread constraints requiring UI operations to execute on the dedicated event loop thread.
- **State Mutability & Thread Affinity**:
  - `ThreadBound<T>` encapsulates thread-unsafe pointers (`CFRetained<AXUIElement>`, `NSScreen`), granting `Send + Sync` guarantees by enforcing that access and destruction occur exclusively on the originating event loop thread.
  - `KeybindingListener` utilizes `Arc<Mutex<HashMap<Key, Vec<Keybinding>>>>` for atomic hotkey map updates and `Arc<AtomicBool>` for state toggles across hooks.
  - `Dispatcher` shares shutdown status across threads using `Arc<AtomicBool>` with `Ordering::SeqCst`.
- **Locking Mechanisms**: `std::sync::Mutex` for keybinding map protection, `std::sync::mpsc` channels for synchronous dispatch returns, atomic flags (`AtomicBool`) for listener state management.
