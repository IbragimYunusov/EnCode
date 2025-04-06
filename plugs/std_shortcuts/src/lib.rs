use gtk4::prelude::*;
use idl::get_attr;


macro_rules! create_shortcuts {
    {$app:expr => {$(
        $submenu_preffix:ident {$(
            $section_preffix:ident {$(
                $suffix:ident, $($shortcut:literal),+
            );*$(;)?}
        );*$(;)?}
    );*$(;)?}} => {
        $($($($app.set_accels_for_action(
            &format!(
                "{}.{}__{}",
                stringify!($submenu_preffix),
                stringify!($section_preffix),
                stringify!($suffix),
            ),
            &[$($shortcut),*],
        );)*)*)*
    };
}


#[unsafe(no_mangle)]
pub extern "C" fn before_showing_window(data: idl::Data) -> idl::Ret
{
    unsafe{gtk4::set_initialized();}
    Box::new(|| -> idl::Res {
        create_shortcuts!{get_attr!([(*data).app.as_ref()]?) => {
            project {
                project {
                    new, "<Ctrl><Alt>N";
                    open, "<Ctrl><Alt>O";
                };
                file {
                    new, "<Ctrl>N";
                    open, "<Ctrl>O";
                    save, "<Ctrl>S";
                    save_all, "<Ctrl><Shift>S";
                    remove, "<Ctrl>R";
                };
                dir {
                    new, "<Ctrl><Shift>N";
                    remove, "<Ctrl><Shift>R";
                };
                other {
                    exit, "<Alt>F4";
                };
            };
            edit {
                tab {
                    close, "<Ctrl>W";
                    close_all, "<Ctrl><Shift>W";
                };
                undo_redo {
                    undo, "<Ctrl>Z";
                    redo, "<Ctrl><Shift>Z";
                };
                ctrl_axcv {
                    select_all, "<Ctrl>A";
                    cut, "<Ctrl>X";
                    copy, "<Ctrl>C";
                    paste, "<Ctrl>V";
                };
                search {
                    find, "<Ctrl>F";
                    replace, "<Ctrl>H";
                };
            };
            run {
                run {
                    run, "<Ctrl>F5";
                };
            };
        }}
        return Ok(());
    }().err().map(|e| e.to_string()))
}
