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
    dialog.set_icon_name(parent.icon_name().as_ref().map(|g| g.as_str()));

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
                    idl::show_error_dialog(&parent, e);
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
    directory_button.connect_clicked(clone!(
        #[weak] parent,
        #[weak] directory_entry,
        move |_| directory_choose(&parent, &directory_entry),
    ));

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

    use_env_checkbutton.connect_toggled(clone!(
        #[weak] env_dir_box,
        move |cb| env_dir_box.set_sensitive(cb.is_active()),
    ));

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

    if connect_response {dialog.connect_response(clone!(
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
    return Ok(Np{
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
    });
}

fn directory_choose(parent: &ApplicationWindow, directory_entry: &Entry)
{
    let file_chooser = FileChooserDialog::builder()
        .title("Выбрать Директорию")
        .action(FileChooserAction::SelectFolder)
        .transient_for(parent)
        .modal(true)
        .build();
    file_chooser.add_buttons(&[
        ("Отмена", ResponseType::Cancel),
        ("Выбрать", ResponseType::Accept),
    ]);
    file_chooser.connect_response(clone!(
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
    ));
    file_chooser.show();
}
