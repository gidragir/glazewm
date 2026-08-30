# Bug 1: Zebar and Ignored Windows Permanent Invisibility

## Status
**Fixed** (Resolved in Iteration 4 follow-up).

## Description
When launching GlazeWM alongside Zebar (or any other application/dock bar configured with an `ignore` window rule), Zebar became completely invisible, even though its process was running and active.

## Root Cause
1. In Iteration 4, `native_window.set_cloaked(true)` was unconditionally placed at the very start of `manage_window.rs` before window rules were evaluated.
2. When the window rule matching `process_name: 'zebar'` fired with command `ignore`, `ignore_window.rs` detached the window from the tree.
3. Because the window was detached and no longer tracked as a managed tiling window, `platform_sync.rs` never processed it and never called `set_cloaked(false)`.
4. As a result, the window remained permanently cloaked (invisible to DWM).

## Solution Applied
1. Removed premature top-level cloaking from `manage_window.rs`.
2. Applied cloaking conditionally *only* after `run_window_rules` and *only* if `window.state() == WindowState::Tiling`.
3. Added guaranteed explicit `set_cloaked(false)` in `ignore_window.rs` and the `manage_window.rs` fallback branch.
