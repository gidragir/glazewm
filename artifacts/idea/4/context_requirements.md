## Required Codebase Context
1. **Container Detachment**:
   - [detach_container.rs](file:///data/projects/glazewm/packages/wm/src/commands/container/detach_container.rs): Isolate sibling resize logic for workspace children.
2. **Window Management & Lifecycle**:
   - [manage_window.rs](file:///data/projects/glazewm/packages/wm/src/commands/window/manage_window.rs): Window registration, initial placement, and cloaking/uncloaking flow.
   - [window_listener.rs](file:///data/projects/glazewm/packages/wm-platform/src/platform_impl/windows/window_listener.rs): WinEventHook lifecycle events (`EVENT_OBJECT_CREATE`, `EVENT_OBJECT_SHOW`).
3. **Viewport Auto-Panning**:
   - [platform_sync.rs](file:///data/projects/glazewm/packages/wm/src/commands/general/platform_sync.rs): Viewport panning on new window focus.
