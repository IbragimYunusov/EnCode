use std::os::raw::c_char;


pub type Data = *mut InterData;


#[macro_export]
macro_rules! get_attr {
    (
        $obj:ident
        $(.$attr:ident
            $(::<$($generic:ty),*$(,)?>)?
            ($($params:expr),*$(,)?)
        )*
    ) => {
        unsafe{$obj$(
            .$attr$(::<$($generic),*>)?($($params),*)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Не удалось получить атрибут {} объекта {} или его атрибута",
                    stringify!($attr),
                    stringify!($obj),
                ),
            ))?
        )*}
    };
    (
        $obj:ident
        $(.$attr:ident
            $(::<$($generic:ty),*$(,)?>)?
            ($($params:expr),*$(,)?)
        )*?
    ) => {
        $crate::get_attr!(
            $obj$(.$attr$(::<$($generic),*>)?($($params),*))*
        ).ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Объект {} или его атрибут — None", stringify!($obj)),
        ))?
    };
    (
        [$obj:expr]
        $(.$attr:ident
            $(::<$($generic:ty),*$(,)?>)?
            ($($params:expr),*$(,)?)
        )*
    ) => {
        unsafe{($obj)$(
            .$attr$(::<$($generic),*>)?($($params),*)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Не удалось получить атрибут {} объекта {} или его атрибута",
                    stringify!($attr),
                    stringify!($obj),
                ),
            ))?
        )*}
    };
    (
        [$obj:expr]
        $(.$attr:ident
            $(::<$($generic:ty),*$(,)?>)?
            ($($params:expr),*$(,)?)
        )*?
    ) => {
        $crate::get_attr!(
            [$obj]$(.$attr$(::<$($generic),*>)?($($params),*))*
        ).ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Выражение {} или его атрибут — None", stringify!($obj)),
        ))?
    };
}


#[macro_export]
macro_rules! get_gui_el {
    ($id1:ident$(.$attr1:ident)*, $id2:ident$(.$attr2:ident)*$(, $type:ty)?$(,)?) => {
        unsafe{(*(*$id1)$(.$attr1)*).object$(::<$type>)?(&*unsafe{
            std::ffi::CStr::from_ptr((*$id2)$(.$attr2)*).to_string_lossy()
        })}
    };
}


#[macro_export]
macro_rules! get_str {
    ($data:ident$(.$attr:ident)*) => {
        &*unsafe{std::ffi::CStr::from_ptr((*$data)$(.$attr)*).to_string_lossy()}
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
    pub app: *mut gtk4::Application,
    pub inner_spacing: i32,
    pub outter_spacing: i32,
    pub spacing_delta: i32,
}
