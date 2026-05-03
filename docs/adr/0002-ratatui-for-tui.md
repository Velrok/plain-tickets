# ADR 0002: Use `ratatui` + `crossterm` for the TUI

**Status:** Accepted
**Date:** 2026-05-03

The interactive TUI needs a rendering and event-handling stack. `ratatui` (with `crossterm` backend) is the de facto standard for Rust TUIs — actively maintained, large ecosystem, and the right level of abstraction for a layout-and-navigation UI. `cursive` was considered but is heavier and form-oriented; raw `crossterm` was considered but too low-level to build a Kanban board without reinventing widget primitives.
