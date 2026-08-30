## Definition
An explicit column-grouping and window-isolation mechanism for infinite horizontal canvas window managers, introducing dedicated `consume-or-expel-window-left` and `consume-or-expel-window-right` operations while decoupling standard horizontal translation (`move --direction <left/right>`) from container-merging side effects.

## Value Proposition
Prevents accidental window merging during column repositioning on an infinite canvas while providing deterministic, bidirectional grouping and extraction mechanics inspired by Niri.

## Core Mechanics
1. **Move Isolation (Column Swapping)**: Horizontal move dispatch (`move_window_in_direction`) treats adjacent top-level nodes (both single `TilingWindow` instances and vertical `SplitContainer` columns) as discrete atomic entities, performing positional index swaps within the workspace rather than tree descent.
2. **Column Consumption**: Invoking `consume-or-expel-window` with a directional vector (`Left` | `Right`) on a top-level window targets the adjacent sibling:
   - If the sibling is a vertical `SplitContainer`, moves the window into the container (appended at bottom for `Left`, prepended at top for `Right`).
   - If the sibling is a standalone `TilingWindow`, creates a new vertical `SplitContainer` at the sibling's index and attaches both windows inside it.
3. **Column Expulsion**: Invoking `consume-or-expel-window` on a window nested inside a vertical `SplitContainer` extracts the window to the top-level horizontal workspace immediately adjacent to the column (before the column for `Left`, after for `Right`).
4. **Column Flattening Guard**: When a vertical `SplitContainer` is reduced to 1 child after expulsion, it flattens into a single window. If a workspace contains only 1 column, automatic container flattening is suppressed to prevent canvas orientation mutation.
