use std::{env, fs};
use std::io::Write;

use gtk4::prelude::*;
use gtk4::glib::object::Cast;

use idl::{get_attr, get_gui_el};


macro_rules! create_actions {
    {$window:expr, $data:expr => {$(
        $submenu_preffix:ident {$(
            $section_preffix:ident {$(
                $suffix:ident, $func:expr
            );*$(;)?}
        );*$(;)?}
    );*$(;)?}} => {
        let window = &$window;
        $(
            let action_group = gio::SimpleActionGroup::new();
            action_group.add_action_entries([$($(
                gio::ActionEntry::builder(&format!(
                    "{}__{}",
                    stringify!($section_preffix),
                    stringify!($suffix),
                )).activate(move |_, _, _| if let Some(e) = *$func($data) {
                    idl::show_error_dialog(window as &gtk4::ApplicationWindow, e);
                }).build(),
            )*)*]);
            window.insert_action_group(
                stringify!($submenu_preffix),
                Some(&action_group),
            );
        )*
    };
}


#[no_mangle]
pub extern "C" fn before_showing_window(data: idl::Data) -> idl::Ret
{
    unsafe{gtk4::set_initialized();}
    Box::new(|| -> idl::Res<()> {
        create_actions!{*get_gui_el!(data.gui.window), data => {
            project {
                file {
                    new, new_file;
                    save, save_cur_file;
                    save_all, save_all_files;
                };
                dir {
                    new, new_dir;
                };
            };
        }}
        return Ok(());
    }().err().map(|e| e.to_string()))
}


#[no_mangle]
pub extern "C" fn save_cur_file(data: idl::Data) -> idl::Ret
{
    Box::new(|| -> idl::Res<()> {
        let notebook = get_gui_el!(data.gui.notebook);
        let cur_page = notebook
            .nth_page(notebook.current_page())
            .ok_or("Не удалось получить содержимое текущей открытой вкладки")?;
        let vbox = get_attr!(cur_page.downcast_ref::<gtk4::Box>());
        let scrolled_window = get_attr!(vbox.first_child());
        let scrolled_window = get_attr!(
            scrolled_window.downcast_ref::<gtk4::ScrolledWindow>()
        );
        let view = get_attr!(scrolled_window.child());
        let view = get_attr!(view.downcast_ref::<sourceview5::View>());
        let buf = view.buffer();
        if !buf.is_modified() {
            return Ok(());
        }
        let cur_tab_label = notebook
            .tab_label(&cur_page)
            .ok_or("Не удалось получить название текущей открытой вкладки")?
            .downcast::<gtk4::Box>();
        let cur_tab_label = get_attr!(cur_tab_label.ok().first_child())
            .downcast::<gtk4::Label>();
        let cur_tab_label = get_attr!(cur_tab_label.ok()).label();
        let file = std::env::current_dir()?.join(&cur_tab_label);
        fs::write(&file, buf.text(&buf.start_iter(), &buf.end_iter(), true))?;
        buf.set_modified(false);
        return Ok(());
    }().err().map(|e| e.to_string()))
}


#[no_mangle]
pub extern "C" fn save_all_files(data: idl::Data) -> idl::Ret
{
    Box::new(|| -> idl::Res<()> {
        let notebook = get_gui_el!(data.gui.notebook);
        for i in 0..notebook.n_pages() {
            let notebook = get_gui_el!(data.gui.notebook);
            let cur_page = notebook
                .nth_page(Some(i))
                .ok_or("Не удалось получить содержимое текущей открытой вкладки")?;
            let vbox = get_attr!(cur_page.downcast_ref::<gtk4::Box>());
            let scrolled_window = get_attr!(vbox.first_child());
            let scrolled_window = get_attr!(
                scrolled_window.downcast_ref::<gtk4::ScrolledWindow>()
            );
            let view = get_attr!(scrolled_window.child());
            let view = get_attr!(view.downcast_ref::<sourceview5::View>());
            let buf = view.buffer();
            if !buf.is_modified() {
                continue;
            }
            let cur_tab_label = notebook
                .tab_label(&cur_page)
                .ok_or("Не удалось получить название текущей открытой вкладки")?
                .downcast::<gtk4::Box>();
            let cur_tab_label = get_attr!(cur_tab_label.ok().first_child())
                .downcast::<gtk4::Label>();
            let cur_tab_label = get_attr!(cur_tab_label.ok()).label();
            let file = std::env::current_dir()?.join(&cur_tab_label);
            fs::write(&file, buf.text(&buf.start_iter(), &buf.end_iter(), true))?;
            buf.set_modified(false);
        }
        return Ok(());
    }().err().map(|e| e.to_string()))
}


fn directory_choose(parent: &gtk4::Dialog, directory_entry: &gtk4::Entry)
{
    let file_chooser = gtk4::FileChooserDialog::builder()
        .title("Выбрать Директорию")
        .action(gtk4::FileChooserAction::SelectFolder)
        .transient_for(parent)
        .build();
    let _ = file_chooser.set_current_folder(
        std::env::current_dir()
            .ok()
            .map(gio::File::for_path)
            .as_ref(),
    );
    file_chooser.add_buttons(&[
        ("Отмена", gtk4::ResponseType::Cancel),
        ("Выбрать", gtk4::ResponseType::Accept),
    ]);
    file_chooser.connect_response(glib::clone!(
        #[weak] directory_entry,
        move |dialog, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        directory_entry.set_text(&*path.to_string_lossy());
                    }
                }
            }
            dialog.close();
        },
    ));
    file_chooser.present();
}


#[no_mangle]
pub extern "C" fn new_file(data: idl::Data) -> idl::Ret
{
    Box::new(|| -> idl::Res<()> {
        let dialog = gtk4::Dialog::builder()
            .transient_for(get_gui_el!(data.gui.window))
            .title("Новый Файл")
            .build();
        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(unsafe{(*data).inner_spacing})
            .margin_top(unsafe{(*data).outter_spacing})
            .margin_bottom(unsafe{(*data).outter_spacing})
            .margin_start(unsafe{(*data).outter_spacing})
            .margin_end(unsafe{(*data).outter_spacing})
            .build();

        let name_label = gtk4::Label::builder()
            .label("Название\t")
            .halign(gtk4::Align::Start)
            .build();
        let name_entry = gtk4::Entry::builder()
            .hexpand(true)
            .build();
        let name_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(unsafe{(*data).inner_spacing})
            .build();
        name_box.append(&name_label);
        name_box.append(&name_entry);

        let directory_label = gtk4::Label::builder()
            .label("Директория\t")
            .halign(gtk4::Align::Start)
            .build();
        let directory_entry = gtk4::Entry::builder()
            .hexpand(true)
            .build();
        if let Ok(default_dir) = env::current_dir(){
            directory_entry.set_text(&*default_dir.to_string_lossy());
        }
        let directory_button = gtk4::Button::builder()
            .label("📂")
            .build();
        directory_button.connect_clicked(glib::clone!(
            #[weak] dialog,
            #[weak] directory_entry,
            move |_| directory_choose(&dialog, &directory_entry),
        ));
        let directory_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(unsafe{(*data).inner_spacing})
            .build();
        directory_box.append(&directory_label);
        directory_box.append(&directory_entry);
        directory_box.append(&directory_button);

        let button_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_top(unsafe{(*data).spacing_delta})
            .spacing(unsafe{(*data).inner_spacing})
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::End)
            .build();
        let cancel_button = dialog.add_button("Отмена", gtk4::ResponseType::Cancel);
        cancel_button.unparent();
        button_box.append(&cancel_button);
        let ok_button = dialog.add_button("Открыть", gtk4::ResponseType::Accept);
        ok_button.unparent();
        button_box.append(&ok_button);

        vbox.append(&name_box);
        vbox.append(&directory_box);
        vbox.append(&button_box);

        dialog.connect_response(glib::clone!(
            #[weak] name_entry,
            #[weak] directory_entry,
            move |dialog, response| if let Err(e) = || -> idl::Res<()> {
                if response == gtk4::ResponseType::Accept {
                    let path = std::path::PathBuf::from(directory_entry.text())
                        .join(name_entry.text());
                    std::fs::File::create(path)?;
                    dialog.close();
                    if let Some(e) = *update_tree_view(data) {
                        Err(std::io::Error::other(e))?
                    }
                }
                return Ok(());
            }() {let _ = || -> idl::Res<()> {
                idl::show_error_dialog(get_gui_el!(data.gui.window), e);
                return Ok(());
            }();},
        ));
        dialog.set_child(Some(&vbox));
        dialog.present();
        return Ok(());
    }().err().map(|e| e.to_string()))
}


#[no_mangle]
pub extern "C" fn new_dir(data: idl::Data) -> idl::Ret
{
    Box::new(|| -> idl::Res<()> {
        let dialog = gtk4::Dialog::builder()
            .transient_for(get_gui_el!(data.gui.window))
            .title("Новая Директория")
            .build();
        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(unsafe{(*data).inner_spacing})
            .margin_top(unsafe{(*data).outter_spacing})
            .margin_bottom(unsafe{(*data).outter_spacing})
            .margin_start(unsafe{(*data).outter_spacing})
            .margin_end(unsafe{(*data).outter_spacing})
            .build();

        let name_label = gtk4::Label::builder()
            .label("Название\t")
            .halign(gtk4::Align::Start)
            .build();
        let name_entry = gtk4::Entry::builder()
            .hexpand(true)
            .build();
        let name_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(unsafe{(*data).inner_spacing})
            .build();
        name_box.append(&name_label);
        name_box.append(&name_entry);

        let directory_label = gtk4::Label::builder()
            .label("Директория\t")
            .halign(gtk4::Align::Start)
            .build();
        let directory_entry = gtk4::Entry::builder()
            .hexpand(true)
            .build();
        if let Ok(default_dir) = env::current_dir(){
            directory_entry.set_text(&*default_dir.to_string_lossy());
        }
        let directory_button = gtk4::Button::builder()
            .label("📂")
            .build();
        directory_button.connect_clicked(glib::clone!(
            #[weak] dialog,
            #[weak] directory_entry,
            move |_| directory_choose(&dialog, &directory_entry),
        ));
        let directory_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(unsafe{(*data).inner_spacing})
            .build();
        directory_box.append(&directory_label);
        directory_box.append(&directory_entry);
        directory_box.append(&directory_button);

        let button_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_top(unsafe{(*data).spacing_delta})
            .spacing(unsafe{(*data).inner_spacing})
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::End)
            .build();
        let cancel_button = dialog.add_button("Отмена", gtk4::ResponseType::Cancel);
        cancel_button.unparent();
        button_box.append(&cancel_button);
        let ok_button = dialog.add_button("Открыть", gtk4::ResponseType::Accept);
        ok_button.unparent();
        button_box.append(&ok_button);

        vbox.append(&name_box);
        vbox.append(&directory_box);
        vbox.append(&button_box);

        dialog.connect_response(glib::clone!(
            #[weak] name_entry,
            #[weak] directory_entry,
            move |dialog, response| if let Err(e) = || -> idl::Res<()> {
                if response == gtk4::ResponseType::Accept {
                    let path = std::path::PathBuf::from(directory_entry.text())
                        .join(name_entry.text());
                    std::fs::create_dir(path)?;
                    dialog.close();
                    if let Some(e) = *update_tree_view(data) {
                        Err(std::io::Error::other(e))?
                    }
                }
                return Ok(());
            }() {let _ = || -> idl::Res<()> {
                idl::show_error_dialog(get_gui_el!(data.gui.window), e);
                return Ok(());
            }();},
        ));
        dialog.set_child(Some(&vbox));
        dialog.present();
        return Ok(());
    }().err().map(|e| e.to_string()))
}


pub extern "C" fn update_tree_view(data: idl::Data) -> idl::Ret
{
    fn load_dir(
        data: idl::Data,
        store: &gtk4::TreeStore,
        parent: Option<gtk4::TreeIter>,
        path: &std::path::Path,
    ) -> idl::Res<()> {
        for entry in std::fs::read_dir(path)?.filter_map(Result::ok) {
            let name = entry.file_name().into_string().unwrap_or_default();
            let iter = store.append(parent.as_ref());
            store.set_value(&iter, 2, &name.to_value());
            let file_type = entry.file_type();
            if let Some(pixbuf) = if let Ok(ft) = file_type {
                if ft.is_file() {unsafe{&*(*data).tree_view_icons.file}}
                else if ft.is_dir() {unsafe{&*(*data).tree_view_icons.dir}}
                else if ft.is_symlink() {unsafe{&*(*data).tree_view_icons.symlink}}
                else {unsafe{&*(*data).tree_view_icons.unknown}}
            } else {unsafe{&*(*data).tree_view_icons.unknown}}.as_ref() {
                store.set(&iter, &[(0, pixbuf)]);
            }
            if file_type.map_or(false, |ft| ft.is_dir()) {
                load_dir(data, store, Some(iter), &entry.path())?;
            }
        }
        return Ok(());
    }
    Box::new(|| -> idl::Res<()> {
        get_gui_el!(data.gui.store).clear();
        load_dir(
            data,
            get_gui_el!(data.gui.store),
            None,
            std::env::current_dir()?.as_path(),
        )
    }().err().map(|e| e.to_string()))
}


pub fn get_hex_fg_color() -> String
{
    return if || -> Option<bool> {
        return Some(
            gtk4::Settings::default()?
                .gtk_theme_name()?
                .to_lowercase()
                .contains("dark"),
        );
    }().unwrap_or_default() {"acb4c1"} else {"ffffff"}.to_string();
}


pub fn get_icon(name: &str) -> idl::Res<gtk4::gdk_pixbuf::Pixbuf>
{
    let exe_path = std::env::current_exe()?;
    let dir = exe_path.parent().ok_or(
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Не удалось получить директорию текущего исполняемого файла",
        ),
    )?;
    let hex_fg_color = get_hex_fg_color();
    let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
    loader.write(std::fs::read_to_string({
        let cached_logo_dir = dir
            .join("cache")
            .join("static")
            .join("img")
            .join("full_format_logo.svg");
        let cached_logo = cached_logo_dir.join(&hex_fg_color);
        if !cached_logo.exists() {
            std::fs::create_dir_all(&cached_logo_dir)?;
            let svg = std::fs::read_to_string(
                dir
                    .join("static")
                    .join("img")
                    .join("full_format_logo.svg"),
            )?;
            std::fs::File::create(&cached_logo)?.write_all(
                svg.replace(
                    "fill=\"black\"",
                    &format!("fill=\"#{}\"", &hex_fg_color),
                ).as_bytes(),
            )?;
        }
        cached_logo
    })?.as_bytes())?;
    loader.close()?;
    let pixbuf = loader.pixbuf().ok_or(
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Не удалось получить объект Pixbuf",
        ),
    )?;
    return Ok(pixbuf);
}
