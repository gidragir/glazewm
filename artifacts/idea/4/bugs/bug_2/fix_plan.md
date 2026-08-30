# Fix Plan: Prevent Workspace Inversion on Vertical Movement (Infinite Canvas)

## Goal Description
Prevent vertical move commands (`move down` / `move up`) from inverting the workspace tiling direction to vertical and wrapping all canvas columns into a single destructive split container.

## Proposed Changes

### `packages/wm` (Movement & Layout Logic)

#### [MODIFY] [move_window_in_direction.rs](file:///data/projects/glazewm/packages/wm/src/commands/window/move_window_in_direction.rs)
- In `move_tiling_window`:
  - When searching for a matching target ancestor, check if the workspace is in horizontal infinite canvas mode.
  - If no suitable vertical ancestor exists:
    - **Disable `invert_workspace_tiling_direction` for horizontal canvas workspaces**.
    - If the user moves `Down` / `Up` on a standalone column, either:
      - Safely no-op if no vertical destination exists in the column.
      - Or support column-merging into a vertical split container without altering the workspace's horizontal tiling direction or touching neighboring columns.

## Detailed Task List

- [x] **Task 1: Guard Workspace Tiling Direction from Inversion**
  - In `move_window_in_direction.rs`, check if the top-level parent is a horizontal `Workspace`.
  - Suppress `invert_workspace_tiling_direction` when `parent.is_workspace()` and direction is `Up`/`Down`.

- [x] **Task 2: Support Vertical Stacking within Column**
  - When moving within a vertical split container, permit swapping siblings.
  - Ensure the parent column retains its exact `tiling_size` (width).

- [x] **Task 3: Verification & Test**
  - Reproduce scenario: Window 1 (25%), Window 2 (75%), Window 3 (50%), Window 4 (50%).
  - On Window 4, press `move down` and `move up`.
  - Return to Window 1; verify Window 2 retains 75% width and canvas geometry remains pristine.
