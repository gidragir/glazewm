# Implementation Plan - Infinite Canvas UX & Column Width Presets (Iteration 2)

Enhance GlazeWM's infinite horizontal canvas layout with persistent column width management and configurable width cycling presets.

## Goal Description

In Iteration 1, the basic infinite horizontal strip canvas was established. However, two critical UX issues remain:
1. **Column Width Distortion / State Loss**: Resizing a column currently redistributes space across sibling columns because `resize_tiling_container` assumes a fixed normalized width sum of 1.0. When columns scroll out of view and back, their proportions are altered or reset.
2. **Missing Width Presets & Quick Cycling**: Users cannot quickly snap columns to standard screen proportions (e.g., 25%, 33%, 50%, 75%).

This plan decouples top-level column widths on the infinite canvas from sibling constraints and introduces a configurable cycling command (`CycleColumnPreset`) bound by default to `Alt+Shift+R`.

---

## User Review Required

> [!IMPORTANT]
> **Column Target Granularity**: In Niri, a "column" can contain multiple vertically stacked windows. When cycling presets on a focused window that belongs to a vertical split within the horizontal workspace, the preset should apply to the **entire column** (the root container under the workspace), not just the inner window.
>
> **Default Presets**: The proposed default presets are `[25%, 33%, 50%, 75%]`. These will be customizable in `config.yaml`.

---

## Proposed Changes

### `packages/wm-common` (Config & Commands)

#### [MODIFY] [app_command.rs](file:///data/projects/glazewm/packages/wm-common/src/app_command.rs)
- Add `CycleColumnPreset` variant to `InvokeCommand` (with optional custom presets argument or fallback to user config).
- Add `SetColumnWidth` variant to `InvokeCommand` for direct length values (e.g. `50%`, `400px`).

#### [MODIFY] [config](file:///data/projects/glazewm/packages/wm-common/src/config/) & [default_config.rs](file:///data/projects/glazewm/packages/wm-common/src/config/default_config.rs)
- Add `column_width_presets` field to configuration schema with default `vec!["25%".into(), "33%".into(), "50%".into(), "75%".into()]`.
- Add `alt+shift+r: cycle-column-preset` to default keybindings.

---

### `packages/wm` (Layout Engine & Commands)

#### [MODIFY] [resize_tiling_container.rs](file:///data/projects/glazewm/packages/wm/src/commands/container/resize_tiling_container.rs)
- Detect if the container is a direct child of a horizontal `Workspace` (an infinite canvas column).
- For canvas columns, update `tiling_size` directly without clamping against sibling sum or stealing width from siblings.

#### [MODIFY] [set_window_size.rs](file:///data/projects/glazewm/packages/wm/src/commands/window/set_window_size.rs)
- Adapt width resizing when target is an infinite canvas column so it sets independent ratio values (e.g. `0.25`, `0.33`, `0.5`, `0.75`, `1.0`).

#### [NEW] [cycle_column_preset.rs](file:///data/projects/glazewm/packages/wm/src/commands/window/cycle_column_preset.rs)
- Implement `cycle_column_preset(window: WindowContainer, state: &mut WmState, config: &UserConfig)`.
- Traverse upward to find the top-level column under the active `Workspace`.
- Inspect current `tiling_size`, locate the next preset in the list (or the next larger preset if custom-sized, wrapping around to the first preset).
- Apply the new width and queue redrawing with `pending_sync`.

#### [MODIFY] [wm.rs](file:///data/projects/glazewm/packages/wm/src/wm.rs)
- Register `InvokeCommand::CycleColumnPreset` in `run_command` dispatcher.

---

## Detailed Task List

- [ ] **Task 1: Config Schema & Command Definition**
  - Add `CycleColumnPreset` to `InvokeCommand` in `wm-common/src/app_command.rs`.
  - Add `column_width_presets` vector to `GeneralConfig` / `UserConfig`.
  - Add default keybinding `Alt+Shift+R -> cycle-column-preset` in default configuration.

- [ ] **Task 2: Decouple Column Resizing on Infinite Canvas**
  - Update `resize_tiling_container.rs` to bypass sibling proportion redistribution when the parent is a horizontal `Workspace`.
  - Update `attach_container.rs` to assign default `tiling_size = 0.5` (or configured default width) to new columns without altering existing column widths.

- [ ] **Task 3: Implement Column Preset Cycling Logic**
  - Create `packages/wm/src/commands/window/cycle_column_preset.rs`.
  - Add helper to resolve top-level column ancestor.
  - Implement circular step logic comparing `current_tiling_size` against `config.column_width_presets`.
  - Wire command execution in `packages/wm/src/wm.rs`.

- [ ] **Task 4: Unit Testing & Verification**
  - Add unit test for independent column resizing in `packages/wm/src/commands/container/resize_tiling_container.rs`.
  - Add unit test for `cycle_column_preset` with various initial widths (exact match, intermediate manual resize, wrap-around).

---

## Verification Plan

### Automated Tests
- Run `cargo test -p wm -p wm-common` to ensure all tests pass.
- Specific unit tests:
  - `test_canvas_column_resize_does_not_affect_siblings`
  - `test_cycle_column_presets_progression`

### Manual Verification
1. Launch GlazeWM with 3-4 open windows on a horizontal workspace.
2. Focus the first window, press `Alt+Shift+R` multiple times, and verify it cycles smoothly through 25% -> 33% -> 50% -> 75% -> 25%.
3. Pan right so the first window is off-screen.
4. Pan left or refocus the first window; verify its width is strictly preserved.
5. Manually resize a window to an arbitrary width (~40%), press `Alt+Shift+R`, and verify it snaps cleanly to 50%.
