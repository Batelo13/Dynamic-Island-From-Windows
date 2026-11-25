# Graph Report - src  (2026-11-25)

## Corpus Check
- Corpus is ~2,541 words - fits in a single context window. You may not need a graph.

## Summary
- 85 nodes · 138 edges · 9 communities (8 shown, 1 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Settings Persistence
- Layout Math
- Composition Renderer
- File Logging
- App State & Tray
- Window Creation
- Window Procedure
- Localization
- Entry Point

## God Nodes (most connected - your core abstractions)
1. `App` - 13 edges
2. `wndproc()` - 10 edges
3. `Renderer` - 9 edges
4. `Settings` - 9 edges
5. `Rect` - 8 edges
6. `place()` - 8 edges
7. `tmp_dir()` - 6 edges
8. `create()` - 6 edges
9. `Monitor` - 5 edges
10. `r()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `enum_monitors()` --references--> `Monitor`  [EXTRACTED]
  window.rs → layout.rs
- `App` --references--> `Renderer`  [EXTRACTED]
  window.rs → render.rs
- `App` --references--> `Settings`  [EXTRACTED]
  window.rs → settings.rs

## Import Cycles
- None detected.

## Communities (9 total, 1 thin omitted)

### Community 0 - "Settings Persistence"
Cohesion: 0.18
Nodes (16): Default, corrupt_file_is_backed_up(), default_path(), load(), margin_is_clamped(), missing_file_gives_defaults(), roundtrip_and_partial_json(), Option (+8 more)

### Community 1 - "Layout Math"
Cohesion: 0.24
Nodes (11): centered_with_margin_at_100_percent(), Monitor, pick(), place(), Placement, r(), Rect, respects_work_area_offset_top_taskbar_and_second_monitor() (+3 more)

### Community 2 - "Composition Renderer"
Cohesion: 0.18
Nodes (9): CompositionRoundedRectangleGeometry, CompositionSpriteShape, DesktopWindowTarget, DispatcherQueueController, Renderer, HWND, Result, Self (+1 more)

### Community 3 - "File Logging"
Cohesion: 0.36
Nodes (7): append(), error(), info(), rotates_and_caps_disk_usage(), Path, warn(), write()

### Community 4 - "App State & Tray"
Cohesion: 0.33
Nodes (4): NOTIFYICONDATAW, App, Option, PathBuf

### Community 5 - "Window Creation"
Cohesion: 0.33
Nodes (6): FnOnce, Vec, create(), enum_monitors(), Result, with_app()

### Community 6 - "Window Procedure"
Cohesion: 0.53
Nodes (6): LPARAM, LRESULT, request_relayout(), HWND, wndproc(), WPARAM

### Community 7 - "Localization"
Cohesion: 0.83
Nodes (3): is_portuguese(), t(), Text

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `App State & Tray` to `Settings Persistence`, `Composition Renderer`, `Window Creation`, `Window Procedure`, `Entry Point`?**
  _High betweenness centrality (0.535) - this node is a cross-community bridge._
- **Why does `Settings` connect `Settings Persistence` to `App State & Tray`?**
  _High betweenness centrality (0.314) - this node is a cross-community bridge._
- **Why does `enum_monitors()` connect `Window Creation` to `Layout Math`, `App State & Tray`, `Window Procedure`?**
  _High betweenness centrality (0.290) - this node is a cross-community bridge._