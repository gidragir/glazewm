# Move and Merge Window Commands Separation

## Goal Description

In the current implementation of the infinite horizontal canvas, `move_window_in_direction` exhibits inconsistent behavior when moving a window horizontally (Left/Right):
- If the adjacent sibling is a single `TilingWindow`, the window **swaps** positions with it.
- If the adjacent sibling is a `SplitContainer` (a column), the window **merges** into it.

This causes a usability issue where extracting a window from a column and moving it past another column accidentally merges it into that column.

The goal is to separate these behaviors:
1. **Standard Move (`Alt+Shift+Left/Right`)**: Should *always* swap positions (reorder columns), regardless of whether the sibling is a single window or a split container.
2. **Explicit Merge (`Alt+Ctrl+Shift+Left/Right`)**: Should explicitly merge the window with the adjacent sibling into a single column. If the sibling is a split container, it joins it. If the sibling is a single window, it creates a new vertical split container for both.

## Open Questions

> [!IMPORTANT]
> **1. Command API Design**
> Should we introduce a brand new command for this, or add a flag to the existing move command?
> - **Option A:** New command: `merge-window --direction left`
> - **Option B:** Flag on move: `move --direction left --merge`
> 
> **2. Vertical Merging?**
> Do you want this explicit merge command to also apply vertically? (e.g., merging a window `Up` into a horizontal split). The implementation will be generic over directions, but the primary use case is horizontal columns.

## Proposed Changes

### `packages/wm-common`

#### [MODIFY] [`app_command.rs`](file:///data/projects/glazewm/packages/wm-common/src/app_command.rs)
- Add the new command or flag representation to `InvokeCommand` / `InvokeMoveCommand`.

### `packages/wm`

#### [MODIFY] [`move_window_in_direction.rs`](file:///data/projects/glazewm/packages/wm/src/commands/window/move_window_in_direction.rs)
- Update `move_to_sibling_container()`: When handling `TilingContainer::Split(sibling_split)`, change the logic to use `move_container_within_tree` to swap the window's index with the split container's index within their mutual parent (the workspace), instead of inserting it as a descendant.

#### [NEW] `merge_window_in_direction.rs` (or added to existing module)
- Implement `merge_window_in_direction`:
  - Find the sibling in the given direction.
  - If the sibling is a `SplitContainer`, move the window into it (reusing the old `descendant_in_direction` logic).
  - If the sibling is a `TilingWindow`, create a new `SplitContainer` with `TilingDirection::Vertical`. Place the sibling window and the moving window into this new split container, and insert the split container into the workspace in place of the original sibling.

#### [MODIFY] `packages/wm/src/commands/window/mod.rs`
- Export the new command.

#### [MODIFY] `packages/wm/src/command_executor.rs`
- Register the handler for the new `merge-window` command.

## Verification Plan

### Automated Tests
- Write unit tests ensuring `move_window_in_direction` horizontally swaps a window with an adjacent `SplitContainer` without altering the split container's children.
- Write unit tests ensuring `merge_window_in_direction` horizontally into a `TilingWindow` creates a new `SplitContainer`.
- Write unit tests ensuring `merge_window_in_direction` horizontally into a `SplitContainer` adds the window as a child.

### Manual Verification
- Rebuild the binaries and deploy to `/srv/Shared`.
- Test on the Windows VM using the requested `Alt+Shift+Left/Right` and `Alt+Ctrl+Shift+Left/Right` keybindings.
