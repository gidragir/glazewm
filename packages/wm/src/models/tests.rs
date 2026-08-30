use wm_common::{GapsConfig, TilingDirection};

use crate::{
  models::{
    Container, Monitor, NonTilingWindow, SplitContainer, TilingContainer,
    TilingWindow, Workspace,
  },
  traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
};

#[test]
fn container_hierarchy_parent_and_children_navigation() {
  let win_a = TilingWindow::mock().title("Window A".into()).call();
  let win_b = TilingWindow::mock().title("Window B".into()).call();

  let split = SplitContainer::mock()
    .tiling_direction(TilingDirection::Vertical)
    .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
    .call();

  let workspace = Workspace::mock()
    .tiling_direction(TilingDirection::Horizontal)
    .tiling_containers(vec![split.clone().into()])
    .call();

  let monitor = Monitor::mock().workspaces(vec![workspace.clone()]).call();

  // Test root monitor level
  assert_eq!(monitor.child_count(), 1);
  assert!(monitor.is_detached());
  assert!(monitor.parent().is_none());

  // Test workspace level
  assert_eq!(workspace.parent().map(|p| p.id()), Some(monitor.id()));
  assert_eq!(workspace.child_count(), 1);
  assert_eq!(workspace.index(), 0);
  assert!(!workspace.is_detached());

  // Test split container level
  assert_eq!(split.parent().map(|p| p.id()), Some(workspace.id()));
  assert_eq!(split.child_count(), 2);
  assert_eq!(split.index(), 0);

  // Test leaf windows level
  assert_eq!(win_a.parent().map(|p| p.id()), Some(split.id()));
  assert_eq!(win_b.parent().map(|p| p.id()), Some(split.id()));
  assert_eq!(win_a.index(), 0);
  assert_eq!(win_b.index(), 1);
  assert!(!win_a.has_children());
  assert_eq!(win_a.child_count(), 0);
}

#[test]
fn container_ancestors_traversal() {
  let win = TilingWindow::mock().call();
  let split = SplitContainer::mock()
    .tiling_containers(vec![win.clone().into()])
    .call();
  let workspace = Workspace::mock()
    .tiling_containers(vec![split.clone().into()])
    .call();
  let monitor = Monitor::mock().workspaces(vec![workspace.clone()]).call();

  let ancestors: Vec<Container> = win.ancestors().collect();
  assert_eq!(ancestors.len(), 3);
  assert_eq!(ancestors[0].id(), split.id());
  assert_eq!(ancestors[1].id(), workspace.id());
  assert_eq!(ancestors[2].id(), monitor.id());

  let self_and_ancestors: Vec<Container> =
    win.self_and_ancestors().collect();
  assert_eq!(self_and_ancestors.len(), 4);
  assert_eq!(self_and_ancestors[0].id(), win.id());
  assert_eq!(self_and_ancestors[1].id(), split.id());

  // Direct ancestor lookups
  assert_eq!(win.workspace().map(|w| w.id()), Some(workspace.id()));
  assert_eq!(win.monitor().map(|m| m.id()), Some(monitor.id()));
  assert_eq!(win.direction_container().map(|d| d.id()), Some(split.id()));
}

#[test]
fn container_descendants_traversal() {
  let win_a = TilingWindow::mock().call();
  let win_b = TilingWindow::mock().call();
  let win_c = TilingWindow::mock().call();

  let inner_split = SplitContainer::mock()
    .tiling_containers(vec![win_b.clone().into(), win_c.clone().into()])
    .call();

  let root_split = SplitContainer::mock()
    .tiling_containers(vec![
      win_a.clone().into(),
      inner_split.clone().into(),
    ])
    .call();

  let workspace = Workspace::mock()
    .tiling_containers(vec![root_split.clone().into()])
    .call();

  let descendants: Vec<Container> = workspace.descendants().collect();
  assert_eq!(descendants.len(), 5);
  assert_eq!(descendants[0].id(), root_split.id());
  assert_eq!(descendants[1].id(), win_a.id());
  assert_eq!(descendants[2].id(), inner_split.id());
  assert_eq!(descendants[3].id(), win_b.id());
  assert_eq!(descendants[4].id(), win_c.id());

  let self_descendants: Vec<Container> =
    workspace.self_and_descendants().collect();
  assert_eq!(self_descendants.len(), 6);
  assert_eq!(self_descendants[0].id(), workspace.id());
}

#[test]
fn container_siblings_traversal() {
  let win_a = TilingWindow::mock().call();
  let win_b = TilingWindow::mock().call();
  let win_c = TilingWindow::mock().call();

  let _split = SplitContainer::mock()
    .tiling_containers(vec![
      win_a.clone().into(),
      win_b.clone().into(),
      win_c.clone().into(),
    ])
    .call();

  // Test siblings for middle window (win_b)
  let siblings: Vec<Container> = win_b.siblings().collect();
  assert_eq!(siblings.len(), 2);
  assert_eq!(siblings[0].id(), win_a.id());
  assert_eq!(siblings[1].id(), win_c.id());

  // Test prev_siblings and next_siblings
  let prev_siblings: Vec<Container> = win_b.prev_siblings().collect();
  assert_eq!(prev_siblings.len(), 1);
  assert_eq!(prev_siblings[0].id(), win_a.id());

  let next_siblings: Vec<Container> = win_b.next_siblings().collect();
  assert_eq!(next_siblings.len(), 1);
  assert_eq!(next_siblings[0].id(), win_c.id());

  // Test tiling_siblings
  let tiling_sibs: Vec<TilingContainer> =
    win_a.tiling_siblings().collect();
  assert_eq!(tiling_sibs.len(), 2);
  assert_eq!(tiling_sibs[0].id(), win_b.id());
  assert_eq!(tiling_sibs[1].id(), win_c.id());
}

#[test]
fn container_conversions_and_subenums() {
  let tiling_win = TilingWindow::mock().call();
  let split = SplitContainer::mock().call();
  let non_tiling_win = NonTilingWindow::mock().call();
  let workspace = Workspace::mock().call();

  let container_tiling: Container = tiling_win.clone().into();
  let container_split: Container = split.clone().into();
  let container_non_tiling: Container = non_tiling_win.clone().into();
  let container_workspace: Container = workspace.clone().into();

  // Tiling container conversions
  assert!(container_tiling.as_tiling_container().is_ok());
  assert!(container_split.as_tiling_container().is_ok());
  assert!(container_non_tiling.as_tiling_container().is_err());
  assert!(container_workspace.as_tiling_container().is_err());

  // Window container conversions
  assert!(container_tiling.as_window_container().is_ok());
  assert!(container_non_tiling.as_window_container().is_ok());
  assert!(container_split.as_window_container().is_err());

  // Direction container conversions
  assert!(container_split.as_direction_container().is_ok());
  assert!(container_workspace.as_direction_container().is_ok());
  assert!(container_tiling.as_direction_container().is_err());
}

#[test]
fn container_child_by_id_lookup() {
  let win_a = TilingWindow::mock().call();
  let win_b = TilingWindow::mock().call();

  let split = SplitContainer::mock()
    .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
    .call();

  assert_eq!(
    split.child_by_id(&win_a.id()).map(|c| c.id()),
    Some(win_a.id())
  );
  assert_eq!(
    split.child_by_id(&win_b.id()).map(|c| c.id()),
    Some(win_b.id())
  );
  assert!(split.child_by_id(&uuid::Uuid::new_v4()).is_none());
}

#[test]
fn tiling_size_and_position_getters() {
  let win = TilingWindow::mock().tiling_size(0.75).call();
  assert!((win.tiling_size() - 0.75).abs() < f32::EPSILON);

  win.set_tiling_size(0.5);
  assert!((win.tiling_size() - 0.5).abs() < f32::EPSILON);

  let split = SplitContainer::mock()
    .tiling_direction(TilingDirection::Vertical)
    .call();
  assert_eq!(split.tiling_direction(), TilingDirection::Vertical);

  split.set_tiling_direction(TilingDirection::Horizontal);
  assert_eq!(split.tiling_direction(), TilingDirection::Horizontal);
}

#[test]
fn workspace_gaps_and_dto_serialization() {
  let win = TilingWindow::mock().title("Main Editor".into()).call();
  let workspace = Workspace::mock()
    .name("2".into())
    .tiling_containers(vec![win.clone().into()])
    .call();

  let _monitor =
    Monitor::mock().workspaces(vec![workspace.clone()]).call();

  // Test workspace outer gaps
  let gaps = workspace.outer_gaps();
  assert_eq!(gaps, GapsConfig::default().outer_gap);

  // Test to_dto serialization
  let dto_res = workspace.to_dto();
  assert!(dto_res.is_ok());

  let win_dto = win.to_dto();
  assert!(win_dto.is_ok());
}

#[test]
fn container_focus_and_descendant_focus_order() {
  let win_a = TilingWindow::mock().call();
  let win_b = TilingWindow::mock().call();

  let split = SplitContainer::mock()
    .tiling_containers(vec![win_a.clone().into(), win_b.clone().into()])
    .call();

  let workspace = Workspace::mock()
    .tiling_containers(vec![split.clone().into()])
    .call();

  let _monitor =
    Monitor::mock().workspaces(vec![workspace.clone()]).call();

  // Focus order is tracked by child_focus_order
  assert_eq!(win_a.focus_index(), 0);
  assert_eq!(win_b.focus_index(), 1);

  // Has focus checks (assuming 0th element in focus order has focus)
  assert!(win_a.has_focus(None));
  assert!(!win_b.has_focus(None));

  let leaf_focus: Vec<Container> =
    workspace.descendant_focus_order().collect();
  assert_eq!(leaf_focus.len(), 2);
  assert_eq!(leaf_focus[0].id(), win_a.id());
  assert_eq!(leaf_focus[1].id(), win_b.id());
}
