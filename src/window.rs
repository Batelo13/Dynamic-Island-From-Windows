//! Windows of the island (design.md D2/D3):
//! - host:   hidden top-level window; system broadcasts + tray owner (can take foreground for the tray menu).
//! - visual: click-through composition surface, never receives mouse input.
//! - input:  invisible (alpha 1) window clipped to the island shape; receives mouse input, never activates.
use crate::{
    i18n::{t, Text},
    island::{self, Event, Island, Visual},
    layout, log,
    media::{self, Command, Media, Snapshot},
    render::{Frame, Renderer},
    settings::{self, Settings},
    theme,
};
use std::{cell::RefCell, path::PathBuf, time::Instant};
use windows::{
    Win32::{
        Foundation::{COLORREF, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT, TRUE, WPARAM},
        Graphics::Gdi::{CreateRoundRectRgn, EnumDisplayMonitors, GetMonitorInfoW, GetStockObject, SetWindowRgn, BLACK_BRUSH, HBRUSH, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW},
        System::{
            LibraryLoader::GetModuleHandleW,
            RemoteDesktop::{WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION},
            Threading::{RegisterWaitForSingleObject, INFINITE, WT_EXECUTEONLYONCE},
        },
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT},
            Shell::{Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW},
            WindowsAndMessaging::*,
        },
    },
    core::{w, Result, BOOL, PCWSTR},
};

const WM_TRAY: u32 = WM_APP + 1;
const WM_RELAYOUT: u32 = WM_APP + 2;
const WM_DEVICE_LOST: u32 = WM_APP + 3;
const WM_MEDIA: u32 = WM_APP + 4;
const ID_TOGGLE: usize = 1;
const ID_QUIT: usize = 2;
#[cfg(debug_assertions)]
const ID_DEMO_START: usize = 3;
#[cfg(debug_assertions)]
const ID_DEMO_END: usize = 4;
const TIMER_COLLAPSE: usize = 1;
const TIMER_REVEAL: usize = 2;
const TIMER_PAUSE: usize = 3;
/// Refreshes the media times once a second, only while the media Expanded is visible.
const TIMER_TIMES: usize = 4;
const WM_MOUSELEAVE: u32 = 0x02A3;
const MONITORINFOF_PRIMARY: u32 = 1;
const WTS_SESSION_LOCK: usize = 0x7;
const WTS_SESSION_UNLOCK: usize = 0x8;
const PBT_APMSUSPEND: usize = 0x4;
const PBT_APMRESUMEAUTOMATIC: usize = 0x12;

struct App {
    host: HWND,
    visual: HWND,
    input: HWND,
    renderer: Renderer,
    settings: Settings,
    settings_path: PathBuf,
    taskbar_created: u32,
    startup: Option<Instant>,
    island: Island,
    placement: Option<layout::Placement>,
    dpi: u32,
    /// Windows "Animation effects" (design.md D8).
    animations: bool,
    /// TrackMouseEvent armed (cursor inside the input window).
    hovering: bool,
    /// Last input-window geometry (rect, corner radius). Re-applying an identical region cancels
    /// mouse tracking and fakes a MouseLeave, so unchanged geometry is skipped.
    input_shape: Option<(layout::Rect, i32)>,
    media: Media,
    now_playing: Option<Snapshot>,
    /// Last visual applied and when it was entered (media buttons wait for the content fade-in).
    shown: (Visual, Instant),
    /// Media button under the cursor (Expanded), for the hover highlight and the hand cursor.
    hovered_button: Option<usize>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

pub fn create(startup: Instant) -> Result<()> {
    let hinstance = unsafe { GetModuleHandleW(None)? };
    let class = w!("DynamicIslandWindow");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: hinstance.into(),
        lpszClassName: class,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        hbrBackground: HBRUSH(unsafe { GetStockObject(BLACK_BRUSH) }.0),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&wc) } == 0 {
        return Err(windows::core::Error::from_thread());
    }
    let make = |ex: WINDOW_EX_STYLE, title: PCWSTR| unsafe {
        CreateWindowExW(ex, class, title, WS_POPUP, 0, 0, 0, 0, None, None, Some(hinstance.into()), None)
    };
    let tool = WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
    let host = make(WS_EX_TOOLWINDOW, w!("Dynamic Island Host"))?;
    let visual = make(tool | WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOREDIRECTIONBITMAP, w!("Dynamic Island"))?;
    let input = make(tool | WS_EX_TOPMOST | WS_EX_LAYERED, w!("Dynamic Island Input"))?;
    unsafe {
        // Layered windows need attributes before they show. Input at alpha 1: invisible but hit-testable.
        SetLayeredWindowAttributes(visual, COLORREF(0), 255, LWA_ALPHA)?;
        SetLayeredWindowAttributes(input, COLORREF(0), 1, LWA_ALPHA)?;
        if let Err(e) = WTSRegisterSessionNotification(host, NOTIFY_FOR_THIS_SESSION) {
            log::warn(&format!("session notifications unavailable: {e}"));
        }
    }

    let settings_path = settings::default_path();
    let app = App {
        host,
        visual,
        input,
        renderer: Renderer::new(visual)?,
        settings: settings::load(&settings_path),
        settings_path,
        taskbar_created: unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) },
        startup: Some(startup),
        island: Island::default(),
        placement: None,
        dpi: 96,
        animations: animations_enabled(),
        hovering: false,
        input_shape: None,
        media: media::start(host, WM_MEDIA),
        now_playing: None,
        shown: (Visual::Hidden, Instant::now()),
        hovered_button: None,
    };
    app.add_tray_icon();
    APP.with(|a| *a.borrow_mut() = Some(app));
    with_app(|app| app.relayout());
    Ok(())
}

fn with_app(f: impl FnOnce(&mut App)) {
    // try_borrow: Win32 can re-enter the wndproc synchronously (e.g. SetWindowPos → WM_DPICHANGED).
    APP.with(|a| {
        if let Ok(mut guard) = a.try_borrow_mut()
            && let Some(app) = guard.as_mut()
        {
            f(app)
        }
    });
}

impl App {
    fn relayout(&mut self) {
        let monitors = enum_monitors();
        let Some(mon) = layout::pick(&monitors, self.settings.monitor.as_deref()) else {
            log::warn("no monitors found");
            return;
        };
        let p = layout::place(mon.work, mon.dpi, self.settings.top_margin);
        let v = p.visual;
        let result = (|| -> Result<()> {
            unsafe { SetWindowPos(self.visual, Some(HWND_TOPMOST), v.left, v.top, v.width(), v.height(), SWP_NOACTIVATE | SWP_NOOWNERZORDER)? };
            self.renderer.set_window(v.width() as f32, v.height() as f32, p.scale)
        })();
        if let Err(e) = result {
            log::error(&format!("relayout failed: {e}"));
        }
        self.placement = Some(p);
        self.dpi = mon.dpi;
        self.apply_shape(false);
        self.apply_visibility();
    }

    /// Push the island's target shape to the input window (region = destination, design.md D2) and renderer.
    fn apply_shape(&mut self, animate: bool) {
        let Some(p) = self.placement else { return };
        let visual = self.island.visual();
        let hidden = visual == Visual::Hidden;
        let size = self.island.shape();
        let c = p.shape(size);
        let radius = p.px(size.radius);
        let v = p.visual;
        // Hidden: park the shape (and its shadow) above the visual window so the window edge clips it;
        // revealing then slides it down with the same spring (design.md D12).
        let y = if hidden { -(c.height() + p.px(theme::SHADOW_PAD_DIP)) } else { c.top - v.top };
        // While hidden the input window is only the reveal strip.
        let input = if hidden { p.strip() } else { c };
        let frame = Frame {
            x: (c.left - v.left) as f32,
            y: y as f32,
            w: c.width() as f32,
            h: c.height() as f32,
            radius: radius as f32,
        };
        let shape = (input, if hidden { 0 } else { radius });
        let moved = self.input_shape != Some(shape);
        self.input_shape = Some(shape);
        let result = (|| -> Result<()> {
            if moved {
                unsafe {
                    // ponytail: region jumps to the target shape; ~400 ms mismatch while the spring runs.
                    // Upgrade: sample the spring in-process during transitions if phantom clicks show up.
                    SetWindowPos(self.input, Some(HWND_TOPMOST), input.left, input.top, input.width(), input.height(), SWP_NOACTIVATE | SWP_NOOWNERZORDER)?;
                    // Region coords are window-relative; +1 because GDI excludes right/bottom edges. Strip: plain rect.
                    let rgn = (!hidden).then(|| CreateRoundRectRgn(0, 0, c.width() + 1, c.height() + 1, radius * 2, radius * 2));
                    SetWindowRgn(self.input, rgn, true); // system owns rgn after this call
                }
            }
            self.renderer.apply(frame, visual, self.island.activity(), self.now_playing.as_ref(), self.dpi, animate && self.animations)
        })();
        if self.shown.0 != visual {
            self.shown = (visual, Instant::now());
            self.set_hovered_button(None);
            // Times tick only while they're on screen (spec: Custo da música).
            unsafe {
                if visual == Visual::Expanded && self.now_playing.is_some() {
                    SetTimer(Some(self.host), TIMER_TIMES, 1000, None);
                } else {
                    let _ = KillTimer(Some(self.host), TIMER_TIMES);
                }
            }
        }
        if let Err(e) = result {
            log::error(&format!("apply shape failed: {e}"));
        }
        self.watch_device();
    }

    fn send(&mut self, event: Event, animate: bool) {
        let timer = self.island.on(event);
        match timer {
            island::Timer::StartReveal => unsafe {
                SetTimer(Some(self.host), TIMER_REVEAL, theme::REVEAL_DELAY_MS, None);
            },
            island::Timer::StartLeave => unsafe {
                SetTimer(Some(self.host), TIMER_COLLAPSE, theme::COLLAPSE_AFTER_LEAVE_MS, None);
            },
            island::Timer::Stop => unsafe {
                let _ = KillTimer(Some(self.host), TIMER_COLLAPSE);
                let _ = KillTimer(Some(self.host), TIMER_REVEAL);
            },
            island::Timer::StartPause => unsafe {
                SetTimer(Some(self.host), TIMER_PAUSE, theme::PAUSE_HIDE_MS, None);
            },
            island::Timer::StopPause => unsafe {
                let _ = KillTimer(Some(self.host), TIMER_PAUSE);
            },
            island::Timer::Keep => {}
        }
        self.apply_shape(animate);
    }

    /// New media state from the media thread: translate the change into island events.
    fn media_changed(&mut self) {
        let new = self.media.latest.lock().ok().and_then(|l| l.clone());
        let old = std::mem::replace(&mut self.now_playing, new.clone());
        match (old, new) {
            (None, Some(n)) if !n.playing => {
                // Already paused when it appeared (e.g. player opened paused): register it as dormant
                // without rendering in between — a one-frame Compact would load the text GPU stack for nothing.
                self.island.on(Event::ActivityStart);
                self.send(Event::PauseTimeout, true);
            }
            (None, Some(_)) => self.send(Event::ActivityStart, true),
            (Some(_), None) => self.send(Event::ActivityEnd, true),
            (Some(o), Some(n)) if o.playing != n.playing => self.send(if n.playing { Event::Resumed } else { Event::Paused }, true),
            _ => self.apply_shape(true), // same activity, new track/position: redraw what changed
        }
    }

    /// Enabled media button at (x, y) in input-window client coords (= shape coords), once the
    /// media Expanded content is visible (it fades in after the morph).
    fn media_button_at(&self, x: i32, y: i32) -> Option<usize> {
        let (visual, since) = self.shown;
        let (Some(m), Some(p)) = (&self.now_playing, self.placement) else { return None };
        if visual != Visual::Expanded || since.elapsed().as_millis() < (theme::CONTENT_FADE_IN_DELAY_MS + theme::CONTENT_FADE_IN_MS) as u128 {
            return None;
        }
        let w = p.shape(self.island.shape()).width();
        layout::media_buttons(w, p.scale)
            .iter()
            .position(|b| (b.left..b.right).contains(&x) && (b.top..b.bottom).contains(&y))
            .filter(|&i| m.can[i])
    }

    fn media_click(&mut self, x: i32, y: i32) {
        if let Some(i) = self.media_button_at(x, y) {
            self.media.command([Command::Previous, Command::TogglePlayPause, Command::Next][i]);
        }
    }

    /// Hover highlight only changes when the button under the cursor changes (mouse moves are frequent).
    fn set_hovered_button(&mut self, button: Option<usize>) {
        if self.hovered_button == button {
            return;
        }
        self.hovered_button = button;
        if let Err(e) = self.renderer.set_button_hover(button, self.animations) {
            log::warn(&format!("button hover failed: {e}"));
        }
    }

    fn mouse_move(&mut self, x: i32, y: i32) {
        self.set_hovered_button(self.media_button_at(x, y));
        if self.hovering {
            return;
        }
        self.hovering = true;
        let mut tme = TRACKMOUSEEVENT { cbSize: size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: self.input, dwHoverTime: 0 };
        let _ = unsafe { TrackMouseEvent(&mut tme) };
        self.send(Event::MouseEnter, true);
    }

    /// Wait (thread pool, no polling) on the D3D device-removed event whenever a new device was created.
    fn watch_device(&mut self) {
        let Some(event) = self.renderer.device_removed.take() else { return };
        unsafe extern "system" fn on_removed(ctx: *mut core::ffi::c_void, _: bool) {
            let _ = unsafe { PostMessageW(Some(HWND(ctx)), WM_DEVICE_LOST, WPARAM(0), LPARAM(0)) };
        }
        let mut wait = HANDLE::default();
        // ponytail: one-shot waits are never unregistered (one tiny handle per device loss, a rare event).
        if let Err(e) = unsafe { RegisterWaitForSingleObject(&mut wait, event, Some(on_removed), Some(self.host.0), INFINITE, WT_EXECUTEONLYONCE) } {
            log::warn(&format!("device-removed watch unavailable: {e}"));
        }
    }

    fn apply_visibility(&mut self) {
        let cmd = if self.settings.visible { SW_SHOWNOACTIVATE } else { SW_HIDE };
        unsafe {
            let _ = ShowWindow(self.visual, cmd);
            let _ = ShowWindow(self.input, cmd);
        }
        if let Some(t0) = self.startup.take() {
            log::info(&format!("startup_ms={}", t0.elapsed().as_millis()));
        }
    }

    fn tray_data(&self) -> NOTIFYICONDATAW {
        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.host,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAY,
            // ponytail: stock icon until the app has its own artwork.
            hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap_or_default(),
            ..Default::default()
        };
        for (dst, src) in nid.szTip.iter_mut().zip("Dynamic Island".encode_utf16()) {
            *dst = src;
        }
        nid
    }

    fn add_tray_icon(&self) {
        if !unsafe { Shell_NotifyIconW(NIM_ADD, &self.tray_data()) }.as_bool() {
            log::warn("tray icon not added");
        }
    }

    fn show_tray_menu(&mut self) {
        let toggle = if self.settings.visible { t(Text::HideIsland) } else { t(Text::ShowIsland) };
        let cmd = unsafe {
            let Ok(menu) = CreatePopupMenu() else { return };
            let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
            let (a, b) = (wide(toggle), wide(t(Text::Quit)));
            let _ = AppendMenuW(menu, MF_STRING, ID_TOGGLE, PCWSTR(a.as_ptr()));
            // Dev-only placeholder activity (design.md D11); not localized, never in release builds.
            #[cfg(debug_assertions)]
            {
                let _ = AppendMenuW(menu, MF_STRING, ID_DEMO_START, w!("Demo: iniciar atividade"));
                let _ = AppendMenuW(menu, MF_STRING, ID_DEMO_END, w!("Demo: encerrar"));
            }
            let _ = AppendMenuW(menu, MF_STRING, ID_QUIT, PCWSTR(b.as_ptr()));
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            // Required so the menu closes when clicking elsewhere (documented TrackPopupMenu quirk).
            let _ = SetForegroundWindow(self.host);
            let cmd = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, None, self.host, None);
            let _ = DestroyMenu(menu);
            cmd.0 as usize
        };
        match cmd {
            ID_TOGGLE => {
                self.settings.visible = !self.settings.visible;
                if let Err(e) = settings::save(&self.settings_path, &self.settings) {
                    log::warn(&format!("settings not saved: {e}"));
                }
                self.apply_visibility();
            }
            #[cfg(debug_assertions)]
            ID_DEMO_START => self.send(Event::ActivityStart, true),
            #[cfg(debug_assertions)]
            ID_DEMO_END => self.send(Event::ActivityEnd, true),
            ID_QUIT => unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &self.tray_data());
                let _ = WTSUnRegisterSessionNotification(self.host);
                PostQuitMessage(0);
            },
            _ => {}
        }
    }
}

fn enum_monitors() -> Vec<layout::Monitor> {
    unsafe extern "system" fn cb(hmon: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
        let out = unsafe { &mut *(data.0 as *mut Vec<layout::Monitor>) };
        let mut info = MONITORINFOEXW { monitorInfo: MONITORINFO { cbSize: size_of::<MONITORINFOEXW>() as u32, ..Default::default() }, ..Default::default() };
        if unsafe { GetMonitorInfoW(hmon, &mut info as *mut _ as *mut MONITORINFO) }.as_bool() {
            let (mut dx, mut dy) = (96, 96);
            let _ = unsafe { GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) };
            let w = info.monitorInfo.rcWork;
            let name_len = info.szDevice.iter().position(|&c| c == 0).unwrap_or(info.szDevice.len());
            out.push(layout::Monitor {
                name: String::from_utf16_lossy(&info.szDevice[..name_len]),
                work: layout::Rect { left: w.left, top: w.top, right: w.right, bottom: w.bottom },
                dpi: dx,
                primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
            });
        }
        TRUE
    }
    let mut out: Vec<layout::Monitor> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(cb), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

fn animations_enabled() -> bool {
    let mut on = TRUE;
    let ok = unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, Some(&mut on as *mut BOOL as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    ok.is_err() || on.as_bool() // unknown → animate
}

fn request_relayout(hwnd: HWND) {
    // Deferred: avoids re-entrancy and coalesces bursts of display messages.
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_RELAYOUT, WPARAM(0), LPARAM(0));
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_MOUSEMOVE => with_app(|app| app.mouse_move((lparam.0 & 0xffff) as i16 as i32, ((lparam.0 >> 16) & 0xffff) as i16 as i32)),
        WM_SETCURSOR => {
            // Hand cursor over a media button.
            let mut on_button = false;
            with_app(|app| on_button = hwnd == app.input && app.hovered_button.is_some());
            if on_button {
                unsafe { SetCursor(LoadCursorW(None, IDC_HAND).ok()) };
                return LRESULT(1);
            }
        }
        WM_MOUSELEAVE => with_app(|app| {
            app.set_hovered_button(None);
            app.hovering = false;
            app.send(Event::MouseLeave, true);
        }),
        WM_LBUTTONUP => with_app(|app| {
            let (x, y) = ((lparam.0 & 0xffff) as i16 as i32, ((lparam.0 >> 16) & 0xffff) as i16 as i32);
            app.media_click(x, y);
            app.send(Event::Click, true);
        }),
        WM_MEDIA => {
            with_app(|app| app.media_changed());
            return LRESULT(0);
        }
        WM_TIMER if wparam.0 == TIMER_TIMES => {
            with_app(|app| {
                if let Err(e) = app.renderer.redraw_front(app.shown.0, app.island.activity()) {
                    log::warn(&format!("times redraw failed: {e}"));
                }
            });
            return LRESULT(0);
        }
        WM_TIMER if wparam.0 == TIMER_REVEAL => {
            with_app(|app| {
                let _ = unsafe { KillTimer(Some(app.host), TIMER_REVEAL) }; // one-shot
                app.send(Event::RevealTimeout, true);
            });
            return LRESULT(0);
        }
        WM_TIMER if wparam.0 == TIMER_PAUSE => {
            with_app(|app| {
                let _ = unsafe { KillTimer(Some(app.host), TIMER_PAUSE) }; // one-shot
                app.send(Event::PauseTimeout, true);
            });
            return LRESULT(0);
        }
        WM_TIMER if wparam.0 == TIMER_COLLAPSE => {
            with_app(|app| {
                let _ = unsafe { KillTimer(Some(app.host), TIMER_COLLAPSE) }; // one-shot: no periodic timer left
                app.send(Event::LeaveTimeout, true);
            });
            return LRESULT(0);
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => request_relayout(hwnd),
        WM_SETTINGCHANGE => {
            with_app(|app| app.animations = animations_enabled());
            if wparam.0 as u32 == SPI_SETWORKAREA.0 {
                request_relayout(hwnd);
            }
        }
        WM_POWERBROADCAST if wparam.0 == PBT_APMSUSPEND => with_app(|app| app.send(Event::SessionLock, false)),
        WM_POWERBROADCAST if wparam.0 == PBT_APMRESUMEAUTOMATIC => request_relayout(hwnd),
        WM_WTSSESSION_CHANGE if wparam.0 == WTS_SESSION_LOCK => with_app(|app| app.send(Event::SessionLock, false)),
        WM_WTSSESSION_CHANGE if wparam.0 == WTS_SESSION_UNLOCK => request_relayout(hwnd),
        WM_DEVICE_LOST => {
            with_app(|app| {
                if let Err(e) = app.renderer.recover_device() {
                    log::error(&format!("device recovery failed: {e}"));
                }
                app.apply_shape(false);
            });
            return LRESULT(0);
        }
        WM_RELAYOUT => {
            // Drop queued duplicates so a burst produces one relayout.
            let mut m = MSG::default();
            while unsafe { PeekMessageW(&mut m, Some(hwnd), WM_RELAYOUT, WM_RELAYOUT, PM_REMOVE) }.as_bool() {}
            with_app(|app| app.relayout());
            return LRESULT(0);
        }
        WM_TRAY => {
            let event = (lparam.0 & 0xffff) as u32;
            if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                with_app(|app| app.show_tray_menu());
            }
            return LRESULT(0);
        }
        _ => {
            let mut handled = false;
            with_app(|app| {
                if msg == app.taskbar_created && hwnd == app.host {
                    app.add_tray_icon();
                    handled = true;
                }
            });
            if handled {
                return LRESULT(0);
            }
        }
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
