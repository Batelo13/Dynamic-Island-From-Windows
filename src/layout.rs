//! Pure placement math (physical pixels). No Win32 calls here so it stays testable.

pub const CAPSULE_W_DIP: f32 = 126.0;
pub const CAPSULE_H_DIP: f32 = 37.0;
/// Extra room around the capsule inside the click-through visual window (future shadow/expansion).
pub const VISUAL_PAD_DIP: f32 = 16.0;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    /// Capsule in screen coordinates (input window).
    pub capsule: Rect,
    /// Visual window in screen coordinates; capsule sits at offset `pad` inside it.
    pub visual: Rect,
    pub pad: i32,
}

pub fn place(work: Rect, dpi: u32, top_margin_dip: f32) -> Placement {
    let scale = dpi as f32 / 96.0;
    let px = |dip: f32| (dip * scale).round() as i32;
    let (w, h) = (px(CAPSULE_W_DIP), px(CAPSULE_H_DIP));
    let left = work.left + (work.width() - w) / 2;
    let top = work.top + px(top_margin_dip);
    let capsule = Rect { left, top, right: left + w, bottom: top + h };
    let pad = px(VISUAL_PAD_DIP);
    Placement { capsule, visual: capsule.inflate(pad), pad }
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
        assert_eq!(p.capsule, r(897, 8, 1023, 45));
        assert_eq!(p.visual, r(881, -8, 1039, 61));
    }

    #[test]
    fn scales_at_150_percent() {
        let p = place(r(0, 0, 2880, 1680), 144, 8.0);
        assert_eq!((p.capsule.width(), p.capsule.height()), (189, 56));
        assert_eq!(p.capsule.top, 12);
        let center = (p.capsule.left + p.capsule.right) / 2;
        assert!((center - 1440).abs() <= 1);
    }

    #[test]
    fn respects_work_area_offset_top_taskbar_and_second_monitor() {
        // Second monitor to the right, taskbar docked at the top (work starts at y=48).
        let p = place(r(1920, 48, 3840, 1080), 96, 0.0);
        assert_eq!(p.capsule.top, 48);
        assert_eq!((p.capsule.left + p.capsule.right) / 2, 2880);
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
