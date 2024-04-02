use extendr_api::prelude::*;
use helpers::config;
use helpers::repo;
use library::get::dvs_get;
use library::status::dvs_status;
pub mod helpers;
pub mod library;
use std::path::PathBuf;
use crate::library::init;
use crate::library::add;
use extendr_api::robj::Robj;
use anyhow::anyhow;
use std::convert::TryFrom;

/// @export
#[extendr]
fn dvs_init_impl(storage_dir: &str, mode: i32, group: &str) -> std::result::Result<(), String> {
    let storage_dir_in = PathBuf::from(storage_dir);
    let mode_in = match u32::try_from(mode) {
        Ok(mode) => mode,
        Err(e) => return Err(anyhow!("could not convert permissions to unsigned integer \n{e}").to_string())
    };
    match init::dvs_init(&storage_dir_in, &mode_in, group) {
        Ok(_) => {},
        Err(e) => return Err(anyhow!(e).to_string())
    };

    Ok(())
} // dvs_init_impl


/// @export
#[extendr]
fn dvs_add_impl(files: Vec<String>, message: &str) -> Robj {
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Robj::from(format!("could not find git repo root - make sure you're in an active git repository: \n{e}")),
    };

    // load the config
    let conf = match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Robj::from(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}")),
    };

    let res = match add::dvs_add(&files, &git_dir, &conf, &String::from(message)) {
        Ok(res) => res,
        Err(e) => return Robj::from(e),
    };

    match res.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_add_impl


/// @export
#[extendr]
fn dvs_get_impl(globs: Vec<String>) -> Robj {
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Robj::from(format!("could not find git repo root - make sure you're in an active git repository: \n{e}")),
    };

    // load the config
    let conf = match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Robj::from(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}")),
    };

    let retrieved_files = match dvs_get(&globs, &conf) {
        Ok(files) => files,
        Err(e) => return Robj::from(e),
    };

    match retrieved_files.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_get_impl


/// @export
#[extendr]
fn dvs_status_impl(files: Vec<String>) -> Robj {
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Robj::from(format!("could not find git repo root - make sure you're in an active git repository: \n{e}")),
    };

    // load the config
    match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Robj::from(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}")),
    };

    let status = match dvs_status(&files, &git_dir) {
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
