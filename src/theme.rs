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

pub const TEXT_PRIMARY: u32 = 0xFFFFFF;
pub const TEXT_SECONDARY: u32 = 0x9A9A9F;
pub const ACCENT: u32 = 0x30D158;
pub const FONT_FAMILY: &str = "Segoe UI Variable Text";
pub const FONT_COMPACT_DIP: f32 = 13.0;
pub const FONT_TITLE_DIP: f32 = 15.0;
pub const FONT_SUBTITLE_DIP: f32 = 12.0;
/// Horizontal inset of Compact slots / Expanded content.
pub const CONTENT_INSET_DIP: f32 = 14.0;
pub const EXPANDED_INSET_DIP: f32 = 22.0;
