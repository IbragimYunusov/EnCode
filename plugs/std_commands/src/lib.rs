use std::ffi::c_char;
use std::ptr::null;

use std::fs;

use gtk4::prelude::*;
use gtk4::glib::object::Cast;

use idl::{get_gui_el, get_attr};
type Res<T> = Result<T, Box<dyn std::error::Error>>;


#[no_mangle]
pub extern "C" fn save_cur_file(data: idl::Data) -> *const c_char
{
    if let Err(e) = (|| -> Res<()> {
        let notebook: gtk4::Notebook = get_attr!(
            [get_gui_el!(data.gui, data.gui_ids.notebook_id)]?
        );
        let cur_page = notebook
            .nth_page(notebook.current_page())
            .ok_or("Не удалось получить содержимое текущей открытой вкладки")?;
        let hbox = get_attr!(cur_page.downcast_ref::<gtk4::Box>());
        let view = get_attr!(hbox.first_child());
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
        buf.set_modified(true);
        return Ok(());
    })() {e.to_string().as_ptr() as *const c_char} else {null()}
}


/*
#[no_mangle]
pub extern "C" fn save_all_files(data: idl::Data) -> Res<()>
{
    let notebook = get_attr!(data.gui.as_ref.notebook);
    for i in 0..notebook.n_pages() {
        let cur_page = &notebook
            .nth_page(Some(i))
            .ok_or("Не удалось получить содержимое текущей открытой вкладки")?;
        let hbox = cur_page.downcast_ref::<gtk4::Box>();
        let view = get_attr!(hbox.as_ref.first_child)
            .downcast::<sourceview5::View>();
        let buf = get_attr!(view.ok).buffer();
        if !buf.is_modified() {
            continue;
        }
        let cur_tab_label = notebook
            .tab_label(cur_page)
            .ok_or("Не удалось получить название текущей открытой вкладки")?
            .downcast::<gtk4::Box>();
        let cur_tab_label = get_attr!(cur_tab_label.ok.first_child)
            .downcast::<gtk4::Label>();
        let cur_tab_label = get_attr!(cur_tab_label.ok).label();
        let file = std::env::current_dir()?.join(&cur_tab_label);
        fs::write(&file, buf.text(&buf.start_iter(), &buf.end_iter(), true))?;
        buf.set_modified(true);
    }
    return Ok(());
}


macro_rules! std_dialog_with_entries {
    {
        $func:ident, $title:expr => [$((
            $label:expr,
            $text:expr
            $(,($button_label:expr, $button_f:expr))*$(,)?
        )),+$(,)?] => $func_response:expr
    } => {
        #[no_mangle]
        pub extern "C" fn $func(data: idl::Data) -> Res<()> {

            let dialog = gtk4::Dialog::builder()
                .transient_for(get_attr!(data.gui.as_ref.window))
                .title($title)
                .modal(true)
                .build();

            let vbox = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(*get_attr!(data.inner_spacing))
                .margin_top(*get_attr!(data.outter_spacing))
                .margin_bottom(*get_attr!(data.outter_spacing))
                .margin_start(*get_attr!(data.outter_spacing))
                .margin_end(*get_attr!(data.outter_spacing))
                .build();

            $(
                let hbox = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(*get_attr!(data.inner_spacing))
                    .build();

                let label = gtk4::Label::builder()
                    .label($label)
                    .build();

                let entry = gtk4::Entry::builder()
                    .text($text)
                    .hexpand(true)
                    .build();

                hbox.append(&label);
                hbox.append(&entry);

                $(
                    let button = gtk4::Button::builder()
                        .label($button_label)
                        .build();
                    button.connect_clicked($button_f);
                    hbox.append(&button);
                )*

                vbox.append(&hbox);
            )*

            dialog.connect_response($func_response);

            let button_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .margin_top(*get_attr!(data.spacing_delta))
                .spacing(*get_attr!(data.inner_spacing))
                .halign(gtk4::Align::End)
                .valign(gtk4::Align::End)
                .build();

            let cancel_button = dialog.add_button("Отмена", gtk4::ResponseType::Cancel);
            cancel_button.unparent();
            button_box.append(&cancel_button);

            let ok_button = dialog.add_button("Ок", gtk4::ResponseType::Accept);
            ok_button.unparent();
            button_box.append(&ok_button);

            vbox.append(&button_box);
            dialog.set_child(Some(&vbox));

            Ok(())
        }
    };
    {
        $($func:ident, $title:expr => [$((
            $label:expr,
            $text:expr
            $(,($button_label:expr, $button_f:expr))*$(,)?
        )),+$(,)?] => $func_response:expr);+$(;)?
    } => {
        $(
            std_dialog_with_entries! {
                $func, $title => [
                    $(
                        (
                            $label,
                            $text
                            $(,($button_label, $button_f))*
                        )
                    ),+
                ] => $func_response
            }
        )*
    }
}


std_dialog_with_entries! {
    new_file, "Новый Файл" => [
        ("Название:\t", ""),
        (
            "Расположение:\t",
            &*std::env::current_dir()?.to_string_lossy(),
            (
                "📂",
                {
                    move |button| {|| -> Option<_> {
                        let dialog = button
                            .parent()?
                            .parent()?
                            .parent()?
                            .downcast::<gtk4::Dialog>().ok()?;
                        let entry = button
                            .prev_sibling()?
                            .downcast::<gtk4::Entry>()
                            .ok()?;
                        let file_chooser = gtk4::FileChooserDialog::builder()
                            .title("Выбрать Директорию")
                            .action(gtk4::FileChooserAction::SelectFolder)
                            .transient_for(&dialog)
                            .modal(true)
                            .build();
                        file_chooser.add_buttons(&[
                            ("Отмена", gtk4::ResponseType::Cancel),
                            ("Выбрать", gtk4::ResponseType::Accept),
                        ]);

                        file_chooser.connect_response(
                            glib::clone!(
                                #[weak] entry,
                                move |file_dialog, response| {
                                    if response != gtk4::ResponseType::Accept {
                                        file_dialog.close();
                                        return;
                                    }
                                    if let Some(file) = file_dialog.file() {
                                        if let Some(path) = file.path() {
                                            entry.set_text(&*path.to_string_lossy());
                                        }
                                    }
                                    file_dialog.close();
                                }
                            ),
                        );
                        file_chooser.show();
                        return Some(())
                    }();}
                }
            ),
        ),
    ] => move |dialog, response| {|| -> Option<_> {
        if response != gtk4::ResponseType::Accept {
            dialog.close();
            return Some(());
        }
        let name_box = dialog
            .child()?
            .downcast_ref::<gtk4::Box>()?
            .first_child()?
            .downcast::<gtk4::Box>()
            .ok()?;
        let path_box = name_box
            .next_sibling()?
            .downcast::<gtk4::Box>()
            .ok()?;
        let name_entry = name_box
            .first_child()?
            .next_sibling()?
            .downcast::<gtk4::Entry>()
            .ok()?;
        let path_entry = path_box
            .first_child()?
            .next_sibling()?
            .downcast::<gtk4::Entry>()
            .ok()?;
        std::fs::create_dir_all(&path_entry.text()).ok()?;
        let _ = std::fs::File::create(
            std::path::PathBuf::from(&path_entry.text())
                .join(&name_entry.text()),
        );
        return Some(());
    }();};
    new_dir, "Новая Директория" => [
        ("Название:\t", ""),
        (
            "Расположение:\t",
            &*std::env::current_dir()?.to_string_lossy(),
            (
                "📂",
                {
                    move |button| {|| -> Option<_> {
                        let dialog = button
                            .parent()?
                            .parent()?
                            .parent()?
                            .downcast::<gtk4::Dialog>().ok()?;
                        let entry = button
                            .prev_sibling()?
                            .downcast::<gtk4::Entry>()
                            .ok()?;
                        let file_chooser = gtk4::FileChooserDialog::builder()
                            .title("Выбрать Директорию")
                            .action(gtk4::FileChooserAction::SelectFolder)
                            .transient_for(&dialog)
                            .modal(true)
                            .build();
                        file_chooser.add_buttons(&[
                            ("Отмена", gtk4::ResponseType::Cancel),
                            ("Выбрать", gtk4::ResponseType::Accept),
                        ]);

                        file_chooser.connect_response(
                            glib::clone!(
                                #[weak] entry,
                                move |file_dialog, response| {
                                    if response != gtk4::ResponseType::Accept {
                                        file_dialog.close();
                                        return;
                                    }
                                    if let Some(file) = file_dialog.file() {
                                        if let Some(path) = file.path() {
                                            entry.set_text(&*path.to_string_lossy());
                                        }
                                    }
                                    file_dialog.close();
                                }
                            ),
                        );
                        file_chooser.show();
                        return Some(())
                    }();}
                }
            ),
        ),
    ] => move |dialog, response| {|| -> Option<_> {
        if response != gtk4::ResponseType::Accept {
            dialog.close();
            return Some(());
        }
        let name_box = dialog
            .child()?
            .downcast_ref::<gtk4::Box>()?
            .first_child()?
            .downcast::<gtk4::Box>()
            .ok()?;
        let path_box = name_box
            .next_sibling()?
            .downcast::<gtk4::Box>()
            .ok()?;
        let name_entry = name_box
            .first_child()?
            .next_sibling()?
            .downcast::<gtk4::Entry>()
            .ok()?;
        let path_entry = path_box
            .first_child()?
            .next_sibling()?
            .downcast::<gtk4::Entry>()
            .ok()?;
        let _ = std::fs::create_dir_all(
            std::path::PathBuf::from(&path_entry.text())
                .join(&name_entry.text()),
        );
        return Some(());
    }();};
}
*/
