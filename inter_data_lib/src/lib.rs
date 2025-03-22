use std::os::raw::c_char;


pub type Data = *mut InterData;


#[macro_export]
macro_rules! get_attr {
    ($obj:ident$(.$attr:ident$(($($params:expr),*$(,)?))?)*) => {
        unsafe{(*$obj)$(.$attr$(($($params),*))?)*}
    };
}


#[repr(C)]
pub struct GuiIds {
    pub app_window_id: *const c_char,
    pub paned_id: *const c_char,
    pub tree_view_id: *const c_char,
    pub store_id: *const c_char,
    pub column_id: *const c_char,
    pub renderer_id: *const c_char,
    pub tree_view_scrolled_window_id: *const c_char,
    pub notebook_id: *const c_char,
}


#[repr(C)]
pub struct InterData {
    pub version: *const c_char,
    pub gui_ids: GuiIds,
    pub gui: *mut gtk4::Builder,
    pub app_id: *const c_char,
    pub inner_spacing: i32,
    pub outter_spacing: i32,
    pub spacing_delta: i32,
}
