use std::{
  cell::{Ref, RefMut},
  collections::VecDeque,
};

use ambassador::delegatable_trait;
use uuid::Uuid;
use wm_common::ContainerDto;

use crate::models::{
  Container, DirectionContainer, Monitor, TilingContainer,
  WindowContainer, Workspace,
};

#[delegatable_trait]
pub trait CommonGetters {
  /// A unique identifier for the container.
  fn id(&self) -> Uuid;

  fn as_container(&self) -> Container;

  fn as_tiling_container(&self) -> anyhow::Result<TilingContainer>;

  fn as_window_container(&self) -> anyhow::Result<WindowContainer>;

  fn as_direction_container(&self) -> anyhow::Result<DirectionContainer>;

  fn to_dto(&self) -> anyhow::Result<ContainerDto>;

  /// Gets the parent container, unless this container is the root or
  /// detached.
  fn parent(&self) -> Option<Container>;

  /// Sets the parent container. Pass `None` to detach.
  fn set_parent(&self, parent: Option<&Container>);

  fn borrow_children(&self) -> Ref<'_, VecDeque<Container>>;

  fn borrow_children_mut(&self) -> RefMut<'_, VecDeque<Container>>;

  fn borrow_child_focus_order(&self) -> Ref<'_, VecDeque<Uuid>>;

  fn borrow_child_focus_order_mut(&self) -> RefMut<'_, VecDeque<Uuid>>;

  /// Direct children of this container.
  fn children(&self) -> VecDeque<Container> {
    self.borrow_children().clone()
  }

  /// Number of children that this container has.
  fn child_count(&self) -> usize {
    self.borrow_children().len()
  }

  /// Whether this container has any direct children.
  fn has_children(&self) -> bool {
    !self.borrow_children().is_empty()
  }

  /// Whether this container is detached from the tree (i.e. it does not
  /// have a parent).
  fn is_detached(&self) -> bool {
    self.parent().is_none()
  }

  /// Index of this container amongst its siblings.
  ///
  /// Returns 0 if the container has no parent.
  fn index(&self) -> usize {
    self
      .parent()
      .and_then(|parent| {
        parent
          .borrow_children()
          .iter()
          .position(|child| child.id() == self.id())
      })
      .unwrap_or(0)
  }

  /// Gets child container with the given ID.
  fn child_by_id(&self, child_id: &Uuid) -> Option<Container> {
    self
      .borrow_children()
      .iter()
      .find(|child| &child.id() == child_id)
      .cloned()
  }

  fn child_focus_order_ids(&self) -> Vec<Uuid> {
    self.borrow_child_focus_order().iter().copied().collect()
  }

  fn tiling_children(&self) -> TilingChildren {
    TilingChildren {
      iter: self.children().into_iter(),
    }
  }

  fn descendants(&self) -> Descendants {
    Descendants {
      stack: self.children(),
    }
  }

  fn self_and_descendants(&self) -> Descendants {
    let mut stack = VecDeque::new();
    stack.push_back(self.as_container());
    Descendants { stack }
  }

  /// Children in order of last focus.
  fn child_focus_order(&self) -> ChildFocusOrder {
    ChildFocusOrder {
      container: self.as_container(),
      ids: self.child_focus_order_ids(),
      index: 0,
    }
  }

  /// Leaf nodes (i.e. windows and workspaces) in order of last focus.
  fn descendant_focus_order(&self) -> DescendantFocusOrder {
    DescendantFocusOrder {
      root_id: self.id(),
      stack: vec![self.as_container()],
    }
  }

  fn siblings(&self) -> Siblings {
    Siblings {
      id: self.id(),
      iter: self
        .parent()
        .map_or_else(VecDeque::new, |parent| parent.children())
        .into_iter(),
    }
  }

  fn self_and_siblings(&self) -> SelfAndSiblings {
    SelfAndSiblings {
      iter: self
        .parent()
        .map_or_else(VecDeque::new, |parent| parent.children())
        .into_iter(),
    }
  }

  fn prev_siblings(&self) -> PrevSiblings {
    let index = self.index();
    let prev = self
      .self_and_siblings()
      .take(index)
      .collect::<Vec<_>>()
      .into_iter()
      .rev()
      .collect::<Vec<_>>();
    PrevSiblings {
      iter: prev.into_iter(),
    }
  }

  fn next_siblings(&self) -> NextSiblings {
    let index = self.index();
    let mut iter = self
      .parent()
      .map_or_else(VecDeque::new, |parent| parent.children())
      .into_iter();
    if index < iter.len() {
      iter.nth(index);
    } else {
      iter = VecDeque::new().into_iter();
    }
    NextSiblings { iter }
  }

  fn tiling_siblings(&self) -> TilingSiblings {
    TilingSiblings {
      id: self.id(),
      iter: self
        .parent()
        .map_or_else(VecDeque::new, |parent| parent.children())
        .into_iter(),
    }
  }

  fn ancestors(&self) -> Ancestors {
    Ancestors {
      start: self.parent(),
    }
  }

  fn self_and_ancestors(&self) -> Ancestors {
    Ancestors {
      start: Some(self.as_container()),
    }
  }

  /// Workspace that this container belongs to.
  ///
  /// Note that this might return the container itself.
  fn workspace(&self) -> Option<Workspace> {
    self
      .self_and_ancestors()
      .find_map(|container| container.as_workspace().cloned())
  }

  /// Monitor that this container belongs to.
  ///
  /// Note that this might return the container itself.
  fn monitor(&self) -> Option<Monitor> {
    self
      .self_and_ancestors()
      .find_map(|container| container.as_monitor().cloned())
  }

  /// Nearest direction container (i.e. split container or workspace) that
  /// this container belongs to.
  ///
  /// Note that this might return the container itself.
  fn direction_container(&self) -> Option<DirectionContainer> {
    self
      .self_and_ancestors()
      .find_map(|container| container.try_into().ok())
  }

  /// Index of this container in parent's child focus order.
  ///
  /// Returns 0 if the container has no parent.
  fn focus_index(&self) -> usize {
    self
      .parent()
      .and_then(|parent| {
        parent
          .borrow_child_focus_order()
          .iter()
          .position(|id| id == &self.id())
      })
      .unwrap_or(0)
  }

  /// Whether this container or a descendant has focus.
  ///
  /// If `end_ancestor` is provided, then the check for focus will be up to
  /// and including the `end_ancestor`.
  fn has_focus(&self, end_ancestor: Option<Container>) -> bool {
    self
      .self_and_ancestors()
      .take_while(|ancestor| end_ancestor.as_ref() != Some(ancestor))
      .chain(end_ancestor.clone())
      .all(|ancestor| ancestor.focus_index() == 0)
  }
}

/// An iterator over ancestors of a given container.
pub struct Ancestors {
  start: Option<Container>,
}

impl Iterator for Ancestors {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    self.start.take().inspect(|container| {
      self.start = container.parent();
    })
  }
}

/// An iterator over descendants of a given container.
pub struct Descendants {
  stack: VecDeque<Container>,
}

impl Iterator for Descendants {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    if let Some(container) = self.stack.pop_front() {
      self.stack.extend(container.children());
      return Some(container);
    }
    None
  }
}

/// An iterator over tiling children of a given container.
pub struct TilingChildren {
  iter: std::collections::vec_deque::IntoIter<Container>,
}

impl Iterator for TilingChildren {
  type Item = TilingContainer;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.find_map(|c| c.try_into().ok())
  }
}

/// An iterator over siblings of a given container.
pub struct Siblings {
  id: Uuid,
  iter: std::collections::vec_deque::IntoIter<Container>,
}

impl Iterator for Siblings {
  type Item = Container;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.find(|c| c.id() != self.id)
  }
}

/// An iterator over a container and its siblings.
pub struct SelfAndSiblings {
  iter: std::collections::vec_deque::IntoIter<Container>,
}

impl Iterator for SelfAndSiblings {
  type Item = Container;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

/// An iterator over previous siblings.
pub struct PrevSiblings {
  iter: std::vec::IntoIter<Container>,
}

impl Iterator for PrevSiblings {
  type Item = Container;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

/// An iterator over next siblings.
pub struct NextSiblings {
  iter: std::collections::vec_deque::IntoIter<Container>,
}

impl Iterator for NextSiblings {
  type Item = Container;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

/// An iterator over tiling siblings.
pub struct TilingSiblings {
  id: Uuid,
  iter: std::collections::vec_deque::IntoIter<Container>,
}

impl Iterator for TilingSiblings {
  type Item = TilingContainer;

  fn next(&mut self) -> Option<Self::Item> {
    for c in self.iter.by_ref() {
      if c.id() != self.id
        && let Ok(tiling) = c.try_into()
      {
        return Some(tiling);
      }
    }
    None
  }
}

/// An iterator over children in focus order.
pub struct ChildFocusOrder {
  container: Container,
  ids: Vec<Uuid>,
  index: usize,
}

impl Iterator for ChildFocusOrder {
  type Item = Container;

  fn next(&mut self) -> Option<Self::Item> {
    while self.index < self.ids.len() {
      let id = self.ids[self.index];
      self.index += 1;
      if let Some(child) = self.container.child_by_id(&id) {
        return Some(child);
      }
    }
    None
  }
}

/// An iterator over descendants in focus order.
pub struct DescendantFocusOrder {
  root_id: Uuid,
  stack: Vec<Container>,
}

impl Iterator for DescendantFocusOrder {
  type Item = Container;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(current) = self.stack.pop() {
      if current.id() != self.root_id && !current.has_children() {
        return Some(current);
      }

      let focus_ids = current.child_focus_order_ids();
      for focus_child_id in focus_ids.into_iter().rev() {
        if let Some(focus_child) = current.child_by_id(&focus_child_id) {
          self.stack.push(focus_child);
        }
      }
    }
    None
  }
}

/// Implements the `CommonGetters` trait for a branch container struct.
#[macro_export]
macro_rules! impl_common_getters {
  ($struct_name:ident) => {
    impl CommonGetters for $struct_name {
      fn id(&self) -> ::uuid::Uuid {
        self.0.borrow().id
      }

      fn as_container(&self) -> $crate::models::Container {
        self.clone().into()
      }

      fn as_tiling_container(
        &self,
      ) -> anyhow::Result<$crate::models::TilingContainer> {
        TryInto::<$crate::models::TilingContainer>::try_into(
          self.as_container(),
        )
        .map_err(anyhow::Error::msg)
      }

      fn as_window_container(
        &self,
      ) -> anyhow::Result<$crate::models::WindowContainer> {
        TryInto::<$crate::models::WindowContainer>::try_into(
          self.as_container(),
        )
        .map_err(anyhow::Error::msg)
      }

      fn as_direction_container(
        &self,
      ) -> anyhow::Result<$crate::models::DirectionContainer> {
        TryInto::<$crate::models::DirectionContainer>::try_into(
          self.as_container(),
        )
        .map_err(anyhow::Error::msg)
      }

      fn to_dto(&self) -> anyhow::Result<::wm_common::ContainerDto> {
        self.to_dto()
      }

      fn parent(&self) -> Option<$crate::models::Container> {
        self
          .0
          .borrow()
          .parent
          .as_ref()
          .and_then($crate::models::WeakContainer::upgrade)
      }

      fn set_parent(&self, parent: Option<&$crate::models::Container>) {
        self.0.borrow_mut().parent =
          parent.map($crate::models::WeakContainer::from_container);
      }

      fn children(
        &self,
      ) -> ::std::collections::VecDeque<$crate::models::Container> {
        self.0.borrow().children.clone()
      }

      fn child_count(&self) -> usize {
        self.0.borrow().children.len()
      }

      fn has_children(&self) -> bool {
        !self.0.borrow().children.is_empty()
      }

      fn child_by_id(
        &self,
        child_id: &::uuid::Uuid,
      ) -> Option<$crate::models::Container> {
        self
          .0
          .borrow()
          .children
          .iter()
          .find(|child| &child.id() == child_id)
          .cloned()
      }

      fn child_focus_order_ids(&self) -> ::std::vec::Vec<::uuid::Uuid> {
        self.0.borrow().child_focus_order.iter().copied().collect()
      }

      fn borrow_children(
        &self,
      ) -> ::std::cell::Ref<
        '_,
        ::std::collections::VecDeque<$crate::models::Container>,
      > {
        ::std::cell::Ref::map(self.0.borrow(), |inner| &inner.children)
      }

      fn borrow_children_mut(
        &self,
      ) -> ::std::cell::RefMut<
        '_,
        ::std::collections::VecDeque<$crate::models::Container>,
      > {
        ::std::cell::RefMut::map(self.0.borrow_mut(), |inner| {
          &mut inner.children
        })
      }

      fn borrow_child_focus_order(
        &self,
      ) -> ::std::cell::Ref<'_, ::std::collections::VecDeque<::uuid::Uuid>>
      {
        ::std::cell::Ref::map(self.0.borrow(), |inner| {
          &inner.child_focus_order
        })
      }

      fn borrow_child_focus_order_mut(
        &self,
      ) -> ::std::cell::RefMut<
        '_,
        ::std::collections::VecDeque<::uuid::Uuid>,
      > {
        ::std::cell::RefMut::map(self.0.borrow_mut(), |inner| {
          &mut inner.child_focus_order
        })
      }
    }
  };
}

/// Implements the `CommonGetters` trait for a leaf window struct (no
/// children fields).
#[macro_export]
macro_rules! impl_leaf_common_getters {
  ($struct_name:ident) => {
    impl CommonGetters for $struct_name {
      fn id(&self) -> ::uuid::Uuid {
        self.0.borrow().id
      }

      fn as_container(&self) -> $crate::models::Container {
        self.clone().into()
      }

      fn as_tiling_container(
        &self,
      ) -> anyhow::Result<$crate::models::TilingContainer> {
        TryInto::<$crate::models::TilingContainer>::try_into(
          self.as_container(),
        )
        .map_err(anyhow::Error::msg)
      }

      fn as_window_container(
        &self,
      ) -> anyhow::Result<$crate::models::WindowContainer> {
        TryInto::<$crate::models::WindowContainer>::try_into(
          self.as_container(),
        )
        .map_err(anyhow::Error::msg)
      }

      fn as_direction_container(
        &self,
      ) -> anyhow::Result<$crate::models::DirectionContainer> {
        TryInto::<$crate::models::DirectionContainer>::try_into(
          self.as_container(),
        )
        .map_err(anyhow::Error::msg)
      }

      fn to_dto(&self) -> anyhow::Result<::wm_common::ContainerDto> {
        self.to_dto()
      }

      fn parent(&self) -> Option<$crate::models::Container> {
        self
          .0
          .borrow()
          .parent
          .as_ref()
          .and_then($crate::models::WeakContainer::upgrade)
      }

      fn set_parent(&self, parent: Option<&$crate::models::Container>) {
        self.0.borrow_mut().parent =
          parent.map($crate::models::WeakContainer::from_container);
      }

      fn children(
        &self,
      ) -> ::std::collections::VecDeque<$crate::models::Container> {
        ::std::collections::VecDeque::new()
      }

      fn child_count(&self) -> usize {
        0
      }

      fn has_children(&self) -> bool {
        false
      }

      fn child_by_id(
        &self,
        _child_id: &::uuid::Uuid,
      ) -> Option<$crate::models::Container> {
        None
      }

      fn child_focus_order_ids(&self) -> ::std::vec::Vec<::uuid::Uuid> {
        ::std::vec::Vec::new()
      }

      fn borrow_children(
        &self,
      ) -> ::std::cell::Ref<
        '_,
        ::std::collections::VecDeque<$crate::models::Container>,
      > {
        panic!("Cannot borrow children of a leaf window");
      }

      fn borrow_children_mut(
        &self,
      ) -> ::std::cell::RefMut<
        '_,
        ::std::collections::VecDeque<$crate::models::Container>,
      > {
        panic!("Cannot borrow children mutably for a leaf container");
      }

      fn borrow_child_focus_order(
        &self,
      ) -> ::std::cell::Ref<'_, ::std::collections::VecDeque<::uuid::Uuid>>
      {
        panic!("Cannot borrow child focus order of a leaf window");
      }

      fn borrow_child_focus_order_mut(
        &self,
      ) -> ::std::cell::RefMut<
        '_,
        ::std::collections::VecDeque<::uuid::Uuid>,
      > {
        panic!(
          "Cannot borrow child focus order mutably for a leaf container"
        );
      }
    }
  };
}
