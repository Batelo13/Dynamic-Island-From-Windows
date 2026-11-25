//! UI strings. ponytail: in-code table for 3 strings; move to files when the settings UI lands.
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub enum Text {
    ShowIsland,
    HideIsland,
    Quit,
}

fn is_portuguese() -> bool {
    static PT: OnceLock<bool> = OnceLock::new();
    // LANG_PORTUGUESE = 0x16 (primary language id = low 10 bits).
    *PT.get_or_init(|| unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff == 0x16)
}

pub fn t(text: Text) -> &'static str {
    match (text, is_portuguese()) {
        (Text::ShowIsland, true) => "Mostrar ilha",
        (Text::ShowIsland, false) => "Show island",
        (Text::HideIsland, true) => "Ocultar ilha",
        (Text::HideIsland, false) => "Hide island",
        (Text::Quit, true) => "Sair",
        (Text::Quit, false) => "Quit",
    }
}
