# Implementation Plan - Infinite Canvas Bugfixes & Spawn Refinement (Iteration 4)

Fix window size corruption upon container closure and eliminate the centered-window visual flash when spawning new applications on the infinite horizontal canvas.

## Goal Description

During practical usage of the infinite horizontal canvas, two critical defects were identified:
1. **Size Corruption on Window Close**: Closing any window causes off-screen and visible columns to expand massively because `detach_container` distributes the closed window's `tiling_size` across all siblings, erroneously assuming a bounded 1.0 sum.
2. **Centered Spawn Flash on New Windows**: When new applications (e.g. `explorer.exe`) launch, Windows OS renders them initially at default screen-centered coordinates before GlazeWM tiles them, creating a jarring center-flicker that ruins canvas continuity.

---

## User Review Required

> [!IMPORTANT]
> **Detach Behavior on Infinite Strip**: When a column is closed on the horizontal workspace, remaining columns will keep their exact pixel/percentage width. Rightward columns simply slide left to close the gap without resizing.
>
> **Pre-Show Cloaking on Windows**: Windows will be cloaked/hidden during initial positioning in `manage_window` and revealed only once their target strip coordinates have been committed to DWM.

---

## Proposed Changes

### `packages/wm` (Container Lifecycle & Placement)

#### [MODIFY] [detach_container.rs](file:///data/projects/glazewm/packages/wm/src/commands/container/detach_container.rs)
- Check `if parent.as_workspace().is_none()` before redistributing `size_delta` to siblings.
- Keep sibling column widths strictly constant on horizontal workspaces.

#### [MODIFY] [manage_window.rs](file:///data/projects/glazewm/packages/wm/src/commands/window/manage_window.rs)
- Cloak newly detected manageable windows immediately before attaching and calculating their frame.
- Ensure the window uncloaks only after initial layout placement is queued and applied via batch `DeferWindowPos`.

#### [MODIFY] [platform_sync.rs](file:///data/projects/glazewm/packages/wm/src/commands/general/platform_sync.rs)
- Ensure auto-panning on window spawn smoothly positions the camera on the newly created column.

---

## Detailed Task List

- [ ] **Task 1: Fix Window Close Size Inflation**
  - In `detach_container.rs`, guard the sibling resize loop with `if parent.as_workspace().is_none()`.
  - Add unit test verifying closing a container on a workspace does not alter remaining containers' `tiling_size`.

- [ ] **Task 2: Implement Pre-Show Cloaking on Window Spawn**
  - In `manage_window.rs`, cloak window upon detection before attaching.
  - Position window directly at target strip bounds in `platform_sync.rs` before uncloaking.

- [ ] **Task 3: Verification & Edge Case Testing**
  - Open 5 Explorer and Notepad windows on a horizontal strip.
  - Close windows from the beginning, middle, and end of the strip; verify no size inflation occurs.
  - Launch new Explorer instances repeatedly; verify zero center-screen flickering.

---

## Verification Plan

### Automated Tests
- `cargo check --tests --target x86_64-pc-windows-gnu`
- `cargo clippy --target x86_64-pc-windows-gnu -- -D warnings`

### Manual Verification
1. Open 4 windows of varying preset sizes (`25%`, `33%`, `50%`, `75%`).
2. Close window 2; verify window 1 and windows 3-4 retain their exact dimensions.
3. Press `Win+E` to open Windows Explorer; verify it appears cleanly on the canvas without flashing in the center of the monitor.
