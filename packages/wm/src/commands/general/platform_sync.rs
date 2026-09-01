use anyhow::Context;
#[cfg(target_os = "windows")]
use wm_common::WindowEffectConfig;
use wm_common::{
  CursorJumpTrigger, DisplayState, HideCorner, HideMethod, UniqueExt,
  WindowState, WmEvent,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
#[cfg(target_os = "windows")]
use wm_platform::{CornerStyle, OpacityValue};
use wm_platform::{Rect, WindowZOrder};

use crate::{
  models::{Container, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn platform_sync(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  _ = auto_pan_all_viewports(state, config);

  if state.pending_sync.needs_focus_update() {
    sync_focus(&focused_container, state)?;
  }

  if !state.pending_sync.containers_to_redraw().is_empty()
    || !state.pending_sync.workspaces_to_reorder().is_empty()
  {
    redraw_containers(&focused_container, state, config)?;
  }

  if state.pending_sync.needs_cursor_jump()
    && config.value.general.cursor_jump.enabled
  {
    jump_cursor(&focused_container, state, config)?;
  }

  if state.pending_sync.needs_focused_effect_update()
    || state.pending_sync.needs_all_effects_update()
  {
    // Keep reference to the previous window that had focus effects
    // applied.
    let prev_effects_window = state.prev_effects_window.clone();

    if let Ok(window) = focused_container.as_window_container() {
      apply_window_effects(&window, true, config);
      state.prev_effects_window = Some(window.clone());
    } else {
      state.prev_effects_window = None;
    }

    // Get windows that should have the unfocused border applied to them.
    // For the sake of performance, we only update the border of the
    // previously focused window. If the `reset_window_effects` flag is
    // passed, the unfocused border is applied to all unfocused windows.
    let unfocused_windows =
      if state.pending_sync.needs_all_effects_update() {
        state.windows()
      } else {
        prev_effects_window.into_iter().collect()
      }
      .into_iter()
      .filter(|window| window.id() != focused_container.id());

    for window in unfocused_windows {
      apply_window_effects(&window, false, config);
    }
  }

  state.pending_sync.clear();

  Ok(())
}

fn sync_focus(
  focused_container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let native_window = focused_container.as_window_container().ok();

  // Sets focus to the appropriate target:
  // - If the container is a window, focuses that window.
  // - If the container is a workspace, "resets" focus by focusing the
  //   desktop window.
  //
  // In either case, a `PlatformEvent::WindowFocused` event is subsequently
  // triggered.
  let result = if let Some(window) = native_window {
    tracing::info!("Setting focus to window: {window}");
    window.native().focus()
  } else {
    tracing::info!("Setting focus to the desktop window.");
    state.dispatcher.reset_focus()
  };

  if let Err(err) = result {
    tracing::warn!("Failed to set focus: {}", err);
  }

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container.to_dto()?,
  });

  Ok(())
}

/// Finds windows that should be brought to the top of their workspace's
/// z-order.
///
/// Windows are brought to front if they match the focused window's state
/// (floating/tiling) and any of these conditions are met:
///  * Focus has changed to a different window.
///  * Focused window's state has changed (e.g. tiling -> floating).
///  * Focused window has moved to a different workspace.
fn windows_to_bring_to_front(
  focused_container: &Container,
  state: &WmState,
) -> anyhow::Result<Vec<WindowContainer>> {
  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  // Add focused workspace if there's been a focus change.
  let workspaces_to_reorder = state
    .pending_sync
    .workspaces_to_reorder()
    .iter()
    .chain(
      state
        .pending_sync
        .needs_focus_update()
        .then_some(&focused_workspace),
    )
    .unique_by(|workspace| workspace.id());

  // Bring forward windows that match the focused state. Only do this for
  // tiling/floating windows.
  let windows_to_bring_to_front = workspaces_to_reorder
    .flat_map(|workspace| {
      let focused_descendant = workspace
        .descendant_focus_order()
        .next()
        .and_then(|container| container.as_window_container().ok());

      match focused_descendant {
        Some(focused_descendant) => workspace
          .descendants()
          .filter_map(|descendant| descendant.as_window_container().ok())
          .filter(|window| {
            let is_floating_or_tiling = matches!(
              window.state(),
              WindowState::Floating(_) | WindowState::Tiling
            );

            is_floating_or_tiling
              && window.state().is_same_state(&focused_descendant.state())
          })
          .collect(),
        None => vec![],
      }
    })
    .collect::<Vec<_>>();

  Ok(windows_to_bring_to_front)
}

#[allow(clippy::too_many_lines)]
fn redraw_containers(
  focused_container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let windows_to_redraw = state.windows_to_redraw();
  let windows_to_bring_to_front =
    windows_to_bring_to_front(focused_container, state)?;

  let windows_to_update = {
    let mut windows = windows_to_redraw
      .iter()
      .chain(&windows_to_bring_to_front)
      .unique_by(|window| window.id())
      .collect::<Vec<_>>();

    let descendant_focus_order = state
      .root_container
      .descendant_focus_order()
      .collect::<Vec<_>>();

    // Sort the windows to update by their focus order. The most recently
    // focused window will be updated first.
    // TODO: To reduce flicker, redraw windows that will be shown first,
    // then redraw the ones to be hidden last.
    windows.sort_by_key(|window| {
      descendant_focus_order
        .iter()
        .position(|order| order.id() == window.id())
    });

    windows
  };

  // Get monitors by their optimal hide corner.
  let monitors_by_hide_corner = state.monitors_by_hide_corner();

  #[cfg(target_os = "windows")]
  {
    let mut batch_positions: Vec<(wm_platform::NativeWindow, Rect)> =
      Vec::new();
    for window in &windows_to_update {
      if !windows_to_redraw.contains(window) {
        continue;
      }
      let Some(workspace) = window.workspace() else {
        continue;
      };

      if !workspace.is_displayed() {
        continue;
      }
      if matches!(window.state(), WindowState::Tiling)
        && window.active_drag().is_none()
        && let Ok(rect) = window.to_rect()
        && let Ok(delta) = window.total_border_delta()
      {
        let rect_with_delta = rect.apply_delta(&delta, None);
        if let Ok(physical_rect) =
          calculate_physical_rect(window, &rect_with_delta, true)
        {
          batch_positions.push((window.native().clone(), physical_rect));
        }
      }
    }

    if !batch_positions.is_empty()
      && let Err(err) =
        wm_platform::apply_window_positions(&batch_positions)
    {
      tracing::warn!("Failed to batch apply window positions: {}", err);
    }
  }

  for window in windows_to_update.iter().rev() {
    let should_bring_to_front = windows_to_bring_to_front.contains(window);

    let workspace =
      window.workspace().context("Window has no workspace.")?;

    let monitor = window.monitor().context("No monitor.")?;
    let hide_corner = monitors_by_hide_corner
      .iter()
      .find(|(m, _)| m.id() == monitor.id())
      .map(|(_, hide_corner)| hide_corner)
      .context("Monitor not found in hide corner map.")?;

    // Whether the window should be shown above all other windows.
    let z_order = match window.state() {
      WindowState::Floating(config) if config.shown_on_top => {
        WindowZOrder::TopMost
      }
      WindowState::Fullscreen(config) if config.shown_on_top => {
        WindowZOrder::TopMost
      }
      _ if should_bring_to_front => {
        let focused_descendant = workspace
          .descendant_focus_order()
          .next()
          .and_then(|container| container.as_window_container().ok());

        if let Some(focused_descendant) = focused_descendant {
          if window.id() == focused_descendant.id() {
            WindowZOrder::Normal
          } else {
            WindowZOrder::AfterWindow(focused_descendant.native().id())
          }
        } else {
          WindowZOrder::Normal
        }
      }
      _ => WindowZOrder::Normal,
    };

    // Set the z-order of the window.
    //
    // NOTE: macOS doesn't have a robust public API for setting the z-order
    // of a window. See `NativeWindow::raise` for more details.
    #[cfg(target_os = "windows")]
    if should_bring_to_front && !windows_to_redraw.contains(window) {
      tracing::info!("Updating window z-order: {window}");

      if let Err(err) = window.native().set_z_order(&z_order) {
        tracing::warn!("Failed to set window z-order: {}", err);
      }
    }

    // Skip updating the window's position if it only required a z-order
    // change.
    if !windows_to_redraw.contains(window) {
      continue;
    }

    // Transition display state depending on whether window will be
    // shown or hidden.
    window.set_display_state(
      match (window.display_state(), workspace.is_displayed()) {
        (DisplayState::Hidden | DisplayState::Hiding, true) => {
          DisplayState::Showing
        }
        (DisplayState::Shown | DisplayState::Showing, false) => {
          DisplayState::Hiding
        }
        _ => window.display_state(),
      },
    );

    let is_visible = matches!(
      window.display_state(),
      DisplayState::Showing | DisplayState::Shown
    );

    if let Err(err) =
      reposition_window(window, *hide_corner, &z_order, is_visible, config)
    {
      tracing::warn!("Failed to set window position: {}", err);
    }

    // Whether the window is either transitioning to or from fullscreen.
    // TODO: This check can be improved since `prev_state` can be
    // fullscreen without it needing to be marked as not fullscreen.
    #[cfg(target_os = "windows")]
    {
      let is_transitioning_fullscreen =
        match (window.prev_state(), window.state()) {
          (Some(_), WindowState::Fullscreen(s)) if !s.maximized => true,
          (Some(WindowState::Fullscreen(_)), _) => true,
          _ => false,
        };

      if is_transitioning_fullscreen
        && let Err(err) = window.native().mark_fullscreen(matches!(
          window.state(),
          WindowState::Fullscreen(_)
        ))
      {
        tracing::warn!("Failed to mark window as fullscreen: {}", err);
      }
    }

    // Skip setting taskbar visibility if the window is hidden (has no
    // effect). Since cloaked windows are normally always visible in the
    // taskbar, we only need to set visibility if `show_all_in_taskbar` is
    // `false`.
    #[cfg(target_os = "windows")]
    if config.value.general.hide_method == HideMethod::Cloak
      && !config.value.general.show_all_in_taskbar
      && matches!(
        window.display_state(),
        DisplayState::Showing | DisplayState::Hiding
      )
      && let Err(err) = window.native().set_taskbar_visibility(is_visible)
    {
      tracing::warn!("Failed to set taskbar visibility: {}", err);
    }
  }

  Ok(())
}

fn reposition_window(
  window: &WindowContainer,
  hide_corner: HideCorner,
  // LINT: `z_order` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  z_order: &WindowZOrder,
  is_visible: bool,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let rect = window
    .to_rect()?
    .apply_delta(&window.total_border_delta()?, None);

  // For `HideMethod::PlaceInCorner`, we need to reposition hidden windows
  // to the corner of the monitor.
  if config.value.general.hide_method == HideMethod::PlaceInCorner
    && !is_visible
  {
    const VISIBLE_SLIVER: i32 = 1;

    let monitor_rect = window
      .monitor()
      .context("No monitor.")?
      .native_properties()
      .working_area;

    let frame = window.native_properties().frame;

    let position_y = monitor_rect.bottom - VISIBLE_SLIVER;
    let position_x = match hide_corner {
      HideCorner::BottomLeft => {
        monitor_rect.left + VISIBLE_SLIVER - frame.width()
      }
      HideCorner::BottomRight => monitor_rect.right - VISIBLE_SLIVER,
    };

    // Even though the window size is unchanged, `NativeWindow::set_frame`
    // is used instead of `NativeWindow::reposition` because the latter
    // resulted in occasional incorrect positionings on macOS.
    window.native().set_frame(&Rect::from_xy(
      position_x,
      position_y,
      frame.width(),
      frame.height(),
    ))?;

    return Ok(());
  }

  let physical_rect = calculate_physical_rect(window, &rect, is_visible)?;

  if window.active_drag().is_some() {
    window.native().resize(physical_rect.width(), physical_rect.height())?;
  } else {
    #[cfg(target_os = "macos")]
    window.native().set_frame(&physical_rect)?;

    #[cfg(target_os = "windows")]
    {
      use wm_platform::{
        SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE,
        SWP_NOCOPYBITS, SWP_NOSENDCHANGING, WS_MAXIMIZEBOX,
      };

      // Restore window if it's minimized/maximized and shouldn't be. This
      // is needed to be able to move and resize it.
      let should_restore = match &window.state() {
        // Need to restore window if transitioning from maximized
        // fullscreen to non-maximized fullscreen.
        WindowState::Fullscreen(fullscreen) => {
          !fullscreen.maximized && window.native().is_maximized()?
        }
        // No need to restore window if it'll be minimized. Transitioning
        // from maximized to minimized works without having to
        // restore.
        WindowState::Minimized => false,
        _ => {
          window.native().is_minimized()?
            || window.native().is_maximized()?
        }
      };

      if should_restore {
        // Restoring to position has the same effect as `ShowWindow` with
        // `SW_RESTORE`, but doesn't cause a flicker.
        window.native().restore(Some(&physical_rect))?;
      }

      let mut swp_flags = SWP_NOACTIVATE
        | SWP_NOCOPYBITS
        | SWP_NOSENDCHANGING
        | SWP_ASYNCWINDOWPOS;

      match &window.state() {
        WindowState::Minimized => {
          if !window.native().is_minimized()? {
            window.native().minimize()?;
          }
        }
        WindowState::Fullscreen(fullscreen)
          if fullscreen.maximized
            && window.native().has_window_style(WS_MAXIMIZEBOX) =>
        {
          if !window.native().is_maximized()? {
            window.native().maximize()?;
          }

          window.native().set_window_pos(z_order, &physical_rect, swp_flags)?;
        }
        _ => {
          swp_flags |= SWP_FRAMECHANGED;

          window.native().set_window_pos(z_order, &physical_rect, swp_flags)?;

          // When there's a mismatch between the DPI of the monitor and the
          // window, the window might be sized incorrectly after the first
          // move. If we set the position twice, inconsistencies after the
          // first move are resolved.
          if window.has_pending_dpi_adjustment() {
            window.native().set_window_pos(z_order, &physical_rect, swp_flags)?;
          }
        }
      }

      // Set visibility based on the hide method.
      if config.value.general.hide_method == HideMethod::Cloak {
        window.native().set_cloaked(!is_visible)?;
      } else if is_visible {
        window.native().show()?;
      } else {
        window.native().hide()?;
      }
    }
  }

  Ok(())
}

fn calculate_physical_rect(
  window: &WindowContainer,
  rect: &Rect,
  is_visible: bool,
) -> anyhow::Result<Rect> {
  let monitor = window.monitor().context("No monitor.")?;
  let monitor_rect = monitor.native_properties().working_area;

  if is_visible && matches!(window, WindowContainer::TilingWindow(_)) {
    if rect.right <= monitor_rect.left || rect.left >= monitor_rect.right {
      // Completely offscreen relative to parent monitor.
      // Park at safe virtual desktop coordinates far outside all monitors,
      // keeping original dimensions so taskbar and Alt+Tab remain functional.
      const SAFE_PARK_X: i32 = 50_000;
      const SAFE_PARK_Y: i32 = 50_000;
      Ok(Rect::from_xy(
        SAFE_PARK_X,
        SAFE_PARK_Y,
        rect.width(),
        rect.height(),
      ))
    } else if rect.left < monitor_rect.left && rect.right <= monitor_rect.right {
      // Partially visible on the left edge of monitor. Clamp to monitor left bound.
      let visible_width = (rect.right - monitor_rect.left).max(1);
      Ok(Rect::from_xy(
        monitor_rect.left,
        rect.y(),
        visible_width,
        rect.height(),
      ))
    } else if rect.left >= monitor_rect.left && rect.right > monitor_rect.right {
      // Partially visible on the right edge of monitor. Clamp to monitor right bound.
      let visible_width = (monitor_rect.right - rect.left).max(1);
      Ok(Rect::from_xy(
        rect.left,
        rect.y(),
        visible_width,
        rect.height(),
      ))
    } else if rect.left < monitor_rect.left && rect.right > monitor_rect.right {
      // Wider than monitor working area.
      Ok(Rect::from_xy(
        monitor_rect.left,
        rect.y(),
        monitor_rect.width(),
        rect.height(),
      ))
    } else {
      Ok(rect.clone())
    }
  } else {
    Ok(rect.clone())
  }
}

fn jump_cursor(
  focused_container: &Container,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let cursor_jump = &config.value.general.cursor_jump;

  let target_monitor = focused_container.monitor().context("No monitor.")?;
  let monitor_rect = target_monitor.native_properties().working_area;
  let cursor_pos = state.dispatcher.cursor_position().ok();

  let should_jump = match cursor_jump.trigger {
    CursorJumpTrigger::WindowFocus => true,
    CursorJumpTrigger::MonitorFocus => {
      let cursor_monitor =
        cursor_pos.as_ref().and_then(|pos| state.monitor_at_point(pos));

      // Jump to the target monitor if the cursor is not already on it.
      cursor_monitor
        .is_none_or(|monitor| monitor.id() != target_monitor.id())
    }
  };

  if should_jump {
    let center = if let Ok(window) = focused_container.as_window_container() {
      if let Ok(rect) = window.to_rect()
        && let Ok(physical_rect) =
          calculate_physical_rect(&window, &rect, true)
      {
        if physical_rect.left >= 40_000 || physical_rect.top >= 40_000 {
          monitor_rect.center_point()
        } else {
          wm_platform::Point {
            x: physical_rect
              .center_point()
              .x
              .clamp(monitor_rect.left, monitor_rect.right.saturating_sub(1)),
            y: physical_rect
              .center_point()
              .y
              .clamp(monitor_rect.top, monitor_rect.bottom.saturating_sub(1)),
          }
        }
      } else {
        monitor_rect.center_point()
      }
    } else {
      monitor_rect.center_point()
    };

    if let Err(err) = state.dispatcher.set_cursor_position(&center) {
      tracing::warn!("Failed to set cursor position: {}", err);
    }
  }

  Ok(())
}

fn apply_window_effects(
  // LINT: `window` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  window: &WindowContainer,
  is_focused: bool,
  config: &UserConfig,
) {
  let window_effects = &config.value.window_effects;

  // LINT: `effect_config` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  let effect_config = if is_focused {
    &window_effects.focused_window
  } else {
    &window_effects.other_windows
  };

  // Skip if both focused + non-focused window effects are disabled.
  #[cfg(target_os = "windows")]
  if window_effects.focused_window.border.enabled
    || window_effects.other_windows.border.enabled
  {
    apply_border_effect(window, effect_config);
  }

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.hide_title_bar.enabled
    || window_effects.other_windows.hide_title_bar.enabled
  {
    apply_hide_title_bar_effect(window, effect_config);
  }

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.corner_style.enabled
    || window_effects.other_windows.corner_style.enabled
  {
    apply_corner_effect(window, effect_config);
  }

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.transparency.enabled
    || window_effects.other_windows.transparency.enabled
  {
    apply_transparency_effect(window, effect_config);
  }
}

#[cfg(target_os = "windows")]
fn apply_border_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let border_color = if effect_config.border.enabled {
    Some(&effect_config.border.color)
  } else {
    None
  };

  _ = window.native().set_border_color(border_color);

  let native = window.native().clone();
  let border_color = border_color.cloned();

  // Re-apply border color after a short delay to better handle
  // windows that change it themselves.
  tokio::task::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    _ = native.set_border_color(border_color.as_ref());
  });
}

#[cfg(target_os = "windows")]
fn apply_hide_title_bar_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  _ = window
    .native()
    .set_title_bar_visibility(!effect_config.hide_title_bar.enabled);
}

#[cfg(target_os = "windows")]
fn apply_corner_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let corner_style = if effect_config.corner_style.enabled {
    &effect_config.corner_style.style
  } else {
    &CornerStyle::Default
  };

  _ = window.native().set_corner_style(corner_style);
}

#[cfg(target_os = "windows")]
fn apply_transparency_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let transparency = if effect_config.transparency.enabled {
    &effect_config.transparency.opacity
  } else {
    // Reset the transparency to default.
    &OpacityValue::from_alpha(u8::MAX)
  };

  _ = window.native().set_transparency(transparency);
}

/// Smoothly animates the workspace `offset_x` to `target_offset` using
/// lightweight discrete step interpolation.
pub fn animate_pan_workspace(
  workspace: &Workspace,
  target_offset: f64,
  config: &UserConfig,
) {
  let start_offset = workspace.offset_x();
  let delta = target_offset - start_offset;

  #[cfg(target_os = "windows")]
  if config.value.general.animation_enabled && delta.abs() > 20.0 {
    let duration_ms = config.value.general.animation_duration_ms.max(20);
    let steps = (duration_ms / 16).clamp(3, 8);
    let step_delay = std::time::Duration::from_millis(
      u64::from(duration_ms) / u64::from(steps),
    );

    let tiling_windows: Vec<WindowContainer> = workspace
      .descendants()
      .filter_map(|c| c.as_window_container().ok())
      .filter(|w| matches!(w.state(), WindowState::Tiling))
      .collect();

    for step in 1..steps {
      #[allow(clippy::cast_precision_loss)]
      let t = f64::from(step) / f64::from(steps);
      // Ease-out quadratic formula
      let progress = 1.0 - (1.0 - t) * (1.0 - t);
      let intermediate_offset = start_offset + delta * progress;

      workspace.set_offset_x(intermediate_offset);

      let mut step_positions = Vec::new();
      for win in &tiling_windows {
        if let Ok(rect) = win.to_rect()
          && let Ok(border_delta) = win.total_border_delta()
        {
          let unconstrained_rect = rect.apply_delta(&border_delta, None);
          let physical_rect = calculate_physical_rect(win, &unconstrained_rect, true)
            .unwrap_or(unconstrained_rect);
          step_positions.push((win.native().clone(), physical_rect));
        }
      }

      if !step_positions.is_empty() {
        _ = wm_platform::apply_window_positions(&step_positions);
      }

      std::thread::sleep(step_delay);
    }
  }

  workspace.set_offset_x(target_offset);
}

fn auto_pan_all_viewports(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let displayed_workspaces: Vec<Workspace> = state
    .monitors()
    .into_iter()
    .filter_map(|m| m.displayed_workspace())
    .collect();

  for workspace in displayed_workspaces {
    auto_pan_workspace_viewport(&workspace, state, config)?;
  }

  Ok(())
}

fn auto_pan_workspace_viewport(
  workspace: &Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace_rect = workspace.to_rect()?;

  let tiling_children: Vec<crate::models::TilingContainer> =
    workspace.tiling_children().collect();

  if tiling_children.is_empty() {
    if workspace.offset_x().abs() > 0.001 {
      workspace.set_offset_x(0.0);
      state
        .pending_sync
        .queue_container_to_redraw(workspace.clone());
    }
    return Ok(());
  }

  #[allow(clippy::cast_possible_truncation)]
  let current_offset = workspace.offset_x() as i32;

  // Find rightmost tiling child to compute total content span from workspace.left.
  let last_child_rect = tiling_children
    .last()
    .context("No tiling child.")?
    .to_rect()?;

  let total_content_span =
    (last_child_rect.right + current_offset - workspace_rect.left).max(0);

  #[allow(clippy::cast_precision_loss)]
  let max_offset =
    f64::from((total_content_span - workspace_rect.width()).max(0));

  // Find the focused window or container in this workspace.
  let focused_container = workspace
    .descendant_focus_order()
    .find(|c| c.as_window_container().is_ok())
    .or_else(|| tiling_children.first().map(|c| c.clone().into()));

  let mut new_offset = workspace.offset_x();

  if let Some(focused) = focused_container
    && let Ok(focused_rect) = focused.to_rect()
  {
    if focused_rect.left < workspace_rect.left {
      let delta = focused_rect.left - workspace_rect.left;
      new_offset = f64::from((current_offset + delta).max(0));
    } else if focused_rect.right > workspace_rect.right {
      let delta = focused_rect.right - workspace_rect.right;
      new_offset = f64::from((current_offset + delta).max(0));
    }
  }

  // Clamp new_offset within [0.0, max_offset]
  new_offset = new_offset.clamp(0.0, max_offset);

  if (new_offset - workspace.offset_x()).abs() > 0.001 {
    animate_pan_workspace(workspace, new_offset, config);
    state
      .pending_sync
      .queue_container_to_redraw(workspace.clone());
  }

  Ok(())
}
