# Summary: Iteration 8 Q&A Synthesis

## Initial Proposition
Conduct a comprehensive review and architectural audit of iterations 1-7 against Rust async patterns and Apollo GraphQL Rust best practices, formalizing improvements as Iteration 8.

## Clarifications
- **Frame Driving**: Use non-blocking async architecture aligned with Niri and Apollo Rust best practices (Tokio interval + Dispatcher marshaling).
- **Scope & Delivery**: Comprehensive multi-layer improvements rolled out via structured iterations and tasklists following Test-Driven Development (TDD).
- **Animation Interruption**: Follow Niri's viewport mechanics by allowing active transitions to complete or seamlessly redirect toward new targets without visual jitter.
- **Resource Management**: Apply Niri-grade zero-allocation scratch buffers and pass-by-reference idioms in high-frequency paths.
- **Verification Strategy**: Automated performance benchmarking (before vs. after profiling metrics) paired with unit and integration test suites.

## Perceived Pitfalls
- **Main-Thread Contention**: Risk of UI thread stalls if frame timers execute synchronous sleeps inside `platform_sync`.
- **Win32 Concurrent Invocations**: Asynchronous background tasks directly mutating `HWND` outside the Win32 message pump thread.
- **Multiplexed IPC Loss**: Dropping broadcasted window events when polling for single-request client responses in `wm-ipc-client`.
- **Animation Race Conditions**: Multiple overlapping auto-pan requests causing viewport oscillation if transitions are not cleanly queued or coalesced.
