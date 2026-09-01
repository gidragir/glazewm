use anyhow::Context;
use tracing::info;

use super::activate_workspace;
use crate::{
  commands::{
    container::set_focused_descendant, workspace::deactivate_workspace,
  },
  models::WorkspaceTarget,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Focuses a workspace by a given target.
///
/// This target can be a workspace name, the most recently focused
/// workspace, the next workspace, the previous workspace, or the workspace
/// in a given direction from the currently focused workspace.
///
/// The workspace will be activated if it isn't already active.
pub fn focus_workspace(
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let (target_workspace_name, target_workspace) =
    state.workspace_by_target(&focused_workspace, target, config)?;

  // Retrieve or activate the target workspace by its name.
  let target_workspace = match target_workspace {
    Some(_) => anyhow::Ok(target_workspace),
    _ => match target_workspace_name {
      Some(name) => {
        activate_workspace(Some(&name), None, state, config)?;

        Ok(state.workspace_by_name(&name))
      }
      _ => Ok(None),
    },
  }?;

  if let Some(target_workspace) = target_workspace {
    info!("Focusing workspace: {target_workspace}");

    // Get the currently displayed workspace on the same monitor that the
    // workspace to focus is on.
    let displayed_workspace = target_workspace
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .context("No workspace is currently displayed.")?;

    // Set focus to whichever window last had focus in workspace. If the
    // workspace has no windows, then set focus to the workspace itself.
    let container_to_focus = target_workspace
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| target_workspace.clone().into());

    set_focused_descendant(&container_to_focus, None);
    state.pending_sync.queue_focus_change();

    // Display the workspace to switch focus to.
    state
      .pending_sync
      .queue_container_to_redraw(displayed_workspace)
      .queue_container_to_redraw(target_workspace);

    // Get empty workspace to destroy (if one is found). Cannot destroy
    // empty workspaces if they're the only workspace on the monitor.
    let workspace_to_destroy =
      state.workspaces().into_iter().find(|workspace| {
        !workspace.config().keep_alive
          && !workspace.has_children()
          && !workspace.is_displayed()
      });

    if let Some(workspace) = workspace_to_destroy {
      deactivate_workspace(workspace, state)?;
    }

    // Save the currently focused workspace as recent.
    state.recent_workspace_name = Some(focused_workspace.config().name);
    state.pending_sync.queue_cursor_jump();
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::TilingDirection;
  use wm_platform::{Direction, Rect};

  use super::*;
  use crate::{
    commands::container::attach_container,
    models::{Monitor, TilingWindow, Workspace},
  };

  #[test]
  fn test_focus_workspace_in_direction_on_multi_monitor() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win1 = TilingWindow::mock().call();
    let ws1 = Workspace::mock()
      .name("1".to_string())
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![win1.clone().into()])
      .call();

    let mon1 = Monitor::mock()
      .bounds(Rect::from_xy(0, 0, 1920, 1080))
      .working_area(Rect::from_xy(0, 0, 1920, 1080))
      .workspaces(vec![ws1.clone()])
      .call();

    let win2 = TilingWindow::mock().call();
    let ws2 = Workspace::mock()
      .name("2".to_string())
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![win2.clone().into()])
      .call();

    let mon2 = Monitor::mock()
      .bounds(Rect::from_xy(1920, 0, 1920, 1080))
      .working_area(Rect::from_xy(1920, 0, 1920, 1080))
      .workspaces(vec![ws2.clone()])
      .call();

    attach_container(
      &mon1.clone().into(),
      &state.root_container.clone().into(),
      None,
    )
    .unwrap();
    attach_container(
      &mon2.clone().into(),
      &state.root_container.clone().into(),
      None,
    )
    .unwrap();

    set_focused_descendant(&win1.clone().into(), None);
    assert_eq!(state.focused_container().unwrap().id(), win1.id());

    // Focus workspace to the Right (should focus ws2/win2 on monitor 2)
    focus_workspace(
      WorkspaceTarget::Direction(Direction::Right),
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(state.focused_container().unwrap().id(), win2.id());
    assert!(state.pending_sync.needs_cursor_jump());
    assert!(state.pending_sync.needs_focus_update());
  }

  #[test]
  fn test_focus_monitor_index() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win1 = TilingWindow::mock().call();
    let ws1 = Workspace::mock()
      .name("1".to_string())
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![win1.clone().into()])
      .call();

    let mon1 = Monitor::mock()
      .bounds(Rect::from_xy(0, 0, 1920, 1080))
      .working_area(Rect::from_xy(0, 0, 1920, 1080))
      .workspaces(vec![ws1.clone()])
      .call();

    let win2 = TilingWindow::mock().call();
    let ws2 = Workspace::mock()
      .name("2".to_string())
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![win2.clone().into()])
      .call();

    let mon2 = Monitor::mock()
      .bounds(Rect::from_xy(1920, 0, 1920, 1080))
      .working_area(Rect::from_xy(1920, 0, 1920, 1080))
      .workspaces(vec![ws2.clone()])
      .call();

    attach_container(
      &mon1.clone().into(),
      &state.root_container.clone().into(),
      None,
    )
    .unwrap();
    attach_container(
      &mon2.clone().into(),
      &state.root_container.clone().into(),
      None,
    )
    .unwrap();

    set_focused_descendant(&win1.clone().into(), None);

    // Focus monitor index 1 (mon2)
    crate::commands::monitor::focus_monitor(1, &mut state, &config)
      .unwrap();

    assert_eq!(state.focused_container().unwrap().id(), win2.id());
    assert!(state.pending_sync.needs_cursor_jump());
  }
}
