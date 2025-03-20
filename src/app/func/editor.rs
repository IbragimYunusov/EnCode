use gtk4::prelude::*;


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
            store.set_value(&iter, 0, &name.to_value());

            if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                load_directory(store, Some(iter), &entry.path());
            }
        }
    }
}
