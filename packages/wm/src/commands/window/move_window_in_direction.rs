use anyhow::Context;
use wm_common::{TilingDirection, WindowState};
use wm_platform::{Direction, Rect};

use crate::{
  commands::container::{
    flatten_split_container, move_container_within_tree,
    set_focused_descendant,
  },
  models::{
    DirectionContainer, Monitor, NonTilingWindow, TilingContainer,
    TilingWindow, WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// The distance in pixels to snap the window to the monitor's edge.
const SNAP_DISTANCE: i32 = 15;

pub fn move_window_in_direction(
  window: WindowContainer,
  direction: &Direction,
  state: &mut WmState,
  _config: &UserConfig,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      move_tiling_window(window, direction, state)
    }
    WindowContainer::NonTilingWindow(non_tiling_window) => {
      match non_tiling_window.state() {
        WindowState::Floating(_) => {
          move_floating_window(non_tiling_window, direction, state)
        }
        WindowState::Fullscreen(_) => move_to_workspace_in_direction(
          &non_tiling_window.into(),
          direction,
          state,
        ),
        _ => Ok(()),
      }
    }
  }
}

fn move_tiling_window(
  window_to_move: TilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Flatten the parent split container if it only contains the window.
  if let Some(split_parent) = window_to_move
    .parent()
    .and_then(|parent| parent.as_split().cloned())
  {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent)?;
    }
  }

  let parent = window_to_move
    .direction_container()
    .context("No direction container.")?;

  let has_matching_tiling_direction = parent.tiling_direction()
    == TilingDirection::from_direction(direction);

  // Attempt to swap or move the window into a sibling container.
  if has_matching_tiling_direction {
    if let Some(sibling) =
      tiling_sibling_in_direction(&window_to_move, direction)
    {
      return move_to_sibling_container(
        window_to_move,
        sibling,
        direction,
        state,
      );
    }
  }

  // Attempt to move the window to workspace in given direction.
  if (has_matching_tiling_direction
    || window_to_move.tiling_siblings().count() == 0)
    && parent.is_workspace()
  {
    return move_to_workspace_in_direction(
      &window_to_move.into(),
      direction,
      state,
    );
  }

  // The window cannot be moved within the parent container, so traverse
  // upwards to find an ancestor that has the correct tiling direction.
  let target_ancestor = parent.ancestors().find_map(|ancestor| {
    ancestor.as_direction_container().ok().filter(|ancestor| {
      ancestor.tiling_direction()
        == TilingDirection::from_direction(direction)
    })
  });

  if let Some(target_ancestor) = target_ancestor {
    insert_into_ancestor(
      &window_to_move,
      &target_ancestor,
      direction,
      state,
    )?;
  }

  Ok(())
}

/// Gets the next sibling `TilingWindow` or `SplitContainer` in the given
/// direction.
fn tiling_sibling_in_direction(
  window: &TilingWindow,
  direction: &Direction,
) -> Option<TilingContainer> {
  match direction {
    Direction::Up | Direction::Left => window
      .prev_siblings()
      .find_map(|sibling| sibling.as_tiling_container().ok()),
    _ => window
      .next_siblings()
      .find_map(|sibling| sibling.as_tiling_container().ok()),
  }
}

fn move_to_sibling_container(
  window_to_move: TilingWindow,
  target_sibling: TilingContainer,
  _direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window_to_move.parent().context("No parent.")?;

  // Swap the window with sibling in given direction.
  move_container_within_tree(
    &window_to_move.clone().into(),
    &parent,
    target_sibling.index(),
    state,
  )?;

  state
    .pending_sync
    .queue_container_to_redraw(target_sibling)
    .queue_container_to_redraw(window_to_move);

  Ok(())
}

fn move_to_workspace_in_direction(
  window_to_move: &WindowContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window_to_move.parent().context("No parent.")?;
  let workspace = window_to_move.workspace().context("No workspace.")?;
  let monitor = parent.monitor().context("No monitor.")?;

  let target_workspace = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|monitor| monitor.displayed_workspace());

  if let Some(target_workspace) = target_workspace {
    // Since the window is crossing monitors, adjustments might need to be
    // made because of DPI.
    if monitor.has_dpi_difference(&target_workspace.clone().into())? {
      window_to_move.set_has_pending_dpi_adjustment(true);
    }

    // Update floating placement since the window has to cross monitors.
    window_to_move.set_floating_placement(
      window_to_move
        .floating_placement()
        .translate_to_center(&target_workspace.to_rect()?),
    );

    if let WindowContainer::NonTilingWindow(window_to_move) =
      &window_to_move
    {
      window_to_move.set_insertion_target(None);
    }

    let target_index = match direction {
      Direction::Down | Direction::Right => 0,
      _ => target_workspace.child_count(),
    };

    // Focus should be reassigned within the original workspace after the
    // window is moved out. For example, if the focus order is 1. tiling
    // window and 2. fullscreen window, then we'd want to retain focus on a
    // tiling window on move.
    let focus_target = state.focus_target_after_removal(window_to_move);

    move_container_within_tree(
      &window_to_move.clone().into(),
      &target_workspace.clone().into(),
      target_index,
      state,
    )?;

    if let Some(focus_target) = focus_target {
      set_focused_descendant(
        &focus_target,
        Some(&workspace.clone().into()),
      );
    }

    state
      .pending_sync
      .queue_container_to_redraw(window_to_move.clone())
      .queue_containers_to_redraw(target_workspace.tiling_children())
      .queue_containers_to_redraw(parent.tiling_children())
      .queue_cursor_jump()
      .queue_workspace_to_reorder(target_workspace);
  }

  Ok(())
}


fn insert_into_ancestor(
  window_to_move: &TilingWindow,
  target_ancestor: &DirectionContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Traverse upwards to find container whose parent is the target
  // ancestor. Then, depending on the direction, insert before or after
  // that container.
  let window_ancestor = window_to_move
    .ancestors()
    .find(|container| {
      container
        .parent()
        .is_some_and(|parent| parent == target_ancestor.clone().into())
    })
    .context("Window ancestor not found.")?;

  let target_index = match direction {
    Direction::Up | Direction::Left => window_ancestor.index(),
    _ => window_ancestor.index() + 1,
  };

  // Move the window into the container above.
  move_container_within_tree(
    &window_to_move.clone().into(),
    &target_ancestor.clone().into(),
    target_index,
    state,
  )?;

  state
    .pending_sync
    .queue_containers_to_redraw(target_ancestor.tiling_children());

  Ok(())
}

fn move_floating_window(
  window_to_move: NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let new_position =
    new_floating_position(&window_to_move, direction, state)?;

  if let Some((position_rect, target_monitor)) = new_position {
    let monitor = window_to_move.monitor().context("No monitor.")?;

    // Mark window as needing DPI adjustment if it crosses monitors. The
    // handler for `PlatformEvent::LocationChanged` will update the
    // window's workspace if it goes out of bounds of its current
    // workspace.
    if monitor.id() != target_monitor.id()
      && monitor.has_dpi_difference(&target_monitor.into())?
    {
      window_to_move.set_has_pending_dpi_adjustment(true);
    }

    window_to_move.set_floating_placement(position_rect);
    state.pending_sync.queue_container_to_redraw(window_to_move);
  }

  Ok(())
}

/// Returns a tuple of the new floating position and the target monitor.
fn new_floating_position(
  window_to_move: &NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<Option<(Rect, Monitor)>> {
  let monitor = window_to_move.monitor().context("No monitor.")?;
  let monitor_rect = monitor.native_properties().working_area;
  let window_pos = window_to_move.native_properties().frame;

  let is_on_monitor_edge = match direction {
    Direction::Up => window_pos.top == monitor_rect.top,
    Direction::Down => window_pos.bottom == monitor_rect.bottom,
    Direction::Left => window_pos.left == monitor_rect.left,
    Direction::Right => window_pos.right == monitor_rect.right,
  };

  // Window is on the edge of the monitor and should be moved to a
  // different monitor in the given direction.
  if is_on_monitor_edge {
    let next_monitor = state.monitor_in_direction(&monitor, direction)?;

    if let Some(next_monitor) = next_monitor {
      let monitor_rect = next_monitor.native().working_area()?.clone();

      let position = snap_to_monitor_edge(
        &window_pos,
        &monitor_rect,
        &direction.inverse(),
      )
      .clamp(&monitor_rect);

      return Ok(Some((position, next_monitor)));
    }

    return Ok(None);
  }

  let (monitor_length, window_length) = match direction {
    Direction::Up | Direction::Down => {
      (monitor_rect.height(), window_pos.height())
    }
    _ => (monitor_rect.width(), window_pos.width()),
  };

  let length_delta = monitor_length - window_length;

  // Calculate the distance the window should move based on the ratio of
  // the window's length to the monitor's length.
  #[allow(clippy::cast_precision_loss)]
  let move_distance = match window_length as f32 / monitor_length as f32 {
    x if (0.0..0.2).contains(&x) => length_delta / 5,
    x if (0.2..0.4).contains(&x) => length_delta / 4,
    x if (0.4..0.6).contains(&x) => length_delta / 3,
    _ => length_delta / 2,
  };

  // Snap the window to the current monitor's edge if it's within 15px of
  // it after the move.
  let should_snap_to_edge = match direction {
    Direction::Up => {
      window_pos.top - move_distance - SNAP_DISTANCE < monitor_rect.top
    }
    Direction::Down => {
      window_pos.bottom + move_distance + SNAP_DISTANCE
        > monitor_rect.bottom
    }
    Direction::Left => {
      window_pos.left - move_distance - SNAP_DISTANCE < monitor_rect.left
    }
    Direction::Right => {
      window_pos.right + move_distance + SNAP_DISTANCE > monitor_rect.right
    }
  };

  if should_snap_to_edge {
    let position =
      snap_to_monitor_edge(&window_pos, &monitor_rect, direction);

    return Ok(Some((position, monitor)));
  }

  // Snap the window to the current monitor's inverse edge if it's in
  // between two monitors or outside the bounds of the current monitor.
  let should_snap_to_inverse_edge = match direction {
    Direction::Up => window_pos.bottom > monitor_rect.bottom,
    Direction::Down => window_pos.top < monitor_rect.top,
    Direction::Left => window_pos.right > monitor_rect.right,
    Direction::Right => window_pos.left < monitor_rect.left,
  };

  let position = if should_snap_to_inverse_edge {
    snap_to_monitor_edge(&window_pos, &monitor_rect, &direction.inverse())
  } else {
    window_pos.translate_in_direction(direction, move_distance)
  };

  Ok(Some((position, monitor)))
}

fn snap_to_monitor_edge(
  window_pos: &Rect,
  monitor_rect: &Rect,
  edge: &Direction,
) -> Rect {
  let (x, y) = match edge {
    Direction::Up => (window_pos.x(), monitor_rect.top),
    Direction::Down => {
      (window_pos.x(), monitor_rect.bottom - window_pos.height())
    }
    Direction::Left => (monitor_rect.left, window_pos.y()),
    Direction::Right => {
      (monitor_rect.right - window_pos.width(), window_pos.y())
    }
  };

  window_pos.translate_to_coordinates(x, y)
}

#[cfg(test)]
mod tests {
  use wm_common::TilingDirection;
  use wm_platform::Direction;

  use super::*;
  use crate::{
    models::{SplitContainer, Workspace},
    traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
  };

  #[test]
  fn vertical_move_on_horizontal_workspace_preserves_layout_and_sizes() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win1 = TilingWindow::mock().tiling_size(0.25).call();
    let win2 = TilingWindow::mock().tiling_size(0.75).call();
    let win3 = TilingWindow::mock().tiling_size(0.50).call();
    let win4 = TilingWindow::mock().tiling_size(0.50).call();

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![
        win1.clone().into(),
        win2.clone().into(),
        win3.clone().into(),
        win4.clone().into(),
      ])
      .call();

    let monitor = Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let _ = monitor;

    // Moving win4 Down should be a safe no-op on infinite horizontal canvas
    move_window_in_direction(
      win4.clone().into(),
      &Direction::Down,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(workspace.child_count(), 4);
    assert!((win1.tiling_size() - 0.25).abs() < f32::EPSILON);
    assert!((win2.tiling_size() - 0.75).abs() < f32::EPSILON);
    assert!((win3.tiling_size() - 0.50).abs() < f32::EPSILON);
    assert!((win4.tiling_size() - 0.50).abs() < f32::EPSILON);

    // Moving win4 Up should also be a safe no-op
    move_window_in_direction(
      win4.clone().into(),
      &Direction::Up,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(workspace.child_count(), 4);
    assert!((win1.tiling_size() - 0.25).abs() < f32::EPSILON);
    assert!((win2.tiling_size() - 0.75).abs() < f32::EPSILON);
    assert!((win3.tiling_size() - 0.50).abs() < f32::EPSILON);
    assert!((win4.tiling_size() - 0.50).abs() < f32::EPSILON);
  }

  #[test]
  fn vertical_move_within_vertical_split_swaps_siblings() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win_a = TilingWindow::mock().tiling_size(0.4).call();
    let win_b = TilingWindow::mock().tiling_size(0.6).call();

    let split = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
      .call();
    split.set_tiling_size(0.5);

    let win_c = TilingWindow::mock().tiling_size(0.5).call();

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![split.clone().into(), win_c.clone().into()])
      .call();

    let monitor = Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let _ = monitor;

    // Moving win_a Down should swap with win_b in the vertical column
    move_window_in_direction(
      win_a.clone().into(),
      &Direction::Down,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(split.child_count(), 2);
    assert_eq!(win_b.index(), 0);
    assert_eq!(win_a.index(), 1);
    assert!((split.tiling_size() - 0.5).abs() < f32::EPSILON);
    assert!((win_c.tiling_size() - 0.5).abs() < f32::EPSILON);

    // Moving win_a Down again (now at bottom) should safely no-op
    move_window_in_direction(
      win_a.clone().into(),
      &Direction::Down,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(split.child_count(), 2);
    assert_eq!(win_a.index(), 1);
  }

  #[test]
  fn horizontal_move_swaps_with_split_container_without_merging() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win_a = TilingWindow::mock().call();
    let win_b = TilingWindow::mock().call();

    let split = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
      .call();

    let win_c = TilingWindow::mock().call();

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![split.clone().into(), win_c.clone().into()])
      .call();

    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    // Moving win_c Left should swap places with the `split` column, NOT merge into it
    move_window_in_direction(
      win_c.clone().into(),
      &Direction::Left,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace.child_count(), 2);
    assert_eq!(win_c.index(), 0);
    assert_eq!(split.index(), 1);
    assert_eq!(split.child_count(), 2);
    assert_eq!(win_a.index(), 0);
    assert_eq!(win_b.index(), 1);
  }
}
