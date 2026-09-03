# Architectural Hypotheses: Iteration 8

## H1: Non-Blocking Dispatcher Frame Driving
- **Description**: Replacing synchronous `std::thread::sleep` in `animate_pan_workspace` with an asynchronous frame driver (`tokio::time::interval` emitting actions to `wm_platform::Dispatcher`) maintains 60/120/144 FPS smooth panning while reducing main-thread Win32 message pump latency from ~90ms to <0.1ms per frame.
- **Validation Condition**: Windows message queue latency benchmark shows 0 dropped input messages during rapid workspace panning and focus switching.

## H2: Strict Single-Threaded UI Marshaling
- **Description**: Routing delayed border and z-order mutations in `apply_border_effect` and `set_z_order` through the `Dispatcher` UI queue eliminates concurrent `HWND` access from background Tokio worker threads.
- **Validation Condition**: Win32 thread safety assertion (`GetCurrentThreadId() == main_ui_thread_id`) holds true across all window style and position mutations.

## H3: Zero-Allocation Scratch Buffering in Hot Paths
- **Description**: Reusing persistent scratch buffers for window position collection in `PendingSync` and container traversal reduces heap allocations per layout cycle to zero.
- **Validation Condition**: Memory allocation profiler / benchmark verifies zero heap allocations in `platform_sync` passes during continuous mouse tracking or viewport panning.

## H4: Lossless Multiplexed IPC Client
- **Description**: Decoupling event subscription reception from request-response awaiting in `wm-ipc-client` via dedicated unbounded mpsc buffers prevents dropping broadcasted `WmEvent` payloads under concurrent load.
- **Validation Condition**: Integration test streaming 1,000 rapid state queries while subscribing to events receives 100% of emitted events without drops.

## H5: TDD Benchmark Regression Suite
- **Description**: Automated Criterion/custom performance benchmarks measuring throughput, layout calculation time, and message pump latency before and after refactoring reliably quantify performance gains.
- **Validation Condition**: Benchmark suite executes via `mise run benchmark` and records reproducible before/after statistical deltas.
