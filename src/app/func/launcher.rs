use std::error::Error;
use std::io::Write;

use gtk4::prelude::*;

use std::env;
use std::process::Command;

use std::path::PathBuf;
use std::fs;

use crate::app::ui::launcher as ui_launcher;
use crate::sql::general_data::{Plugin, Project};


pub fn new_project(
    ret: ui_launcher::new_project::Ret,
    v_open_project: bool,
) -> Result<(), Box<dyn Error>>
{
    fs::create_dir(PathBuf::from(&ret.directory).join(&ret.name))?;
    if ret.use_env {
        Command::new("python").args(&[
            "-m",
            "venv",
            &*PathBuf::from(&ret.directory)
                .join(&ret.name)
                .join(&ret.env_dir)
                .to_string_lossy(),
        ]).spawn()?;
    }
    let project = Project{
        id: Project::get_max_id().unwrap_or(0) + 1,
        dir: ret.directory,
        dev_name: ret.name,
    };
    project.insert()?;
    return if v_open_project {open_project(project)} else {Ok(())};
}


pub fn open_project(project: Project) -> Result<(), Box<dyn Error>>
{
    let _ = Command::new(env::current_exe()?)
        .args(&[PathBuf::from(&project.dir).join(&project.dev_name).as_os_str()])
        .spawn()?
        .id();
    return Ok(());
}


pub fn get_full_format_logo(height_request: i32) -> Result<gtk4::Image, Box<dyn Error>>
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
    let image = gtk4::Image::builder()
        .height_request(height_request)
        .valign(gtk4::Align::Center)
        .build();
    image.set_from_pixbuf(Some(&pixbuf));
    return Ok(image);
}


pub fn dependencies_check(selected: Vec<&str>) -> Result<(), Box<dyn Error>>
{
    use std::collections::HashSet;
    let dependencies: HashSet<String> = HashSet::from_iter(
        Plugin::select_all()?
            .iter()
            .filter_map(|it| it.as_ref().ok().map(|plugin| &plugin.dependencies))
            .flat_map(|deps| deps.iter())
            .cloned(),
    );
    let selected_set: HashSet<String> = selected.iter().map(|&s| s.to_string()).collect();
    if dependencies.is_subset(&selected_set) {
        return Ok(());
    }
    return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Кажется, вы не учли зависимости плагинов",
    )));
}
