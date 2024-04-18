pub mod helpers;
pub mod library;
use library::{init, add, get, status};
use extendr_api::{prelude::*, robj::Robj};
use std::path::PathBuf;
use anyhow::anyhow;

#[extendr]
fn dvs_init_impl(storage_dir: &str, mode: i32, group: &str, strict: bool) -> std::result::Result<(), String> {
    match init::dvs_init(&PathBuf::from(storage_dir), &mode, group, strict) {
        Ok(_) => {return Ok(())},
        Err(e) => return Err(anyhow!(e).to_string())
    };
} // dvs_init_impl



#[extendr]
fn dvs_add_impl(globs: Vec<String>, message: &str, strict: bool) -> Robj {
    // dvs add
    let added_files = match add::add(&globs, &String::from(message), strict) {
        Ok(files) => files,
        Err(e) => return Robj::from(e),
    };

    // let added_files = add::dvs_add(&files, &String::from(message), strict).map_err(|e| {
    //         Error::Other(e.to_string())
    //     })?.iter().map(|el| {
    //         el.map_err(|e| {
    //             Error::Other(e.to_string())
    //         })
    //     }).map(|el| AddedFileFlattened {
    //         error: el.err().map(|f| f.to_string()),
    //         path: el.ok().map(|f| f.absolute_path),
    //     })
    //     .collect::<Vec<AddedFileFlattened>>();

    // convert to data frame
    match added_files.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_add_impl



#[extendr]
fn dvs_get_impl(globs: Vec<String>) -> Robj {
    // dvs get
    let retrieved_files = match get::dvs_get(&globs) {
        Ok(files) => files,
        Err(e) => return Robj::from(e),
    };

    // convert to data frame
    match retrieved_files.into_dataframe() {
        Ok(dataframe) => dataframe.as_robj().clone(),
        Err(e) => Robj::from(format!("Error converting to DataFrame: {}", e)),
    }
} // dvs_get_impl



#[extendr]
fn dvs_status_impl(files: Vec<String>) -> Robj {
    // dvs status
    let status = match status::dvs_status(&files) {
        Ok(files) => files,
        Err(e) => return Robj::from(e),
    };

    // convert to data frame
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(1+1, 2);
    }

}