# Graph Report - src  (2026-11-27)

## Corpus Check
- 10 files · ~0 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 209 nodes · 437 edges · 17 communities (12 shown, 5 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 5 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Media Source
- Window & Messages
- Island State Machine
- Layout Math
- Settings Persistence
- Community 5
- Content Drawing
- Composition Tree
- File Logging
- GPU Device Lifecycle
- Content Drawing
- Localization
- Community 12
- Community 13
- Community 14
- Settings Persistence
- Community 16

## God Nodes (most connected - your core abstractions)
1. `Renderer` - 29 edges
2. `App` - 27 edges
3. `Snapshot` - 23 edges
4. `wndproc()` - 17 edges
5. `draw()` - 16 edges
6. `Rect` - 12 edges
7. `run()` - 12 edges
8. `Media` - 10 edges
9. `Island` - 9 edges
10. `place()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `draw()` --references--> `Visual`  [EXTRACTED]
  render.rs → island.rs
- `App` --references--> `Visual`  [EXTRACTED]
  window.rs → island.rs
- `App` --references--> `Island`  [EXTRACTED]
  window.rs → island.rs
- `App` --references--> `Rect`  [EXTRACTED]
  window.rs → layout.rs
- `enum_monitors()` --references--> `Monitor`  [EXTRACTED]
  window.rs → layout.rs

## Import Cycles
- None detected.

## Communities (17 total, 5 thin omitted)

### Community 0 - "Media Source"
Cohesion: 0.11
Nodes (30): A, Arc, ArtCache, IRandomAccessStreamReference, IWICImagingFactory, Command, content_key_ignores_position(), decode_art() (+22 more)

### Community 1 - "Window & Messages"
Cohesion: 0.15
Nodes (18): FnOnce, LPARAM, LRESULT, NOTIFYICONDATAW, animations_enabled(), App, create(), enum_monitors() (+10 more)

### Community 2 - "Island State Machine"
Cohesion: 0.17
Nodes (20): activity_end_hides_unless_cursor_is_on_it(), activity_shows_compact_without_gesture_and_collapses_to_it(), click_on_revealed_idle_expands_clock_then_hides(), Event, hover_grows_within_8_percent_and_reverts(), Island, pause_keeps_visible_then_goes_dormant_and_hidden(), pause_timeout_under_cursor_waits_for_the_cursor_to_leave() (+12 more)

### Community 3 - "Layout Math"
Cohesion: 0.19
Nodes (14): centered_with_margin_at_100_percent(), media_buttons(), media_buttons_are_centered_on_the_controls_row_without_overlap(), Monitor, pick(), place(), Placement, r() (+6 more)

### Community 4 - "Settings Persistence"
Cohesion: 0.18
Nodes (16): Default, corrupt_file_is_backed_up(), default_path(), load(), margin_is_clamped(), missing_file_gives_defaults(), roundtrip_and_partial_json(), Option (+8 more)

### Community 5 - "Community 5"
Cohesion: 0.27
Nodes (8): CVisual, Visual, Frame, ms(), Option, v2(), TimeSpan, Vector2

### Community 6 - "Content Drawing"
Cohesion: 0.22
Nodes (11): D2D1_COLOR_F, color(), color_f(), draw(), mmss(), now_strings(), Duration, HWND (+3 more)

### Community 7 - "Composition Tree"
Cohesion: 0.18
Nodes (11): CompositionEllipseGeometry, CompositionRoundedRectangleGeometry, CompositionSpriteShape, CompositionVisualSurface, Compositor, ContainerVisual, ContentKey, DesktopWindowTarget (+3 more)

### Community 8 - "File Logging"
Cohesion: 0.36
Nodes (7): append(), error(), info(), rotates_and_caps_disk_usage(), Path, warn(), write()

### Community 9 - "GPU Device Lifecycle"
Cohesion: 0.39
Nodes (6): HANDLE, ID2D1Device, ID3D11Device, create_device(), Result, watch_device()

### Community 10 - "Content Drawing"
Cohesion: 0.33
Nodes (6): CompositionDrawingSurface, CompositionGraphicsDevice, IDWriteFactory, Content, Gpu, SpriteVisual

### Community 11 - "Localization"
Cohesion: 0.83
Nodes (3): is_portuguese(), t(), Text

## Knowledge Gaps
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App` connect `Window & Messages` to `Media Source`, `Island State Machine`, `Layout Math`, `Settings Persistence`, `Community 5`, `Composition Tree`?**
  _High betweenness centrality (0.473) - this node is a cross-community bridge._
- **Why does `Renderer` connect `Composition Tree` to `Media Source`, `Window & Messages`, `Community 5`, `Content Drawing`, `GPU Device Lifecycle`, `Content Drawing`?**
  _High betweenness centrality (0.225) - this node is a cross-community bridge._
- **Why does `Snapshot` connect `Media Source` to `Window & Messages`, `Community 5`, `Content Drawing`, `Composition Tree`?**
  _High betweenness centrality (0.215) - this node is a cross-community bridge._
- **Should `Media Source` be split into smaller, more focused modules?**
  _Cohesion score 0.11428571428571428 - nodes in this community are weakly interconnected._
- **Should `Window & Messages` be split into smaller, more focused modules?**
  _Cohesion score 0.14516129032258066 - nodes in this community are weakly interconnected._