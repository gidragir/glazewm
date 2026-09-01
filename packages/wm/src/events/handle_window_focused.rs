use anyhow::Context;
use tracing::info;
use wm_common::{DisplayState, WindowRuleEvent, WmEvent};
use wm_platform::NativeWindow;

use crate::{
  commands::{
    container::set_focused_descendant, window::run_window_rules,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_focused(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);
  let focused_container =
    state.focused_container().context("No focused container.")?;

  // Update the focus sync state. If the OS focused window is not same as
  // the WM's focused container, then the focus is not synced.
  state.is_focus_synced = match focused_container.as_window_container() {
    Ok(window) => *window.native() == *native_window,
    _ => native_window.is_desktop_window().unwrap_or(false),
  };

  // Handle overriding focus on close/minimize. After a window is closed
  // or minimized, the OS or the closed application might automatically
  // switch focus to a different window. To force focus to go to the WM's
  // target focus container, we reassign any focus events 100ms after
  // close/minimize. This will cause focus to briefly flicker to the OS
  // focus target and then to the WM's focus target.
  if should_override_focus(state) {
    state.pending_sync.queue_focus_change();
    return Ok(());
  }

  // Ignore the focus event if window is being hidden by the WM.
  if let Some(window) = &found_window
    && window.display_state() == DisplayState::Hiding
  {
    return Ok(());
  }

  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace")?;

    // Native focus has been synced to the WM's focused container.
    if focused_container == window.clone().into() {
      state.is_focus_synced = true;
      state.pending_sync.queue_workspace_to_reorder(workspace);
      state.pending_sync.queue_focused_effect_update();
      return Ok(());
    }

    // Check if the window is visible on a currently displayed workspace.
    // If the window is hidden, minimized, or on an inactive workspace, it is
    // an unsolicited background focus signal (e.g. from Discord, Telegram, etc.).
    // We reject the focus shift and restore OS focus back to the user's
    // active container.
    let is_on_displayed_workspace = workspace
      .monitor()
      .and_then(|m| m.displayed_workspace())
      .is_some_and(|displayed_ws| displayed_ws.id() == workspace.id());

    if window.display_state() != DisplayState::Shown || !is_on_displayed_workspace {
      info!("Prevented unsolicited background focus from: {window}");
      state.pending_sync.queue_focus_change();
      return Ok(());
    }

    info!("Window manually focused: {window}");

    // Focus effect should be updated for legitimate focus change.
    state.pending_sync.queue_focused_effect_update();

    // Update the WM's focus state.
    set_focused_descendant(&window.clone().into(), None);

    // Auto-pan viewport to bring focused window into view if needed.
    if let (Ok(workspace_rect), Ok(window_rect)) =
      (workspace.to_rect(), window.to_rect())
    {
      #[allow(clippy::cast_possible_truncation)]
      let current_offset = workspace.offset_x() as i32;
      let mut new_offset = current_offset;

      if window_rect.left < workspace_rect.left {
        let delta = window_rect.left - workspace_rect.left;
        new_offset = (current_offset + delta).max(0);
      } else if window_rect.right > workspace_rect.right {
        let delta = window_rect.right - workspace_rect.right;
        new_offset = (current_offset + delta).max(0);
      }

      let target_offset = f64::from(new_offset);
      if (target_offset - workspace.offset_x()).abs() > 0.001 {
        crate::commands::general::animate_pan_workspace(
          &workspace,
          target_offset,
          config,
        );
        state
          .pending_sync
          .queue_container_to_redraw(workspace.clone());
      }
    }

    // Run window rules for focus events.
    run_window_rules(
      window.clone(),
      &WindowRuleEvent::Focus,
      state,
      config,
    )?;

    state.is_focus_synced = true;
    state.pending_sync.queue_workspace_to_reorder(workspace);

    // Broadcast the focus change event.
    state.emit_event(WmEvent::FocusChanged {
      focused_container: window.to_dto()?,
    });
  } else {
    // An unmanaged or desktop window received focus.
    state.pending_sync.queue_focused_effect_update();
  }

  Ok(())
}

/// Returns true if focus should be reassigned to the WM's focus container.
fn should_override_focus(state: &WmState) -> bool {
  let has_recent_unmanage = state
    .unmanaged_or_minimized_timestamp
    .is_some_and(|time| time.elapsed().as_millis() < 100);

  has_recent_unmanage && !state.is_focus_synced
}

