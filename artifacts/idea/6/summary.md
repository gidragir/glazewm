## Initial Proposition
Eliminate focus stealing and unsolicited window/workspace jumping when background applications or running services emit system signals or call foreground activation APIs.

## Clarifications
1. **Strict User-Only Invariant**: Focus transitions, workspace activations, and viewport panning must be triggered exclusively by user actions (keyboard navigation shortcuts, mouse clicks on visible windows, CLI commands) or new window creations on the active workspace.
2. **Elimination of Auto-Switching Legacy Code**: Completely purge legacy logic in `handle_window_focused` that force-activated off-screen/hidden workspaces when background applications received OS focus.
3. **Parity with Windows and Niri Compositor Models**:
   - Windows Desktop: Enforce foreground locking to prevent background `SetForegroundWindow` calls from stealing user keystrokes.
   - Niri Compositor: Suppress automatic workspace switching and infinite canvas panning when background signals or off-screen events occur; preserve active column focus.
4. **Spawn vs Signal Decoupling**: Newly spawned windows on the active workspace receive focus and auto-pan the canvas naturally; already-running background windows that signal in the background are strictly prevented from altering active focus.

## Perceived Pitfalls
1. Re-assertion oscillation: endless focus battle loops between an aggressive background application calling `SetForegroundWindow` and the window manager repeatedly reclaiming focus.
2. Breaking intentional mouse clicks: misclassifying user clicks on unfocused tiled windows on the active workspace as unsolicited background signals.
3. Broken window creation rules: new windows intended for hidden background workspaces accidentally grabbing focus and pulling the user away from their active workspace.
