# Required Codebase Context: Iteration 8

## Required Codebase Context
- [ ] `packages/wm/src/commands/general/platform_sync.rs`: Analyze `animate_pan_workspace` sleep loop, scratch buffer candidate sites, and border effect worker spawn.
- [ ] `packages/wm-platform/src/dispatcher.rs`: Inspect thread dispatching primitives, message pump loop integration, and frame tick injection.
- [ ] `packages/wm-platform/src/platform_impl/windows/native_window.rs`: Inspect `set_z_order` background task spawn, `apply_window_positions` batch execution, and handle validity checks.
- [ ] `packages/wm-ipc-client/src/lib.rs`: Audit message multiplexing, response awaiting loops, and subscription streams.
- [ ] `packages/wm/src/ipc_server.rs`: Review connection handling, broadcast channel buffer sizing, and `JoinSet`/cancellation lifecycles.
- [ ] `packages/wm/src/models/` & `packages/wm/src/wm_state.rs`: Evaluate `PendingSync` scratch capacity and container iteration reference patterns.
- [ ] `packages/wm/tests/` & `benches/`: Establish baseline performance benchmarks measuring layout time and message queue latency.
