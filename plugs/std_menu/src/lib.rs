use std::ffi::c_char;
use std::ptr::null;

use gio::prelude::*;
use gtk4::prelude::*;

use libloading::Library;
use idl::{get_gui_el, get_str, get_attr};

type Res<T> = Result<T, Box<dyn std::error::Error>>;


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
pub extern "C" fn before_showing_window(data: idl::Data) -> *const c_char
{
    if let Err(e) = (|| -> Res<()> {
        gtk4::init()?;
        println!("{}", get_str!(data.app_id));
        let menu = get_attr!([create_menu(data)].ok());
        unsafe{(*(*data).app).set_menubar(Some(&menu));}
        let paned = get_attr!(
            [get_gui_el!(data.gui, data.gui_ids.paned_id, gtk4::Paned)]?
        );
        paned.unparent();
        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        vbox.append(&gtk4::PopoverMenuBar::from_model(Some(&menu)));
        vbox.append(&paned);
        get_attr!([get_gui_el!(
            data.gui,
            data.gui_ids.app_window_id,
            gtk4::ApplicationWindow,
        )]?).set_child(Some(&vbox));
        return Ok(());
    })() {e.to_string().as_ptr() as *const c_char} else {null()}
}


fn create_menu(data: idl::Data) -> Res<gio::Menu>
{
    let menu = gio::Menu::new();
    menu.append_submenu(
        Some("Проект"),
        &create_project_menu(data)?,
    );
    return Ok(menu);
}


fn create_project_menu(data: idl::Data) -> Res<gio::Menu>
{
    println!("kdsjfklsdflsk");
    let menu = gio::Menu::new();
    menu.append_item(
        &gio::MenuItem::new(
            Some("Сохранить Текущий Файл"),
            Some("project.save_cur_file"),
        ),
    );
    let action_group = gio::SimpleActionGroup::new();
    action_group.add_action(&gio::SimpleAction::new(
        "project.save_cur_file",
        None,
    ));
    action_group.connect_action_added(
        Some("project.save_cur_file"),
        move |_, _| {|| -> Option<_> {
            unsafe {
                let _ = STD_COMMANDS
                    .get::<unsafe extern "C" fn(idl::Data) -> Res<()>>(b"save_cur_file")
                    .ok()?
                    (data);
            };
            return Some(());
        }();},
    );
    let app_window: gtk4::ApplicationWindow
        = get_attr!([get_gui_el!(data.gui, data.gui_ids.app_window_id)]?);
    app_window.insert_action_group("project", Some(&action_group));
    return Ok(menu);
}
