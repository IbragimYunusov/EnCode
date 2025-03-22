use std::cell::RefCell;
use std::os::raw::c_char;


pub const VERSION: &str = "0.0.0";

thread_local! {
    pub static DATA: RefCell<idl::InterData> = RefCell::new(idl::InterData {
        version: b"0.0.0\0".as_ptr() as *const c_char,
        gui_ids: idl::GuiIds {
            app_window_id: b"encode.app_window\0".as_ptr() as *const c_char,
            paned_id: b"encode.paned\0".as_ptr() as *const c_char,
            tree_view_id: b"encode.tree_view\0".as_ptr() as *const c_char,
            store_id: b"encode.store\0".as_ptr() as *const c_char,
            column_id: b"encode.column\0".as_ptr() as *const c_char,
            renderer_id: b"encode.renderer\0".as_ptr() as *const c_char,
            tree_view_scrolled_window_id: b"encode.tree_view_scrolled_window_id\0".as_ptr() as *const c_char,
            notebook_id: b"encode.notebook\0".as_ptr() as *const c_char,
        },
        gui: std::ptr::null_mut(),
        app_id: crate::app::APP_ID.as_ptr() as *const c_char,
        inner_spacing: crate::app::ui::INNER_SPACING,
        outter_spacing: crate::app::ui::OUTTER_SPACING,
        spacing_delta: crate::app::ui::SPACING_DELTA,
    });
}
