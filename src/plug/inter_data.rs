use std::cell::RefCell;
use std::os::raw::c_char;


thread_local! {
    pub static DATA: RefCell<idl::InterData> = RefCell::new(idl::InterData {
        version: b"0.0.0\0".as_ptr() as *const c_char,
        gui: None,
        app: None,
        inner_spacing: crate::app::ui::INNER_SPACING,
        outter_spacing: crate::app::ui::OUTTER_SPACING,
        spacing_delta: crate::app::ui::SPACING_DELTA,
    });
}
