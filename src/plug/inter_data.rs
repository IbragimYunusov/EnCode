use std::rc::Rc;
use std::cell::RefCell;


pub const VERSION: &str = "0.0.0";
thread_local!
{
    pub static DATA: idl::Data = Rc::new(RefCell::new(
        idl::InterData::V0_0_0 {
            version: std::ffi::CString::new("0.0.0").unwrap(),
            gui: None,
            app: None,
            inner_spacing: crate::app::ui::INNER_SPACING,
            outter_spacing: crate::app::ui::OUTTER_SPACING,
            spacing_delta: crate::app::ui::SPACING_DELTA,
        },
    ));
}
