use serde::Serialize;
use wm_platform::Direction;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum WorkspaceTarget {
  Name(String),
  Recent,
  NextActive,
  PreviousActive,
  NextActiveInMonitor,
  PreviousActiveInMonitor,
  Next,
  Previous,
  Direction(Direction),
}
