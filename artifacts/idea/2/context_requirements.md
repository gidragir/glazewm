## Required Codebase Context
1. **Tiling Size Management**: Investigate `TilingSizeGetters` and how `Container` size properties are stored and mutated. Currently, `TilingContainer` manages sizes.
2. **Command Registration**: Identify how `AppCommand` and user shortcuts are parsed and registered (e.g., `packages/wm-common/src/app_command.rs`, `packages/wm/src/commands/`).
3. **Configuration Schema**: Understand `UserConfig` parsing (likely using `serde`) to add the new `column_width_presets` field in `config.yaml`.
4. **State Persistence**: Locate where the tree of containers is updated during viewport panning (`workspace.rs` / `offset_x` logic from iteration 1) to ensure width fields are retained across layout recalculations.
