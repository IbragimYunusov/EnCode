use gtk4::{
    prelude::*,
    ApplicationWindow,
    Box,
    CellRendererPixbuf,
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
use gtk4::gdk_pixbuf::Pixbuf;
use sourceview5::{prelude::*, Buffer, StyleSchemeManager, View};
use glib::clone;

use std::path::PathBuf;
use std::sync::Arc;

use crate::app::func::editor as f_editor;


struct BuildTreeViewRet
{
    pub tree_view_vbox: Box,
    pub tree_view: TreeView,
    pub store: TreeStore,
    pub column: TreeViewColumn,
    pub renderer0: CellRendererPixbuf,
    pub renderer1: CellRendererPixbuf,
    pub renderer2: CellRendererText,
    pub tree_view_scrolled_window: ScrolledWindow,
}

struct BuildNotebookRet
{
    pub notebook: Notebook,
}


pub fn build_ui(window: ApplicationWindow, dir: &PathBuf) -> idl::Gui
{
    window.set_title(Some(&format!("EnCode — {}", &*dir.to_string_lossy())));
    window.set_default_width(1000);
    window.set_default_height(600);

    let dir = Arc::new(dir.clone());
    let vbox = Box::new(Orientation::Vertical, 0);
    let paned = Paned::builder()
        .orientation(Orientation::Horizontal)
        .vexpand(true)
        .position(150)
        .build();
    vbox.append(&paned);
    let build_tree_view_ret = build_tree_view(&paned, Arc::clone(&dir));
    let build_notebook_ret = build_notebook(
        Arc::clone(&dir),
        &paned,
        &build_tree_view_ret.tree_view,
    );
    window.set_child(Some(&vbox));
    return idl::Gui{
        window,
        vbox,
        paned,
        tree_view_vbox: build_tree_view_ret.tree_view_vbox,
        tree_view: build_tree_view_ret.tree_view,
        store: build_tree_view_ret.store,
        column: build_tree_view_ret.column,
        renderer0: build_tree_view_ret.renderer0,
        renderer1: build_tree_view_ret.renderer1,
        renderer2: build_tree_view_ret.renderer2,
        tree_view_scrolled_window: build_tree_view_ret.tree_view_scrolled_window,
        notebook:build_notebook_ret.notebook,
    };
}


fn build_tree_view(paned: &Paned, dir: Arc<PathBuf>) -> BuildTreeViewRet
{
    let store = TreeStore::new(&[
        Pixbuf::static_type(),
        Pixbuf::static_type(),
        String::static_type(),
    ]);
    f_editor::load_directory(&store, None, &dir);

    let tree_view_vbox = Box::new(Orientation::Vertical, 0);
    let tree_view = TreeView::builder()
        .model(&store)
        .headers_visible(false)
        .build();
    tree_view.connect_row_expanded(clone!(
        #[weak] store,
        move |_, iter, _| crate::app::ui::FILLED_DIR_ICON.with(
            |p| if let Some(pb) = p {store.set_value(iter, 0, &pb.to_value())}
        ),
    ));
    tree_view.connect_row_collapsed(clone!(
        #[weak] store,
        move |_, iter, _| crate::app::ui::DIR_ICON.with(
            |p| if let Some(pb) = p {store.set_value(iter, 0, &pb.to_value())}
        ),
    ));
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(|_, _, _, _| glib::Propagation::Stop);
    tree_view.add_controller(key_controller);
    let column = TreeViewColumn::new();
    let renderer0 = CellRendererPixbuf::new();
    let renderer1 = CellRendererPixbuf::new();
    let renderer2 = CellRendererText::new();
    column.pack_start(&renderer0, false);
    column.add_attribute(&renderer0, "pixbuf", 0);
    column.pack_start(&renderer1, false);
    column.add_attribute(&renderer1, "pixbuf", 1);
    renderer1.set_fixed_size(10, 14);
    column.pack_start(&renderer2, true);
    column.add_attribute(&renderer2, "text", 2);
    tree_view.append_column(&column);

    let tree_view_scrolled_window = ScrolledWindow::builder()
        .propagate_natural_height(true)
        .child(&tree_view)
        .vexpand(true)
        .build();
    tree_view_vbox.append(&tree_view_scrolled_window);
    paned.set_start_child(Some(&tree_view_vbox));

    return BuildTreeViewRet{
        tree_view_vbox,
        tree_view,
        store,
        column,
        renderer0,
        renderer1,
        renderer2,
        tree_view_scrolled_window,
    };
}


fn build_notebook(
    dir: Arc<PathBuf>,
    paned: &Paned,
    tree_view: &TreeView,
) -> BuildNotebookRet
{
    let notebook = Notebook::builder()
        .scrollable(true)
        .enable_popup(true)
        .build();
    paned.set_end_child(Some(&notebook));
    paned.set_focus_child(Some(&notebook));
    tree_view.selection().connect_changed(clone!(
        #[weak] tree_view,
        #[weak] notebook,
        move |selection| {
            if let Some((model, iter)) = selection.selected() {
                let path = model.path(&iter);
                let mut path_end = PathBuf::new();
                let mut indices = Vec::new();
                for index in path.indices() {
                    indices.push(index);
                    if let Some(iter) = model.iter(&TreePath::from_indices(&indices)) {
                        let name: String = model.get::<String>(&iter, 2);
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
            tree_view.selection().unselect_all();
        },
    ));
    return BuildNotebookRet{notebook};
}


fn build_tab(
    notebook: &Notebook,
    content: String,
    path_end: PathBuf,
) {
    notebook.set_current_page(Some({
        let text_area = build_text_area(
            &Buffer::builder().text(content).build(),
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
        tab_box.append(&{
            let close_label = Label::new(Some("✕"));
            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(
                #[weak] notebook,
                move |_, _, _, _| notebook.remove_page(notebook.current_page()),
            ));
            close_label.add_controller(gesture);
            close_label
        });
        let ret = notebook.append_page(
            &text_area,
            Some(&tab_box),
        );
        notebook.set_tab_detachable(&text_area, true);
        notebook.set_tab_reorderable(&text_area, true);
        ret
    }));
}


fn build_text_area(buffer: &Buffer) -> Box
{
    let bottom_hbox = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(super::INNER_SPACING)
        .margin_end(super::OUTTER_SPACING)
        .margin_top(super::INNER_SPACING)
        .margin_start(super::OUTTER_SPACING)
        .margin_bottom(super::INNER_SPACING)
        .build();
    let pos_label = Label::builder()
        .hexpand(true)
        .halign(gtk4::Align::End)
        .build();
    bottom_hbox.append(&pos_label);
    let view = View::builder()
        .buffer(buffer)
        .vexpand(true)
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
    if let Ok((scheme_name, dir)) = crate::app::func::get_name_and_path_for_color_scheme() {
        let style_scheme_manager = StyleSchemeManager::default();
        style_scheme_manager.append_search_path(&*dir.to_string_lossy());
        let style_scheme = style_scheme_manager.scheme(&scheme_name);
        buffer.set_style_scheme(style_scheme.as_ref());
    }
    buffer.connect_cursor_position_notify(clone!(
        #[weak] pos_label,
        move |buf| {
            let iter = buf.iter_at_offset(buf.cursor_position());
            pos_label.set_label(&format!(
                "{}:{}",
                iter.line() + 1,
                iter.line_offset(),
            ));
        },
    ));
    let text_scrolled_window = ScrolledWindow::builder()
        .propagate_natural_height(true)
        .child(&view)
        .build();
    let vbox = Box::new(Orientation::Vertical, 0);
    vbox.append(&text_scrolled_window);
    vbox.append(&bottom_hbox);
    return vbox;
}
