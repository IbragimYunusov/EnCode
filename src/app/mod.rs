use std::path::PathBuf;
use std::env;

use std::rc::Rc;
use std::cell::RefCell;
use lazy_static::lazy_static;

use gtk4::prelude::*;
use gtk4::Application;
use glib::ExitCode;
use gio::ApplicationFlags;

pub mod ui;
pub mod func;


pub enum AppType {
    LAUNCHER,
    EDITOR(PathBuf),
}

pub fn define_app_type() -> AppType {
    return match env::args().collect::<Vec<String>>().as_slice() {
        [_] => AppType::LAUNCHER,
        [_, dir] => AppType::EDITOR(PathBuf::from(dir)),
        _ => panic!("Unsupported args matching..."),
    };
}

pub fn define_app_id() -> String {
    return match &*APP_TYPE {
        AppType::LAUNCHER => "EnCode.launcher".to_string(),
        AppType::EDITOR(_) => "EncCode.editor".to_string(),
    };
}

lazy_static!
{
    pub static ref APP_TYPE: AppType = define_app_type();
    pub static ref APP_ID: String = define_app_id();
}
thread_local!
{
    pub static APP: Rc<RefCell<Application>> = Rc::new(RefCell::new(
        Application::builder()
            .application_id(&*APP_ID)
            .flags(ApplicationFlags::HANDLES_OPEN)
            .build(),
    ));
}


pub fn main() -> ExitCode {
    return APP.with(
        |data| {
            let app = data.borrow();
            app.connect_activate(|app| {ui::build_ui(app);});
            app.connect_open(|application, _, _| {
                let ret = ui::build_ui(application);
                crate::plug::inter_data::DATA.with(|data| {
                    match *data.borrow_mut() {
                        idl::InterData::V0_0_0{ref mut app, ref mut gui, ..} => {
                            *gui = Some(ret);
                            *app = Some(application.clone());
                        },
                    }
                });
                let _ = crate::plug::inter_data::DATA.with(
                    |rdata| -> Result<(), Box<dyn std::error::Error>> {
                        let data = rdata.borrow();
                        println!("{}\texpected 6", idl::get_attr!(data.inner_spacing));
                        println!("{}\texpected 8", idl::get_attr!(data.outter_spacing));
                        println!("{}\texpected 2", idl::get_attr!(data.spacing_delta));
                        crate::plug::funcs::start(Rc::clone(rdata));
                        return Ok(());
                    },
                );
            });
            return app.run();
        },
    );
}
