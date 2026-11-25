//! Windows of the island (design.md D2/D3):
//! - host:   hidden top-level window; system broadcasts + tray owner (can take foreground for the tray menu).
//! - visual: click-through composition surface, never receives mouse input.
//! - input:  invisible (alpha 1) window clipped to the capsule shape; receives clicks, never activates.
use crate::{i18n::{t, Text}, layout, log, render::Renderer, settings::{self, Settings}};
use std::{cell::RefCell, path::PathBuf, time::Instant};
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM, TRUE},
        Graphics::Gdi::{CreateRoundRectRgn, EnumDisplayMonitors, GetMonitorInfoW, GetStockObject, SetWindowRgn, BLACK_BRUSH, HBRUSH, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW},
        System::{LibraryLoader::GetModuleHandleW, RemoteDesktop::{WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION}},
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            Shell::{Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW},
            WindowsAndMessaging::*,
        },
    },
    core::{w, Result, BOOL, PCWSTR},
};

const WM_TRAY: u32 = WM_APP + 1;
const WM_RELAYOUT: u32 = WM_APP + 2;
const ID_TOGGLE: usize = 1;
const ID_QUIT: usize = 2;
const MONITORINFOF_PRIMARY: u32 = 1;
const WTS_SESSION_UNLOCK: usize = 0x8;
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
        let (v, c) = (p.visual, p.capsule);
        let flags = SWP_NOACTIVATE | SWP_NOOWNERZORDER;
        let result = (|| -> Result<()> {
            unsafe {
                SetWindowPos(self.visual, Some(HWND_TOPMOST), v.left, v.top, v.width(), v.height(), flags)?;
                SetWindowPos(self.input, Some(HWND_TOPMOST), c.left, c.top, c.width(), c.height(), flags)?;
                // Region coords are window-relative; +1 because GDI excludes right/bottom edges.
                let rgn = CreateRoundRectRgn(0, 0, c.width() + 1, c.height() + 1, c.height(), c.height());
                SetWindowRgn(self.input, Some(rgn), true); // system owns rgn after this call
            }
            self.renderer.set_capsule(v.width() as f32, v.height() as f32, p.pad as f32, c.width() as f32, c.height() as f32)
        })();
        if let Err(e) = result {
            log::error(&format!("relayout failed: {e}"));
        }
        self.apply_visibility();
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

fn request_relayout(hwnd: HWND) {
    // Deferred: avoids re-entrancy and coalesces bursts of display messages.
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_RELAYOUT, WPARAM(0), LPARAM(0));
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_LBUTTONUP => {
            #[cfg(debug_assertions)]
            log::info("capsule click");
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => request_relayout(hwnd),
        WM_SETTINGCHANGE if wparam.0 as u32 == SPI_SETWORKAREA.0 => request_relayout(hwnd),
        WM_POWERBROADCAST if wparam.0 == PBT_APMRESUMEAUTOMATIC => request_relayout(hwnd),
        WM_WTSSESSION_CHANGE if wparam.0 == WTS_SESSION_UNLOCK => request_relayout(hwnd),
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
