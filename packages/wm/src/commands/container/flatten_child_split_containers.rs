use super::flatten_split_container;
use crate::{
  models::{Container, DirectionContainer, SplitContainer},
  traits::{CommonGetters, TilingDirectionGetters},
};

fn flatten_single_child_split(
  parent: &DirectionContainer,
  split_child: SplitContainer,
) -> anyhow::Result<()> {
  flatten_split_container(split_child)?;
  parent.set_tiling_direction(parent.tiling_direction().inverse());
  Ok(())
}

fn flatten_matching_direction_split(
  split_child: SplitContainer,
) -> anyhow::Result<()> {
  if split_child.child_count() == 1
    && let Some(split_grandchild) = split_child.children()[0].as_split()
  {
    flatten_split_container(split_grandchild.clone())?;
  }

  flatten_split_container(split_child)?;
  Ok(())
}

fn flatten_split_children(
  parent: &DirectionContainer,
  tiling_children: &[Container],
) -> anyhow::Result<()> {
  let split_children = tiling_children
    .iter()
    .filter_map(|child| child.as_split().cloned())
    .filter(|split_child| {
      split_child.tiling_direction() == parent.tiling_direction()
    });

  for split_child in split_children {
    flatten_matching_direction_split(split_child)?;
  }

  Ok(())
}

/// Flattens any redundant split containers at the top-level of the given
/// parent container.
///
/// For example:
/// ```ignore,compile_fail
/// H[1 H[V[2, 3]]] -> H[1, 2, 3]
/// H[1 H[2, 3]] -> H[1, 2, 3]
/// H[V[1]] -> V[1]
/// ```
pub fn flatten_child_split_containers(
  parent: &Container,
) -> anyhow::Result<()> {
  let Ok(parent) = parent.as_direction_container() else {
    return Ok(());
  };

  // Get children that are either tiling windows or split containers.
  let tiling_children = parent
    .children()
    .into_iter()
    .filter(|child| child.is_tiling_window() || child.is_split())
    .collect::<Vec<_>>();

  if tiling_children.len() == 1 && parent.is_split() {
    if let Some(split_child) = tiling_children[0].as_split() {
      flatten_single_child_split(&parent, split_child.clone())?;
    }
  } else {
    flatten_split_children(&parent, &tiling_children)?;
  }

  Ok(())
}
