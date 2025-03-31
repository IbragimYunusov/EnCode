pub mod inter_data;
pub mod funcs;

use std::collections::HashMap;

use libloading::Library;
use lazy_static::lazy_static;

use crate::sql::general_data::Plugin;


lazy_static!
{
    pub static ref LIBRARIES:
        Option<HashMap<String, Library>>
        = if let crate::app::AppType::EDITOR(_) = *crate::app::APP_TYPE {
            if let Ok(plugins) = Plugin::select_all() {
                Some((*plugins).iter().filter_map(|plug| if plug.as_ref().ok()?.will_be_used {Some((
                    plug.as_ref().ok()?.dev_name.clone(),
                    unsafe {Library::new(
                        crate::app::func::get_dir()?.join(
                            format!(
                                "{}{}{}",
                                std::env::consts::DLL_PREFIX,
                                &plug.as_ref().ok()?.dev_name,
                                std::env::consts::DLL_SUFFIX,
                            ),
                        ),
                    ).ok()?},
                ))} else {None}).collect())
            } else {
                eprintln!("Couldn't include plugins");
                None
            }
        } else {None};
}
