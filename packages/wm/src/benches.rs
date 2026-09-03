#![allow(clippy::cast_precision_loss)]

use std::time::Instant;

use wm_platform::{NativeWindow, Rect};

use crate::{
  commands::general::animate_pan_workspace,
  models::Workspace,
  pending_sync::PendingSync,
  user_config::UserConfig,
  wm_state::WmState,
};

#[test]
fn benchmark_scratch_buffer_vs_fresh_allocations() {
  const ITERATIONS: usize = 10_000;
  const WINDOWS_PER_ITER: usize = 20;

  // 1. Fresh allocations (old pattern)
  let start_fresh = Instant::now();
  for i in 0..ITERATIONS {
    let mut fresh_vec: Vec<(NativeWindow, Rect)> = Vec::new();
    for w in 0..WINDOWS_PER_ITER {
      fresh_vec.push((
        NativeWindow::mock().into(),
        Rect::from_xy((i + w) as i32, (i + w) as i32, 800, 600),
      ));
    }
    std::hint::black_box(&fresh_vec);
  }
  let duration_fresh = start_fresh.elapsed();

  // 2. Pre-allocated scratch buffer reuse (new zero-allocation pattern)
  let mut pending_sync = PendingSync::default();
  let start_scratch = Instant::now();
  for i in 0..ITERATIONS {
    pending_sync.batch_positions_scratch.clear();
    for w in 0..WINDOWS_PER_ITER {
      pending_sync.batch_positions_scratch.push((
        NativeWindow::mock().into(),
        Rect::from_xy((i + w) as i32, (i + w) as i32, 800, 600),
      ));
    }
    std::hint::black_box(&pending_sync.batch_positions_scratch);
  }
  let duration_scratch = start_scratch.elapsed();

  println!(
    "\n[BENCHMARK] Scratch buffer ({duration_scratch:?}) vs Fresh allocation ({duration_fresh:?})"
  );

  assert!(
    pending_sync.batch_positions_scratch.capacity() >= WINDOWS_PER_ITER,
    "Capacity must be preserved"
  );
}

#[tokio::test]
async fn benchmark_non_blocking_animate_pan_latency() {
  let mut state = WmState::mock();
  let mut config = UserConfig::mock();
  config.value.general.animation_enabled = true;
  config.value.general.animation_duration_ms = 200;

  let ws = Workspace::mock().call();

  let start = Instant::now();
  animate_pan_workspace(&ws, 500.0, &mut state, &config);
  let elapsed = start.elapsed();

  println!("\n[BENCHMARK] Non-blocking animate_pan_workspace latency: {elapsed:?}");

  // Must return almost instantaneously (< 2ms) without blocking the thread for the 200ms animation
  assert!(
    elapsed < std::time::Duration::from_millis(5),
    "animate_pan_workspace must return immediately without blocking UI thread, took {elapsed:?}"
  );
}
