use std::ffi::CStr;

use gtk4::{
    prelude::*,
    gdk::Display,
    Application,
    ApplicationWindow,
    Builder,
    IconTheme,
    Widget,
};
use crate::app::func;
use crate::plug::inter_data::DATA as INTER_DATA;

pub mod launcher;
pub mod editor;


pub const INNER_SPACING: i32 = 6;
pub const OUTTER_SPACING: i32 = 8;
pub const SPACING_DELTA: i32 = OUTTER_SPACING - INNER_SPACING;


pub fn build_ui(app: &Application) -> Builder
{
    let name = if let Some(default_display) = Display::default() {
        let icon_theme = IconTheme::for_display(&default_display);
        if let Ok((name, path)) = func::get_name_and_path_for_search_icon() {
            icon_theme.add_search_path(&path);
            Some(name)
        } else {None}
    } else {None};
    let window = ApplicationWindow::builder()
        .application(app)
        .build();
    window.set_icon_name(name.as_ref().map(String::as_str));
    let ret = match *super::APP_TYPE {
        super::AppType::LAUNCHER => launcher::build_ui(window),
        super::AppType::EDITOR(ref dir) => editor::build_ui(window, dir),
    };
    return ret;
}


pub fn show_error_dialog(parent: &ApplicationWindow, err: impl std::string::ToString)
{
    let dialog = gtk4::MessageDialog::new(
        Some(parent),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Error,
        gtk4::ButtonsType::Ok,
        format!("Ошибка: {}", err.to_string()),
    );
    dialog.set_title(Some("Ошибка"));
    dialog.connect_response(|dialog, _| dialog.destroy());
    dialog.show();
}
