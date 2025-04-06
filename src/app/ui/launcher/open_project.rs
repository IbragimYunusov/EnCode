use gtk4::{
    prelude::*,
    Align,
    ApplicationWindow,
    Box,
    Button,
    Dialog,
    FileChooserAction,
    FileChooserDialog,
    Label,
    ListBox,
    ListBoxRow,
    Orientation,
    ResponseType,
};
use glib::clone;
use std::error::Error;
use std::boxed::Box as Box_;

pub fn f(parent: &ApplicationWindow, connect_response: bool) -> Result<Dialog, Box_<dyn Error>>
{
    let projects = crate::sql::general_data::Project::select_all();
    if let Err(e) = projects {
        idl::show_error_dialog(&parent, &e);
        return Err(Box_::new(e));
    }

    let dialog = Dialog::builder()
        .transient_for(parent)
        .title("Открытие Проекта")
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

    let list_box = ListBox::new();

    for project in projects.unwrap().iter().rev() {
        if let Err(e) = project {
            idl::show_error_dialog(&parent, e);
            continue;
        }
        match project.as_ref() {
            Ok(ref proj) => list_box.append(&new_row(proj)),
            Err(e) => idl::show_error_dialog(&parent, e),
        }
    }

    let buttons_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(super::super::SPACING_DELTA)
        .spacing(super::super::INNER_SPACING)
        .halign(Align::End)
        .valign(Align::End)
        .build();
    let cancel_button = dialog.add_button("Отмена", ResponseType::Cancel);
    cancel_button.unparent();
    buttons_box.append(&cancel_button);
    let ok_button = dialog.add_button("Открыть", ResponseType::Accept);
    ok_button.unparent();
    ok_button.set_sensitive(false);
    buttons_box.append(&ok_button);

    let tool_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::End)
        .spacing(super::super::INNER_SPACING)
        .build();
    // let edit_button = Button::builder()
    //     .label("✏️")
    //     .sensitive(false)
    //     .build();
    let delete_button = Button::builder()
        .label("🗑️")
        .sensitive(false)
        .build();
    delete_button.connect_clicked(clone!(
        #[weak] parent,
        #[weak] list_box,
        move |_| {if let Some(row) = list_box.selected_row() {
            let id = if let Some(box_child) = row.child() {
                if let Some(box_) = box_child.downcast_ref::<Box>() {
                    if let Some(lbl_child) = box_.first_child(){
                        if let Some(lbl) = lbl_child.downcast_ref::<Label>() {
                            lbl.text().parse::<i32>()
                        } else {return}
                    } else {return}
                } else {return}
            } else {return};
            if let Err(e) = id {
                idl::show_error_dialog(&parent, e);
                return;
            }
            let id = id.unwrap();
            let project = crate::sql::general_data::Project::get(id);
            if let Err(e) = project {
                idl::show_error_dialog(&parent, e);
                return;
            }
            let project = project.unwrap();
            if let Err(e) = std::fs::remove_dir(
                &*std::path::PathBuf::from(&project.dir).join(&project.dev_name),
            ) {
                idl::show_error_dialog(&parent, e);
                return;
            }
            if let Err(e) = project.remove() {
                idl::show_error_dialog(&parent, e);
                return;
            }
            list_box.remove(&row);
        }},
    ));

    let new_button = Button::builder()
        .label("➕")
        .build();
    new_button.connect_clicked(clone!(
        #[weak] parent,
        #[weak] list_box,
        move |_| {if let Ok(np) = super::new_project::f(&parent, false) {
            np.dialog.connect_response(clone!(
                #[weak(rename_to=name_entry)] np.name_entry,
                #[weak(rename_to=directory_entry)] np.directory_entry,
                #[weak(rename_to=use_env_checkbutton)] np.use_env_checkbutton,
                #[weak(rename_to=env_dir_entry)] np.env_dir_entry,
                move |np_dialog, response| {
                    if response != ResponseType::Accept {
                        np_dialog.close();
                        return;
                    }
                    let ret = super::new_project::Ret{
                        name: name_entry.text().to_string(),
                        directory: directory_entry.text().to_string(),
                        use_env: use_env_checkbutton.is_active(),
                        env_dir: env_dir_entry.text().to_string(),
                    };
                    let name = ret.name.clone();
                    let directory = ret.directory.clone();
                    if let Err(e) = crate::app::func::launcher::new_project(ret, false) {
                        idl::show_error_dialog(&parent, e);
                        return;
                    }
                    list_box.insert(
                        &new_row(&crate::sql::general_data::Project{
                            id: match crate::sql::general_data::Project::get_max_id() {
                                Ok(id) => id + 1,
                                Err(e) => {
                                    idl::show_error_dialog(&parent, e);
                                    return;
                                },
                            },
                            dev_name: name,
                            dir: directory,
                        }),
                        0,
                    );
                    np_dialog.close();
                },
            ));
        }},
    ));

    let from_dir_button = Button::builder()
        .margin_start(super::super::SPACING_DELTA)
        .label("📂")
        .build();
    from_dir_button.connect_clicked(clone!(
        #[weak] parent,
        #[weak] list_box,
        move |_| {
            let file_chooser = FileChooserDialog::builder()
                .title("Выбрать Директорию")
                .action(FileChooserAction::SelectFolder)
                .transient_for(&parent)
                .modal(true)
                .build();
            file_chooser.add_buttons(&[
                ("Отмена", ResponseType::Cancel),
                ("Выбрать", ResponseType::Accept),
            ]);
            file_chooser.connect_response(clone!(
                #[weak] parent,
                move |dialog, response| {
                    if response == ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                let project = crate::sql::general_data::Project{
                                    id: crate::sql::general_data::Project::get_max_id().unwrap_or(0) + 1,
                                    dir: if let Some(grand_dir) = path.parent() {
                                        grand_dir.to_string_lossy().to_string()
                                    } else {
                                        idl::show_error_dialog(
                                            &parent,
                                            "Не удалось получить родительскую директорию",
                                        );
                                        dialog.close();
                                        return;
                                    },
                                    dev_name: if let Some(name) = path.file_name() {
                                        name.to_string_lossy().to_string()
                                    } else {
                                        idl::show_error_dialog(
                                            &parent,
                                            "Не удалось получить саму директорию",
                                        );
                                        dialog.close();
                                        return;
                                    },
                                };
                                if let Err(e) = project.insert() {
                                    idl::show_error_dialog(&parent, e);
                                    dialog.close();
                                    return;
                                };
                                let row = new_row(&project);
                                list_box.insert(&row, 0);
                                list_box.select_row(Some(&row));
                            }
                        }
                    }
                    dialog.close();
                },
            ));
            file_chooser.show();
        },
    ));

    list_box.connect_row_activated(clone!(
        #[weak] ok_button,
        // #[weak] edit_button,
        #[weak] delete_button,
        move |_, _| {
            ok_button.set_sensitive(true);
            // edit_button.set_sensitive(true);
            delete_button.set_sensitive(true);
        },
    ));
    // tool_box.append(&edit_button);
    tool_box.append(&delete_button);
    tool_box.append(&from_dir_button);
    tool_box.append(&new_button);

    vbox.append(&tool_box);
    vbox.append(&list_box);
    vbox.append(&buttons_box);
    dialog.set_child(Some(&vbox));

    if connect_response {dialog.connect_response(clone!(
        #[weak] parent,
        #[weak] list_box,
        move |dialog, response| {
            if response == ResponseType::Accept {
                if let Err(e) = super::f_launcher::open_project(
                    {
                        let id = if let Some(row) = list_box.selected_row() {
                            if let Some(box_child) = row.child() {
                                if let Some(box_) = box_child.downcast_ref::<Box>() {
                                    if let Some(lbl_child) = box_.first_child(){
                                        if let Some(lbl) = lbl_child.downcast_ref::<Label>() {
                                            lbl.text().parse::<i32>()
                                        } else {return}
                                    } else {return}
                                } else {return}
                            } else {return}
                        } else {return};
                        if let Err(e) = id {
                            idl::show_error_dialog(&parent, e);
                            return;
                        }
                        let id = id.unwrap();
                        let r = crate::sql::general_data::Project::get(id);
                        if let Err(e) = r {
                            idl::show_error_dialog(&parent, e);
                            return;
                        } else {r.unwrap()}
                    }
                ) {
                    idl::show_error_dialog(&parent, e);
                } else {
                    dialog.close();
                    parent.destroy();
                }
            } else {
                dialog.close();
            }
        },
    ));}
    dialog.show();
    return Ok(dialog);
}

pub fn new_row(proj: &crate::sql::general_data::Project) -> ListBoxRow {
    let row = ListBoxRow::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .build();
    let rbox = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::Fill)
        .margin_top(super::super::INNER_SPACING)
        .margin_bottom(super::super::INNER_SPACING)
        .margin_start(super::super::INNER_SPACING)
        .margin_end(super::super::INNER_SPACING)
        .spacing(super::super::INNER_SPACING)
        .build();
    rbox.append(&Label::builder().label(&proj.id.to_string()).opacity(0.5).build());
    rbox.append(&Label::new(Some(&proj.dev_name)));
    rbox.append(&Box::builder().hexpand(true).build());
    rbox.append(&Label::builder().label(&proj.dir).halign(Align::End).opacity(0.5).build());
    row.set_child(Some(&rbox));
    return row;
}
