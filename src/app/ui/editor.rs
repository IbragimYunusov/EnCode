use gtk4::{
    prelude::*,
    ApplicationWindow,
    Box,
    Builder,
    CellRendererText,
    EventControllerKey,
    GestureClick,
    Label,
    Notebook,
    Orientation,
    Paned,
    ScrolledWindow,
    TreePath,
    TreeStore,
    TreeView,
    TreeViewColumn,
    WrapMode,
};
use sourceview5::{prelude::*, Buffer, LanguageManager, StyleSchemeManager};
use glib::clone;

use std::path::PathBuf;
use std::sync::Arc;
use std::ffi::CStr;

use crate::app::func::editor as f_editor;
use crate::plug::inter_data::DATA as INTER_DATA;


struct BuildTreeViewRet
{
    pub tree_view: TreeView,
    pub store: TreeStore,
    pub column: TreeViewColumn,
    pub renderer: CellRendererText,
    pub tree_view_scrolled_window: ScrolledWindow,
}

struct BuildNotebookRet
{
    pub notebook: Notebook,
}


pub fn build_ui(window: ApplicationWindow, dir: &PathBuf) -> Builder
{
    let builder = Builder::new();

    window.set_title(Some(&format!("EnCode — {}", &*dir.to_string_lossy())));
    window.set_default_width(956);
    window.set_default_height(546);
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.app_window_id).to_string_lossy()},
        ),
        &window,
    );

    let dir = Arc::new(dir.clone());
    let _ = std::env::set_current_dir(&*dir);

    let paned = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(100)
        .build();
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.paned_id).to_string_lossy()},
        ),
        &paned,
    );
    let build_tree_view_ret = build_tree_view(&builder, &paned, Arc::clone(&dir));
    build_notebook(
        &builder,
        Arc::clone(&dir),
        &paned,
        &build_tree_view_ret.tree_view,
    );
    window.set_child(Some(&paned));
    return builder;
}


fn build_tree_view(
    builder: &Builder,
    paned: &Paned,
    dir: Arc<PathBuf>,
) -> BuildTreeViewRet
{
    let store = TreeStore::new(&[String::static_type()]);
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.store_id).to_string_lossy()},
        ),
        &store,
    );
    f_editor::load_directory(&store, None, &dir);

    let tree_view = TreeView::builder()
        .model(&store)
        //.enable_tree_lines(true)
        .headers_visible(false)
        .build();
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.tree_view_id).to_string_lossy()},
        ),
        &tree_view,
    );
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(|_, _, _, _| glib::Propagation::Stop);
    tree_view.add_controller(key_controller);
    let column = TreeViewColumn::builder()
        .title(&*dir.file_name().unwrap_or_default().to_string_lossy())
        .build();
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.column_id).to_string_lossy()},
        ),
        &column,
    );
    let renderer = CellRendererText::new();
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.renderer_id).to_string_lossy()},
        ),
        &renderer,
    );
    column.pack_start(&renderer, true);
    column.add_attribute(&renderer, "text", 0);
    tree_view.append_column(&column);

    let tree_view_scrolled_window = ScrolledWindow::builder()
        .propagate_natural_height(true)
        .child(&tree_view)
        .build();
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(
                d.gui_ids.tree_view_scrolled_window_id
            ).to_string_lossy()},
        ),
        &tree_view_scrolled_window,
    );
    paned.set_start_child(Some(&tree_view_scrolled_window));

    return BuildTreeViewRet{
        tree_view,
        store,
        column,
        renderer,
        tree_view_scrolled_window,
    };
}


fn build_notebook(
    builder: &Builder,
    dir: Arc<PathBuf>,
    paned: &Paned,
    tree_view: &TreeView,
) -> BuildNotebookRet
{
    let notebook = Notebook::builder()
        .scrollable(true)
        .enable_popup(true)
        .build();
    builder.expose_object(
        &*INTER_DATA.with_borrow(
            |d| unsafe{CStr::from_ptr(d.gui_ids.notebook_id).to_string_lossy()},
        ),
        &notebook,
    );
    paned.set_end_child(Some(&notebook));
    paned.set_focus_child(Some(&notebook));
    tree_view.selection().connect_changed(
        clone!(
            #[weak] notebook,
            move |selection| {
                if let Some((model, iter)) = selection.selected() {
                    let path = model.path(&iter);
                    let mut path_end = PathBuf::new();
                    let mut indices = Vec::new();
                    for index in path.indices() {
                        indices.push(index);
                        if let Some(iter) = model.iter(&TreePath::from_indices(&indices)) {
                            let name: String = model.get::<String>(&iter, 0);
                            path_end.push(name);
                        }
                    }
                    if let Some(n) = (0..notebook.n_pages()).filter_map(
                        |n| Some(
                            notebook
                                .tab_label(&notebook.nth_page(Some(n))?)?
                                .downcast::<Box>()
                                .ok()?
                                .first_child()?
                                .downcast::<Label>()
                                .ok()?
                                .label(),
                        ),
                    ).position(|s| s == &*path_end.to_string_lossy()) {
                        notebook.set_current_page(Some(n as u32));
                    } else {
                        let file_path = dir.join(&path_end);
                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                            build_tab(&notebook, content, path_end);
                        }
                    }
                }
            },
        ),
    );
    return BuildNotebookRet{notebook};
}


fn build_tab(notebook: &Notebook, content: String, path_end: PathBuf)
{
    notebook.set_current_page(
        Some(
            {
                let text_area = build_text_area(
                    &Buffer::builder().text(content).build(),
                    &*path_end.extension().unwrap_or_default().to_string_lossy(),
                );
                let tab_box = Box::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(super::INNER_SPACING)
                    .build();
                tab_box.append(
                    &Label::builder()
                        .label(&*path_end.to_string_lossy())
                        .halign(gtk4::Align::End)
                        .build(),
                );
                tab_box.append(
                    &{
                        let close_label = Label::new(Some("✕"));
                        let gesture = GestureClick::new();
                        gesture.connect_pressed(
                            clone!(
                                #[weak] notebook,
                                move |_, _, _, _| {
                                    notebook.remove_page(notebook.current_page());
                                },
                            )
                        );
                        close_label.add_controller(gesture);
                        close_label
                    },
                );
                let ret = notebook.append_page(
                    &text_area,
                    Some(&tab_box),
                );
                notebook.set_tab_detachable(&text_area, true);
                notebook.set_tab_reorderable(&text_area, true);
                ret
            },
        ),
    );
}


fn build_text_area(buffer: &Buffer, ext: &str) -> ScrolledWindow
{
    let view = sourceview5::View::builder()
        .buffer(buffer)
        .wrap_mode(WrapMode::None)
        .highlight_current_line(true)
        .show_line_numbers(true)
        .smart_backspace(true)
        .accepts_tab(true)
        .indent_on_tab(true)
        .auto_indent(true)
        .right_margin(super::INNER_SPACING)
        .left_margin(super::INNER_SPACING)
        .monospace(true)
        .smart_home_end(sourceview5::SmartHomeEndType::Always)
        .insert_spaces_instead_of_tabs(true)
        .tab_width(4)
        .build();
    let language_manager = LanguageManager::new();
    buffer.set_highlight_matching_brackets(true);
    buffer.set_highlight_syntax(true);
    if let Ok((scheme_name, dir)) = crate::app::func::get_name_and_path_for_color_scheme() {
        let style_scheme_manager = StyleSchemeManager::default();
        style_scheme_manager.append_search_path(&*dir.to_string_lossy());
        let style_scheme = style_scheme_manager.scheme(&scheme_name);
        buffer.set_style_scheme(style_scheme.as_ref());
    }
    buffer.set_language(
        match ext {
            "py" => language_manager.language("python3"),
            _ => None,
        }.as_ref(),
    );
    let text_scrolled_window = ScrolledWindow::builder()
        .propagate_natural_height(true)
        .child(&view)
        .build();
    return text_scrolled_window;
}
