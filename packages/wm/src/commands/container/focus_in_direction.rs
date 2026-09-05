use anyhow::Context;
use wm_common::{TilingDirection, WindowState};
use wm_platform::Direction;

use super::set_focused_descendant;
use crate::{
  models::{Container, TilingContainer},
  traits::{CommonGetters, TilingDirectionGetters, WindowGetters},
  wm_state::WmState,
};

pub fn focus_in_direction(
  origin_container: &Container,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let focus_target = match origin_container {
    Container::TilingWindow(_) => {
      // In infinite horizontal canvas workspaces, directional focus stays
      // strictly within the current workspace and does not jump across monitors.
      tiling_focus_target(origin_container, direction)?
    }
    Container::NonTilingWindow(non_tiling_window) => {
      if matches!(non_tiling_window.state(), WindowState::Floating(_)) {
        floating_focus_target(origin_container, direction)
      } else if matches!(non_tiling_window.state(), WindowState::Fullscreen(_)) {
        fullscreen_focus_target(non_tiling_window, direction)
      } else {
        None
      }
    }
    _ => None,
  };

  // Set focus to the target container.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change().queue_cursor_jump();
  }

  Ok(())
}

fn fullscreen_focus_target(
  origin_window: &crate::models::NonTilingWindow,
  direction: &Direction,
) -> Option<Container> {
  if !matches!(direction, Direction::Left | Direction::Right) {
    return None;
  }

  let insertion_target = origin_window.insertion_target()?;

  let column_or_window = insertion_target.target_parent;
  let siblings = match direction {
    Direction::Left => column_or_window
      .prev_siblings()
      .find_map(|c| c.as_tiling_container().ok()),
    Direction::Right => column_or_window
      .next_siblings()
      .find_map(|c| c.as_tiling_container().ok()),
    _ => None,
  };

  if let Some(sibling) = siblings {
    return match sibling {
      TilingContainer::TilingWindow(_) => Some(sibling.into()),
      TilingContainer::Split(split) => split
        .descendant_in_direction(&direction.inverse())
        .map(Into::into),
    };
  }

  None
}

fn floating_focus_target(
  origin_container: &Container,
  direction: &Direction,
) -> Option<Container> {
  let is_floating = |sibling: &Container| {
    sibling.as_non_tiling_window().is_some_and(|window| {
      matches!(window.state(), WindowState::Floating(_))
    })
  };

  let mut floating_siblings =
    origin_container.siblings().filter(is_floating);

  // Wrap if next/previous floating window is not found.
  match direction {
    Direction::Left => origin_container
      .next_siblings()
      .find(is_floating)
      .or_else(|| floating_siblings.last()),
    Direction::Right => origin_container
      .prev_siblings()
      .find(is_floating)
      .or_else(|| floating_siblings.next()),
    // Cannot focus vertically from a floating window.
    _ => None,
  }
}

/// Gets a focus target within the current workspace. Traverse upwards from
/// the origin container to find an adjacent container that can be focused.
fn tiling_focus_target(
  origin_container: &Container,
  direction: &Direction,
) -> anyhow::Result<Option<Container>> {
  let tiling_direction = TilingDirection::from_direction(direction);
  let mut origin_or_ancestor = origin_container.clone();

  // Traverse upwards from the focused container. Stop searching when a
  // workspace is encountered.
  while !origin_or_ancestor.is_workspace() {
    let parent = origin_or_ancestor
      .parent()
      .and_then(|parent| parent.as_direction_container().ok())
      .context("No direction container.")?;

    // Skip if the tiling direction doesn't match.
    if parent.tiling_direction() != tiling_direction {
      origin_or_ancestor = parent.into();
      continue;
    }

    // Get the next/prev tiling sibling depending on the tiling direction.
    let focus_target = match direction {
      Direction::Up | Direction::Left => origin_or_ancestor
        .prev_siblings()
        .find_map(|c| c.as_tiling_container().ok()),
      _ => origin_or_ancestor
        .next_siblings()
        .find_map(|c| c.as_tiling_container().ok()),
    };

    if let Some(target) = focus_target {
      // If the target column has an active fullscreen window, focus that fullscreen window.
      if let Some(ws) = origin_or_ancestor.workspace() {
        let fullscreen_target = ws
          .children()
          .into_iter()
          .filter_map(|c| c.as_non_tiling_window().cloned())
          .find(|w| {
            matches!(w.state(), WindowState::Fullscreen(_))
              && w.insertion_target().is_some_and(|it| it.target_parent.id() == target.id())
          });

        if let Some(fs_win) = fullscreen_target {
          return Ok(Some(fs_win.into()));
        }
      }

      // Return once a suitable focus target is found.
      return Ok(match target {
        TilingContainer::TilingWindow(_) => Some(target.into()),
        TilingContainer::Split(split) => split
          .descendant_in_direction(&direction.inverse())
          .map(Into::into),
      });
    }

    // Check if an adjacent column is in fullscreen mode.
    if matches!(direction, Direction::Left | Direction::Right)
      && let Some(ws) = origin_or_ancestor.workspace()
    {
      let fullscreen_target = ws
        .children()
        .into_iter()
        .filter_map(|c| c.as_non_tiling_window().cloned())
        .find(|w| {
          if !matches!(w.state(), WindowState::Fullscreen(_)) {
            return false;
          }
          w.insertion_target().is_some_and(|target| match direction {
            Direction::Left => origin_or_ancestor
              .prev_siblings()
              .any(|s| s.id() == target.target_parent.id()),
            Direction::Right => origin_or_ancestor
              .next_siblings()
              .any(|s| s.id() == target.target_parent.id()),
            _ => false,
          })
        });

      if let Some(fs_win) = fullscreen_target {
        return Ok(Some(fs_win.into()));
      }
    }

    origin_or_ancestor = parent.into();
  }

  Ok(None)
}

