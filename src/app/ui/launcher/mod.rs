use gtk4::{
    prelude::*,
    Align,
    ApplicationWindow,
    Box,
    Button,
    Orientation,
};
use glib::clone;

use crate::app::func::launcher as f_launcher;

pub mod new_project;
pub mod open_project;
pub mod plugin_settings;


pub fn build_ui(window: ApplicationWindow) -> ApplicationWindow
{
    window.set_title(Some("EnCode — Лаунчер"));
    window.set_default_width(1000);
    window.set_default_height(600);
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(super::INNER_SPACING)
        .margin_end(super::OUTTER_SPACING)
        .margin_top(super::OUTTER_SPACING)
        .margin_start(super::OUTTER_SPACING)
        .margin_bottom(super::OUTTER_SPACING)
        .build();
    create_image(&window, &vbox);
    create_buttons(&window, &vbox);
    window.set_child(Some(&vbox));
    return window;
}


fn create_image(window: &ApplicationWindow, vbox: &Box)
{
    match f_launcher::get_full_format_logo(300) {
        Ok(image) => {
            let image_box = Box::builder()
                .orientation(Orientation::Vertical)
                .vexpand(true)
                .build();
            let spacer_top = Box::builder()
                .orientation(Orientation::Vertical)
                .vexpand(true)
                .build();
            image_box.append(&spacer_top);
            image_box.append(&image);
            let spacer_bottom = Box::builder()
                .orientation(Orientation::Vertical)
                .vexpand(true)
                .build();
            image_box.append(&spacer_bottom);
            vbox.append(&image_box);
        },
        Err(e) => idl::show_error_dialog(&window, e),
    }
}


fn create_buttons(window: &ApplicationWindow, vbox: &Box)
{
    let button_box = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::End)
        .spacing(vbox.spacing())
        .build();

    let new_project_button = Button::builder()
        .label("Новый Проект")
        .build();
    new_project_button.connect_clicked(
        clone!(
            #[weak] window,
            move |_| {
                let _ = new_project::f(&window, true);
            },
        ),
    );

    let open_project_button = Button::builder()
        .label("Открыть Проект")
        .build();
    open_project_button.connect_clicked(
        clone!(
            #[weak] window,
            move |_| {
                let _ = open_project::f(&window, true);
            },
        ),
    );

    let plugin_settings_button = Button::builder()
        .label("Список Плагинов")
        .build();
    plugin_settings_button.connect_clicked(
        clone!(
            #[weak] window,
            move |_| plugin_settings::f(&window),
        ),
    );

    button_box.append(&new_project_button);
    button_box.append(&open_project_button);
    button_box.append(&plugin_settings_button);
    vbox.append(&button_box);
}
