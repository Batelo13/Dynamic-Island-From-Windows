#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod island;
mod layout;
mod log;
mod render;
mod settings;
mod theme;
mod window;

use std::time::Instant;
use windows::{
    Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateMutexW,
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
        },
    },
    core::w,
};

fn main() {
    let startup = Instant::now();
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    log::init();

    // Single instance per session; handle intentionally kept for the process lifetime.
    let _mutex = unsafe { CreateMutexW(None, true, w!("Local\\DynamicIslandForWindows")) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return;
    }

    if let Err(e) = window::create(startup) {
        log::error(&format!("startup failed: {e}"));
        return;
    }

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
