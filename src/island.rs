//! Pure island state machine (design.md D6, D12). No Win32 here so it stays testable.
use crate::theme::{self, ShapeSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    /// Nothing on screen; the shape waits above the top edge (design.md D12).
    Hidden,
    Idle,
    Compact,
    Expanded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    MouseEnter,
    MouseLeave,
    Click,
    RevealTimeout,
    LeaveTimeout,
    ActivityStart,
    ActivityEnd,
    /// The activity paused (music): stays visible `theme::PAUSE_HIDE_MS`, then goes dormant.
    Paused,
    Resumed,
    PauseTimeout,
    SessionLock,
}

/// What the window must do with its one-shot timers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer {
    Keep,
    /// Cursor parked on the top strip: reveal after `theme::REVEAL_DELAY_MS`.
    StartReveal,
    /// Cursor left: collapse/hide after `theme::COLLAPSE_AFTER_LEAVE_MS`.
    StartLeave,
    /// Cancel both.
    Stop,
    /// Activity paused: go dormant after `theme::PAUSE_HIDE_MS`.
    StartPause,
    StopPause,
}

#[derive(Default)]
pub struct Island {
    // ponytail: a single bool activity until real sources exist (Phase 3+ brings payloads).
    activity: bool,
    /// Activity paused long enough to hide (design.md D3); revealing still shows it.
    dormant: bool,
    /// Shown by the top-center gesture while hidden.
    revealed: bool,
    expanded: bool,
    hover: bool,
}

impl Island {
    pub fn on(&mut self, event: Event) -> Timer {
        match event {
            Event::MouseEnter => {
                self.hover = true;
                if self.visual() == Visual::Hidden {
                    return Timer::StartReveal;
                }
                return Timer::Stop;
            }
            Event::MouseLeave => {
                self.hover = false;
                if self.visual() == Visual::Hidden {
                    return Timer::Stop; // left the strip before the reveal delay
                }
                if self.expanded || !self.activity || self.dormant {
                    return Timer::StartLeave;
                }
            }
            Event::RevealTimeout => self.revealed |= self.hover,
            Event::Click => self.expanded |= matches!(self.visual(), Visual::Idle | Visual::Compact),
            Event::LeaveTimeout => {
                if !self.hover {
                    self.expanded = false;
                    self.revealed = false;
                }
            }
            Event::ActivityStart => {
                self.activity = true;
                self.dormant = false;
            }
            Event::ActivityEnd => {
                self.activity = false;
                self.dormant = false;
                self.expanded = false;
                self.revealed = self.hover; // stay as Idle under the cursor, else slide away
                return Timer::StopPause;
            }
            Event::Paused => return Timer::StartPause,
            Event::Resumed => {
                self.dormant = false;
                return Timer::StopPause;
            }
            Event::PauseTimeout => {
                self.dormant = self.activity;
                // Under the cursor it stays (revealed) until the cursor leaves; otherwise it slides away.
                self.revealed = self.hover;
                self.expanded &= self.hover;
            }
            Event::SessionLock => {
                *self = Island { activity: self.activity, dormant: self.dormant, ..Island::default() };
                return Timer::Stop;
            }
        }
        Timer::Keep
    }

    pub fn activity(&self) -> bool {
        self.activity
    }

    pub fn visual(&self) -> Visual {
        let shown = (self.activity && !self.dormant) || self.revealed;
        match (shown, self.expanded, self.activity) {
            (false, _, _) => Visual::Hidden,
            (true, true, _) => Visual::Expanded,
            (true, false, true) => Visual::Compact,
            (true, false, false) => Visual::Idle,
        }
    }

    /// Target shape in DIP: state size, grown slightly while hovered (Idle/Compact only).
    /// Hidden keeps the Idle size; the window parks it above the top edge.
    pub fn shape(&self) -> ShapeSize {
        let base = match self.visual() {
            Visual::Hidden => return theme::IDLE,
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

    fn revealed() -> Island {
        island(&[Event::MouseEnter, Event::RevealTimeout])
    }

    #[test]
    fn hidden_by_default_and_click_on_strip_does_nothing() {
        let i = island(&[Event::Click]);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn strip_reveals_only_after_delay_with_cursor_still_there() {
        let mut i = Island::default();
        assert_eq!(i.on(Event::MouseEnter), Timer::StartReveal);
        assert_eq!(i.visual(), Visual::Hidden);
        i.on(Event::RevealTimeout);
        assert_eq!(i.visual(), Visual::Idle);

        let mut quick = Island::default();
        quick.on(Event::MouseEnter);
        assert_eq!(quick.on(Event::MouseLeave), Timer::Stop);
        quick.on(Event::RevealTimeout); // stale timer racing the leave
        assert_eq!(quick.visual(), Visual::Hidden);
    }

    #[test]
    fn revealed_idle_hides_one_second_after_leaving() {
        let mut i = revealed();
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn returning_before_timeout_keeps_it() {
        let mut i = revealed();
        i.on(Event::MouseLeave);
        assert_eq!(i.on(Event::MouseEnter), Timer::Stop);
        i.on(Event::LeaveTimeout); // stale
        assert_eq!(i.visual(), Visual::Idle);
    }

    #[test]
    fn hover_grows_within_8_percent_and_reverts() {
        let mut i = revealed();
        let s = i.shape();
        assert!(s.w > theme::IDLE.w && s.w <= theme::IDLE.w * 1.08);
        assert!(s.h > theme::IDLE.h && s.h <= theme::IDLE.h * 1.08);
        assert_eq!(s.radius, s.h / 2.0);
        i.on(Event::MouseLeave);
        assert_eq!(i.shape(), theme::IDLE);
    }

    #[test]
    fn click_on_revealed_idle_expands_clock_then_hides() {
        let mut i = revealed();
        i.on(Event::Click);
        assert_eq!(i.visual(), Visual::Expanded);
        assert!(!i.activity());
        assert_eq!(i.shape(), theme::EXPANDED);
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn activity_shows_compact_without_gesture_and_collapses_to_it() {
        let mut i = island(&[Event::ActivityStart]);
        assert_eq!(i.visual(), Visual::Compact);
        i.on(Event::MouseEnter);
        assert_eq!(i.on(Event::MouseLeave), Timer::Keep); // Compact stays; nothing to time out
        i.on(Event::MouseEnter);
        i.on(Event::Click);
        assert_eq!(i.visual(), Visual::Expanded);
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Compact);
    }

    #[test]
    fn activity_end_hides_unless_cursor_is_on_it() {
        let i = island(&[Event::ActivityStart, Event::Click, Event::ActivityEnd]);
        assert_eq!(i.visual(), Visual::Hidden);
        let mut i = island(&[Event::ActivityStart, Event::MouseEnter, Event::ActivityEnd]);
        assert_eq!(i.visual(), Visual::Idle);
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn session_lock_hides_or_shows_compact_without_hover() {
        let mut i = revealed();
        i.on(Event::Click);
        assert_eq!(i.on(Event::SessionLock), Timer::Stop);
        assert_eq!(i.visual(), Visual::Hidden);

        let mut i = island(&[Event::ActivityStart, Event::MouseEnter, Event::Click]);
        i.on(Event::SessionLock);
        assert_eq!(i.visual(), Visual::Compact);
        assert_eq!(i.shape(), theme::COMPACT);
    }

    fn playing() -> Island {
        island(&[Event::ActivityStart])
    }

    #[test]
    fn pause_keeps_visible_then_goes_dormant_and_hidden() {
        let mut i = playing();
        assert_eq!(i.on(Event::Paused), Timer::StartPause);
        assert_eq!(i.visual(), Visual::Compact);
        i.on(Event::PauseTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
        assert!(i.activity());
    }

    #[test]
    fn resume_before_timeout_keeps_it_visible() {
        let mut i = playing();
        i.on(Event::Paused);
        assert_eq!(i.on(Event::Resumed), Timer::StopPause);
        assert_eq!(i.visual(), Visual::Compact);
    }

    #[test]
    fn resume_after_dormant_shows_compact_again() {
        let mut i = island(&[Event::ActivityStart, Event::Paused, Event::PauseTimeout]);
        i.on(Event::Resumed);
        assert_eq!(i.visual(), Visual::Compact);
    }

    #[test]
    fn revealing_a_dormant_activity_shows_its_compact_and_expands() {
        let mut i = island(&[Event::ActivityStart, Event::Paused, Event::PauseTimeout]);
        assert_eq!(i.on(Event::MouseEnter), Timer::StartReveal);
        i.on(Event::RevealTimeout);
        assert_eq!(i.visual(), Visual::Compact);
        i.on(Event::Click);
        assert_eq!(i.visual(), Visual::Expanded);
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn revealed_dormant_compact_hides_after_leaving() {
        let mut i = island(&[Event::ActivityStart, Event::Paused, Event::PauseTimeout, Event::MouseEnter, Event::RevealTimeout]);
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn pause_timeout_under_cursor_waits_for_the_cursor_to_leave() {
        let mut i = island(&[Event::ActivityStart, Event::MouseEnter, Event::Paused, Event::PauseTimeout]);
        assert_eq!(i.visual(), Visual::Compact);
        assert_eq!(i.on(Event::MouseLeave), Timer::StartLeave);
        i.on(Event::LeaveTimeout);
        assert_eq!(i.visual(), Visual::Hidden);
    }

    #[test]
    fn session_lock_keeps_dormancy() {
        let mut i = island(&[Event::ActivityStart, Event::Paused, Event::PauseTimeout]);
        i.on(Event::SessionLock);
        assert_eq!(i.visual(), Visual::Hidden);
    }
}
