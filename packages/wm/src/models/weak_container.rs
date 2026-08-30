use std::{
  cell::RefCell,
  rc::{Rc, Weak},
};

use crate::models::{
  Container, Monitor, MonitorInner, NonTilingWindow, NonTilingWindowInner,
  RootContainer, RootContainerInner, SplitContainer, SplitContainerInner,
  TilingWindow, TilingWindowInner, Workspace, WorkspaceInner,
};

#[derive(Clone, Debug)]
pub enum WeakContainer {
  Root(Weak<RefCell<RootContainerInner>>),
  Monitor(Weak<RefCell<MonitorInner>>),
  Workspace(Weak<RefCell<WorkspaceInner>>),
  Split(Weak<RefCell<SplitContainerInner>>),
  TilingWindow(Weak<RefCell<TilingWindowInner>>),
  NonTilingWindow(Weak<RefCell<NonTilingWindowInner>>),
}

impl WeakContainer {
  #[must_use]
  pub fn upgrade(&self) -> Option<Container> {
    match self {
      Self::Root(w) => {
        w.upgrade().map(|rc| Container::Root(RootContainer(rc)))
      }
      Self::Monitor(w) => {
        w.upgrade().map(|rc| Container::Monitor(Monitor(rc)))
      }
      Self::Workspace(w) => {
        w.upgrade().map(|rc| Container::Workspace(Workspace(rc)))
      }
      Self::Split(w) => {
        w.upgrade().map(|rc| Container::Split(SplitContainer(rc)))
      }
      Self::TilingWindow(w) => w
        .upgrade()
        .map(|rc| Container::TilingWindow(TilingWindow(rc))),
      Self::NonTilingWindow(w) => w
        .upgrade()
        .map(|rc| Container::NonTilingWindow(NonTilingWindow(rc))),
    }
  }

  #[must_use]
  pub fn from_container(container: &Container) -> Self {
    match container {
      Container::Root(c) => Self::Root(Rc::downgrade(&c.0)),
      Container::Monitor(c) => Self::Monitor(Rc::downgrade(&c.0)),
      Container::Workspace(c) => Self::Workspace(Rc::downgrade(&c.0)),
      Container::Split(c) => Self::Split(Rc::downgrade(&c.0)),
      Container::TilingWindow(c) => {
        Self::TilingWindow(Rc::downgrade(&c.0))
      }
      Container::NonTilingWindow(c) => {
        Self::NonTilingWindow(Rc::downgrade(&c.0))
      }
    }
  }
}
