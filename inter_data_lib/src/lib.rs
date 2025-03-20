use std::cell::RefCell;
use std::rc::Rc;
use std::ffi::CString;

pub type Data = Rc<RefCell<InterData>>;


#[macro_export]
macro_rules! get_attr {($obj:ident$(.$attr:ident)*) => {
    $obj$(
        .$attr()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Не удалось получить атрибут {} объекта {} или его атрибутов",
                stringify!($attr),
                stringify!($data),
            ),
        ))?
    )*
};}


#[macro_export]
macro_rules! generate {
    {$(
        $enum:ident {
            $($variants:ident{$($v_field:ident: $v_field_type:ty),*$(,)?}),*$(,)?
        } => {
            $($field:ident: $field_type:ty >- $($impl_variants:ident),*$(,)?);*$(;)?
        }
    );+$(;)?} => {$(
        #[repr(C)]
        pub enum $enum {$($variants{$($v_field: $v_field_type),*}),*}
        impl $enum {$(
            pub extern "C" fn $field(&self) -> Option<&$field_type> {
                return match self {
                    $(Self::$impl_variants{$field: ref value, ..})|* => Some(value),
                    #[allow(unreachable_patterns)] _ => None,
                };
            }
        )*}
    )+};
}


generate!{
    InterData {
        V0_0_0 {
            version: CString,
            gui: Option<Gui>,
            app: Option<gtk4::Application>,
            inner_spacing: i32,
            outter_spacing: i32,
            spacing_delta: i32,
        },
    } => {
        version: CString >- V0_0_0;
        gui: Option<Gui> >- V0_0_0;
        app: Option<gtk4::Application> >- V0_0_0;
        inner_spacing: i32 >- V0_0_0;
        outter_spacing: i32 >- V0_0_0;
        spacing_delta: i32 >- V0_0_0;
    };
    Gui {
        LAUNCHER {
            window: gtk4::ApplicationWindow,
        },
        V0_0_0 {
            window: gtk4::ApplicationWindow,
            paned: gtk4::Paned,
            tree_view: gtk4::TreeView,
            store: gtk4::TreeStore,
            column: gtk4::TreeViewColumn,
            renderer: gtk4::CellRendererText,
            tree_view_scrolled_window: gtk4::ScrolledWindow,
            notebook: gtk4::Notebook,
        },
    } => {
        window: gtk4::ApplicationWindow >- LAUNCHER, V0_0_0;
        paned: gtk4::Paned >- V0_0_0;
        tree_view: gtk4::TreeView >- V0_0_0;
        store: gtk4::TreeStore >- V0_0_0;
        column: gtk4::TreeViewColumn >- V0_0_0;
        renderer: gtk4::CellRendererText >- V0_0_0;
        tree_view_scrolled_window: gtk4::ScrolledWindow >- V0_0_0;
        notebook: gtk4::Notebook >- V0_0_0;
    };
}
