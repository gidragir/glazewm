use anyhow::Context;
use tracing::info;
use wm_common::{TilingDirection, WindowState};

use crate::{
  commands::{
    container::{
      attach_container, move_container_within_tree, set_focused_descendant,
    },
    workspace::activate_workspace,
  },
  models::{
    Monitor, SplitContainer, WindowContainer, Workspace, WorkspaceTarget,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

fn resolve_target_workspace(
  current_workspace: &Workspace,
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Option<Workspace>> {
  let (target_workspace_name, target_workspace) =
    state.workspace_by_target(current_workspace, target, config)?;

  match target_workspace {
    Some(_) => Ok(target_workspace),
    None => match target_workspace_name {
      Some(name) => {
        activate_workspace(Some(&name), None, state, config)?;
        Ok(state.workspace_by_name(&name))
      }
      None => Ok(None),
    },
  }
}

fn adjust_window_for_target_monitor(
  window: &WindowContainer,
  current_monitor: &Monitor,
  target_monitor: &Monitor,
  target_workspace: &Workspace,
) -> anyhow::Result<()> {
  if current_monitor.has_dpi_difference(&target_monitor.clone().into())? {
    window.set_has_pending_dpi_adjustment(true);
  }

  if target_monitor.id() != current_monitor.id() {
    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&target_workspace.to_rect()?),
    );
  }

  if let WindowContainer::NonTilingWindow(non_tiling) = window {
    non_tiling.set_insertion_target(None);
  }

  Ok(())
}

fn insert_window_into_workspace(
  window: &WindowContainer,
  target_workspace: &Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let insertion_sibling = target_workspace
    .descendant_focus_order()
    .filter_map(|descendant| descendant.as_window_container().ok())
    .find(|descendant| descendant.state() == WindowState::Tiling);

  match (window.is_tiling_window(), insertion_sibling) {
    (true, Some(sibling)) => {
      move_container_within_tree(
        &window.clone().into(),
        &sibling.clone().parent().context("No parent.")?,
        sibling.index() + 1,
        state,
      )?;
    }
    (true, None) => {
      let split_container = SplitContainer::new(
        TilingDirection::Vertical,
        config.value.gaps.clone(),
      );
      attach_container(
        &split_container.clone().into(),
        &target_workspace.clone().into(),
        None,
      )?;
      move_container_within_tree(
        &window.clone().into(),
        &split_container.into(),
        0,
        state,
      )?;
    }
    (false, _) => {
      move_container_within_tree(
        &window.clone().into(),
        &target_workspace.clone().into(),
        target_workspace.child_count(),
        state,
      )?;
    }
  }

  Ok(())
}

fn queue_move_workspace_sync(
  window: WindowContainer,
  current_workspace: &Workspace,
  target_workspace: &Workspace,
  state: &mut WmState,
) {
  set_focused_descendant(&window.clone().into(), None);
  state.pending_sync.queue_focus_change().queue_cursor_jump();

  match window {
    WindowContainer::NonTilingWindow(_) => {
      state.pending_sync.queue_container_to_redraw(window);
    }
    WindowContainer::TilingWindow(_) => {
      state
        .pending_sync
        .queue_containers_to_redraw(current_workspace.tiling_children())
        .queue_containers_to_redraw(target_workspace.tiling_children());
    }
  }

  state
    .pending_sync
    .queue_workspace_to_reorder(target_workspace.clone());
}

pub fn move_window_to_workspace(
  window: WindowContainer,
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let current_workspace = window.workspace().context("No workspace.")?;
  let current_monitor =
    current_workspace.monitor().context("No monitor.")?;

  let Some(target_workspace) =
    resolve_target_workspace(&current_workspace, target, state, config)?
  else {
    return Ok(());
  };

  if target_workspace.id() == current_workspace.id() {
    return Ok(());
  }

  info!(
    "Moving window to workspace: '{}'.",
    target_workspace.config().name
  );

  let target_monitor =
    target_workspace.monitor().context("No monitor.")?;

  adjust_window_for_target_monitor(
    &window,
    &current_monitor,
    &target_monitor,
    &target_workspace,
  )?;

  let focus_reset_target = if target_workspace.is_displayed() {
    None
  } else {
    target_monitor.descendant_focus_order().next()
  };

  insert_window_into_workspace(&window, &target_workspace, state, config)?;

  if let Some(focus_reset_target) = focus_reset_target {
    set_focused_descendant(
      &focus_reset_target,
      Some(&target_monitor.into()),
    );
  }

  queue_move_workspace_sync(
    window,
    &current_workspace,
    &target_workspace,
    state,
  );

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
  fn test_move_window_in_direction_to_another_monitor() {
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

    // Move win1 to monitor on the right
    move_window_to_workspace(
      win1.clone().into(),
      WorkspaceTarget::Direction(Direction::Right),
      &mut state,
      &config,
    )
    .unwrap();

    // win1 should now be in ws2 and still retain focused state
    assert_eq!(win1.workspace().unwrap().id(), ws2.id());
    assert_eq!(state.focused_container().unwrap().id(), win1.id());
    assert!(state.pending_sync.needs_cursor_jump());
  }
}
