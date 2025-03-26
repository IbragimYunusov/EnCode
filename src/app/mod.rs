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

use crate::plug::inter_data::DATA as INTER_DATA;


pub enum AppType {
    LAUNCHER,
    EDITOR(PathBuf),
}

pub fn define_app_type() -> AppType {
    return match env::args().collect::<Vec<String>>().as_slice() {
        [_] => AppType::LAUNCHER,
        [_, dir] => AppType::EDITOR(PathBuf::from(dir)),
        _ => panic!("Неверные аргументы..."),
    };
}

pub fn define_app_id() -> String {
    return match *APP_TYPE {
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
        app.connect_activate(|app| {
            ui::build_ui(app).window().present();
        });
        app.connect_open(|application, _, _| {
            let ui_ret = ui::build_ui(application);
            INTER_DATA.with_borrow_mut(|data| {
                if let AppType::EDITOR(ref dir) = *APP_TYPE {
                    let _ = std::env::set_current_dir(dir);
                }
                data.gui = ui_ret.as_gui();
                data.app = Some(application.clone());
                unsafe {
                    crate::plug::funcs::before_showing_window(&mut *data);
                }
                println!("fdjhfgjhkjhkfghjkhjk");
                data.gui.as_ref().map(|g| g.window.present());
                println!("dkfjsdklfj");
                unsafe {
                    crate::plug::funcs::after_showing_window(&mut *data);
                }
            });
        });
        println!("app.run()");
        let ret = app.run();
        if let AppType::EDITOR(_) = *APP_TYPE {
            INTER_DATA.with_borrow_mut(|data| unsafe {
                crate::plug::funcs::end(&mut *data);
            });
        }
        return ret;
    });
}
