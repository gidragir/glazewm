use anyhow::Context;
use wm_common::ParsedConfig;
use wm_platform::LengthValue;

use crate::{
  commands::container::resize_tiling_container,
  models::{Container, TilingContainer},
  traits::{CommonGetters, PositionGetters, TilingSizeGetters},
  wm_state::WmState,
};

const EPSILON: f32 = 0.01;

/// Finds the top-level column ancestor (immediate child of the horizontal `Workspace`).
#[must_use]
pub fn find_column_ancestor(container: &Container) -> Option<TilingContainer> {
  let mut current = container.clone();
  while let Some(parent) = current.parent() {
    if parent.as_workspace().is_some() {
      return current.as_tiling_container().ok();
    }
    current = parent;
  }
  None
}

/// Cycles the width of the focused column through configured presets.
pub fn cycle_column_preset(
  subject_container: &Container,
  custom_presets: Option<&[LengthValue]>,
  state: &mut WmState,
  config: &ParsedConfig,
) -> anyhow::Result<()> {
  let column_container = find_column_ancestor(subject_container)
    .context("Focused container is not inside a workspace column.")?;

  let parent = column_container
    .parent()
    .context("Column container has no parent.")?;

  let parent_width = parent.to_rect()?.width();

  let presets = custom_presets
    .unwrap_or(&config.general.column_width_presets);

  if presets.is_empty() {
    return Ok(());
  }

  let preset_fractions: Vec<f32> = presets
    .iter()
    .map(|p| p.to_percentage(parent_width))
    .collect();

  let current_size = column_container.tiling_size();

  let matching_index = preset_fractions
    .iter()
    .position(|&p| (p - current_size).abs() < EPSILON);

  let next_index = matching_index.map_or_else(
    || {
      preset_fractions
        .iter()
        .position(|&p| p > current_size + 0.005)
        .unwrap_or(0)
    },
    |idx| (idx + 1) % preset_fractions.len(),
  );

  let target_size = preset_fractions[next_index];
  tracing::debug!(
    "Cycling column {} preset from {:.2} to {:.2} (index {}/{})",
    column_container.id(),
    current_size,
    target_size,
    next_index + 1,
    preset_fractions.len()
  );

  resize_tiling_container(&column_container, target_size);

  state
    .pending_sync
    .queue_containers_to_redraw(parent.tiling_children());

  Ok(())
}

/// Sets the width of the focused column directly to a specified length value.
pub fn set_column_width(
  subject_container: &Container,
  width: &LengthValue,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let column_container = find_column_ancestor(subject_container)
    .context("Focused container is not inside a workspace column.")?;

  let parent = column_container
    .parent()
    .context("Column container has no parent.")?;

  let parent_width = parent.to_rect()?.width();
  let target_size = width.to_percentage(parent_width);

  tracing::debug!(
    "Setting column {} width to {:.2}",
    column_container.id(),
    target_size
  );

  resize_tiling_container(&column_container, target_size);

  state
    .pending_sync
    .queue_containers_to_redraw(parent.tiling_children());

  Ok(())
}
