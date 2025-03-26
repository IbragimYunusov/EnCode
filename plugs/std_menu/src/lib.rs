use gtk4::prelude::*;

use libloading::Library;
use idl::{get_gui_el, get_attr};


macro_rules! menu {
    {$(
        $submenu:literal, $submenu_preffix:ident {$(
            $section:literal, $section_preffix:ident {$(
                $item:literal, $suffix:ident
            );*$(;)?}
        );*$(;)?}
    );*$(;)?} => {{
        let menu = gio::Menu::new();
        $(
            let submenu = gio::Menu::new();
            $(
                let section = gio::Menu::new();
                $(
                    let item = gio::MenuItem::new(
                        Some($item),
                        Some(&format!(
                            "{}.{}__{}",
                            stringify!($submenu_preffix),
                            stringify!($section_preffix),
                            stringify!($suffix),
                        )),
                    );
                    section.append_item(&item);
                )*
                submenu.append_section(
                    if $section == "" {None} else {Some($section)},
                    &section,
                );
            )*
            menu.append_submenu(Some($submenu), &submenu);
        )*
        menu
    }};
}


lazy_static::lazy_static! {
    pub static ref STD_COMMANDS: Library = unsafe {
        Library::new(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("libstd_commands.so"),
        ).unwrap()
    };
}


#[no_mangle]
pub extern "C" fn before_showing_window(data: idl::Data) -> idl::Ret
{
    Box::new(|| -> idl::Res<()> {
        unsafe{gtk4::set_initialized();}
        let menu = get_attr!([create_menu()].ok());
        unsafe{(*data).app.as_ref().map(|app| app.set_menubar(Some(&menu)));}
        let vbox = get_gui_el!(data.gui.vbox);
        vbox.prepend(&gtk4::PopoverMenuBar::from_model(Some(&menu)));
        println!("ok {}", unsafe{(*data).inner_spacing});
        return Ok(());
    }().err().map(|e| e.to_string()))
}


fn create_menu() -> idl::Res<gio::Menu>
{
    let menu = menu!{
        "Проект", project {
            "Проект", project {
                "Создать Новый", new;
                "Открыть", open;
            };
            "Файл", file {
                "Создать Новый", new;
                "Открыть", open;
                "Сохранить", save;
                "Сохранить Все", save_all;
                "Удалить", remove;
            };
            "Директория", dir {
                "Создать Новую", new;
                "Удалить", remove;
            };
            "", other {
                "Выход", exit;
            };
        };
        "Правка", edit {
            "", undo_redo {
                "Отмена", undo;
                "Заново", redo;
            };
            "", ctrl_axcv {
                "Выделить Все", select_all;
                "Вырезать", cut;
                "Копировать", copy;
                "Вставить", paste;
            };
            "", search {
                "Искать", find;
                "Заменить", replace;
            };
        };
        "Запуск", run {
            "", run {
                "Запуск", run;
                "Использовать виртуальное окружение", use_env;
            };
        };
    };
    return Ok(menu);
}
