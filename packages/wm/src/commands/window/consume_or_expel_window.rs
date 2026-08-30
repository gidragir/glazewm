use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::Direction;

use crate::{
  commands::container::{
    flatten_split_container, move_container_within_tree,
    wrap_in_split_container,
  },
  models::{SplitContainer, TilingContainer, TilingWindow, WindowContainer},
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn consume_or_expel_window(
  window: WindowContainer,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let WindowContainer::TilingWindow(window_to_act) = window else {
    return Ok(());
  };

  let parent = window_to_act.parent().context("No parent container.")?;

  // Check if the window is inside a vertical split container (i.e. inside a column).
  if let Some(split_parent) = parent.as_split().cloned() {
    if split_parent.tiling_direction() == TilingDirection::Vertical {
      return expel_window_from_column(
        &window_to_act,
        split_parent,
        direction,
        state,
      );
    }
  }

  // Otherwise, if the window is in a horizontal container (workspace), consume it into the sibling column.
  consume_window_into_column(&window_to_act, direction, state, config)
}

fn expel_window_from_column(
  window_to_expel: &TilingWindow,
  split_parent: SplitContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let workspace = split_parent
    .parent()
    .context("Column has no parent container.")?;

  let target_index = match direction {
    Direction::Left => split_parent.index(),
    _ => split_parent.index() + 1,
  };

  move_container_within_tree(
    &window_to_expel.clone().into(),
    &workspace,
    target_index,
    state,
  )?;

  if split_parent.child_count() == 1 {
    flatten_split_container(split_parent)?;
  }

  state
    .pending_sync
    .queue_containers_to_redraw(workspace.tiling_children())
    .queue_container_to_redraw(window_to_expel.clone());

  Ok(())
}

fn consume_window_into_column(
  window_to_consume: &TilingWindow,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let sibling = match direction {
    Direction::Left => window_to_consume
      .prev_siblings()
      .find_map(|s| s.as_tiling_container().ok()),
    Direction::Right => window_to_consume
      .next_siblings()
      .find_map(|s| s.as_tiling_container().ok()),
    _ => None,
  };

  let Some(target_sibling) = sibling else {
    // No sibling in the given direction to consume into.
    return Ok(());
  };

  let parent = window_to_consume.parent().context("No parent.")?;

  match target_sibling {
    TilingContainer::Split(sibling_split) => {
      let target_index = match direction {
        Direction::Left => sibling_split.child_count(),
        _ => 0,
      };

      move_container_within_tree(
        &window_to_consume.clone().into(),
        &sibling_split.clone().into(),
        target_index,
        state,
      )?;

      state
        .pending_sync
        .queue_container_to_redraw(sibling_split)
        .queue_containers_to_redraw(parent.tiling_children());
    }
    TilingContainer::TilingWindow(sibling_window) => {
      let split_container = SplitContainer::new(
        TilingDirection::Vertical,
        config.value.gaps.clone(),
      );

      wrap_in_split_container(
        &split_container,
        &parent,
        &[sibling_window.into()],
      )?;

      let target_index = match direction {
        Direction::Left => 1,
        _ => 0,
      };

      move_container_within_tree(
        &window_to_consume.clone().into(),
        &split_container.clone().into(),
        target_index,
        state,
      )?;

      state
        .pending_sync
        .queue_container_to_redraw(split_container)
        .queue_containers_to_redraw(parent.tiling_children());
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::TilingDirection;
  use wm_platform::Direction;

  use super::*;
  use crate::{
    models::{Monitor, Workspace},
    traits::{CommonGetters, TilingDirectionGetters},
  };

  #[test]
  fn test_consume_window_left_into_tiling_window() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win_a = TilingWindow::mock().call();
    let win_b = TilingWindow::mock().call();

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
      .call();

    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    // Consume win_b into win_a (to the left)
    consume_or_expel_window(
      win_b.clone().into(),
      &Direction::Left,
      &mut state,
      &config,
    )
    .unwrap();

    // Workspace should now contain 1 column (SplitContainer) with 2 children [win_a, win_b]
    assert_eq!(workspace.child_count(), 1);
    let split = workspace
      .tiling_children()
      .next()
      .unwrap()
      .as_split()
      .cloned()
      .unwrap();

    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(split.child_count(), 2);
    assert_eq!(win_a.index(), 0);
    assert_eq!(win_b.index(), 1);
  }

  #[test]
  fn test_consume_window_right_into_tiling_window() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win_a = TilingWindow::mock().call();
    let win_b = TilingWindow::mock().call();

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
      .call();

    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    // Consume win_a into win_b (to the right)
    consume_or_expel_window(
      win_a.clone().into(),
      &Direction::Right,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace.child_count(), 1);
    let split = workspace
      .tiling_children()
      .next()
      .unwrap()
      .as_split()
      .cloned()
      .unwrap();

    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(split.child_count(), 2);
    assert_eq!(win_a.index(), 0);
    assert_eq!(win_b.index(), 1);
  }

  #[test]
  fn test_expel_window_left_and_right() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();

    let win_a = TilingWindow::mock().call();
    let win_b = TilingWindow::mock().call();

    let split = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
      .call();

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![split.clone().into()])
      .call();

    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    // Expel win_b to the left of the column
    consume_or_expel_window(
      win_b.clone().into(),
      &Direction::Left,
      &mut state,
      &config,
    )
    .unwrap();

    // Since win_a was left alone in split, split flattens.
    // Workspace now has [win_b, win_a]
    assert_eq!(workspace.child_count(), 2);
    assert_eq!(win_b.index(), 0);
    assert_eq!(win_a.index(), 1);
  }
}
