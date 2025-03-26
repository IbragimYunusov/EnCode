use std::os::raw::c_char;


pub type Data = *mut InterData;
pub type Ret = Box<Option<String>>;

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;


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
        ).ok_or(std::io::Error::other(
            format!("Выражение {} или его атрибут — None", stringify!($obj)),
        ))?
    };
}


#[macro_export]
macro_rules! get_gui_el {
    ($id1:ident.$gui:ident.$el:ident) => {
        unsafe{&$crate::get_attr!([(*$id1).$gui.as_ref()]?).$el}
    };
}


#[macro_export]
macro_rules! get_str {
    ($data:ident$(.$attr:ident)*) => {
        &*unsafe{std::ffi::CStr::from_ptr((*$data)$(.$attr)*).to_string_lossy()}
    };
}


#[repr(C)]
pub struct Gui {
    pub window: gtk4::ApplicationWindow,
    pub vbox: gtk4::Box,
    pub paned: gtk4::Paned,
    pub tree_view: gtk4::TreeView,
    pub store: gtk4::TreeStore,
    pub column: gtk4::TreeViewColumn,
    pub renderer: gtk4::CellRendererText,
    pub tree_view_scrolled_window: gtk4::ScrolledWindow,
    pub notebook: gtk4::Notebook,
}


#[repr(C)]
pub struct InterData {
    pub version: *const c_char,
    pub gui: Option<Gui>,
    pub app: Option<gtk4::Application>,
    pub inner_spacing: i32,
    pub outter_spacing: i32,
    pub spacing_delta: i32,
}


pub fn show_error_dialog(
    parent: &gtk4::ApplicationWindow,
    err: impl std::string::ToString,
) {
    use gtk4::prelude::*;
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
