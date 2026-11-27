//! Design tokens (design.md D9). All sizes in DIP; colors as 0xRRGGBB.
//! Starting values — tuned against the visual references, not user-configurable.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeSize {
    pub w: f32,
    pub h: f32,
    pub radius: f32,
}

pub const IDLE: ShapeSize = ShapeSize { w: 126.0, h: 37.0, radius: 18.5 };
pub const COMPACT: ShapeSize = ShapeSize { w: 250.0, h: 37.0, radius: 18.5 };
pub const EXPANDED: ShapeSize = ShapeSize { w: 360.0, h: 160.0, radius: 32.0 };
/// Largest shape the island can take (sizes the fixed visual window, design.md D3).
pub const MAX_W: f32 = EXPANDED.w;
pub const MAX_H: f32 = EXPANDED.h;
pub const HOVER_SCALE: f32 = 1.06;

pub const FILL: u32 = 0x000000;
pub const STROKE: u32 = 0x2A2A2A;
pub const STROKE_DIP: f32 = 1.0;
pub const SHADOW_BLUR_DIP: f32 = 12.0;
pub const SHADOW_OFFSET_Y_DIP: f32 = 2.0;
pub const SHADOW_OPACITY: f32 = 0.35;
/// Room around the largest shape inside the visual window: shadow blur + offset.
pub const SHADOW_PAD_DIP: f32 = 16.0;

pub const SPRING_DAMPING: f32 = 0.8;
pub const SPRING_PERIOD_MS: i64 = 60;
pub const CONTENT_FADE_OUT_MS: i64 = 80;
/// Content fades in after the shape has mostly settled.
pub const CONTENT_FADE_IN_DELAY_MS: i64 = 260;
pub const CONTENT_FADE_IN_MS: i64 = 120;
pub const COLLAPSE_AFTER_LEAVE_MS: u32 = 1000;
/// Invisible top-center strip that reveals the hidden island (design.md D12): height, and dwell time.
pub const REVEAL_STRIP_DIP: f32 = 2.0;
pub const REVEAL_DELAY_MS: u32 = 150;
/// Paused activity (music) stays visible this long, then hides (media-now-playing D3).
pub const PAUSE_HIDE_MS: u32 = 30_000;

pub const TEXT_PRIMARY: u32 = 0xFFFFFF;
pub const TEXT_SECONDARY: u32 = 0x9A9A9F;
pub const ACCENT: u32 = 0x30D158;
pub const FONT_FAMILY: &str = "Segoe UI Variable Text";
pub const FONT_COMPACT_DIP: f32 = 13.0;
pub const FONT_TITLE_DIP: f32 = 15.0;
pub const FONT_SUBTITLE_DIP: f32 = 12.0;
pub const FONT_CLOCK_DIP: f32 = 40.0;
/// Horizontal inset of Compact slots / Expanded content.
pub const CONTENT_INSET_DIP: f32 = 14.0;
pub const EXPANDED_INSET_DIP: f32 = 22.0;

// Media (media-now-playing D4/D5). Positions are relative to the shape, in DIP.
pub const ICON_FONT: &str = "Segoe Fluent Icons";
pub const ART_PX: u32 = 160; // decoded album-art size: 56 DIP up to ~285% without upscaling (~100 KB)
pub const ART_RADIUS_DIP: f32 = 10.0;
pub const ART_EXPANDED_DIP: f32 = 56.0;
pub const ART_COMPACT_DIP: f32 = 25.0;
/// Left inset of the Compact art: clear of the pill's rounded end (user feedback: 6 DIP looked cramped).
pub const ART_COMPACT_INSET_DIP: f32 = 10.0;
pub const ART_PLACEHOLDER: u32 = 0x2C2C2E;
pub const BAR_COLOR: u32 = 0xE5E5EA;
pub const BAR_COUNT: usize = 4;
pub const BAR_W_DIP: f32 = 3.0;
pub const BAR_GAP_DIP: f32 = 2.5;
pub const BAR_H_DIP: f32 = 15.0;
pub const BAR_PAUSED_SCALE: f32 = 0.3;
pub const PROGRESS_TRACK: u32 = 0x3A3A3C;
pub const PROGRESS_Y_DIP: f32 = 102.0;
pub const PROGRESS_H_DIP: f32 = 4.0;
/// Room left of/right of the progress track for the elapsed/remaining times.
pub const PROGRESS_SIDE_DIP: f32 = 44.0;
pub const FONT_TIME_DIP: f32 = 11.0;
pub const BUTTONS_Y_DIP: f32 = 136.0;
pub const BUTTON_SPACING_DIP: f32 = 64.0;
pub const BUTTON_HIT_DIP: f32 = 36.0;
pub const FONT_BUTTON_DIP: f32 = 22.0;
pub const DISABLED: u32 = 0x5A5A5E;
