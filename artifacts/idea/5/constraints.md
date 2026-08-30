## Logical Conflicts
1. **Ambiguous Consume Directionality**: When a window is positioned at the canvas boundary (leftmost or rightmost column), invoking consume in the direction of the boundary has no target node; must safely no-op without triggering viewport scroll or focus loss.
2. **Expel Position Collision**: Expelling a window from a column into an already dense workspace must not overwrite existing column indices or invalidate cached sibling iterators.

## Edge Cases
1. **Two-Window Column Disassembly**: Expelling one window from a two-window vertical column leaves a single remaining window; the parent `SplitContainer` must auto-flatten to avoid redundant single-child split container wrappers.
2. **Multi-Window Column Expulsion**: Expelling an intermediate window (index 1 of a 3-window column) must re-normalize the remaining sibling tiling sizes $(\sum \text{size} = 1.0)$ without resizing unrelated columns on the canvas.
3. **Floating / Fullscreen Invocations**: Invoking consume/expel on non-tiling windows (floating, minimized, or fullscreen) must be filtered at the command boundary to prevent panics or invalid tree mutations.

## Performance Risks
1. **Redundant Layout Passes**: Tree restructuring (detaching, wrapping, and attaching containers) during consume/expel operations triggers geometry updates; must batch redraws via `pending_sync` to prevent multiple `SetWindowPos` calls per invocation.
2. **Focus Index Desynchronization**: Moving windows across container boundaries without updating `child_focus_order` on both source and target containers risks focusing null or detached container IDs.
