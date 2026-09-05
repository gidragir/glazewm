use anyhow::Context;
use tracing::info;
use wm_common::{DisplayState, WindowRuleEvent, WmEvent};
use wm_platform::NativeWindow;

use crate::{
  commands::{
    container::set_focused_descendant, window::run_window_rules,
  },
  models::{Container, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

fn is_unsolicited_focus(
  window: &WindowContainer,
  workspace: &Workspace,
) -> bool {
  let is_on_displayed_workspace = workspace
    .monitor()
    .and_then(|m| m.displayed_workspace())
    .is_some_and(|displayed_ws| displayed_ws.id() == workspace.id());

  let is_visible = matches!(
    window.display_state(),
    DisplayState::Shown | DisplayState::Showing
  );

  !is_visible || !is_on_displayed_workspace
}

fn auto_pan_viewport_if_needed(
  workspace: &Workspace,
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) {
  let (Ok(workspace_rect), Ok(window_rect)) =
    (workspace.to_rect(), window.to_rect())
  else {
    return;
  };

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
      workspace,
      target_offset,
      state,
      config,
    );
    state
      .pending_sync
      .queue_container_to_redraw(workspace.clone());
  }
}

fn handle_managed_window_focus(
  window: &WindowContainer,
  focused_container: &Container,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let workspace = window.workspace().context("No workspace")?;

  // Native focus has been synced to the WM's focused container.
  if *focused_container == window.clone().into() {
    state.is_focus_synced = true;
    state.pending_sync.queue_workspace_to_reorder(workspace);
    state.pending_sync.queue_focused_effect_update();
    return Ok(());
  }

  if is_unsolicited_focus(window, &workspace) {
    info!("Prevented unsolicited background focus from: {window}");
    state.pending_sync.queue_focus_change();
    return Ok(());
  }

  info!("Window manually focused: {window}");
  state.pending_sync.queue_focused_effect_update();
  set_focused_descendant(&window.clone().into(), None);

  auto_pan_viewport_if_needed(&workspace, window, state, config);

  run_window_rules(
    window.clone(),
    &WindowRuleEvent::Focus,
    state,
    config,
  )?;

  state.is_focus_synced = true;
  state.pending_sync.queue_workspace_to_reorder(workspace);

  state.emit_event(WmEvent::FocusChanged {
    focused_container: window.to_dto()?,
  });

  Ok(())
}

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

  // Handle overriding focus on close/minimize.
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
    handle_managed_window_focus(&window, &focused_container, state, config)?;
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

