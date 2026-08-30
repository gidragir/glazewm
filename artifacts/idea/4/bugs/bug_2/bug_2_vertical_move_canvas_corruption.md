# Bug 2: Workspace Inversion & Column Width Corruption on Vertical Move

## Status
**Fixed**

## Description
When working on an infinite horizontal canvas with multiple columns of varying widths (e.g. Window 1 = 25%, Window 2 = 75%, Window 3 = 50%, Window 4 = 50%):
- Moving focus to Window 4 and invoking the `move down` command (e.g. `alt+shift+j`) breaks the horizontal canvas layout completely.
- When moving back up / returning focus to Window 1, Window 2 loses its 75% width and gets compressed to a completely different size.

## Root Cause
In `packages/wm/src/commands/window/move_window_in_direction.rs`:
1. When a window attempts to move `Down` or `Up`, and has no vertical siblings or vertical parent container, GlazeWM's legacy tiling engine falls back to `invert_workspace_tiling_direction`.
2. `invert_workspace_tiling_direction` does the following:
   - Wraps ALL other windows on the canvas (Windows 1, 2, 3) inside a single horizontal `SplitContainer`.
   - **Inverts the entire Workspace tiling direction from `Horizontal` to `Vertical`**.
   - Moves Window 4 below the wrapped split container and resets sizes to `0.5 / 0.5`.
3. This destroys the infinite horizontal canvas topology. When returning back or attempting to undo, `flatten_split_container` divides sizes equally among children, permanently wiping out Window 2's custom 75% width.

## Expected Behavior on Infinite Canvas
1. The `Workspace` tiling direction must remain permanently `Horizontal` (infinite canvas mode) and **must never be inverted to Vertical**.
2. When invoking vertical movement (`move down` / `move up`):
   - If the window is already in a vertical split column, it moves up/down within that column.
   - If the window is a standalone column and there is no vertical space/sibling, it should either:
     - **Option A (Safe no-op)**: Ignore the vertical move command if no vertical target exists.
     - **Option B (Column stacking)**: Allow merging into an adjacent column as a vertical split while preserving column widths.
3. Under no circumstances should the workspace flip to vertical or wrap unrelated columns into a split container.
