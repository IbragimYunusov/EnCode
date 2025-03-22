macro_rules! create_default_func {
    {$name:ident($($params:ident: $types:ty),*) -> $ret_type:ty} => {
        pub unsafe fn $name($($params: $types),*) -> std::option::Option<
            std::collections::HashMap<
                std::string::String,
                std::result::Result<
                    $ret_type,
                    std::boxed::Box<dyn std::error::Error>,
                >,
            >,
        >
        {
            println!(stringify!($name));
            println!("{:?}", (*super::LIBRARIES).as_ref().map(|s| s.values()));
            // $(println!("{:?}", $params);)*
            let ret = std::option::Option::Some(
                std::collections::HashMap::from_iter(
                    (*super::LIBRARIES).as_ref()?.iter().filter_map(
                        |items| std::option::Option::Some((
                            items.0.clone(),
                            unsafe {items.1.get::<
                                unsafe extern "C" fn($($types),*)
                                    -> std::result::Result<
                                        $ret_type,
                                        Box<dyn std::error::Error>,
                                    >
                            >(stringify!($name).as_bytes()).ok()?($($params),*)},
                        )),
                    ),
                ),
            );
            println!("{:?}", ret);
            return ret;
        }
    };
    {$($name:ident($($params:ident: $types:ty),*) -> $ret_type:ty);+;} => {
        $(create_default_func!{$name($($params: $types),*) -> $ret_type})+
    };
}


create_default_func!{
    define_inter_data_version(version: &str) -> ();
    start(data: idl::Data) -> ();
    before_showing_window(data: idl::Data) -> ();
    end(data: idl::Data) -> ();
}
