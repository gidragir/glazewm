## Standard Patterns
1. **Windows Foreground Lock & Flash Pattern**: Win32 native behavior via `LockSetForegroundWindow` and `SystemParametersInfo(SPI_SETFOREGROUNDLOCKTIMEOUT)` where background focus requests are downgraded to flashing window borders (`FLASHW_ALL`) rather than window elevation.
2. **Wayland XDG-Activation Token Verification (Niri / Sway / GNOME)**: Wayland compositors require an activation token issued from a recent user input event to grant focus to an external surface. Requests without valid tokens are converted to `demands_attention` / `urgency` hints, leaving the active view and scroll position completely unaffected.
3. **i3 / bspwm Focus Stealing Prevention**: Tiling managers provide `focus_on_window_activation none|smart|urgent` policies to prevent background processes from switching active workspaces.

## Alternative Approaches
1. **Toast Notification Sniffing**: Intercepting Windows toast and action center hooks to detect alerts before they reach window message pumps. (Rejected: fragile, requires private WinRT hooks).
2. **Demoting Background Windows to Minimization**: Automatically minimizing or hiding rogue windows that request focus. (Rejected: corrupts user layout and window state).
3. **Strict User-Only Focus Filter (Selected)**: Enforcing Win32 foreground lock while filtering WM focus event queues against container display states and displayed monitor workspaces.
