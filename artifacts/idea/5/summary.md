## Initial Proposition
Decouple window translation from column merging in infinite horizontal canvas mode, adding dedicated keybindings/commands to consume adjacent windows into vertical columns and expel them back to the canvas.

## Clarifications
1. **Command Naming & Semantics**: Adopt Niri's `consume-or-expel-window-left` and `consume-or-expel-window-right` command semantics instead of overloaded `--merge` flags.
2. **Infinite Canvas Orientation**: Workspace tiling direction is strictly horizontal. Tiling columns are exclusively vertical stacks (typically $\le 3$ windows per column).
3. **No Vertical Merging**: Consume/expel operations are strictly horizontal across adjacent columns; vertical merging across rows is omitted to preserve infinite canvas invariants.
4. **Isolated Move Behavior**: Standard directional movement (`Alt+Shift+Left/Right`) must strictly reorder columns along the horizontal axis without nesting windows into neighboring containers.

## Perceived Pitfalls
1. Re-nesting loops: moving an extracted window past another column triggering unintended container insertion.
2. Layout engine collapsing or inverting the horizontal workspace when a column becomes the sole child in the workspace.
3. Asymmetric window ordering when consuming leftward vs. rightward into vertical split containers.
