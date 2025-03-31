use gtk4::{
    prelude::*,
    gdk_pixbuf::Pixbuf,
    TreeIter,
    TreePath,
    TreeStore,
};
use pyright::*;

use idl::{get_gui_el, get_attr};

use std::env::current_dir;
use std::process::Command;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};


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
        7,
        14,
        gtk4::gdk_pixbuf::InterpType::Nearest,
    ));
    return Ok(pixbuf);
}


#[unsafe(no_mangle)]
pub extern "C" fn before_showing_window(data: idl::Data) -> idl::Ret
{
    unsafe{gtk4::set_initialized();}
    Box::new(|| -> idl::Res {
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
) -> idl::Res {
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
    add_highlighting_in_views(
        data,
        get_gui_el!(data.gui.notebook),
        pyright_output.general_diagnostics,
    )?;
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


pub fn load_diag(store: &TreeStore, diags: &Vec<Diagnostic>) -> idl::Res
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


#[derive(Debug, Clone)]
pub struct AlmostDiagnostic {
    pub severity: Sev,
    pub rule: Option<String>,
    pub message: String,
    pub range: Rng,
}
#[derive(Debug, Clone)]
pub enum Sev {
    Error,
    Warning,
    Information,
}
impl From<Severity> for Sev {
    fn from(s: Severity) -> Self {
        return match s {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Information => Self::Information,
        };
    }
}
#[derive(Debug, Clone)]
pub struct Rng {
    pub start: Pos,
    pub end: Pos,
}
impl From<Range> for Rng {
    fn from(r: Range) -> Self {
        return Self{
            start: Pos::from(r.start),
            end: Pos::from(r.end),
        };
    }
}
#[derive(Debug, Clone)]
pub struct Pos {
    pub line: u64,
    pub character: u64,
}
impl From<Position> for Pos {
    fn from(p: Position) -> Self {
        return Self{
            line: p.line,
            character: p.character,
        };
    }
}


pub fn add_highlighting_in_views(
    data: idl::Data,
    notebook: &gtk4::Notebook,
    diags: Vec<Diagnostic>,
) -> idl::Res {
    let mut map: Rc<RefCell<HashMap<PathBuf, Vec<AlmostDiagnostic>>>>
        = Rc::new(RefCell::new(HashMap::new()));
    for diag in diags {
        map
            .borrow_mut()
            .entry(diag.file)
            .or_insert_with(Vec::new)
            .push(AlmostDiagnostic{
                severity: Sev::from(diag.severity),
                rule: diag.rule,
                message: diag.message,
                range: Rng::from(diag.range),
            });
    }
    let modificated: Rc<RefCell<HashSet<String>>>
        = Rc::new(RefCell::new(HashSet::new()));
    let modificated_cloned = modificated.clone();
    let map_cloned = map.clone();
    notebook.connect_page_added(move |nb, page, _| add_hs_in_vs(
        nb,
        page,
        modificated_cloned.clone(),
        map_cloned.clone(),
        data,
    ));
    for n in 0..notebook.n_pages() {
        if let Some(page) = notebook.nth_page(Some(n)) {
            add_hs_in_vs(
                &notebook,
                &page,
                modificated.clone(),
                map.clone(),
                data,
            );
        }
    }
    return Ok(());
}


fn add_hs_in_vs(
    nb: &gtk4::Notebook,
    page: &gtk4::Widget,
    modificated: Rc<RefCell<HashSet<String>>>,
    map: Rc<RefCell<HashMap<PathBuf, Vec<AlmostDiagnostic>>>>,
    data: idl::Data,
) {
    if let Err(e) = || -> idl::Res {
        let s = get_attr!(nb.tab_label(page));
        let s = get_attr!(s.downcast_ref::<gtk4::Box>());
        let s = get_attr!(s.first_child());
        let s = get_attr!(s.downcast_ref::<gtk4::Label>());
        let s = s.label().to_string();
        println!("{:#?}", modificated);
        if modificated.borrow().contains(&s) {
            return Ok(());
        }
        let vbox = get_attr!(page.downcast_ref::<gtk4::Box>());
        let scrolled_window = get_attr!(vbox.first_child());
        let scrolled_window = get_attr!(
            scrolled_window.downcast_ref::<gtk4::ScrolledWindow>()
        );
        let view_widget = get_attr!(scrolled_window.child());
        let view: &sourceview5::View = get_attr!(view_widget.downcast_ref());
        let buffer = view.buffer();
        let buffer: &sourceview5::Buffer = get_attr!(buffer.downcast_ref());
        let err_tag = sourceview5::Tag::builder()
            .name("pyright_error")
            .underline(gtk4::pango::Underline::Error)
            .underline_rgba(&gtk4::gdk::RGBA::parse("#ff4d4d")?)
            .build();
        let warn_tag = sourceview5::Tag::builder()
            .name("pyright_warning")
            .underline(gtk4::pango::Underline::Error)
            .underline_rgba(&gtk4::gdk::RGBA::parse("#ffaa00")?)
            .build();
        let info_tag = sourceview5::Tag::builder()
            .name("pyright_information")
            .underline(gtk4::pango::Underline::Error)
            .underline_rgba(&gtk4::gdk::RGBA::parse("#4d88ff")?)
            .build();
        if let None = buffer
                .tag_table()
                .lookup(&err_tag.name().unwrap_or_default()) {
            buffer.tag_table().add(&err_tag);
        }
        if let None = buffer
                .tag_table()
                .lookup(&warn_tag.name().unwrap_or_default()) {
            buffer.tag_table().add(&warn_tag);
        }
        if let None = buffer
                .tag_table()
                .lookup(&info_tag.name().unwrap_or_default()) {
            buffer.tag_table().add(&info_tag);
        }
        let s_cloned = s.clone();
        let modificated_cloned = Rc::clone(&modificated);
        buffer.connect_changed(move |buf| {
            modificated_cloned.borrow_mut().insert(s_cloned.clone());
            for tag_name in [
                "pyright_error",
                "pyright_warning",
                "pyright_information",
            ] {
                buf.remove_tag_by_name(
                    tag_name,
                    &buf.start_iter(),
                    &buf.end_iter(),
                );
            }
        });

        let range_content:
            Rc<RefCell<HashMap<(i32, i32, i32, i32), Rc<(Option<String>, String)>>>>
            = Rc::new(RefCell::new(HashMap::new()));
        let path = current_dir()?.join(&s);
        let adiags = {
            let map_cloned = map.clone();
            let map_cb = map_cloned.borrow();
            map_cb.get(&path).cloned()
        };
        if let Some(adiags) = adiags {
            for adiag in adiags.iter() {|| -> Option<()> {
                let mut start = buffer.iter_at_line(adiag.range.start.line as i32)?;
                if !start.forward_chars(adiag.range.start.character as i32) {
                    return None;
                };
                let mut end = buffer.iter_at_line(adiag.range.end.line as i32)?;
                if !end.forward_chars(adiag.range.end.character as i32) {
                    return None;
                }
                buffer.apply_tag(
                    match adiag.severity {
                        Sev::Error => &err_tag,
                        Sev::Warning => &warn_tag,
                        Sev::Information => &info_tag,
                    },
                    &start,
                    &end,
                );
                range_content.borrow_mut().insert(
                    (
                        adiag.range.start.line as i32,
                        adiag.range.start.character as i32,
                        adiag.range.end.line as i32,
                        adiag.range.end.character as i32,
                    ),
                    Rc::new((adiag.rule.clone(), adiag.message.clone())),
                );
                return Some(());
            }();}
        }
        let popover = gtk4::Popover::new();
        let popover_hbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(unsafe{(*data).outter_spacing})
            .margin_top(unsafe{(*data).outter_spacing})
            .margin_bottom(unsafe{(*data).outter_spacing})
            .margin_start(unsafe{(*data).outter_spacing})
            .margin_end(unsafe{(*data).outter_spacing})
            .build();
        popover.set_focusable(false);
        popover.set_child(Some(&popover_hbox));
        popover.set_parent(&view_widget);
        let range_content_cloned = range_content.clone();
        let motion_controller = gtk4::EventControllerMotion::new();
        motion_controller.connect_motion(glib::clone!(
            #[weak] view,
            move |_, x, y| {let _ = || -> idl::Res {
                let (bx, by) = view.window_to_buffer_coords(
                    gtk4::TextWindowType::Widget,
                    x as i32,
                    y as i32,
                );
                let iter = get_attr!(view.iter_at_location(bx, by));
                let pyright_tags = iter.tags().into_iter().filter(|tag| [
                    Some(glib::GString::from("pyright_error")),
                    Some(glib::GString::from("pyright_warning")),
                    Some(glib::GString::from("pyright_information")),
                ].contains(&tag.name())).collect::<Vec<_>>();
                if pyright_tags.len() != 0 {
                    let mut start = iter.clone();
                    let mut end = iter.clone();
                    for rule_msg in pyright_tags.into_iter().filter_map(|tag| {
                        while start.backward_to_tag_toggle(Some(&tag)) && start.has_tag(&tag) {}
                        if !start.starts_tag(Some(&tag)) {
                            start.forward_to_tag_toggle(Some(&tag));
                        }
                        while end.forward_to_tag_toggle(Some(&tag)) && end.has_tag(&tag) {}
                        if !end.ends_tag(Some(&tag)) {
                            end.backward_to_tag_toggle(Some(&tag));
                        }
                        let key = (
                            start.line(),
                            start.line_offset(),
                            end.line(),
                            end.line_offset(),
                        );
                        range_content_cloned.borrow().get(&key).map(Rc::clone)
                    }) {
                        let vbox = gtk4::Box::new(
                            gtk4::Orientation::Vertical,
                            unsafe{(*data).inner_spacing},
                        );
                        if let Some(ref rule) = rule_msg.0 {
                            let rule_lbl = gtk4::Label::builder()
                                .label(rule)
                                .halign(gtk4::Align::Start)
                                .build();
                            vbox.append(&rule_lbl);
                        }
                        let msg_lbl = gtk4::Label::builder()
                            .label(&rule_msg.1)
                            .halign(gtk4::Align::Start)
                            .build();
                        vbox.append(&msg_lbl);
                        popover_hbox.append(&vbox);
                        popover.connect_closed(glib::clone!(
                            #[weak] vbox,
                            move |_| vbox.unparent(),
                        ));
                    }
                    popover.unparent();
                    popover.set_parent(&get_attr!(view.parent()));
                    let start_rect = view.iter_location(&start);
                    let end_rect = view.iter_location(&end);
                    let delta_x = end_rect.x() - start_rect.x();
                    let delta_y = end_rect.y() + end_rect.height() - start_rect.y();
                    let (sr_bx, sr_by) = view.buffer_to_window_coords(
                        gtk4::TextWindowType::Widget,
                        start_rect.x(),
                        start_rect.y(),
                    );
                    let rect = gtk4::gdk::Rectangle::new(
                        sr_bx,
                        sr_by,
                        delta_x,
                        delta_y,
                    );
                    popover.set_pointing_to(Some(&rect));
                    let motion_controller = gtk4::EventControllerMotion::new();
                    motion_controller.connect_motion(glib::clone!(
                        #[weak] popover,
                        move |cont, x, y| if !rect.contains_point(x as i32, y as i32)
                                && !popover.contains(x, y) {
                            popover.popdown();
                            popover.remove_controller(cont);
                        },
                    ));
                    popover.add_controller(motion_controller);
                    popover.popup();
                    view.grab_focus();
                }
                return Ok(());
            }();},
        ));
        view.add_controller(motion_controller);
        println!("{:#?}", modificated.borrow());
        return Ok(());
    }() {
        if let Some(gui) = unsafe{(*data).gui.as_ref()} {
            idl::show_error_dialog(&gui.window, e);
        }
    }
}
