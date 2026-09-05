use anyhow::Context;
use wm_common::{
  ActiveDrag, ActiveDragOperation, DisplayState, FloatingStateConfig,
  FullscreenStateConfig, HideMethod, WindowState, try_warn,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
#[cfg(target_os = "macos")]
use wm_platform::{LengthValue, MouseButton, RectDelta};
use wm_platform::{NativeWindow, Rect};

use crate::{
  commands::{
    container::{flatten_split_container, move_container_within_tree},
    window::update_window_state,
  },
  events::handle_window_moved_or_resized_end,
  models::{Monitor, NonTilingWindow, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

fn is_duplicate_frame(old: &Rect, new: &Rect) -> bool {
  old == new
    || ((old.x() - new.x()).abs() <= 2
      && (old.y() - new.y()).abs() <= 2
      && (old.width() - new.width()).abs() <= 2
      && (old.height() - new.height()).abs() <= 2)
}

fn handle_active_drag(
  window: &WindowContainer,
  frame_position: &Rect,
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  is_interactive_end: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let is_drag_end = {
    #[cfg(target_os = "windows")]
    {
      is_interactive_end
    }
    #[cfg(target_os = "macos")]
    {
      !state.dispatcher.is_mouse_down(&MouseButton::Left)
    }
  };

  if is_drag_end {
    handle_window_moved_or_resized_end(window, state, config)
  } else {
    update_drag_state(window, frame_position, state, config)
  }
}

fn check_is_drag_start(
  window: &WindowContainer,
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  frame_position: &Rect,
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  is_interactive_start: bool,
  state: &WmState,
) -> bool {
  if state.is_paused {
    return false;
  }

  #[cfg(target_os = "windows")]
  {
    let _ = (frame_position, state);
    is_interactive_start && !matches!(window.state(), WindowState::Minimized)
  }

  #[cfg(target_os = "macos")]
  {
    let is_valid_state = !matches!(
      window.state(),
      WindowState::Fullscreen(FullscreenStateConfig {
        maximized: true,
        ..
      }) | WindowState::Minimized
    );

    let is_dragging_other =
      state.windows().iter().any(|w| w.active_drag().is_some());
    let is_left_click =
      state.dispatcher.is_mouse_down(&MouseButton::Left);

    if !is_valid_state || is_dragging_other || !is_left_click {
      return false;
    }

    let frame_to_check = frame_position.apply_delta(
      &RectDelta::new(
        LengthValue::from_px(40),
        LengthValue::from_px(40),
        LengthValue::from_px(40),
        LengthValue::from_px(40),
      ),
      None,
    );
    let Ok(cursor_position) = state.dispatcher.cursor_position() else {
      return false;
    };
    frame_to_check.contains_point(&cursor_position)
  }
}

fn check_should_fullscreen(
  window: &WindowContainer,
  nearest_monitor: &Monitor,
  old_frame_position: &Rect,
  frame_position: &Rect,
) -> anyhow::Result<bool> {
  let workspace = nearest_monitor
    .displayed_workspace()
    .context("No workspace.")?;
  let should_fullscreen = window.should_fullscreen(&workspace)?;

  if let WindowState::Fullscreen(fullscreen) = window.state()
    && !fullscreen.maximized
    && should_fullscreen
  {
    let workspace_rect = workspace.max_workspace_rect()?;
    let old_frame = old_frame_position
      .apply_delta(&window.border_delta().inverse(), None);
    let new_frame = frame_position
      .apply_delta(&window.border_delta().inverse(), None);

    let old_exceeded = old_frame.inset(1).contains_rect(&workspace_rect);
    let new_exceeds = new_frame.inset(1).contains_rect(&workspace_rect);

    if old_exceeded && !new_exceeds {
      return Ok(false);
    }
  }

  Ok(should_fullscreen)
}

fn handle_fullscreen_or_maximized(
  window: &WindowContainer,
  is_maximized: bool,
  should_fullscreen: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let is_same_state = is_maximized
    && matches!(
      window.state(),
      WindowState::Fullscreen(FullscreenStateConfig {
        maximized: true,
        ..
      })
    )
    || should_fullscreen
      && matches!(
        window.state(),
        WindowState::Fullscreen(FullscreenStateConfig {
          maximized: false,
          ..
        })
      );

  if is_same_state {
    return Ok(());
  }

  let fullscreen_state = if let WindowState::Fullscreen(
    fullscreen_state,
  ) = window.state()
  {
    fullscreen_state
  } else {
    config
      .value
      .window_behavior
      .state_defaults
      .fullscreen
      .clone()
  };

  let updated_window = update_window_state(
    window.clone(),
    WindowState::Fullscreen(FullscreenStateConfig {
      maximized: is_maximized,
      ..fullscreen_state
    }),
    state,
    config,
  )?;

  if is_maximized {
    state
      .pending_sync
      .dequeue_container_from_redraw(updated_window);
  }

  Ok(())
}

fn update_corner_display_state(
  window: &WindowContainer,
  frame_position: &Rect,
  nearest_monitor: &Monitor,
) -> bool {
  let is_in_corner = is_in_corner(
    frame_position,
    &nearest_monitor.native_properties().working_area,
  );

  let display_state = match (window.display_state(), is_in_corner) {
    (DisplayState::Hiding, true) => DisplayState::Hidden,
    (DisplayState::Showing, false) => DisplayState::Shown,
    _ => window.display_state(),
  };

  if display_state == window.display_state() {
    false
  } else {
    window.set_display_state(display_state);
    true
  }
}

fn handle_restored_or_floating(
  window: WindowContainer,
  frame_position: Rect,
  nearest_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match window.state() {
    WindowState::Fullscreen(_) => {
      tracing::info!("Restoring window from fullscreen: {window}");

      update_window_state(
        window.clone(),
        window.toggled_state(window.state(), config),
        state,
        config,
      )?;
    }
    WindowState::Floating(_) => {
      if let WindowContainer::NonTilingWindow(window) = window {
        update_floating_window_position(
          &window,
          frame_position,
          nearest_monitor,
          state,
        )?;
      }
    }
    _ => {}
  }

  Ok(())
}

pub fn handle_window_moved_or_resized(
  native_window: &NativeWindow,
  // LINT: `is_interactive_start` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  is_interactive_start: bool,
  // LINT: `is_interactive_end` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  is_interactive_end: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(window) = state.window_from_native(native_window) else {
    return Ok(());
  };

  let old_frame_position = window.native_properties().frame;
  let frame_position = try_warn!(window.native().frame());

  window.update_native_properties(|properties| {
    properties.frame = frame_position.clone();
  });

  // Handle windows that are actively being dragged.
  if !state.is_paused && window.active_drag().is_some() {
    return handle_active_drag(
      &window,
      &frame_position,
      is_interactive_end,
      state,
      config,
    );
  }

  let old_is_maximized = window.native_properties().is_maximized;
  let is_maximized = try_warn!(window.native().is_maximized());

  // Ignore duplicate or near-duplicate move/resize events (e.g. from DWM
  // subpixel rounding). Window position changes can trigger multiple
  // events. For example, restoring from maximized can trigger as many as
  // 4 identical events on Windows.
  if is_duplicate_frame(&old_frame_position, &frame_position)
    && old_is_maximized == is_maximized
    && !is_interactive_start
  {
    return Ok(());
  }

  window.update_native_properties(|properties| {
    properties.is_maximized = is_maximized;
  });

  // If the window is not maximized, update its cached shadow borders.
  #[cfg(target_os = "windows")]
  {
    let shadow_borders = try_warn!(window.native().shadow_borders());
    if !is_maximized {
      window.update_native_properties(|properties| {
        properties.shadow_borders = shadow_borders;
      });
    }
  }

  let is_minimized = try_warn!(window.native().is_minimized());
  if is_minimized {
    return Ok(());
  }

  // Detect whether the window is starting to be interactively moved or
  // resized by the user (e.g. via the window's drag handles).
  if check_is_drag_start(&window, &frame_position, is_interactive_start, state) {
    tracing::info!("Window started dragging: {window}");

    window.set_active_drag(Some(ActiveDrag {
      operation: None,
      is_from_floating: matches!(
        window.state(),
        WindowState::Floating(_)
      ),
      #[cfg(target_os = "windows")]
      initial_position: old_frame_position.clone(),
      #[cfg(target_os = "macos")]
      initial_position: frame_position.clone(),
    }));

    #[cfg(target_os = "windows")]
    update_drag_state(&window, &frame_position, state, config)?;

    return Ok(());
  }

  let nearest_monitor = state
    .nearest_monitor(&window.native())
    .context("No nearest monitor.")?;

  // For `HideMethod::PlaceInCorner`, update DisplayState if moved to corner.
  if config.value.general.hide_method == HideMethod::PlaceInCorner
    && update_corner_display_state(&window, &frame_position, &nearest_monitor)
  {
    return Ok(());
  }

  let should_fullscreen = check_should_fullscreen(
    &window,
    &nearest_monitor,
    &old_frame_position,
    &frame_position,
  )?;

  // Handle a window being maximized or entering fullscreen.
  if is_maximized || should_fullscreen {
    return handle_fullscreen_or_maximized(
      &window,
      is_maximized,
      should_fullscreen,
      state,
      config,
    );
  }

  handle_restored_or_floating(
    window,
    frame_position,
    &nearest_monitor,
    state,
    config,
  )
}

// TODO: Move to shared location. `handle_window_moved_or_resized_end.rs`
// also uses this.
pub fn update_floating_window_position(
  window: &NonTilingWindow,
  frame_position: Rect,
  nearest_monitor: &Monitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  tracing::info!(
    "Updating floating window position: {}",
    window.as_window_container()?
  );

  // Update state with the new location of the floating window.
  window.set_floating_placement(frame_position);
  window.set_has_custom_floating_placement(true);

  let monitor = window.monitor().context("No monitor.")?;

  // Update the window's workspace if it goes out of bounds of its
  // current workspace.
  if monitor.id() != nearest_monitor.id() {
    let updated_workspace = nearest_monitor
      .displayed_workspace()
      .context("Failed to get workspace of nearest monitor.")?;

    tracing::info!(
      "Floating window moved to new workspace: {updated_workspace}",
    );

    window.set_insertion_target(None);

    move_container_within_tree(
      &window.clone().into(),
      &updated_workspace.clone().into(),
      updated_workspace.child_count(),
      state,
    )?;
  }

  Ok(())
}

fn determine_drag_operation(
  window: &WindowContainer,
  active_drag: &ActiveDrag,
  frame_position: &Rect,
) -> bool {
  if let Some(operation) = active_drag.operation {
    return matches!(operation, ActiveDragOperation::Move);
  }

  let is_move = *frame_position != active_drag.initial_position
    && frame_position.height() == active_drag.initial_position.height()
    && frame_position.width() == active_drag.initial_position.width();

  let operation = if is_move {
    ActiveDragOperation::Move
  } else {
    ActiveDragOperation::Resize
  };

  window.set_active_drag(Some(ActiveDrag {
    operation: Some(operation),
    ..active_drag.clone()
  }));

  is_move
}

fn transition_drag_to_floating(
  window: &WindowContainer,
  active_drag: &ActiveDrag,
  frame_position: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let move_distance = frame_position
    .center_point()
    .distance_between(&active_drag.initial_position.center_point());

  let is_maximized = matches!(
    window.state(),
    WindowState::Fullscreen(FullscreenStateConfig {
      maximized: true,
      ..
    })
  );

  if move_distance < 10.0 && !is_maximized {
    return Ok(());
  }

  let parent = window.parent().context("No parent")?;
  let is_fullscreen =
    matches!(window.state(), WindowState::Fullscreen(_)) && !is_maximized;

  let window = update_window_state(
    window.clone(),
    WindowState::Floating(FloatingStateConfig {
      centered: false,
      ..config.value.window_behavior.state_defaults.floating
    }),
    state,
    config,
  )?;

  if !is_fullscreen {
    state
      .pending_sync
      .dequeue_container_from_redraw(window.clone());
  }

  if let Some(split_parent) = parent.as_split()
    && split_parent.child_count() == 1
    && split_parent.parent().is_some_and(|p| p.as_workspace().is_none())
  {
    let root_parent = split_parent.parent();
    flatten_split_container(split_parent.clone())?;

    if let Some(root_parent) = root_parent {
      state
        .pending_sync
        .queue_container_to_redraw(root_parent);
    }
  }

  Ok(())
}

/// Updates the window operation based on changes in frame position.
///
/// This function determines whether a window is being moved or resized and
/// updates its operation state accordingly. If the window is being moved,
/// it's set to floating mode.
fn update_drag_state(
  window: &WindowContainer,
  frame_position: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(active_drag) = window.active_drag() else {
    return Ok(());
  };

  // Ignore if the window position has not changed yet.
  if *frame_position == active_drag.initial_position {
    return Ok(());
  }

  let is_move = determine_drag_operation(window, &active_drag, frame_position);

  // Transition window to be floating while it's being dragged, but only
  // after it has been moved at least 10px from its initial position. The
  // 10px threshold is to account for small movements that may be
  // accidental.
  if is_move && !matches!(window.state(), WindowState::Floating(_)) {
    transition_drag_to_floating(window, &active_drag, frame_position, state, config)?;
  }

  Ok(())
}

/// Gets whether the window is in the corner of the monitor.
fn is_in_corner(window_frame: &Rect, monitor_rect: &Rect) -> bool {
  // Visible portion of the window used when positioning windows in the
  // monitor's corner. See `platform_sync` for how hidden windows are
  // positioned.
  const VISIBLE_SLIVER_PX: i32 = 1;

  // Allow 1px of leeway.
  let is_left_corner =
    (window_frame.right - VISIBLE_SLIVER_PX - monitor_rect.left).abs()
      <= 1;

  // Allow 1px of leeway.
  let is_right_corner =
    (window_frame.x() + VISIBLE_SLIVER_PX - monitor_rect.right).abs() <= 1;

  // On macOS, the window's title bar is prevented from being positioned
  // outside of monitor's working area, so we need to allow ~55px of
  // vertical leeway. Title bar height varies, but can be up to 52px.
  // TODO: See if possible to make this dynamic based on the window's title
  // bar height.
  let is_bottom_of_monitor =
    (window_frame.y() - monitor_rect.bottom).abs() <= 55;

  (is_left_corner || is_right_corner) && is_bottom_of_monitor
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::is_in_corner;

  #[test]
  fn matches_corner_positions() {
    let monitor = Rect::from_xy(0, 0, 1920, 1080);

    let frame_in_right_corner = Rect::from_xy(1919, 1050, 600, 600);
    assert!(is_in_corner(&frame_in_right_corner, &monitor));

    let frame_in_left_corner = Rect::from_xy(-599, 1050, 600, 600);
    assert!(is_in_corner(&frame_in_left_corner, &monitor));
  }

  #[test]
  fn does_not_match_non_corner_positions() {
    let monitor = Rect::from_xy(0, 0, 1920, 1080);
    let frame = Rect::from_xy(100, 100, 800, 600);

    assert!(!is_in_corner(&frame, &monitor));
  }
}
