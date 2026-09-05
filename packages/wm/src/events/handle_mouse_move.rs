use anyhow::Context;
#[cfg(target_os = "macos")]
use wm_common::try_warn;
use wm_platform::{MouseButton, MouseEvent, Point, PressedButtons};

use crate::{
  commands::container::set_focused_descendant, models::WindowContainer,
  traits::CommonGetters, user_config::UserConfig, wm_state::WmState,
};
#[cfg(target_os = "macos")]
use crate::{
  events::handle_window_moved_or_resized_end, traits::WindowGetters,
};

#[cfg(target_os = "macos")]
fn handle_macos_drag_end(
  button: MouseButton,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if button == MouseButton::Left {
    let active_drag_windows = state
      .windows()
      .into_iter()
      .filter(|window| window.active_drag().is_some());

    for window in active_drag_windows {
      let new_rect = try_warn!(window.native().frame());

      window.update_native_properties(|properties| {
        properties.frame = new_rect;
      });

      handle_window_moved_or_resized_end(&window, state, config)?;
    }
  }

  Ok(())
}

fn should_ignore_mouse_move(
  pressed_buttons: PressedButtons,
  state: &WmState,
  config: &UserConfig,
) -> bool {
  pressed_buttons.contains(&MouseButton::Left)
    || pressed_buttons.contains(&MouseButton::Right)
    || !state.is_focus_synced
    || !config.value.general.focus_follows_cursor
}

fn resolve_window_under_cursor(
  position: &Point,
  #[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
  window_below_cursor: Option<wm_platform::WindowId>,
  state: &WmState,
) -> anyhow::Result<Option<WindowContainer>> {
  #[cfg(target_os = "macos")]
  {
    Ok(window_below_cursor.and_then(|window_id| {
      state
        .windows()
        .into_iter()
        .find(|w| w.native().id() == window_id)
    }))
  }
  #[cfg(target_os = "windows")]
  {
    Ok(
      state
        .dispatcher
        .window_from_point(position)?
        .and_then(|native| state.window_from_native(&native)),
    )
  }
}

fn handle_cursor_window_focus(
  window: &WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  if focused_container.id() != window.id() {
    set_focused_descendant(&window.as_container(), None);
    state.pending_sync.queue_focus_change();
  }

  Ok(())
}

fn handle_cursor_monitor_focus(
  position: &Point,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let cursor_monitor = state
    .monitor_at_point(position)
    .context("No monitor under cursor.")?;

  let focused_monitor = state
    .focused_container()
    .context("No focused container.")?
    .monitor()
    .context("Focused container has no monitor.")?;

  // Avoid setting focus to the same monitor.
  if cursor_monitor.id() != focused_monitor.id() {
    set_focused_descendant(&cursor_monitor.as_container(), None);
    state.pending_sync.queue_focus_change();
  }

  Ok(())
}

pub fn handle_mouse_move(
  event: &MouseEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Ignore mouse move events if the WM is paused.
  if state.is_paused {
    return Ok(());
  }

  #[cfg(target_os = "macos")]
  if let MouseEvent::ButtonUp { button, .. } = event {
    return handle_macos_drag_end(*button, state, config);
  }

  if let MouseEvent::Move {
    pressed_buttons,
    window_below_cursor,
    position,
    ..
  } = event
  {
    if should_ignore_mouse_move(*pressed_buttons, state, config) {
      return Ok(());
    }

    let window_under_cursor =
      resolve_window_under_cursor(position, *window_below_cursor, state)?;

    // Set focus to whichever window or monitor is currently under the cursor.
    if let Some(window) = window_under_cursor {
      handle_cursor_window_focus(&window, state)?;
    } else {
      handle_cursor_monitor_focus(position, state)?;
    }
  }

  Ok(())
}
