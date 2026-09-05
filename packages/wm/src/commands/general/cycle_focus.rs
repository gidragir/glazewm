use anyhow::Context;
use wm_common::WindowState;

use crate::{
  commands::container::set_focused_descendant,
  models::{WindowContainer, Workspace},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

#[allow(clippy::struct_excessive_bools)]
struct OmitFlags {
  floating: bool,
  fullscreen: bool,
  minimized: bool,
  tiling: bool,
}

fn is_state_omitted(state: &WindowState, omit: &OmitFlags) -> bool {
  match state {
    WindowState::Floating(_) => omit.floating,
    WindowState::Fullscreen(_) => omit.fullscreen,
    WindowState::Minimized => omit.minimized,
    WindowState::Tiling => omit.tiling,
  }
}

fn find_window_matching_state(
  workspace: &Workspace,
  target_state: &WindowState,
) -> Option<WindowContainer> {
  workspace
    .descendant_focus_order()
    .filter_map(|descendant| descendant.as_window_container().ok())
    .find(|descendant| descendant.state().is_same_state(target_state))
}

fn find_next_window_to_cycle(
  workspace: &Workspace,
  current: &WindowState,
  omit: &OmitFlags,
  config: &UserConfig,
) -> Option<WindowContainer> {
  let mut next = next_state(current, config);

  while !current.is_same_state(&next) {
    if !is_state_omitted(&next, omit)
      && let Some(window) = find_window_matching_state(workspace, &next)
    {
      return Some(window);
    }
    next = next_state(&next, config);
  }

  None
}

/// Cycles focus through windows of different states. In order, this will
/// change from tiling -> floating -> fullscreen -> minimized, then back to
/// tiling.
///
/// Does nothing if a workspace is focused.
#[allow(clippy::fn_params_excessive_bools)]
pub fn cycle_focus(
  omit_floating: bool,
  omit_fullscreen: bool,
  omit_minimized: bool,
  omit_tiling: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let Ok(window) = focused_container.as_window_container() else {
    return Ok(());
  };

  let workspace = window.workspace().context("No workspace.")?;
  let omit = OmitFlags {
    floating: omit_floating,
    fullscreen: omit_fullscreen,
    minimized: omit_minimized,
    tiling: omit_tiling,
  };

  if let Some(target_window) =
    find_next_window_to_cycle(&workspace, &window.state(), &omit, config)
  {
    set_focused_descendant(&target_window.into(), None);
    state.pending_sync.queue_focus_change().queue_cursor_jump();
  }

  Ok(())
}

fn next_state(
  current_state: &WindowState,
  config: &UserConfig,
) -> WindowState {
  match current_state {
    WindowState::Floating(_) => WindowState::Fullscreen(
      config
        .value
        .window_behavior
        .state_defaults
        .fullscreen
        .clone(),
    ),
    WindowState::Fullscreen(_) => WindowState::Minimized,
    WindowState::Minimized => WindowState::Tiling,
    WindowState::Tiling => WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ),
  }
}
