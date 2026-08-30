# Backlog: Zebar Strip Minimap (Canvas Indicator)

## Overview
Status bars (specifically Zebar) should display a visual minimap / scrollbar widget showing the user's current position and viewport bounds on the infinite horizontal canvas strip.

## IPC Data Model
GlazeWM already provides `offset_x` in `WorkspaceDto`:
```json
{
  "id": "...",
  "name": "1",
  "displayName": "1",
  "width": 1920,
  "height": 1080,
  "offsetX": 540.0,
  "children": [
    {
      "id": "...",
      "x": 0,
      "y": 0,
      "width": 960,
      "height": 1040
    },
    {
      "id": "...",
      "x": 980,
      "y": 0,
      "width": 960,
      "height": 1040
    }
  ]
}
```

## Proposed Zebar Widget Component
1. **Total Canvas Span**: Calculate $W_{\text{canvas}} = \max_i (\text{child.x} + \text{child.width})$.
2. **Viewport Indicator**: Render a bounding rectangle of width $W_{\text{screen}} / W_{\text{canvas}}$ at relative position $\text{offsetX} / W_{\text{canvas}}$.
3. **Columns Representation**: Render miniature blocks for each column, highlighting the active/focused column in the accent theme color.
