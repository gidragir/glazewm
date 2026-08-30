## H1: Atomic Top-Level Node Reordering via Index Swapping
- **Description**: Replacing recursive descent in `move_to_sibling_container` with direct sibling index shifting in the parent direction container ensures $O(1)$ column swapping without mutating internal container hierarchies.
- **Validation Condition**: Executing horizontal move across a sequence of mixed single-window and multi-window columns swaps node indices without altering child counts or tiling sizes of split containers.

## H2: Bidirectional Consume/Expel State Invariant
- **Description**: Executing `consume-or-expel-window-left` on a window followed immediately by `consume-or-expel-window-right` (or vice versa) restores the exact initial workspace tree topology and window ordering.
- **Validation Condition**: Unit test verifying round-trip consumption and expulsion of standalone windows and vertical columns produces an identical serialized container tree.

## H3: Workspace Container Flattening Immunity
- **Description**: Guarding `flatten_child_split_containers` to execute only when `parent.is_split()` prevents single-column workspaces from collapsing into unconstrained horizontal window lists.
- **Validation Condition**: Construct a workspace containing a single vertical split container with 2 windows; trigger tree sync passes and verify the `Workspace` remains `Horizontal` with exactly 1 `Vertical` `SplitContainer` child.

## H4: IPC & Config Command Parsing Parity
- **Description**: Registering `ConsumeOrExpelWindowLeft` and `ConsumeOrExpelWindowRight` in `InvokeCommand` enables kebab-case CLI invocations (`consume-or-expel-window-left`) and config-bound hotkeys with zero runtime dispatch overhead.
- **Validation Condition**: Verify CLI command dispatch `glazewm.exe command consume-or-expel-window-left` executes correctly across running window manager sessions.
