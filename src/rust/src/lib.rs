use extendr_api::prelude::*;
use library::get;
use library::status;
pub mod helpers;
pub mod library;
use std::path::PathBuf;
use crate::library::init;
use crate::library::add;
use extendr_api::robj::Robj;
use anyhow::anyhow;

#[extendr]
fn dvs_init_impl(storage_dir: &str, mode: i32, group: &str) -> std::result::Result<(), String> {
    let storage_dir_in = PathBuf::from(storage_dir);

    match init::dvs_init(&storage_dir_in, &mode, group) {
        Ok(_) => {},
        Err(e) => return Err(anyhow!(e).to_string())
    };

    Ok(())
} // dvs_init_impl



#[extendr]
fn dvs_add_impl(files: Vec<String>, message: &str) -> Robj {
    let res = match add::dvs_add(&files, &String::from(message)) {
        Ok(res) => res,
        Err(e) => return Robj::from(e),
    };

    match res.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_add_impl



#[extendr]
fn dvs_get_impl(globs: Vec<String>) -> Robj {
    let retrieved_files = match get::dvs_get(&globs) {
        Ok(files) => files,
        Err(e) => return Robj::from(e),
    };

    match retrieved_files.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_get_impl



#[extendr]
fn dvs_status_impl(files: Vec<String>) -> Robj {
    let status = match status::dvs_status(&files) {
        Ok(files) => files,
        Err(e) => return Robj::from(e),
    };

    match status.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_status_impl


// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod Rdevious;
    fn dvs_init_impl;
    fn dvs_add_impl;
    fn dvs_get_impl;
    fn dvs_status_impl;
}