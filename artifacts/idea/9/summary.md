## Initial Proposition
Resolution of 7 critical pre-iteration-8 bugs:
1. Fullscreen command error `Focused container is not inside a workspace column`.
2. Monitor switching requiring double hotkey actuation.
3. Windows squished from 100% to 60% on focus change instead of being displaced.
4. Fullscreen windows isolated in separate z-order layer preventing bidirectional focus traversal.
5. Window focus stolen when background applications issue signals/alerts.
6. Rules `move --workspace 3` + `set-tiling` causing window cloaking, taskbar flashing, and column absence.
7. Newly managed windows trapped in cloaked state flashing in taskbar.

## Clarifications
- Fullscreen mode must preserve underlying tiling logic identically to Niri: the window remains part of the horizontal strip, and directional focus moves freely to adjacent tiling columns.
- Deep architectural refactoring of the container hierarchy and Win32 focus pipeline is authorized where required.
- Canvas displacement must display windows instantaneously without artificial frame animations during layout transition.
- Focus guarding must withstand multi-stage application initialization (e.g. 1C Enterprise, Teams, Outlook) where windows spawn sub-windows or assert foreground delayed after focus has already shifted.
- Target functionality strictly mirrors Niri Wayland compositor semantics on the Windows platform.

## Perceived Pitfalls
- Displaced canvas windows extending across display boundaries could project pixel slivers onto adjacent monitors if virtual coordinate parking is miscalculated.
- Premature window cloaking breaks OS message delivery and taskbar click routing if `EVENT_SYSTEM_FOREGROUND` is dropped by WinEvent hooks.
- Asynchronous Win32 focus assertions can deadlock if `AttachThreadInput` is retained across thread context boundaries or error exits.
