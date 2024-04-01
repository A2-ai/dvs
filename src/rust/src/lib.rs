use extendr_api::prelude::*;
use extendr_api::robj::{Robj, IntoRobj};
pub mod helpers;
pub mod library;
use std::path::PathBuf;
use crate::library::init;
use crate::helpers::repo;
use crate::helpers::config;
use std::fs::create_dir;
use path_absolutize::Absolutize;
use file_owner::Group;
use anyhow::{anyhow, Context, Result};
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
        Err(e) => return Err(anyhow!(e).to_string())
    };
    let group_in = String::from(group);
    match init::dvs_init(&storage_dir_in, &mode_in, &group_in) {
        Ok(_) => {},
        Err(e) => return Err(anyhow!(e).to_string())
    };

    Ok(())
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod Rdevious;
    fn hello_world;
    fn dvs_init_r;
}
