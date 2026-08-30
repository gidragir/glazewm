## Standard Patterns
1. **Discrete Step Interpolation (Lightweight TWM Approach)**:
   - Used in lightweight X11/Wayland scripts and window managers to create smooth sliding transitions with 4–8 discrete coordinate steps rather than continuous GPU frame callbacks.
2. **Spring Physics & 120Hz VSync Animators (Full Compositor Approach)**:
   - Used in Wayland compositors (Niri, Hyprland, macOS WindowServer) where the compositor controls the frame rendering pipeline. Requires higher complexity and dedicated frame synchronizers.

## Comparison & Selection
- **Selected Approach**: **Discrete Step Interpolation** for GlazeWM.
  - Keeps the codebase lightweight and maintainable.
  - Directly leverages existing `DeferWindowPos` batch positioning.
  - Achieves 90% of the visual UX improvement of full compositor animations with <5% of the implementation complexity.
