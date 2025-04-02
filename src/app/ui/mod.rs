use std::ffi::c_char;

use gtk4::{
    prelude::*,
    gdk::Display,
    gdk_pixbuf::Pixbuf,
    Application,
    ApplicationWindow,
    IconTheme,
};
use crate::app::func;

pub mod launcher;
pub mod editor;


pub const INNER_SPACING: i32 = 6;
pub const OUTTER_SPACING: i32 = 8;
pub const SPACING_DELTA: i32 = OUTTER_SPACING - INNER_SPACING;


thread_local!{
    pub static FILE_ICON: Option<Pixbuf>
        = func::editor::get_icon("file.svg").ok();
    pub static DIR_ICON: Option<Pixbuf>
        = func::editor::get_icon("dir.svg").ok();
    pub static FILLED_DIR_ICON: Option<Pixbuf>
        = func::editor::get_icon("filled_dir.svg").ok();
    pub static SYMLINK_ICON: Option<Pixbuf>
        = func::editor::get_icon("symlink.svg").ok();
    pub static UNKNOWN_ICON: Option<Pixbuf>
        = func::editor::get_icon("unknown.svg").ok();
}


pub enum BuildUIRet {
    Launcher(ApplicationWindow),
    Editor(idl::Gui),
}
impl BuildUIRet {
    pub fn window(&self) -> &ApplicationWindow {
        return match self {
            Self::Launcher(ref window) => window,
            Self::Editor(ref gui) => &gui.window,
        };
    }
    pub fn as_gui(self) -> Option<idl::Gui> {
        return match self {
            Self::Editor(gui) => Some(gui),
            _ => None,
        }
    }
}


pub fn build_ui(app: &Application) -> BuildUIRet
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
    crate::plug::inter_data::DATA.with_borrow_mut(
        |data| if let Some(icon_name) = name {
            data.icon_name = icon_name.as_ptr() as *const c_char;
        }
    );
    let ret = match *super::APP_TYPE {
        super::AppType::LAUNCHER =>
            BuildUIRet::Launcher(launcher::build_ui(window)),
        super::AppType::EDITOR(ref dir) =>
            BuildUIRet::Editor(editor::build_ui(window, dir)),
    };
    return ret;
}
