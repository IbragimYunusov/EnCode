use gtk4::{
    prelude::*,
    Align,
    ApplicationWindow,
    Box,
    Builder,
    Button,
    Orientation,
};
use glib::clone;

use std::ffi::CStr;

use crate::app::func::launcher as f_launcher;
use super::show_error_dialog;
use crate::plug::inter_data::DATA as INTER_DATA;


pub fn build_ui(window: ApplicationWindow) -> Builder
{
    let builder = Builder::new();
    window.set_title(Some("EnCode — Лаунчер"));
    window.set_default_width(800);
    window.set_default_height(500);
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.app_window_id).to_string_lossy()},
        ),
        &window,
    );
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
    return builder;
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
        Err(e) => show_error_dialog(&window, e),
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


pub mod new_project
{
    use gtk4::{
        prelude::*,
        Align,
        ApplicationWindow,
        Button,
        Box,
        CheckButton,
        Dialog,
        Entry,
        FileChooserAction,
        FileChooserDialog,
        Label,
        Orientation,
        ResponseType,
        Widget,
    };
    use glib::clone;
    use std::{env, fs};
    use std::error::Error;
    use std::boxed::Box as Box_;

    pub struct Ret {
        pub name: String,
        pub directory: String,
        pub use_env: bool,
        pub env_dir: String,
    }

    pub struct Np {
        pub dialog: Dialog,
        pub vbox: Box,
        pub name_box: Box,
        pub name_label: Label,
        pub name_entry: Entry,
        pub directory_box: Box,
        pub directory_label: Label,
        pub directory_entry: Entry,
        pub directory_button: Button,
        pub use_env_checkbutton: CheckButton,
        pub env_dir_box: Box,
        pub env_dir_label: Label,
        pub env_dir_entry: Entry,
        pub button_box: Box,
        pub cancel_button: Widget,
        pub ok_button: Widget,
    }

    pub fn f(parent: &ApplicationWindow, connect_response: bool) -> Result<Np, Box_<dyn Error>>
    {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .title("Создание Нового Проекта")
            .modal(true)
            .build();

        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(super::super::INNER_SPACING)
            .margin_top(super::super::OUTTER_SPACING)
            .margin_bottom(super::super::OUTTER_SPACING)
            .margin_start(super::super::OUTTER_SPACING)
            .margin_end(super::super::OUTTER_SPACING)
            .build();

        let name_label = Label::builder()
            .label("Название\t")
            .halign(Align::Start)
            .build();
        let name_entry = Entry::builder()
            .hexpand(true)
            .build();

        let name_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(super::super::INNER_SPACING)
            .build();
        name_box.append(&name_label);
        name_box.append(&name_entry);

        let directory_label = Label::builder()
            .label("Директория\t")
            .halign(Align::Start)
            .build();
        let directory_entry = Entry::builder()
            .hexpand(true)
            .build();
        if let Ok(exe) = env::current_exe(){
            if let Some(grand_default_dir) = exe.parent() {
                let default_dir = grand_default_dir.join("projects");
                if !default_dir.exists() {
                    if let Err(e) = fs::create_dir(&default_dir) {
                        super::super::show_error_dialog(&parent, e);
                    } else {
                        directory_entry.set_text(&*default_dir.to_string_lossy());
                    }
                } else {
                    directory_entry.set_text(&*default_dir.to_string_lossy());
                }
            }
        }
        let directory_button = Button::builder()
            .label("📂")
            .build();
        directory_button.connect_clicked(
            clone!(
                #[weak] parent,
                #[weak] directory_entry,
                move |_| directory_choose(&parent, &directory_entry),
            )
        );

        let directory_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(super::super::INNER_SPACING)
            .build();
        directory_box.append(&directory_label);
        directory_box.append(&directory_entry);
        directory_box.append(&directory_button);

        let use_env_checkbutton = CheckButton::builder()
            .halign(Align::Start)
            .label("Использовать виртуальное окружение")
            .build();

        let env_dir_label = Label::builder()
            .label("Директория виртуального окружения\t")
            .halign(Align::Start)
            .build();
        let env_dir_entry = Entry::builder()
            .hexpand(true)
            .text(".env")
            .build();

        let env_dir_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(super::super::INNER_SPACING)
            .build();
        env_dir_box.append(&env_dir_label);
        env_dir_box.append(&env_dir_entry);
        env_dir_box.set_sensitive(false);

        use_env_checkbutton.connect_toggled(
            clone!(
                #[weak] env_dir_box,
                move |cb| env_dir_box.set_sensitive(cb.is_active()),
            )
        );

        let button_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(super::super::SPACING_DELTA)
            .spacing(super::super::INNER_SPACING)
            .halign(Align::End)
            .valign(Align::End)
            .build();
        let cancel_button = dialog.add_button("Отмена", ResponseType::Cancel);
        cancel_button.unparent();
        button_box.append(&cancel_button);
        let ok_button = dialog.add_button("Ок", ResponseType::Accept);
        ok_button.unparent();
        button_box.append(&ok_button);

        vbox.append(&name_box);
        vbox.append(&directory_box);
        vbox.append(&use_env_checkbutton);
        vbox.append(&env_dir_box);
        vbox.append(&button_box);
        dialog.set_child(Some(&vbox));

        if connect_response {
            dialog.connect_response(
                clone!(
                    #[weak] parent,
                    #[weak] use_env_checkbutton,
                    #[weak] name_entry,
                    #[weak] directory_entry,
                    #[weak] env_dir_entry,
                    move |dialog, response| {
                        if response == ResponseType::Accept {
                            if let Err(e) = super::f_launcher::new_project(
                                Ret {
                                    name: name_entry.text().to_string(),
                                    directory: directory_entry.text().to_string(),
                                    use_env: use_env_checkbutton.is_active(),
                                    env_dir: env_dir_entry.text().to_string(),
                                },
                                true,
                            ) {
                                super::super::show_error_dialog(&parent, e);
                            } else {
                                dialog.close();
                                parent.destroy();
                            }
                        } else {
                            dialog.close();
                        }
                    },
                ),
            );
        }
        dialog.show();
        return Ok(
            Np{
                dialog,
                vbox,
                name_box,
                name_label,
                name_entry,
                directory_box,
                directory_label,
                directory_entry,
                directory_button,
                use_env_checkbutton,
                env_dir_box,
                env_dir_label,
                env_dir_entry,
                button_box,
                cancel_button,
                ok_button,
            },
        );
    }

    fn directory_choose(parent: &ApplicationWindow, directory_entry: &Entry)
    {
        let file_chooser = FileChooserDialog::builder()
            .title("Выбрать Директорию")
            .action(FileChooserAction::SelectFolder)
            .transient_for(parent)
            .modal(true)
            .build();

        file_chooser.add_buttons(
            &[
                ("Отмена", ResponseType::Cancel),
                ("Выбрать", ResponseType::Accept),
            ],
        );

        file_chooser.connect_response(
            clone!(
                #[weak] directory_entry,
                move |dialog, response| {
                    if response == ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                directory_entry.set_text(&*(path.to_string_lossy()));
                            }
                        }
                    }
                    dialog.close();
                },
            ),
        );

        file_chooser.show();
    }
}


pub mod open_project
{
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
            super::super::show_error_dialog(&parent, &e);
            return Err(Box_::new(e));
        }

        let dialog = Dialog::builder()
            .transient_for(parent)
            .title("Открытие Проекта")
            .modal(true)
            .vexpand(true)
            .build();

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
                super::super::show_error_dialog(&parent, e);
                continue;
            }
            match project.as_ref() {
                Ok(ref proj) => list_box.append(&new_row(proj)),
                Err(e) => super::super::show_error_dialog(&parent, e),
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
        let edit_button = Button::builder()
            .label("✏️")
            .sensitive(false)
            .build();
        let delete_button = Button::builder()
            .label("🗑️")
            .sensitive(false)
            .build();
        delete_button.connect_clicked(
            clone!(
                #[weak] parent,
                #[weak] list_box,
                move |_| {
                    if let Some(row) = list_box.selected_row() {
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
                            super::super::show_error_dialog(&parent, e);
                            return;
                        }
                        let id = id.unwrap();
                        let project = crate::sql::general_data::Project::get(id);
                        if let Err(e) = project {
                            super::super::show_error_dialog(&parent, e);
                            return;
                        }
                        let project = project.unwrap();
                        if let Err(e) = std::fs::remove_dir(
                            &*std::path::PathBuf::from(&project.dir).join(&project.dev_name),
                        ) {
                            super::super::show_error_dialog(&parent, e);
                            return;
                        }
                        if let Err(e) = project.remove() {
                            super::super::show_error_dialog(&parent, e);
                            return;
                        }
                        list_box.remove(&row);
                    }
                },
            ),
        );

        let new_button = Button::builder()
            .label("➕")
            .build();
        new_button.connect_clicked(
            clone!(
                #[weak] parent,
                #[weak] list_box,
                move |_| {
                    if let Ok(np) = super::new_project::f(&parent, false) {
                        np.dialog.connect_response(
                            clone!(
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
                                        super::super::show_error_dialog(&parent, e);
                                        return;
                                    }
                                    list_box.insert(
                                        &new_row(
                                            &crate::sql::general_data::Project{
                                                id: match crate::sql::general_data::Project::get_max_id() {
                                                    Ok(id) => id + 1,
                                                    Err(e) => {
                                                        super::super::show_error_dialog(&parent, e);
                                                        return;
                                                    },
                                                },
                                                dev_name: name,
                                                dir: directory,
                                            },
                                        ),
                                        0,
                                    );
                                    np_dialog.close();
                                },
                            )
                        );
                    }
                },
            ),
        );

        let from_dir_button = Button::builder()
            .margin_start(super::super::SPACING_DELTA)
            .label("📂")
            .build();
        from_dir_button.connect_clicked(
            clone!(
                #[weak] parent,
                #[weak] list_box,
                move |_| {
                    let file_chooser = FileChooserDialog::builder()
                        .title("Выбрать Директорию")
                        .action(FileChooserAction::SelectFolder)
                        .transient_for(&parent)
                        .modal(true)
                        .build();

                    file_chooser.add_buttons(
                        &[
                            ("Отмена", ResponseType::Cancel),
                            ("Выбрать", ResponseType::Accept),
                        ],
                    );

                    file_chooser.connect_response(
                        clone!(
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
                                                    super::super::show_error_dialog(
                                                        &parent,
                                                        "Не удалось получить родительскую директорию",
                                                    );
                                                    dialog.close();
                                                    return;
                                                },
                                                dev_name: if let Some(name) = path.file_name() {
                                                    name.to_string_lossy().to_string()
                                                } else {
                                                    super::super::show_error_dialog(
                                                        &parent,
                                                        "Не удалось получить саму директорию",
                                                    );
                                                    dialog.close();
                                                    return;
                                                },
                                            };
                                            if let Err(e) = project.insert() {
                                                super::super::show_error_dialog(&parent, e);
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
                        ),
                    );

                    file_chooser.show();
                },
            )
        );

        list_box.connect_row_activated(
            clone!(
                #[weak] ok_button,
                #[weak] edit_button,
                #[weak] delete_button,
                move |_, _| {
                    ok_button.set_sensitive(true);
                    edit_button.set_sensitive(true);
                    delete_button.set_sensitive(true);
                },
            ),
        );
        tool_box.append(&edit_button);
        tool_box.append(&delete_button);
        tool_box.append(&from_dir_button);
        tool_box.append(&new_button);

        vbox.append(&tool_box);
        vbox.append(&list_box);
        vbox.append(&buttons_box);
        dialog.set_child(Some(&vbox));

        if connect_response {
            dialog.connect_response(
                clone!(
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
                                        super::super::show_error_dialog(&parent, e);
                                        return;
                                    }
                                    let id = id.unwrap();
                                    let r = crate::sql::general_data::Project::get(id);
                                    if let Err(e) = r {
                                        super::super::show_error_dialog(&parent, e);
                                        return;
                                    } else {r.unwrap()}
                                }
                            ) {
                                super::super::show_error_dialog(&parent, e);
                            } else {
                                dialog.close();
                                parent.destroy();
                            }
                        } else {
                            dialog.close();
                        }
                    },
                ),
            );
        }
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
}


pub mod plugin_settings
{
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
            super::super::show_error_dialog(&parent, e);
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
            super::super::show_error_dialog(
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
        dialog.connect_response(
            clone!(
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
                            super::super::show_error_dialog(&parent, e);
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
                                            super::super::show_error_dialog(&parent, e);
                                        }
                                    }
                                },
                                Err(e) => super::super::show_error_dialog(&parent, e),
                            }
                        }
                    }
                    dialog.close();
                },
            ),
        );

        buttons_box.append(&cancel_button);
        buttons_box.append(&ok_button);

        vbox.append(&scrolled_window);
        vbox.append(&buttons_box);

        dialog.set_child(Some(&vbox));
        dialog.present();
    }
}
