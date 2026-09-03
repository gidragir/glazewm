# Prior Art: Iteration 8

## Standard Patterns
- **Actor / Dispatcher Pattern**: Separating background async I/O / timers (Tokio actor) from thread-bound native OS GUI subsystems via non-blocking mpsc message passing (used in VS Code, Alacritty, Chromium UI loop).
- **Spring / Easing Interpolation Driver**: Niri's Wayland scroll engine driving sub-pixel matrix offsets per frame tick with state transition coalescing.
- **Scratch Buffer / Object Pooling**: Pre-allocated memory reuse patterns from high-frequency trading (HFT) and game engines (Bevy ECS, Unreal Engine slate layout passes).
- **Multiplexed Channel Router**: Demultiplexing request-reply frames and pub-sub streams over a single connection with dedicated async queues (Tonic gRPC, Tokio Tower).
- **Type-State Pattern**: Compile-time protocol state enforcement using Rust zero-sized phantom types (Apollo GraphQL Rust Handbook Ch. 7, embedded-hal).

## Alternative Approaches
- **Native OS Timer Hook (`SetTimer` / `WM_TIMER`)**: Driving animations entirely within the Win32 message loop via OS timer callbacks. *Drawback*: Windows `WM_TIMER` has low resolution (~15.6ms jitter) and poor coalescing compared to Tokio micro-timers.
- **Direct Multi-Threaded Win32 Invocations**: Calling `SetWindowPos` directly from Tokio worker threads with raw handles. *Drawback*: Breaches single-thread UI model, risks DWM corruption and unpredictable deadlocks with Win32 message processing.
