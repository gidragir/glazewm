use std::{
  cell::{Ref, RefCell},
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ActiveDrag, ContainerDto, DisplayState, GapsConfig, TilingDirection,
  WindowDto, WindowRuleConfig, WindowState,
};
use wm_platform::{NativeWindow, Rect, RectDelta};

use crate::{
  impl_container_debug, impl_leaf_common_getters,
  impl_position_getters_as_resizable, impl_tiling_size_getters,
  impl_window_getters,
  models::{
    InsertionTarget, NativeWindowProperties, NonTilingWindow,
    WeakContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters, WindowGetters,
  },
};

#[derive(Clone)]
pub struct TilingWindow(pub(crate) Rc<RefCell<TilingWindowInner>>);

pub(crate) struct TilingWindowInner {
  pub(crate) id: Uuid,
  pub(crate) parent: Option<WeakContainer>,
  pub(crate) tiling_size: f32,
  pub(crate) native: NativeWindow,
  pub(crate) native_properties: NativeWindowProperties,
  pub(crate) state: WindowState,
  pub(crate) prev_state: Option<WindowState>,
  pub(crate) display_state: DisplayState,
  pub(crate) border_delta: RectDelta,
  pub(crate) has_pending_dpi_adjustment: bool,
  pub(crate) floating_placement: Rect,
  pub(crate) has_custom_floating_placement: bool,
  pub(crate) gaps_config: GapsConfig,
  pub(crate) done_window_rules: Vec<WindowRuleConfig>,
  pub(crate) active_drag: Option<ActiveDrag>,
}

impl TilingWindow {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    id: Option<Uuid>,
    native: NativeWindow,
    properties: NativeWindowProperties,
    prev_state: Option<WindowState>,
    border_delta: RectDelta,
    floating_placement: Rect,
    has_custom_floating_placement: bool,
    gaps_config: GapsConfig,
    done_window_rules: Vec<WindowRuleConfig>,
    active_drag: Option<ActiveDrag>,
  ) -> Self {
    let window = TilingWindowInner {
      id: id.unwrap_or_else(Uuid::new_v4),
      parent: None,
      tiling_size: 1.0,
      native,
      native_properties: properties,
      state: WindowState::Tiling,
      prev_state,
      display_state: DisplayState::Shown,
      border_delta,
      has_pending_dpi_adjustment: false,
      floating_placement,
      has_custom_floating_placement,
      gaps_config,
      done_window_rules,
      active_drag,
    };

    Self(Rc::new(RefCell::new(window)))
  }

  pub fn to_non_tiling(
    &self,
    state: WindowState,
    insertion_target: Option<InsertionTarget>,
  ) -> NonTilingWindow {
    NonTilingWindow::new(
      Some(self.id()),
      self.native().clone(),
      self.native_properties().clone(),
      state,
      Some(WindowState::Tiling),
      self.border_delta(),
      insertion_target,
      self.floating_placement(),
      self.has_custom_floating_placement(),
      self.done_window_rules(),
      self.active_drag(),
    )
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;

    Ok(ContainerDto::Window(WindowDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      has_focus: self.has_focus(None),
      tiling_size: Some(self.tiling_size()),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
      state: self.state(),
      prev_state: self.prev_state(),
      display_state: self.display_state(),
      border_delta: self.border_delta(),
      floating_placement: self.floating_placement(),
      #[allow(clippy::cast_possible_wrap, clippy::unnecessary_cast)]
      handle: self.native().id().0 as isize,
      title: self.native_properties().title,
      #[cfg(target_os = "windows")]
      class_name: self.native_properties().class_name,
      process_name: self.native_properties().process_name,
      active_drag: self.active_drag(),
    }))
  }
}

impl_container_debug!(TilingWindow);
impl_leaf_common_getters!(TilingWindow);
impl_tiling_size_getters!(TilingWindow);
impl_position_getters_as_resizable!(TilingWindow);
impl_window_getters!(TilingWindow);
