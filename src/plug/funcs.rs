use std::ffi::{CStr, c_char};
use std::borrow::Cow;
use std::ptr::null;


macro_rules! create_default_func {
    {$name:ident($($params:ident: $types:ty),*)} => {
        pub unsafe fn $name($($params: $types),*) -> std::option::Option<
            std::collections::HashMap<
                std::string::String,
                std::option::Option<Cow<'static, str>>,
            >,
        >
        {
            return std::option::Option::Some(
                std::collections::HashMap::from_iter(
                    (*super::LIBRARIES).as_ref()?.iter().filter_map(
                        |items| std::option::Option::Some((
                            items.0.clone(),
                            unsafe {
                                let ret = items.1.get::<
                                    unsafe extern "C" fn($($types),*) -> *const c_char
                                >(stringify!($name).as_bytes()).ok()?($($params),*);
                                if ret == null() {
                                    None
                                } else {
                                    Some(CStr::from_ptr(ret).to_string_lossy())
                                }
                            },
                        )),
                    ),
                ),
            );
        }
    };
    {$($name:ident($($params:ident: $types:ty),*));+;} => {
        $(create_default_func!{$name($($params: $types),*)})+
    };
}


create_default_func!{
    define_inter_data_version(version: &str);
    start(data: idl::Data);
    before_showing_window(data: idl::Data);
    end(data: idl::Data);
}
