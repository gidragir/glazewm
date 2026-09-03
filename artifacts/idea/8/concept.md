# Concept: Non-Blocking Event-Driven Viewport Engine & Zero-Allocation Layout Pipeline

## Definition
A non-blocking, event-driven animation and synchronization engine inspired by Niri and Rust asynchronous best practices, utilizing Tokio tick dispatching, zero-allocation container buffers, queued viewport transitions, and compile-time type-state safety.

## Value Proposition
Eliminates main-thread message loop stalls (0ms Win32 pump blocking), prevents asynchronous Win32 API race conditions, reduces heap churn during layout cycles to near-zero, and guarantees lossless bidirectional IPC communication under heavy event load.

## Core Mechanics
1. **Asynchronous Frame Dispatching**:
   - Viewport animation frames are scheduled via a non-blocking asynchronous ticker (`tokio::time::interval`).
   - Frame mutations and batch position commands are marshaled to the main OS thread via `Dispatcher` without blocking the Win32 `EventLoop`.
2. **Animation Coalescing & Transition Queuing (Niri model)**:
   - When a pan request arrives mid-animation, the engine finishes the ongoing step interpolation smoothly or repoints the curve towards the newly requested target without teleportation or frame drops.
3. **Thread-Safe Win32 Marshaling**:
   - Delayed UI mutations (`set_border_color`, `set_z_order`) route exclusively through the `Dispatcher` channel to ensure strict single-thread UI execution.
4. **Zero-Allocation Layout Pass**:
   - Container tree iterations and physical rect batch evaluations utilize pre-allocated scratch storage in `WmState`/`PendingSync`, eliminating ad-hoc `Vec` heap allocations during rapid cursor/focus passes.
5. **Lossless IPC Streaming**:
   - `IpcClient` uses multiplexed channels with dedicated task buffers, preventing event starvation and packet loss during concurrent command execution.
