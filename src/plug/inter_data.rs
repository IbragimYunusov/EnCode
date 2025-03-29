use std::cell::RefCell;
use std::os::raw::c_char;

use gtk4::gdk_pixbuf::Pixbuf;


thread_local! {
    pub static DATA: RefCell<idl::InterData> = RefCell::new(idl::InterData {
        version: b"0.1.0\0".as_ptr() as *const c_char,
        gui: None,
        app: None,
        tree_view_icons: idl::TreeViewIcons {
            file: crate::app::ui::FILE_ICON.with(|d| -> *const Option<Pixbuf> {d}),
            dir: crate::app::ui::DIR_ICON.with(|d| -> *const Option<Pixbuf> {d}),
            filled_dir: crate::app::ui::FILLED_DIR_ICON.with(|d| -> *const Option<Pixbuf> {d}),
            symlink: crate::app::ui::SYMLINK_ICON.with(|d| -> *const Option<Pixbuf> {d}),
            unknown: crate::app::ui::UNKNOWN_ICON.with(|d| -> *const Option<Pixbuf> {d}),
        },
        inner_spacing: crate::app::ui::INNER_SPACING,
        outter_spacing: crate::app::ui::OUTTER_SPACING,
        spacing_delta: crate::app::ui::SPACING_DELTA,
    });
}
