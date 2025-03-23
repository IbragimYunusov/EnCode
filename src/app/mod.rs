use std::path::PathBuf;
use std::env;
use std::ffi::CStr;
use std::rc::Rc;
use std::cell::RefCell;
use lazy_static::lazy_static;

use gtk4::prelude::*;
use gtk4::Application;
use glib::ExitCode;
use gio::ApplicationFlags;

pub mod ui;
pub mod func;

use crate::plug::inter_data::DATA as INTER_DATA;


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
        AppType::EDITOR(_) => "EnCode.editor".to_string(),
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
    return APP.with(move |data| {
        let app = data.borrow();
        app.connect_activate(|app| {ui::build_ui(app);});
        app.connect_open(|application, _, _| {
            let mut ui_ret = ui::build_ui(application);
            INTER_DATA.with_borrow_mut(|data| {
                data.gui = &mut ui_ret;
                data.app = &mut application.clone();
                unsafe {crate::plug::funcs::before_showing_window(&mut *data);}
                ui_ret
                    .object::<gtk4::ApplicationWindow>(
                        &*unsafe{
                            CStr::from_ptr(data.gui_ids.app_window_id)
                                .to_string_lossy()
                        },
                    ).as_ref().map(gtk4::ApplicationWindow::present);
                unsafe {crate::plug::funcs::start(&mut *data);}
            });
        });
        return app.run();
    });
}
