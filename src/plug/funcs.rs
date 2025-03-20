use std::rc::Rc;


macro_rules! create_default_func {
    {$name:ident($($params:ident: $types:ty),*) -> $ret_type:ty} => {
        pub fn $name($($params: $types),*)
            -> std::option::Option<
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
                                unsafe extern fn($(&$types),*)
                                    -> std::result::Result<
                                        $ret_type,
                                        Box<dyn std::error::Error>,
                                    >
                            >(stringify!($name).as_bytes()).ok()?($(&$params),*)},
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


macro_rules! default_wrapper {
    {$($params:expr),+ => $func:ident, $new_func:ident -> $ret_type:ty} => {
        pub fn $new_func()
            -> std::option::Option<
                std::collections::HashMap<
                    std::string::String,
                    std::result::Result<
                        $ret_type,
                        std::boxed::Box<dyn std::error::Error>,
                    >,
                >,
            >
        {
            return $func($($params),*);
        }
    };
    {$($($params:expr),+ => $func:ident, $new_func:ident -> $ret_type:ty);+;} => {
        $(default_wrapper!{$($params),* => $func, $new_func -> $ret_type})*
    };
}


create_default_func!{
    define_inter_data_version(version: &str) -> ();
    start(data: idl::Data) -> ();
    before_showing_window(data: idl::Data) -> ();
    end(data: idl::Data) -> ();
}

default_wrapper!{
    super::inter_data::VERSION
        => define_inter_data_version, dflt_define_inter_data_version -> ();
    super::inter_data::DATA.with(Rc::clone) => start, dflt_start -> ();
    super::inter_data::DATA.with(Rc::clone)
        => before_showing_window, dflt_before_showing_window -> ();
    super::inter_data::DATA.with(Rc::clone) => end, dflt_end -> ();
}
