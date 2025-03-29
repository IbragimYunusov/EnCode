use gtk4::{
    prelude::*,
    gdk_pixbuf::Pixbuf,
    TreeIter,
    TreePath,
    TreeStore,
    TreeView,
};
use pyright::*;
use sourceview5::prelude::*;

use idl::{get_gui_el, get_attr};

use std::ffi::OsStr;
use std::process::Command;
use std::path::PathBuf;
use std::collections::HashMap;


thread_local!{
    pub static ERROR_STATUS: Option<Pixbuf>
        = get_status_icon("pyright_error.svg").ok();
    pub static WARNING_STATUS: Option<Pixbuf>
        = get_status_icon("pyright_warning.svg").ok();
    pub static INFORMATION_STATUS: Option<Pixbuf>
        = get_status_icon("pyright_information.svg").ok();
}


pub fn get_status_icon(name: &str) -> idl::Res<Pixbuf>
{
    let exe_path = std::env::current_exe()?;
    let dir = get_attr!(exe_path.parent());
    let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
    loader.write(std::fs::read_to_string(
        dir
            .join("static")
            .join("img")
            .join(name)
    )?.as_bytes())?;
    loader.close()?;
    let pixbuf = get_attr!(loader.pixbuf());
    let pixbuf = get_attr!(pixbuf.scale_simple(
        14,
        14,
        gtk4::gdk_pixbuf::InterpType::Nearest,
    ));
    return Ok(pixbuf);
}


#[unsafe(no_mangle)]
pub extern "C" fn before_showing_window(data: idl::Data) -> idl::Ret
{
    unsafe{gtk4::set_initialized();}
    Box::new(|| -> idl::Res<()> {
        let summary_label = gtk4::Label::builder()
            .label("Ok")
            .halign(gtk4::Align::Start)
            .margin_top(unsafe{(*data).inner_spacing})
            .margin_bottom(unsafe{(*data).inner_spacing})
            .margin_start(unsafe{(*data).outter_spacing})
            .margin_end(unsafe{(*data).outter_spacing})
            .build();
        get_gui_el!(data.gui.tree_view_vbox).append(&summary_label);
        let action_group = gio::SimpleActionGroup::new();
        action_group.add_action_entries([
            gio::ActionEntry::builder("diagnostic__project")
                .activate(move |_, _, _| if let Err(e) = diagnose_project(&summary_label, data) {
                    if let Some(gui) = unsafe{(*data).gui.as_ref()} {
                        idl::show_error_dialog(&gui.window, e);
                    }
                })
                .build(),
        ]);
        get_gui_el!(data.gui.window).insert_action_group(
            "pyright",
            Some(&action_group),
        );
        return Ok(());
    }().err().map(|e| e.to_string()))
}


pub fn diagnose_project(
    summary_label: &gtk4::Label,
    data: idl::Data,
) -> idl::Res<()> {
    let output = Command::new("pyright")
        .args(["-p", ".", "--outputjson"])
        .output()?;
    let output_string = &String::from_utf8(output.stdout)?;
    let pyright_output: PyrightOutput = serde_json::from_str(output_string)?;
    let _ = load_diag(
        get_gui_el!(data.gui.store),
        &pyright_output.general_diagnostics,
    );
    summary_label.set_text(&{
        let mut b = vec![];
        if pyright_output.summary.error_count != 0 {
            b.push(format!("❌ {}", pyright_output.summary.error_count));
        }
        if pyright_output.summary.warning_count != 0 {
            b.push(format!("⚠️ {}", pyright_output.summary.warning_count));
        }
        if pyright_output.summary.information_count != 0 {
            b.push(format!("ℹ️ {}", pyright_output.summary.information_count));
        }
        let s = b.join("  ");
        if s.len() == 0 {"Ok".to_string()} else {s}
    });
    // get_gui_el!(data.gui.window).queue_draw();
    return Ok(());
}


trait Importance {
    fn importance(&self) -> u8;
}
impl Importance for Severity {
    fn importance(&self) -> u8 {
        match self {
            Self::Error => 3,
            Self::Warning => 2,
            Self::Information => 1,
        }
    }
}
impl Importance for Option<Severity> {
    fn importance(&self) -> u8 {
        return self.as_ref().map_or(0, Importance::importance);
    }
}


pub fn load_diag(store: &TreeStore, diags: &Vec<Diagnostic>) -> idl::Res<()>
{
    let mut map: HashMap<PathBuf, u8> = HashMap::new();
    let mut max_depth: usize = 0;
    let root = std::env::current_dir()?;
    let root = root.as_path();
    for diag in diags {
        map.insert(
            diag.file.clone(),
            if let Some(other_importance) = map.get(&diag.file) {
                diag.severity.importance().max(*other_importance)
            } else {
                diag.severity.importance()
            },
        );
        max_depth = max_depth.max(
            diag
                .file
                .strip_prefix(root)
                .map(|p| p.iter().count())
                .unwrap_or(0)
        );
    }
    for (path, importance) in map.clone() {
        let mut parent = path.parent();
        while let Some(parent_) = parent {
            map.insert(
                parent_.to_path_buf(),
                importance.max(*map.get(parent_).unwrap_or(&0)),
            );
            parent = parent_.parent();
            if parent <= Some(root) {
                break;
            }
        }
    }
    println!("{:#?}", map);
    fn inner(
        map: &HashMap<PathBuf, u8>,
        store: &TreeStore,
        iter: Option<&TreeIter>,
        depth: usize,
        max_depth: usize,
    ) {
        if let Some(it) = iter {
            let path = store.path(&it);
            let mut path_end = PathBuf::new();
            let mut indices = Vec::new();
            for index in path.indices() {
                indices.push(index);
                if let Some(iter) = store.iter(&TreePath::from_indices(&indices)) {
                    let name: String = store.get::<String>(&iter, 2);
                    path_end.push(name);
                }
            }
            if let Ok(cwd) = std::env::current_dir() {
                println!("ya v {}", &*path_end.to_string_lossy());
                let ret = *map.get(&cwd.join(&path_end)).unwrap_or(&0);
                println!("{:?}", cwd.join(&path_end));
                match ret {
                    3 => ERROR_STATUS.with(|p| if let Some(pb) = p.as_ref() {
                        store.set_value(it, 1, &pb.to_value())
                    }),
                    2 => WARNING_STATUS.with(|p| if let Some(pb) = p.as_ref() {
                        store.set_value(it, 1, &pb.to_value())
                    }),
                    1 => INFORMATION_STATUS.with(|p| if let Some(pb) = p.as_ref() {
                        store.set_value(it, 1, &pb.to_value())
                    }),
                    _ => (),
                }
            }
        }
        if depth == max_depth {
            return;
        }
        if let Some(child_iter) = store.iter_children(iter) {
            inner(map, store, Some(&child_iter), depth + 1, max_depth);
            while store.iter_next(&child_iter) {
                inner(map, store, Some(&child_iter), depth + 1, max_depth);
            }
        }
    }
    inner(&map, store, None, 0, max_depth);
    return Ok(());
}
