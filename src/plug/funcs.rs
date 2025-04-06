macro_rules! create_default_func {
    {$name:ident($($params:ident: $types:ty),*$(,)?)} => {
        pub unsafe fn $name($($params: $types),*) -> std::option::Option<
            std::collections::HashMap<
                std::string::String,
                std::option::Option<String>,
            >,
        >
        {
            let ret = std::option::Option::Some(
                std::collections::HashMap::from_iter(
                    (*super::LIBRARIES).as_ref()?.iter().filter_map(
                        |items| std::option::Option::Some((
                            items.0.clone(),
                            unsafe {
                                *items.1.get::<
                                    unsafe extern "C" fn($($types),*) -> Box<Option<String>>
                                >(stringify!($name).as_bytes()).ok()?($($params),*)
                            },
                        )),
                    ),
                ),
            );
            return ret;
        }
    };
    {$($name:ident($($params:ident: $types:ty),*));+;} => {
        $(create_default_func!{$name($($params: $types),*)})+
    };
}


create_default_func!{
    before_showing_window(data: idl::Data);
    after_showing_window(data: idl::Data);
    end(data: idl::Data);
}
