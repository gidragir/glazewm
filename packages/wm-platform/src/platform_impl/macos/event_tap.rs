use objc2_core_foundation::{
  CFMachPort, CFRetained, CFRunLoop, kCFRunLoopCommonModes,
};
use objc2_core_graphics::CGEvent;

use crate::{Dispatcher, Error, Result, ThreadBound};

/// Attaches a `CFMachPort` event tap to the current `CFRunLoop` in common modes,
/// enables the tap, and wraps it in a `ThreadBound` handle.
pub(crate) fn attach_tap_to_run_loop(
  tap_port: CFRetained<CFMachPort>,
  dispatcher: &Dispatcher,
) -> Result<ThreadBound<CFRetained<CFMachPort>>> {
  let loop_source =
    CFMachPort::new_run_loop_source(None, Some(&tap_port), 0)
      .ok_or_else(|| {
        Error::Platform("Failed to create loop source".to_string())
      })?;

  let current_loop = CFRunLoop::current().ok_or_else(|| {
    Error::Platform("Failed to get current run loop".to_string())
  })?;

  current_loop
    .add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

  CGEvent::tap_enable(&tap_port, true);

  Ok(ThreadBound::new(tap_port, dispatcher.clone()))
}
