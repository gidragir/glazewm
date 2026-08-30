## Standard Patterns
1. **Pre-Show Placement / Window Cloaking (Komorebi / GlazeWM)**:
   - Modern Win32 window managers use `DwmSetWindowAttribute(DWMWA_CLOAKED)` to keep newly spawned windows invisible to the compositor while calculating and applying initial placement.
2. **Infinite Strip Collapse (Niri / PaperWM)**:
   - In horizontal strip layouts, closing an item leaves neighboring column sizes untouched; the strip simply compacts horizontally by moving rightward elements leftward by $(W_{\text{closed}} + \text{gap})$.
