use gtk4::prelude::*;
use gtk4::gdk_pixbuf::prelude::*;
use std::io::Write;

use crate::app::ui::{
    FILE_ICON,
    DIR_ICON,
    SYMLINK_ICON,
    UNKNOWN_ICON,
};


pub fn load_directory(
    store: &gtk4::TreeStore,
    parent: Option<gtk4::TreeIter>,
    path: &std::path::Path,
)
{
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().into_string().unwrap_or_default();
            let iter = store.append(parent.as_ref());
            let file_type = entry.file_type();

            if let Ok(ft) = file_type {
                if ft.is_file() {&FILE_ICON}
                else if ft.is_dir() {&DIR_ICON}
                else if ft.is_symlink() {&SYMLINK_ICON}
                else {&UNKNOWN_ICON}
            } else {&UNKNOWN_ICON}.with(|data| {
                if let Some(pixbuf) = data.as_ref() {
                    store.set(&iter, &[(0, pixbuf)]);
                }
            });
            store.set(&iter, &[(2, &name)]);
            if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                load_directory(store, Some(iter), &entry.path());
            }
        }
    }
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
    let hex_fg_color = idl::get_hex_fg_color();
    let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
    loader.write(std::fs::read_to_string({
        let cached_logo_dir = dir
            .join("cache")
            .join("static")
            .join("img")
            .join(name);
        let cached_logo = cached_logo_dir.join(&hex_fg_color);
        if !cached_logo.exists() {
            std::fs::create_dir_all(&cached_logo_dir)?;
            let svg = std::fs::read_to_string(
                dir
                    .join("static")
                    .join("img")
                    .join(name),
            )?;
            std::fs::File::create(&cached_logo)?.write_all(
                svg.replace(
                    "\"black\"",
                    &format!("\"#{}\"", &hex_fg_color),
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
    let pixbuf = idl::get_attr!(pixbuf.scale_simple(
        14,
        14,
        gtk4::gdk_pixbuf::InterpType::Nearest,
    ));
    return Ok(pixbuf);
}
