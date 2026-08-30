## Standard Patterns
1. **Tree-Zipper / Split-Container Wrapping**: Dynamically injecting a composite node (`SplitContainer`) to replace a leaf node (`TilingWindow`) upon grouping, moving both operand leaves into the new composite.
2. **Atomic Command Dispatcher**: Isolating command handlers (`move_window_in_direction`, `consume_or_expel_window`) into independent pure-mutation routines operating on an in-memory DOM-like container tree.
3. **Reactive Pending Sync Queue**: Queueing damaged nodes (`pending_sync.queue_container_to_redraw`) during tree mutations, deferring Win32 layout computation until the command batch completes.

## Alternative Approaches
1. **Niri Wayland Protocol (Reference Baseline)**: Niri handles columns natively as a 2D matrix structure (`Workspace -> Column -> Window[]`), where consume/expel shifts windows across column arrays directly without arbitrary nested split trees.
2. **Static Split Containers (Traditional GlazeWM / i3)**: Relying solely on directional movement and explicit container splitting (`split-direction vertical`) before moving windows into the newly split container.
3. **Hybrid Split-Tree Infinite Canvas (Selected Approach)**: Keeping GlazeWM's recursive container tree model while restricting top-level workspace nodes to vertical columns, translating 2D matrix operations (`consume`/`expel`) onto tree wrapping/unwrapping operations.
