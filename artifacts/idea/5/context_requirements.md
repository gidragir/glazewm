## Required Codebase Context
1. **Container Tree Manipulation Modules**:
   - `packages/wm/src/commands/container/wrap_in_split_container.rs`: Splitting leaf windows into composite `SplitContainer` nodes.
   - `packages/wm/src/commands/container/move_container_within_tree.rs`: Cross-container reparenting, focus-order preservation, and lowest common ancestor resolution.
   - `packages/wm/src/commands/container/flatten_split_container.rs` & `flatten_child_split_containers.rs`: Cleanup of single-child split containers.
2. **Command Dispatch & IPC Pipeline**:
   - `packages/wm-common/src/app_command.rs`: Command line parsing, serialization/deserialization, and `InvokeCommand` variants.
   - `packages/wm/src/wm.rs`: Dispatch router connecting incoming IPC commands to window management handlers.
3. **Window Traversal & Sibling Selectors**:
   - `packages/wm/src/traits/common_getters.rs` & `tiling_direction_getters.rs`: Sibling iteration (`prev_siblings`, `next_siblings`) and direction resolution.
4. **State Synchronization**:
   - `packages/wm/src/wm_state.rs` & `pending_sync.rs`: Redraw queueing and event emission.
