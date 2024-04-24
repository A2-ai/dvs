pub mod helpers;
pub mod library;
use library::{init, add, get, status};
use extendr_api::{prelude::*, robj::Robj};
use std::path::PathBuf;

#[extendr]
fn dvs_init_impl(storage_dir: &str, mode: i32, group: &str) -> std::result::Result<(), String> {
    init::dvs_init(&PathBuf::from(storage_dir), &mode, group).map_err(|e|
        Error::Other(format!("{}: {}", e.error_type, e.error_message))
    )?;

    Ok(())
} // dvs_init_impl



#[extendr]
fn dvs_add_impl(globs: Vec<String>, message: &str, strict: bool) -> std::result::Result<List, String> {
    let added_files = add::add(&globs, &String::from(message), strict).map_err(|e| {
        Error::Other(format!("{}: {}", e.error_type, e.error_message))
    })?;

    let success_files = added_files.success_files.into_dataframe().map_err(|e|
        Error::Other(format!("Error converting sucessfully added files to data frame: {e}"))
    )?.as_robj().clone();

    let error_files = added_files.error_files.into_dataframe().map_err(|e|
        Error::Other(format!("Error converting errored added files to data frame: {e}"))
    )?.as_robj().clone();

    return Ok(list!(success_files, error_files))
} // dvs_add_impl



#[extendr]
fn dvs_get_impl(globs: Vec<String>) -> std::result::Result<Robj, String> {
    Ok(get::dvs_get(&globs).map_err(|e|
        Error::Other(format!("{}: {}", e.error_type, e.error_message))
    )?.into_dataframe()?.as_robj().clone())
} // dvs_get_impl



#[extendr]
fn dvs_status_impl(files: Vec<String>) -> std::result::Result<Robj, String> {
    Ok(status::dvs_status(&files)?.into_dataframe()?.as_robj().clone())
} // dvs_status_impl


extendr_module! {
    mod Rdevious;
    fn dvs_init_impl;
    fn dvs_add_impl;
    fn dvs_get_impl;
    fn dvs_status_impl;
}
