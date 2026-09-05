use std::{
  cell::{Ref, RefCell},
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ActiveDrag, ContainerDto, DisplayState, GapsConfig, WindowRuleConfig,
  WindowState,
};
use wm_platform::{NativeWindow, Rect, RectDelta};

use crate::{
  impl_container_debug, impl_leaf_common_getters, impl_window_getters,
  models::{
    InsertionTarget, NativeWindowProperties, TilingWindow, WeakContainer,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
};

#[derive(Clone)]
pub struct NonTilingWindow(pub(crate) Rc<RefCell<NonTilingWindowInner>>);

pub(crate) struct NonTilingWindowInner {
  pub(crate) id: Uuid,
  pub(crate) parent: Option<WeakContainer>,
  pub(crate) native: NativeWindow,
  pub(crate) native_properties: NativeWindowProperties,
  pub(crate) state: WindowState,
  pub(crate) prev_state: Option<WindowState>,
  pub(crate) insertion_target: Option<InsertionTarget>,
  pub(crate) display_state: DisplayState,
  pub(crate) border_delta: RectDelta,
  pub(crate) has_pending_dpi_adjustment: bool,
  pub(crate) floating_placement: Rect,
  pub(crate) has_custom_floating_placement: bool,
  pub(crate) done_window_rules: Vec<WindowRuleConfig>,
  pub(crate) active_drag: Option<ActiveDrag>,
}

impl NonTilingWindow {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    id: Option<Uuid>,
    native: NativeWindow,
    properties: NativeWindowProperties,
    state: WindowState,
    prev_state: Option<WindowState>,
    border_delta: RectDelta,
    insertion_target: Option<InsertionTarget>,
    floating_placement: Rect,
    has_custom_floating_placement: bool,
    done_window_rules: Vec<WindowRuleConfig>,
    active_drag: Option<ActiveDrag>,
  ) -> Self {
    let window = NonTilingWindowInner {
      id: id.unwrap_or_else(Uuid::new_v4),
      parent: None,
      native,
      native_properties: properties,
      state,
      prev_state,
      insertion_target,
      display_state: DisplayState::Shown,
      border_delta,
      has_pending_dpi_adjustment: false,
      floating_placement,
      has_custom_floating_placement,
      done_window_rules,
      active_drag,
    };

    Self(Rc::new(RefCell::new(window)))
  }

  pub fn insertion_target(&self) -> Option<InsertionTarget> {
    self.0.borrow().insertion_target.clone()
  }

  pub fn set_insertion_target(
    &self,
    insertion_target: Option<InsertionTarget>,
  ) {
    self.0.borrow_mut().insertion_target = insertion_target;
  }

  pub fn to_tiling(&self, gaps_config: GapsConfig) -> TilingWindow {
    let prev_state = if self.active_drag().is_some() {
      self.prev_state()
    } else {
      Some(self.state())
    };

    TilingWindow::new(
      Some(self.id()),
      self.native().clone(),
      self.native_properties().clone(),
      prev_state,
      self.border_delta(),
      self.floating_placement(),
      self.has_custom_floating_placement(),
      gaps_config,
      self.done_window_rules(),
      self.active_drag(),
    )
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    crate::traits::window_to_dto(self, None)
  }
}

impl_container_debug!(NonTilingWindow);
impl_leaf_common_getters!(NonTilingWindow);
impl_window_getters!(NonTilingWindow);

impl PositionGetters for NonTilingWindow {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    match self.state() {
      WindowState::Fullscreen(_) => {
        let monitor = self.monitor().context("No monitor.")?;

        #[cfg(target_os = "windows")]
        {
          monitor.to_rect()
        }
        #[cfg(target_os = "macos")]
        {
          // On macOS, the public APIs only allow window placement within
          // the display's working area.
          Ok(monitor.native_properties().working_area)
        }
      }
      _ => Ok(self.floating_placement()),
    }
  }
}
