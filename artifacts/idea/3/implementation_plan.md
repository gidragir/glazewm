# Implementation Plan - Lightweight Smooth Canvas Transitions (Iteration 3)

Introduce lightweight micro-animated step transitions for infinite horizontal canvas viewport panning and document IPC requirements for the Zebar strip minimap backlog.

## Goal Description

When changing window focus across off-screen columns or using manual pan commands, the instantaneous jump of `offset_x` creates visual disorientation. This iteration introduces a lightweight stepping transition (4–6 discrete steps over ~80–120ms) using `DeferWindowPos` (`SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER`). This gives the user clear spatial awareness of motion direction without the architectural overhead of a complex physics/VSync animation engine.

---

## User Review Required

> [!IMPORTANT]
> **Step Interval & Duration**: Default duration is proposed as ~90ms across 5 steps (e.g. ~18ms per step). This is fast enough to feel instantaneous and non-blocking, while slow enough for the human eye to perceive the motion vector.
>
> **Rapid Input Handling**: If a new navigation command arrives during an active transition, the transition immediately updates its target coordinate to ensure zero input lag.

---

## Proposed Changes

### `packages/wm-common` (Configuration & IPC Backlog)

#### [MODIFY] [parsed_config.rs](file:///data/projects/glazewm/packages/wm-common/src/parsed_config.rs)
- Add optional `animation_enabled: bool` (default `true`) and `animation_duration_ms: u32` (default `90`) to `GeneralConfig`.

#### [MODIFY] [workspace_dto.rs](file:///data/projects/glazewm/packages/wm-common/src/dtos/workspace_dto.rs)
- Ensure `offset_x` and window bounding boxes are broadcasted in `WorkspaceDto` on every pan change for status bar (Zebar) minimap rendering.

---

### `packages/wm` (Transition Logic & Viewport Panning)

#### [MODIFY] [platform_sync.rs](file:///data/projects/glazewm/packages/wm/src/commands/general/platform_sync.rs)
- Update `auto_pan_viewport` to smoothly step `offset_x` from `current_offset` to `new_offset` in small discrete increments.
- Apply batch positioning with `DeferWindowPos` on each step.

#### [MODIFY] [wm.rs](file:///data/projects/glazewm/packages/wm/src/wm.rs)
- Apply the same stepped transition for manual panning commands (`PanViewportLeft` and `PanViewportRight`).

---

## Detailed Task List

- [ ] **Task 1: Add Animation Configuration Options**
  - Add `animation_enabled` and `animation_duration_ms` to `GeneralConfig` in `wm-common`.
  - Update `sample-config.yaml` with explanatory comments.

- [ ] **Task 2: Implement Stepped Viewport Transition**
  - Implement a helper function `smooth_pan_viewport(workspace, target_offset, state)` in `platform_sync.rs`.
  - Calculate step deltas based on configured duration and step count (e.g., 5 steps with ease-out increments).
  - Execute batch `DeferWindowPos` updates per step.

- [ ] **Task 3: Connect Manual Pan Commands & Auto-Pan**
  - Integrate smooth transition into `auto_pan_viewport` on focus changes.
  - Integrate smooth transition into `PanViewportLeft` / `PanViewportRight` handlers in `wm.rs`.

- [ ] **Task 4: Backlog Documentation for Zebar Minimap**
  - Create a backlog reference in `artifacts/idea/3/backlog_zebar_minimap.md` describing IPC events and DTO payload requirements for external bars.

- [ ] **Task 5: Verification & Testing**
  - Test rapid focus changes across 5+ off-screen windows.
  - Verify smooth sliding motion and absence of input lag or tearing.

---

## Verification Plan

### Automated Tests
- Run `cargo test -p wm -p wm-common` to ensure all existing and updated layout tests pass cleanly.

### Manual Verification
1. Open 4–5 windows on a horizontal workspace.
2. Focus between the leftmost and rightmost windows using keyboard shortcuts.
3. Observe smooth slide-in motion into view rather than an abrupt teleportation.
4. Rapidly hold or press navigation hotkeys to confirm there is no input buffering or stuttering.
