# Graph Report - src  (2026-11-26)

## Corpus Check
- 9 files · ~0 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 150 nodes · 293 edges · 14 communities (12 shown, 2 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Entry & Win32 Glue
- Island State Machine
- Settings Persistence
- Layout Math
- Shape & Content Apply
- Composition Tree
- Text Drawing Helpers
- File Logging
- GPU Device Lifecycle
- Text Surfaces (A/B)
- Localization
- Fade Animations
- Self Type
- Path Type

## God Nodes (most connected - your core abstractions)
1. `Renderer` - 24 edges
2. `App` - 20 edges
3. `wndproc()` - 14 edges
4. `Rect` - 11 edges
5. `draw()` - 11 edges
6. `place()` - 9 edges
7. `Settings` - 8 edges
8. `Island` - 8 edges
9. `Gpu` - 8 edges
10. `create()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `create_device()` --calls--> `create()`  [INFERRED]
  render.rs → window.rs
- `Renderer` --references--> `Visual`  [EXTRACTED]
  render.rs → island.rs
- `App` --references--> `Island`  [EXTRACTED]
  window.rs → island.rs
- `App` --references--> `Rect`  [EXTRACTED]
  window.rs → layout.rs
- `enum_monitors()` --references--> `Monitor`  [EXTRACTED]
  window.rs → layout.rs

## Import Cycles
- None detected.

## Communities (14 total, 2 thin omitted)

### Community 0 - "Entry & Win32 Glue"
Cohesion: 0.13
Nodes (19): FnOnce, Instant, LPARAM, LRESULT, NOTIFYICONDATAW, PathBuf, Settings, Vec (+11 more)

### Community 1 - "Island State Machine"
Cohesion: 0.21
Nodes (13): activity_end_hides_unless_cursor_is_on_it(), activity_shows_compact_without_gesture_and_collapses_to_it(), click_on_revealed_idle_expands_clock_then_hides(), Event, hover_grows_within_8_percent_and_reverts(), Island, returning_before_timeout_keeps_it(), revealed() (+5 more)

### Community 2 - "Settings Persistence"
Cohesion: 0.18
Nodes (16): Default, corrupt_file_is_backed_up(), default_path(), load(), margin_is_clamped(), missing_file_gives_defaults(), roundtrip_and_partial_json(), Option (+8 more)

### Community 3 - "Layout Math"
Cohesion: 0.24
Nodes (11): centered_with_margin_at_100_percent(), Monitor, pick(), place(), Placement, r(), Rect, respects_work_area_offset_top_taskbar_and_second_monitor() (+3 more)

### Community 4 - "Shape & Content Apply"
Cohesion: 0.33
Nodes (6): Visual, draw(), Frame, v2(), SizeInt32, Vector2

### Community 5 - "Composition Tree"
Cohesion: 0.22
Nodes (9): CompositionRoundedRectangleGeometry, CompositionSpriteShape, CompositionVisualSurface, Compositor, ContainerVisual, DesktopWindowTarget, DispatcherQueueController, Renderer (+1 more)

### Community 6 - "Text Drawing Helpers"
Cohesion: 0.25
Nodes (7): D2D1_COLOR_F, color(), color_f(), now_strings(), HWND, String, Self

### Community 7 - "File Logging"
Cohesion: 0.36
Nodes (7): append(), error(), info(), rotates_and_caps_disk_usage(), Path, warn(), write()

### Community 8 - "GPU Device Lifecycle"
Cohesion: 0.36
Nodes (6): HANDLE, ID2D1Device, create_device(), Option, Result, watch_device()

### Community 9 - "Text Surfaces (A/B)"
Cohesion: 0.33
Nodes (6): CompositionDrawingSurface, CompositionGraphicsDevice, ID3D11Device, IDWriteFactory, Content, Gpu

### Community 10 - "Localization"
Cohesion: 0.83
Nodes (3): is_portuguese(), t(), Text

### Community 11 - "Fade Animations"
Cohesion: 0.50
Nodes (3): ms(), SpriteVisual, TimeSpan

## Knowledge Gaps
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `Entry & Win32 Glue` to `Island State Machine`, `Layout Math`, `Composition Tree`?**
  _High betweenness centrality (0.274) - this node is a cross-community bridge._
- **Why does `Renderer` connect `Composition Tree` to `Entry & Win32 Glue`, `Shape & Content Apply`, `Text Drawing Helpers`, `GPU Device Lifecycle`, `Text Surfaces (A/B)`, `Fade Animations`?**
  _High betweenness centrality (0.235) - this node is a cross-community bridge._
- **Why does `Rect` connect `Layout Math` to `Entry & Win32 Glue`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **Should `Entry & Win32 Glue` be split into smaller, more focused modules?**
  _Cohesion score 0.12903225806451613 - nodes in this community are weakly interconnected._