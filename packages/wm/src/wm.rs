use anyhow::{Context, bail};
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;
#[cfg(target_os = "windows")]
use wm_common::TitleBarVisibility;
use wm_common::{
  FloatingStateConfig, FullscreenStateConfig, InvokeCommand,
  InvokeFocusCommand, InvokeMoveCommand, WindowState, WmEvent,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{
  Direction, Dispatcher, LengthValue, PlatformEvent, RectDelta,
  WindowEvent,
};

use crate::{
  commands::{
    container::{
      focus_container_by_id, focus_in_direction, set_tiling_direction,
      toggle_tiling_direction,
    },
    general::{
      cycle_focus, disable_binding_mode, enable_binding_mode,
      platform_sync, reload_config, shell_exec, toggle_pause,
    },
    monitor::focus_monitor,
    window::{
      WindowPositionTarget, consume_or_expel_window, cycle_column_preset,
      ignore_window, move_window_in_direction, move_window_to_workspace,
      resize_window, set_column_width, set_window_position,
      set_window_size, update_window_state,
    },
    workspace::{
      focus_workspace, move_workspace_in_direction,
      update_workspace_config,
    },
  },
  events::{
    handle_display_settings_changed, handle_mouse_move,
    handle_window_destroyed, handle_window_focused, handle_window_hidden,
    handle_window_minimize_ended, handle_window_minimized,
    handle_window_moved_or_resized, handle_window_shown,
    handle_window_title_changed,
  },
  ipc_server::IpcServer,
  models::{Container, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: mpsc::UnboundedReceiver<WmEvent>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  pub state: WmState,
}

impl WindowManager {
  pub fn new(
    config: &mut UserConfig,
    dispatcher: Dispatcher,
  ) -> anyhow::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();

    let mut state = WmState::new(dispatcher, event_tx, exit_tx);
    state.populate(config)?;

    Ok(Self {
      event_rx,
      exit_rx,
      state,
    })
  }

  pub fn process_event(
    &mut self,
    event: PlatformEvent,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let state = &mut self.state;

    match event {
      PlatformEvent::DisplaySettingsChanged => {
        handle_display_settings_changed(state, config)
      }
      PlatformEvent::Keybinding(keybinding_event) => {
        // Find the keybinding config that matches this keybinding.
        let commands = config
          .active_keybinding_configs(
            &self.state.binding_modes,
            self.state.is_paused,
          )
          .find(|kb_config| {
            kb_config.bindings.contains(&keybinding_event.0)
          })
          .map(|kb_config| kb_config.commands.clone());

        if let Some(commands) = commands {
          self.process_commands(&commands, None, config)?;
        }

        // Return early since we don't want to redraw twice.
        return Ok(());
      }
      PlatformEvent::Mouse(event) => {
        handle_mouse_move(&event, state, config)
      }
      PlatformEvent::Window(window_event) => match window_event {
        WindowEvent::Focused { window, .. } => {
          handle_window_focused(&window, state, config)
        }
        WindowEvent::Shown { window, .. } => {
          handle_window_shown(window, state, config)
        }
        WindowEvent::Hidden { window, .. } => {
          handle_window_hidden(&window, state, config)
        }
        WindowEvent::MovedOrResized {
          window,
          is_interactive_start,
          is_interactive_end,
          ..
        } => handle_window_moved_or_resized(
          &window,
          is_interactive_start,
          is_interactive_end,
          state,
          config,
        ),
        WindowEvent::Minimized { window, .. } => {
          handle_window_minimized(&window, state, config)
        }
        WindowEvent::MinimizeEnded { window, .. } => {
          handle_window_minimize_ended(&window, state, config)
        }
        WindowEvent::TitleChanged { window, .. } => {
          handle_window_title_changed(&window, state, config)
        }
        WindowEvent::Destroyed { window_id, .. } => {
          handle_window_destroyed(window_id, state)
        }
      },
    }?;

    if !state.is_paused && state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    Ok(())
  }

  pub fn process_commands(
    &mut self,
    commands: &Vec<InvokeCommand>,
    subject_container_id: Option<Uuid>,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let state = &mut self.state;

    // Get the container to run WM commands with.
    let subject_container = match subject_container_id {
      Some(id) => state.container_by_id(id).with_context(|| {
        format!("No container found with the given ID '{id}'.")
      })?,
      None => state
        .focused_container()
        .context("No subject container for command.")?,
    };

    let new_subject_container_id = WindowManager::run_commands(
      commands,
      subject_container,
      state,
      config,
    )?;

    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    Ok(new_subject_container_id)
  }

  pub fn run_commands(
    commands: &Vec<InvokeCommand>,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let mut current_subject_container = subject_container;

    for command in commands {
      WindowManager::run_command(
        command,
        current_subject_container.clone(),
        state,
        config,
      )?;

      // Update the subject container in case the container type changes.
      // For example, when going from a tiling to a floating window.
      current_subject_container =
        if current_subject_container.is_detached() {
          match state.container_by_id(current_subject_container.id()) {
            Some(container) => container,
            None => break,
          }
        } else {
          current_subject_container
        }
    }

    Ok(current_subject_container.id())
  }

  pub fn run_command(
    command: &InvokeCommand,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    // No-op if WM is currently paused.
    if state.is_paused && *command != InvokeCommand::WmTogglePause {
      return Ok(());
    }

    if subject_container.is_detached() {
      bail!("Cannot run command because subject container is detached.");
    }

    match command {
      InvokeCommand::Focus(args) => {
        execute_focus_command(args, &subject_container, state, config)
      }
      InvokeCommand::Move(args) => {
        if let Ok(window) = subject_container.as_window_container() {
          execute_move_command(args, window, state, config)?;
        }
        Ok(())
      }
      InvokeCommand::AdjustBorders(_)
      | InvokeCommand::Close
      | InvokeCommand::ConsumeOrExpelWindowLeft
      | InvokeCommand::ConsumeOrExpelWindowRight
      | InvokeCommand::Ignore
      | InvokeCommand::Position(_)
      | InvokeCommand::Resize(_)
      | InvokeCommand::SetFloating { .. }
      | InvokeCommand::SetFullscreen { .. }
      | InvokeCommand::SetMinimized
      | InvokeCommand::SetTiling
      | InvokeCommand::SetTitleBarVisibility { .. }
      | InvokeCommand::SetTransparency(_)
      | InvokeCommand::Size(_)
      | InvokeCommand::ToggleFloating { .. }
      | InvokeCommand::ToggleFullscreen { .. }
      | InvokeCommand::ToggleMinimized
      | InvokeCommand::ToggleTiling => {
        if let Ok(window) = subject_container.as_window_container() {
          execute_window_command(command, window, state, config)?;
        }
        Ok(())
      }
      InvokeCommand::MoveWorkspace { .. }
      | InvokeCommand::UpdateWorkspaceConfig { .. }
      | InvokeCommand::ToggleTilingDirection
      | InvokeCommand::SetTilingDirection { .. }
      | InvokeCommand::CycleColumnPreset { .. }
      | InvokeCommand::SetColumnWidth { .. }
      | InvokeCommand::PanViewportLeft { .. }
      | InvokeCommand::PanViewportRight { .. } => {
        execute_layout_command(command, subject_container, state, config)
      }
      InvokeCommand::WmCycleFocus { .. }
      | InvokeCommand::WmDisableBindingMode { .. }
      | InvokeCommand::WmEnableBindingMode { .. }
      | InvokeCommand::WmExit
      | InvokeCommand::WmRedraw
      | InvokeCommand::WmReloadConfig
      | InvokeCommand::WmTogglePause
      | InvokeCommand::ShellExec { .. } => {
        execute_system_command(command, state, config)
      }
    }
  }

  /// Runs cleanup tasks when the WM is exiting.
  pub(crate) fn cleanup(
    &mut self,
    config: &mut UserConfig,
    ipc_server: &mut IpcServer,
  ) {
    self.state.emit_event(WmEvent::ApplicationExiting);

    // Ensure that the WM is unpaused, otherwise, shutdown commands won't
    // get executed.
    self.state.is_paused = false;

    // Run user's shutdown commands.
    if let Err(err) = self.process_commands(
      &config.value.general.shutdown_commands.clone(),
      None,
      config,
    ) {
      tracing::warn!("Failed to run shutdown commands: {:?}", err);
    }

    // Emit remaining WM events before exiting.
    while let Ok(wm_event) = self.event_rx.try_recv() {
      tracing::info!(
        "Emitting WM event before shutting down: {:?}",
        wm_event
      );

      if let Err(err) = ipc_server.process_event(wm_event) {
        tracing::warn!("{:?}", err);
      }
    }
  }
}

fn execute_focus_command(
  args: &InvokeFocusCommand,
  subject_container: &Container,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  if let Some(direction) = &args.direction {
    focus_in_direction(subject_container, direction, state)?;
  }

  if let Some(container_id) = &args.container_id {
    focus_container_by_id(container_id, state)?;
  }

  if let Some(monitor_index) = &args.monitor {
    focus_monitor(*monitor_index, state, config)?;
  }

  if let Some(target) = args.to_workspace_target() {
    focus_workspace(target, state, config)?;
  }

  Ok(())
}

fn execute_move_command(
  args: &InvokeMoveCommand,
  window: WindowContainer,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  if let Some(direction) = &args.direction {
    move_window_in_direction(
      window.clone(),
      direction,
      state,
      config,
    )?;
  }

  if let Some(target) = args.to_workspace_target() {
    move_window_to_workspace(window, target, state, config)?;
  }

  Ok(())
}

fn execute_set_floating(
  window: WindowContainer,
  command: &InvokeCommand,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let InvokeCommand::SetFloating {
    centered,
    shown_on_top,
    x_pos,
    y_pos,
    width,
    height,
  } = command
  else {
    return Ok(());
  };

  let floating_defaults =
    &config.value.window_behavior.state_defaults.floating;
  let centered = centered.unwrap_or(floating_defaults.centered);

  let window = update_window_state(
    window,
    WindowState::Floating(FloatingStateConfig {
      centered,
      shown_on_top: shown_on_top
        .unwrap_or(floating_defaults.shown_on_top),
    }),
    state,
    config,
  )?;

  // Allow size and position to be set if window has not previously
  // been manually placed.
  if !window.has_custom_floating_placement() {
    if width.is_some() || height.is_some() {
      set_window_size(
        window.clone(),
        width.clone(),
        height.clone(),
        state,
      )?;
    }

    if centered {
      set_window_position(
        window,
        &WindowPositionTarget::Centered,
        state,
      )?;
    } else if x_pos.is_some() || y_pos.is_some() {
      set_window_position(
        window,
        &WindowPositionTarget::Coordinates(*x_pos, *y_pos),
        state,
      )?;
    }
  }

  Ok(())
}

fn execute_toggle_floating(
  window: &WindowContainer,
  centered: Option<bool>,
  shown_on_top: Option<bool>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let floating_defaults =
    &config.value.window_behavior.state_defaults.floating;

  let centered = centered.unwrap_or(floating_defaults.centered);
  let target_state = WindowState::Floating(FloatingStateConfig {
    centered,
    shown_on_top: shown_on_top
      .unwrap_or(floating_defaults.shown_on_top),
  });

  let window = update_window_state(
    window.clone(),
    window.toggled_state(target_state, config),
    state,
    config,
  )?;

  if !window.has_custom_floating_placement() && centered {
    set_window_position(
      window,
      &WindowPositionTarget::Centered,
      state,
    )?;
  }

  Ok(())
}

fn execute_toggle_fullscreen(
  window: &WindowContainer,
  maximized: Option<bool>,
  shown_on_top: Option<bool>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let fullscreen_defaults =
    &config.value.window_behavior.state_defaults.fullscreen;

  let target_state =
    WindowState::Fullscreen(FullscreenStateConfig {
      maximized: maximized
        .unwrap_or(fullscreen_defaults.maximized),
      shown_on_top: shown_on_top
        .unwrap_or(fullscreen_defaults.shown_on_top),
    });

  update_window_state(
    window.clone(),
    window.toggled_state(target_state, config),
    state,
    config,
  )?;

  Ok(())
}

fn execute_window_state_command(
  command: &InvokeCommand,
  window: WindowContainer,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  match command {
    InvokeCommand::SetFloating { .. } => {
      execute_set_floating(window, command, state, config)
    }
    InvokeCommand::ToggleFloating {
      centered,
      shown_on_top,
    } => execute_toggle_floating(&window, *centered, *shown_on_top, state, config),
    InvokeCommand::SetFullscreen {
      maximized,
      shown_on_top,
    } => {
      let fullscreen_defaults =
        &config.value.window_behavior.state_defaults.fullscreen;

      update_window_state(
        window,
        WindowState::Fullscreen(FullscreenStateConfig {
          maximized: maximized
            .unwrap_or(fullscreen_defaults.maximized),
          shown_on_top: shown_on_top
            .unwrap_or(fullscreen_defaults.shown_on_top),
        }),
        state,
        config,
      )?;
      Ok(())
    }
    InvokeCommand::ToggleFullscreen {
      maximized,
      shown_on_top,
    } => execute_toggle_fullscreen(&window, *maximized, *shown_on_top, state, config),
    InvokeCommand::SetMinimized => {
      update_window_state(
        window,
        WindowState::Minimized,
        state,
        config,
      )?;
      Ok(())
    }
    InvokeCommand::ToggleMinimized => {
      let toggled = window.toggled_state(WindowState::Minimized, config);
      update_window_state(
        window,
        toggled,
        state,
        config,
      )?;
      Ok(())
    }
    InvokeCommand::SetTiling => {
      update_window_state(
        window,
        WindowState::Tiling,
        state,
        config,
      )?;
      Ok(())
    }
    InvokeCommand::ToggleTiling => {
      let toggled = window.toggled_state(WindowState::Tiling, config);
      update_window_state(
        window,
        toggled,
        state,
        config,
      )?;
      Ok(())
    }
    _ => Ok(()),
  }
}

fn execute_window_command(
  command: &InvokeCommand,
  window: WindowContainer,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  match command {
    InvokeCommand::AdjustBorders(args) => {
      let args = args.clone();
      let border_delta = RectDelta::new(
        args.left.unwrap_or(LengthValue::from_px(0)),
        args.top.unwrap_or(LengthValue::from_px(0)),
        args.right.unwrap_or(LengthValue::from_px(0)),
        args.bottom.unwrap_or(LengthValue::from_px(0)),
      );

      window.set_border_delta(border_delta);
      state.pending_sync.queue_container_to_redraw(window);
      Ok(())
    }
    InvokeCommand::Close => {
      if let Err(err) = window.native().close() {
        warn!("Failed to close window: {:?}", err);
      }
      Ok(())
    }
    InvokeCommand::ConsumeOrExpelWindowLeft => consume_or_expel_window(
      window,
      &Direction::Left,
      state,
      config,
    ),
    InvokeCommand::ConsumeOrExpelWindowRight => consume_or_expel_window(
      window,
      &Direction::Right,
      state,
      config,
    ),
    InvokeCommand::Ignore => ignore_window(window, state),
    InvokeCommand::Position(args) => {
      if args.centered {
        set_window_position(
          window,
          &WindowPositionTarget::Centered,
          state,
        )
      } else {
        set_window_position(
          window,
          &WindowPositionTarget::Coordinates(args.x_pos, args.y_pos),
          state,
        )
      }
    }
    InvokeCommand::Resize(args) => resize_window(
      &window,
      args.width.clone(),
      args.height.clone(),
      state,
    ),
    InvokeCommand::Size(args) => set_window_size(
      window,
      args.width.clone(),
      args.height.clone(),
      state,
    ),
    InvokeCommand::SetFloating { .. }
    | InvokeCommand::ToggleFloating { .. }
    | InvokeCommand::SetFullscreen { .. }
    | InvokeCommand::ToggleFullscreen { .. }
    | InvokeCommand::SetMinimized
    | InvokeCommand::ToggleMinimized
    | InvokeCommand::SetTiling
    | InvokeCommand::ToggleTiling => {
      execute_window_state_command(command, window, state, config)
    }
    InvokeCommand::SetTitleBarVisibility {
      #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
      visibility,
    } => {
      #[cfg(target_os = "windows")]
      {
        _ = window.native().set_title_bar_visibility(
          *visibility == TitleBarVisibility::Shown,
        );
      }
      Ok(())
    }
    InvokeCommand::SetTransparency(args) => {
      #[cfg(target_os = "windows")]
      {
        if let Some(opacity) = &args.opacity {
          _ = window.native().set_transparency(opacity);
        }

        if let Some(opacity_delta) = &args.opacity_delta {
          _ = window.native().adjust_transparency(opacity_delta);
        }
      }
      #[cfg(not(target_os = "windows"))]
      {
        let _ = args;
      }
      Ok(())
    }
    _ => Ok(()),
  }
}

fn pan_viewport(
  subject_container: &Container,
  delta: f64,
  state: &mut WmState,
  config: &mut UserConfig,
) {
  let workspace = subject_container.workspace().or_else(|| {
    state.focused_container().and_then(|c| c.workspace())
  });
  if let Some(workspace) = workspace {
    let new_offset = (workspace.offset_x() + delta).max(0.0);
    crate::commands::general::animate_pan_workspace(
      &workspace, new_offset, state, config,
    );
    state.pending_sync.queue_container_to_redraw(workspace);
  }
}

fn execute_layout_command(
  command: &InvokeCommand,
  subject_container: Container,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  match command {
    InvokeCommand::MoveWorkspace { direction } => {
      let workspace =
        subject_container.workspace().context("No workspace.")?;
      move_workspace_in_direction(&workspace, direction, state, config)
    }
    InvokeCommand::UpdateWorkspaceConfig {
      workspace,
      new_config,
    } => {
      let workspace = if let Some(workspace_name) = workspace {
        state
          .workspace_by_name(workspace_name)
          .context("Workspace doesn't exist.")?
      } else {
        subject_container.workspace().context("No workspace.")?
      };
      update_workspace_config(&workspace, state, config, new_config)
    }
    InvokeCommand::ToggleTilingDirection => {
      toggle_tiling_direction(subject_container, state, config)
    }
    InvokeCommand::SetTilingDirection { tiling_direction } => {
      set_tiling_direction(
        subject_container,
        state,
        config,
        tiling_direction,
      )
    }
    InvokeCommand::CycleColumnPreset { presets } => cycle_column_preset(
      &subject_container,
      presets.as_deref(),
      state,
      &config.value,
    ),
    InvokeCommand::SetColumnWidth { width } => {
      set_column_width(&subject_container, width, state)
    }
    InvokeCommand::PanViewportLeft { amount } => {
      let delta = amount.unwrap_or(150.0);
      pan_viewport(&subject_container, -delta, state, config);
      Ok(())
    }
    InvokeCommand::PanViewportRight { amount } => {
      let delta = amount.unwrap_or(150.0);
      pan_viewport(&subject_container, delta, state, config);
      Ok(())
    }
    _ => Ok(()),
  }
}

fn execute_system_command(
  command: &InvokeCommand,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  match command {
    InvokeCommand::WmCycleFocus {
      omit_floating,
      omit_fullscreen,
      omit_minimized,
      omit_tiling,
    } => cycle_focus(
      *omit_floating,
      *omit_fullscreen,
      *omit_minimized,
      *omit_tiling,
      state,
      config,
    ),
    InvokeCommand::WmDisableBindingMode { name } => {
      disable_binding_mode(name, state);
      Ok(())
    }
    InvokeCommand::WmEnableBindingMode { name } => {
      enable_binding_mode(name, state, config)
    }
    InvokeCommand::WmExit => state.emit_exit(),
    InvokeCommand::WmRedraw => {
      state
        .pending_sync
        .queue_container_to_redraw(state.root_container.clone());
      Ok(())
    }
    InvokeCommand::WmReloadConfig => reload_config(state, config),
    InvokeCommand::WmTogglePause => {
      toggle_pause(state);
      Ok(())
    }
    InvokeCommand::ShellExec {
      hide_window,
      command,
    } => shell_exec(&command.join(" "), *hide_window, state),
    _ => Ok(()),
  }
}
