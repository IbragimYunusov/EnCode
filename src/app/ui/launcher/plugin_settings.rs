use gtk4::{
    prelude::*,
    ApplicationWindow,
    Align,
    Box,
    CheckButton,
    Dialog,
    Orientation,
    ResponseType,
    ScrolledWindow,
};
use glib::{clone, GString};

pub fn f(parent: &ApplicationWindow)
{
    let dialog = Dialog::builder()
        .transient_for(parent)
        .title("Список Плагинов")
        .modal(true)
        .vexpand(true)
        .build();
    dialog.set_icon_name(parent.icon_name().as_ref().map(|g| g.as_str()));

    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(super::super::INNER_SPACING)
        .margin_top(super::super::OUTTER_SPACING)
        .margin_bottom(super::super::OUTTER_SPACING)
        .margin_start(super::super::OUTTER_SPACING)
        .margin_end(super::super::OUTTER_SPACING)
        .build();

    let scrolled_window = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .propagate_natural_height(true)
        .build();

    let vbox_scrolled = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(super::super::INNER_SPACING)
        .build();

    let plugins = crate::sql::general_data::Plugin::select_all();
    if let Err(e) = plugins {
        idl::show_error_dialog(&parent, e);
        return;
    }
    let plugins = plugins.unwrap();

    let mut errs = Vec::new();
    for plugin in plugins.iter().rev() {
        match plugin {
            Ok(plug) => {
                let check_button = CheckButton::builder()
                    .label(&plug.dev_name)
                    .active(plug.will_be_used)
                    .build();
                vbox_scrolled.append(&check_button);
            },
            Err(e) => errs.push(e),
        }
    }
    if errs.len() != 0 {
        idl::show_error_dialog(
            &parent,
            errs.iter().map(|e| e.to_string()).collect::<Vec<String>>().join("\n"),
        );
    }

    scrolled_window.set_child(Some(&vbox_scrolled));

    let buttons_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(super::super::SPACING_DELTA)
        .spacing(super::super::INNER_SPACING)
        .halign(Align::End)
        .valign(Align::End)
        .build();
    let cancel_button = dialog.add_button("Отмена", ResponseType::Cancel);
    cancel_button.unparent();
    let ok_button = dialog.add_button("Сохранить", ResponseType::Accept);
    ok_button.unparent();
    dialog.connect_response(clone!(
        #[weak] vbox_scrolled,
        #[weak] parent,
        move |dialog, response| {
            if response == ResponseType::Accept {
                let selected_dev_names: Vec<GString> = vbox_scrolled.observe_children()
                    .into_iter()
                    .filter_map(|c| c.ok()?.downcast_ref::<CheckButton>().and_then(CheckButton::label))
                    .collect();
                if let Err(e) = super::f_launcher::dependencies_check(
                    selected_dev_names.iter().map(GString::as_str).collect::<Vec<&str>>(),
                ) {
                    idl::show_error_dialog(&parent, e);
                    return;
                }
                for child in vbox_scrolled.observe_children().into_iter() {
                    match child {
                        Ok(c) => if let Some(check_button) = c.downcast_ref::<CheckButton>() {
                            if let Some(dev_name) = check_button.label() {
                                if let Err(e) = crate::sql::general_data::Plugin::set_will_be_used_from_dev_name(
                                    dev_name,
                                    check_button.is_active(),
                                ) {
                                    idl::show_error_dialog(&parent, e);
                                }
                            }
                        },
                        Err(e) => idl::show_error_dialog(&parent, e),
                    }
                }
            }
            dialog.close();
        },
    ));

    buttons_box.append(&cancel_button);
    buttons_box.append(&ok_button);

    vbox.append(&scrolled_window);
    vbox.append(&buttons_box);

    dialog.set_child(Some(&vbox));
    dialog.present();
}
