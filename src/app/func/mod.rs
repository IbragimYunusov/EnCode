use std::io::Write;
use std::path::PathBuf;
use std::error::Error;

pub mod launcher;
pub mod editor;


pub fn get_dir() -> Option<PathBuf>
{
    return Some(std::env::current_exe().ok()?.parent()?.to_path_buf());
}


pub fn get_name_and_path_for_color_scheme() -> Result<(String, PathBuf), Box<dyn Error>>
{
    let exe_path = std::env::current_exe()?;
    let dir = exe_path.parent().ok_or(
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Не удалось получить директорию текущего исполняемого файла",
        ),
    )?;
    return Ok((
        if idl::is_current_theme_dark() {
            "encode-dark".to_string()
        } else {
            "Adwaita".to_string()
        },
        dir.join("static").join("color_schemes"),
    ));
}


pub fn get_name_and_path_for_search_icon() -> Result<(String, PathBuf), Box<dyn Error>>
{
    let exe_path = std::env::current_exe()?;
    let dir = exe_path.parent().ok_or(
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Не удалось получить директорию текущего исполняемого файла",
        ),
    )?;
    let hex_fg_color = idl::get_hex_fg_color();
    let ret_dir = dir
        .join("cache")
        .join("static")
        .join("img");
    let cached_logo_dir = ret_dir
        .join("hicolor")
        .join("48x48")
        .join("apps");
    let name = format!("encode-{}", &hex_fg_color);
    let cached_logo = cached_logo_dir.join(&format!("{}.svg", &name));
    if !cached_logo.exists() {
        std::fs::create_dir_all(&cached_logo_dir)?;
        let svg = std::fs::read_to_string(
            dir
                .join("static")
                .join("img")
                .join("hicolor")
                .join("48x48")
                .join("apps")
                .join("encode.svg"),
        )?;
        std::fs::File::create(&cached_logo)?.write_all(
            svg.replace(
                "fill=\"black\"",
                &format!("fill=\"#{}\"", &hex_fg_color),
            ).as_bytes(),
        )?;
    }
    return Ok((name, ret_dir));
}
