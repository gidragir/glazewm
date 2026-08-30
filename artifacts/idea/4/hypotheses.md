## H1: Independent Detachment Stability
- **Description**: Bypassing sibling size redistribution in `detach_container` when parent is a `Workspace` guarantees that off-screen and visible columns retain their exact width ratios when any window closes.
- **Validation Condition**: Open 5 windows with varying widths. Close window 2; verify windows 1, 3, 4, 5 retain their exact pixel widths and columns 3-5 shift left cleanly.

## H2: Pre-Show Cloaking Eliminates Spawn Flash
- **Description**: Setting `set_cloaked(true)` immediately upon window creation and uncloaking only after the window has been positioned on the strip completely eliminates the centered spawn flicker for explorer.exe and other apps.
- **Validation Condition**: Open Windows Explorer (`Win+E`) 10 times; confirm zero instances of the window flashing in the screen center before appearing on the canvas.
