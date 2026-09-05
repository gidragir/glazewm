use anyhow::Context;
use tracing::info;
use wm_common::{
  GapsConfig, TilingDirection, WindowRuleEvent, WindowState, WmEvent,
  try_warn,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{NativeWindow, RectDelta};

use crate::{
  commands::{
    container::{attach_container, set_focused_descendant},
    window::{find_column_ancestor, run_window_rules},
  },
  models::{
    Container, Monitor, NativeWindowProperties, NonTilingWindow,
    SplitContainer, TilingWindow, WindowContainer, Workspace,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

#[allow(clippy::needless_pass_by_value)]
pub fn manage_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let Some(native_properties) =
    check_is_manageable(&native_window).unwrap_or(None)
  else {
    return Ok(());
  };

  // Create the window instance. This may fail if the window handle has
  // already been destroyed.
  let window = try_warn!(create_window(
    native_window.clone(),
    native_properties,
    target_parent,
    state,
    config
  ));

  let prev_focused = state.focused_container();

  // Set the newly added window as focus descendant. This means the window
  // rules will be run as if the window is focused.
  set_focused_descendant(&window.clone().into(), None);

  // Window might be detached if `ignore` command has been invoked.
  let updated_window = run_window_rules(
    window.clone(),
    &WindowRuleEvent::Manage,
    state,
    config,
  )?;

  if let Some(window) = updated_window {
    info!("New window managed: {window}");

    state.emit_event(WmEvent::WindowManaged {
      managed_window: window.to_dto()?,
    });

    let is_on_displayed_workspace = window
      .workspace()
      .and_then(|ws| ws.monitor())
      .and_then(|m| m.displayed_workspace())
      .is_some_and(|displayed_ws| {
        window.workspace().is_some_and(|ws| ws.id() == displayed_ws.id())
      });

    if is_on_displayed_workspace {
      // OS focus should be set to the newly added window in case it's not
      // already focused.
      state.pending_sync.queue_focus_change();

      // Normally, a `PlatformEvent::WindowFocused` event is what triggers
      // focus effects and workspace reordering to be applied. However, when
      // a window is first launched, this event can come before the
      // window is managed, and so we need to force an update here.
      state.pending_sync.queue_focused_effect_update();
      state.pending_sync.queue_workspace_to_reorder(
        window.workspace().context("No workspace.")?,
      );
    } else if let Some(prev_focused) = prev_focused {
      // If the window was moved to a hidden workspace, restore focus to
      // the previously focused container.
      set_focused_descendant(&prev_focused, None);
    }

    // Workspace containers need to be redrawn if the window is tiling.
    state.pending_sync.queue_container_to_redraw(
      if window.state() == WindowState::Tiling {
        if let Some(workspace) = window.workspace() {
          workspace.into()
        } else {
          window.parent().context("No parent.")?
        }
      } else {
        window.into()
      },
    );
  } else {
    #[cfg(target_os = "windows")]
    {
      _ = native_window.set_cloaked(false);
    }
  }

  Ok(())
}

/// Checks if a window is manageable and retrieves its native properties.
///
/// Returns `Ok(Some(properties))` if the window is manageable and its
/// properties were retrieved successfully.
fn check_is_manageable(
  native_window: &NativeWindow,
) -> anyhow::Result<Option<NativeWindowProperties>> {
  if !native_window.is_visible()? {
    return Ok(None);
  }

  #[cfg(target_os = "macos")]
  {
    use wm_platform::NativeWindowExtMacOs;

    let is_standard_window = native_window.role()? == "AXWindow"
      && native_window.subrole()? == "AXStandardWindow";

    if !is_standard_window {
      return Ok(None);
    }
  }

  // Ensure window has a valid process name, title, etc.
  let native_properties = NativeWindowProperties::try_from(native_window)?;

  #[cfg(target_os = "windows")]
  {
    use wm_platform::{
      NativeWindowWindowsExt, WS_CAPTION, WS_CHILD, WS_EX_NOACTIVATE,
      WS_EX_TOOLWINDOW,
    };

    // TODO: Temporary fix for managing Flow Launcher until a force manage
    // command is added.
    let is_flow_launcher = native_properties.process_name
      == "Flow.Launcher"
      && native_properties.title == "Flow.Launcher";

    if !is_flow_launcher {
      // Ensure window is top-level (i.e. not a child window). Ignore
      // windows that cannot be focused or if they're unavailable in
      // task switcher (alt+tab menu).
      if native_window.has_window_style(WS_CHILD)
        || native_window
          .has_window_style_ex(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW)
      {
        return Ok(None);
      }

      // Some applications spawn top-level windows for menus that
      // should be ignored. This includes the autocomplete popup in
      // Notepad++ and title bar menu in Keepass. Although not
      // foolproof, these can typically be identified by having an
      // owner window and no title bar.
      if native_window.has_owner_window()
        && !native_window.has_window_style(WS_CAPTION)
      {
        return Ok(None);
      }
    }
  }

  Ok(Some(native_properties))
}

fn create_window(
  native_window: NativeWindow,
  native_properties: NativeWindowProperties,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  let nearest_monitor = state
    .nearest_monitor(&native_window)
    .context("No nearest monitor.")?;

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No nearest workspace.")?;

  let gaps_config = config.value.gaps.clone();
  let window_state =
    window_state_to_create(&native_properties, &nearest_monitor, config)?;

  let target_workspace = if let Some(parent) = &target_parent {
    parent
      .workspace()
      .with_context(|| format!("Target parent {} has no workspace.", parent.id()))?
  } else {
    let focused_container = state
      .focused_container()
      .context("No focused container.")?;
    focused_container
      .workspace()
      .with_context(|| format!("Focused container {} has no workspace.", focused_container.id()))?
  };

  let prefers_centered = config
    .value
    .window_behavior
    .state_defaults
    .floating
    .centered;

  // Calculate where window should be placed when floating is enabled. Use
  // the original width/height of the window and optionally position it in
  // the center of the workspace.
  let is_same_workspace = nearest_workspace.id() == target_workspace.id();
  let floating_placement = {
    let placement = if !is_same_workspace || prefers_centered {
      native_properties
        .frame
        .translate_to_center(&target_workspace.to_rect()?)
    } else {
      native_properties.frame.clone()
    };

    // Clamp the window size to be within the workspace's outer gaps. 10px
    // is arbitrary - helps differentiate from tiling windows.
    let max_workspace_rect = target_workspace.max_workspace_rect()?;
    placement.clamp_size(
      max_workspace_rect.width() - 10,
      max_workspace_rect.height() - 10,
    )
  };

  // Window has no border delta unless it's later changed via the
  // `adjust_borders` command.
  let border_delta = RectDelta::zero();

  let window_container: WindowContainer = match window_state {
    WindowState::Tiling => TilingWindow::new(
      None,
      native_window,
      native_properties,
      None,
      border_delta,
      floating_placement,
      false,
      gaps_config.clone(),
      Vec::new(),
      None,
    )
    .into(),
    _ => NonTilingWindow::new(
      None,
      native_window,
      native_properties,
      window_state,
      None,
      border_delta,
      None,
      floating_placement,
      !prefers_centered,
      Vec::new(),
      None,
    )
    .into(),
  };

  if window_container.state() == WindowState::Tiling {
    attach_tiling_window(
      &window_container,
      target_parent.as_ref(),
      &target_workspace,
      gaps_config,
      state,
    )?;
  } else {
    let (target_parent, target_index) = if let Some(parent) = target_parent {
      (parent, 0)
    } else {
      (target_workspace.clone().into(), target_workspace.child_count())
    };

    attach_container(
      &window_container.clone().into(),
      &target_parent,
      Some(target_index),
    )
    .context("Failed to attach non-tiling window.")?;
  }

  // The OS might spawn the window on a different monitor to the target
  // parent, so adjustments might need to be made because of DPI.
  if nearest_monitor
    .has_dpi_difference(&window_container.clone().into())?
  {
    window_container.set_has_pending_dpi_adjustment(true);
  }

  Ok(window_container)
}

fn attach_tiling_window(
  window: &WindowContainer,
  target_parent: Option<&Container>,
  target_workspace: &Workspace,
  gaps_config: GapsConfig,
  state: &WmState,
) -> anyhow::Result<()> {
  // In infinite horizontal canvas mode, tiling windows are enclosed in
  // vertical columns (SplitContainer) attached directly to the workspace.
  let column = SplitContainer::new(
    TilingDirection::Vertical,
    gaps_config,
  );

  let target_index = match target_parent {
    Some(parent) if parent.as_workspace().is_some() => {
      target_workspace.child_count()
    }
    Some(parent) => {
      find_column_ancestor(parent)
        .map_or_else(|| target_workspace.child_count(), |col| col.index() + 1)
    }
    None => {
      let focused_container = state
        .focused_container()
        .context("No focused container.")?;

      find_column_ancestor(&focused_container).map_or_else(
        || {
          target_workspace
            .descendant_focus_order()
            .find_map(|descendant| find_column_ancestor(&descendant))
            .map_or_else(
              || target_workspace.child_count(),
              |col| col.index() + 1,
            )
        },
        |col| col.index() + 1,
      )
    }
  };

  attach_container(
    &column.clone().into(),
    &target_workspace.clone().into(),
    Some(target_index),
  )
  .context("Failed to attach column to workspace.")?;

  attach_container(
    &window.clone().into(),
    &column.into(),
    Some(0),
  )
  .context("Failed to attach tiling window to column.")?;

  Ok(())
}

/// Gets the initial state for a window based on its native state.
///
/// Note that maximized windows are initialized as tiling.
fn window_state_to_create(
  native_properties: &NativeWindowProperties,
  nearest_monitor: &Monitor,
  config: &UserConfig,
) -> anyhow::Result<WindowState> {
  if native_properties.is_minimized {
    return Ok(WindowState::Minimized);
  }

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No workspace.")?;

  // Only initialize as fullscreen if the window *exceeds* the workspace
  // bounds (due to the 1px inset).
  //
  // For example, with 0px outer gaps and a window that covers the entire
  // workspace, it would still not be initialized as fullscreen. The window
  // needs to be within the workspace's outer gaps by at least 1px on each
  // side.
  if !native_properties.is_maximized
    && native_properties
      .frame
      .inset(1)
      .contains_rect(&nearest_workspace.max_workspace_rect()?)
  {
    return Ok(WindowState::Fullscreen(
      config
        .value
        .window_behavior
        .state_defaults
        .fullscreen
        .clone(),
    ));
  }

  // Initialize windows that can't be resized as floating.
  if !native_properties.is_resizable {
    return Ok(WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ));
  }

  Ok(WindowState::default_from_config(&config.value))
}
