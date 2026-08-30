## Definition
Enhancement of the infinite horizontal scrolling layout (iteration 2) focusing on User Experience (UX). Introduces persistent column width state and configurable width presets (similar to Niri).

## Value Proposition
Improves the consistency and navigability of the horizontal canvas. Preserving column widths when they scroll out of the viewport prevents layout shifting. Preset commands allow users to quickly snap columns to desired proportions (e.g., 25%, 33%, 50%, 75%) using keyboard shortcuts (e.g., `Alt+Shift+R`), enhancing workflow efficiency.

## Core Mechanics
1. **Persistent Column State**: Store explicit width assignments (either relative percentage or absolute pixels) on the container model (`TilingContainer` / `TilingWindow`). When the viewport pans and recalculates positions, it uses the preserved width state instead of resetting to a default equal-split.
2. **Width Presets Command**: A new command `set-preset` (or similar) that applies a predefined width to the currently focused column.
3. **Configuration Driven**: Presets are defined in the user's `config.yaml`. The command can cycle through these presets sequentially.
