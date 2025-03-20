use gio::prelude::*;
use gtk4::prelude::*;

use idl::get_attr;

use libloading::Library;
use std::rc::Rc;
type Res<T> = Result<T, Box<dyn std::error::Error>>;


lazy_static::lazy_static! {
    pub static ref STD_COMMANDS: Library = unsafe {
        Library::new(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("std_commands.rlib"),
        ).unwrap()
    };
}


#[no_mangle]
pub extern "C" fn start(rdata: idl::Data) -> Res<()>
{
    let data = rdata.borrow();
    println!("{}\texpected 6", get_attr!(data.inner_spacing));
    println!("{}\texpected 8", get_attr!(data.outter_spacing));
    println!("{}\texpected 2", get_attr!(data.spacing_delta));
    println!("{:?}", get_attr!(data.app.as_ref).application_id());
    get_attr!(data.app.as_ref).set_menubar(
        create_menu(Rc::clone(&rdata)).ok().as_ref(),
    );
    return Ok(());
}


fn create_menu(rdata: idl::Data) -> Res<gio::Menu>
{
    let menu = gio::Menu::new();
    // menu.append_submenu(
    //     Some("Проект"),
    //     &create_project_menu(Rc::clone(&rdata))?,
    // );
    return Ok(menu);
}


fn create_project_menu(rdata: idl::Data) -> Res<gio::Menu>
{
    let data = rdata.borrow();
    let menu = gio::Menu::new();
    menu.append(
        Some("Save Current File"),
        Some(&format!(
            "{}.project.save_cur_file",
            get_attr!(data.app.as_ref.application_id),
        )),
    );
    let action_group = gio::SimpleActionGroup::new();
    action_group.add_action(&gio::SimpleAction::new(
        "project.save_cur_file",
        None,
    ));
    let rdata_cloned = Rc::clone(&rdata);
    action_group.connect_action_added(
        Some("project.save_cur_file"),
        move |_, _| {|| -> Option<_> {
            unsafe {
                let _ = STD_COMMANDS
                    .get::<unsafe fn(idl::Data) -> Res<()>>(b"save_cur_file")
                    .ok()?
                    (Rc::clone(&rdata_cloned));
            };
            return Some(());
        }();},
    );
    return Ok(menu);
}
