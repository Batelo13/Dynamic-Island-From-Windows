//! Pure placement math (physical pixels). No Win32 calls here so it stays testable.

use crate::theme::{self, ShapeSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    fn inflate(&self, d: i32) -> Rect {
        Rect { left: self.left - d, top: self.top - d, right: self.right + d, bottom: self.bottom + d }
    }
}

#[derive(Clone, Debug)]
pub struct Monitor {
    pub name: String,
    pub work: Rect,
    pub dpi: u32,
    pub primary: bool,
}

/// Configured monitor if present, else primary, else any.
pub fn pick<'a>(monitors: &'a [Monitor], wanted: Option<&str>) -> Option<&'a Monitor> {
    wanted
        .and_then(|w| monitors.iter().find(|m| m.name == w))
        .or_else(|| monitors.iter().find(|m| m.primary))
        .or(monitors.first())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Visual window in screen coordinates: fixed, sized for the largest shape + shadow (design.md D3).
    pub visual: Rect,
    /// Horizontal center and top edge every shape is anchored to (screen px).
    pub center_x: i32,
    pub top: i32,
    /// Top of the work area (the reveal strip hugs it, ignoring the margin).
    pub work_top: i32,
    pub scale: f32,
}

impl Placement {
    pub fn px(&self, dip: f32) -> i32 {
        (dip * self.scale).round() as i32
    }

    /// Screen rect of a shape anchored at the top center.
    pub fn shape(&self, s: ShapeSize) -> Rect {
        let (w, h) = (self.px(s.w), self.px(s.h));
        let left = self.center_x - w / 2;
        Rect { left, top: self.top, right: left + w, bottom: self.top + h }
    }

    /// Reveal strip while hidden: Idle width, a couple of px tall, flush with the work-area top.
    pub fn strip(&self) -> Rect {
        let idle = self.shape(theme::IDLE);
        Rect { left: idle.left, top: self.work_top, right: idle.right, bottom: self.work_top + self.px(theme::REVEAL_STRIP_DIP).max(1) }
    }
}

pub fn place(work: Rect, dpi: u32, top_margin_dip: f32) -> Placement {
    let scale = dpi as f32 / 96.0;
    let px = |dip: f32| (dip * scale).round() as i32;
    // Center derived from the Idle capsule so its position stays identical to Phase 1.
    let idle_w = px(theme::IDLE.w);
    let center_x = work.left + (work.width() - idle_w) / 2 + idle_w / 2;
    let top = work.top + px(top_margin_dip);
    let (max_w, pad) = (px(theme::MAX_W), px(theme::SHADOW_PAD_DIP));
    let left = center_x - max_w / 2;
    let visual = Rect { left, top, right: left + max_w, bottom: top + px(theme::MAX_H) }.inflate(pad);
    Placement { visual, center_x, top, work_top: work.top, scale }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(l: i32, t: i32, rt: i32, b: i32) -> Rect {
        Rect { left: l, top: t, right: rt, bottom: b }
    }

    #[test]
    fn centered_with_margin_at_100_percent() {
        let p = place(r(0, 0, 1920, 1040), 96, 8.0);
        assert_eq!(p.shape(theme::IDLE), r(897, 8, 1023, 45));
        assert_eq!(p.visual, r(780 - 16, 8 - 16, 1140 + 16, 168 + 16));
        // Every state shares the top edge and center.
        let e = p.shape(theme::EXPANDED);
        assert_eq!((e.top, (e.left + e.right) / 2), (8, 960));
        // Reveal strip: same span as Idle, flush with the work-area top, 2 px tall.
        assert_eq!(p.strip(), r(897, 0, 1023, 2));
    }

    #[test]
    fn scales_at_150_percent() {
        let p = place(r(0, 0, 2880, 1680), 144, 8.0);
        let c = p.shape(theme::IDLE);
        assert_eq!((c.width(), c.height()), (189, 56));
        assert_eq!(c.top, 12);
        let center = (c.left + c.right) / 2;
        assert!((center - 1440).abs() <= 1);
    }

    #[test]
    fn respects_work_area_offset_top_taskbar_and_second_monitor() {
        // Second monitor to the right, taskbar docked at the top (work starts at y=48).
        let p = place(r(1920, 48, 3840, 1080), 96, 0.0);
        let c = p.shape(theme::IDLE);
        assert_eq!(c.top, 48);
        assert_eq!((c.left + c.right) / 2, 2880);
    }

    #[test]
    fn pick_falls_back_to_primary() {
        let m = |n: &str, p| Monitor { name: n.into(), work: Rect::default(), dpi: 96, primary: p };
        let mons = vec![m("A", false), m("B", true)];
        assert_eq!(pick(&mons, Some("A")).unwrap().name, "A");
        assert_eq!(pick(&mons, Some("gone")).unwrap().name, "B");
        assert_eq!(pick(&mons, None).unwrap().name, "B");
        assert!(pick(&[], None).is_none());
    }
}
