//! Pure island state machine (design.md D6). No Win32 here so it stays testable.
use crate::theme::{self, ShapeSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    Idle,
    Compact,
    Expanded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    MouseEnter,
    MouseLeave,
    Click,
    LeaveTimeout,
    ActivityStart,
    ActivityEnd,
    SessionLock,
}

/// What the window must do with the one-shot collapse timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer {
    Keep,
    Start,
    Stop,
}

#[derive(Default)]
pub struct Island {
    // ponytail: a single bool activity until real sources exist (Phase 3+ brings payloads).
    activity: bool,
    expanded: bool,
    hover: bool,
}

impl Island {
    pub fn on(&mut self, event: Event) -> Timer {
        match event {
            Event::MouseEnter => {
                self.hover = true;
                return Timer::Stop;
            }
            Event::MouseLeave => {
                self.hover = false;
                if self.expanded {
                    return Timer::Start;
                }
            }
            Event::Click => self.expanded |= self.activity,
            Event::LeaveTimeout => self.expanded = self.hover && self.expanded,
            Event::ActivityStart => self.activity = true,
            Event::ActivityEnd => {
                self.activity = false;
                self.expanded = false;
            }
            Event::SessionLock => {
                self.expanded = false;
                self.hover = false;
                return Timer::Stop;
            }
        }
        Timer::Keep
    }

    pub fn visual(&self) -> Visual {
        match (self.activity, self.expanded) {
            (false, _) => Visual::Idle,
            (true, false) => Visual::Compact,
            (true, true) => Visual::Expanded,
        }
    }

    /// Target shape in DIP: state size, grown slightly while hovered (not when expanded).
    pub fn shape(&self) -> ShapeSize {
        let base = match self.visual() {
            Visual::Idle => theme::IDLE,
            Visual::Compact => theme::COMPACT,
            Visual::Expanded => return theme::EXPANDED,
        };
        if !self.hover {
            return base;
        }
        let s = theme::HOVER_SCALE;
        ShapeSize { w: base.w * s, h: base.h * s, radius: base.h * s / 2.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn island(events: &[Event]) -> Island {
        let mut i = Island::default();
        for &e in events {
            i.on(e);
        }
        i
    }

    #[test]
    fn idle_by_default_and_click_without_activity_does_nothing() {
        let i = island(&[Event::Click]);
        assert_eq!(i.visual(), Visual::Idle);
        assert_eq!(i.shape(), theme::IDLE);
    }

    #[test]
    fn hover_grows_within_8_percent_and_reverts() {
        let mut i = island(&[Event::MouseEnter]);
        let s = i.shape();
        assert!(s.w > theme::IDLE.w && s.w <= theme::IDLE.w * 1.08);
        assert!(s.h > theme::IDLE.h && s.h <= theme::IDLE.h * 1.08);
        assert_eq!(s.radius, s.h / 2.0);
        i.on(Event::MouseLeave);
        assert_eq!(i.shape(), theme::IDLE);
    }

    #[test]
    fn activity_compact_click_expands_no_hover_growth() {
        let i = island(&[Event::ActivityStart, Event::MouseEnter, Event::Click]);
        assert_eq!(i.visual(), Visual::Expanded);
        assert_eq!(i.shape(), theme::EXPANDED);
    }

    #[test]
    fn leave_starts_timer_only_when_expanded_and_timeout_collapses() {
        let mut i = island(&[Event::ActivityStart, Event::MouseEnter]);
        assert_eq!(i.on(Event::MouseLeave), Timer::Keep);
        i.on(Event::MouseEnter);
        i.on(Event::Click);
        assert_eq!(i.on(Event::MouseLeave), Timer::Start);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Compact);
    }

    #[test]
    fn returning_before_timeout_keeps_expanded() {
        let mut i = island(&[Event::ActivityStart, Event::MouseEnter, Event::Click, Event::MouseLeave]);
        assert_eq!(i.on(Event::MouseEnter), Timer::Stop);
        i.on(Event::LeaveTimeout); // stale timer racing the re-enter
        assert_eq!(i.visual(), Visual::Expanded);
    }

    #[test]
    fn activity_end_returns_to_idle() {
        let i = island(&[Event::ActivityStart, Event::Click, Event::ActivityEnd]);
        assert_eq!(i.visual(), Visual::Idle);
        let i = island(&[Event::ActivityEnd, Event::ActivityStart]);
        assert_eq!(i.visual(), Visual::Compact); // end doesn't leave a sticky expand behind
    }

    #[test]
    fn session_lock_collapses_and_clears_hover() {
        let mut i = island(&[Event::ActivityStart, Event::MouseEnter, Event::Click]);
        assert_eq!(i.on(Event::SessionLock), Timer::Stop);
        assert_eq!(i.visual(), Visual::Compact);
        assert_eq!(i.shape(), theme::COMPACT);
    }
}
