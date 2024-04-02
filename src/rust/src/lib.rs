use extendr_api::dataframe;
use extendr_api::prelude::*;
use helpers::config;
use helpers::repo;
// use extendr_api::robj::{Robj, IntoRobj};
pub mod helpers;
pub mod library;
use std::path::PathBuf;
use crate::library::init;
use crate::library::add;
use extendr_api::robj::Robj;
// use crate::helpers::repo;
// use crate::helpers::config;
// use std::fs::create_dir;
// use path_absolutize::Absolutize;
// use file_owner::Group;
use anyhow::{anyhow, Context};
// use anyhow::anyhow;

use std::convert::TryFrom;

/// Return string `"Hello world!"` to R.
/// @export
#[extendr]
fn hello_world() -> &'static str {
    "Hello world!"
}

/// @export
#[extendr]
pub fn dvs_init_r(storage_dir: &str, mode: i32, group: &str) -> std::result::Result<(), String> {
    let storage_dir_in = PathBuf::from(storage_dir);
    let mode_in = match u32::try_from(mode) {
        Ok(mode) => mode,
        Err(e) => return Err(anyhow!("could not convert permissions to unsigned integer {e}").to_string())
    };
    match init::dvs_init(&storage_dir_in, &mode_in, group) {
        Ok(_) => {},
        Err(e) => return Err(anyhow!(e).to_string())
    };

    Ok(())
} // dvs_init_r


/// @export
#[extendr]
pub fn dvs_add_r(files: Vec<String>, message: &str) -> Robj {
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Robj::from(format!("could not find git repo root - make sure you're in an active git repository: {e}")),
    };

    // load the config
    let conf = match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Robj::from(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: {e}")),
    };

    let res = match add::dvs_add(&files, &git_dir, &conf, &String::from(message)) {
        Ok(res) => res,
        Err(e) => return Robj::from(e),
    };

    match res.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_add_r



// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod Rdevious;
    fn hello_world;
    fn dvs_init_r;
    fn dvs_add_r;
    
}
