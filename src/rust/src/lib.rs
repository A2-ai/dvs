pub mod helpers;
pub mod library;
use library::{init, add, get, status, info};
use extendr_api::{prelude::*, robj::Robj};
use std::path::PathBuf;
use std::collections::HashMap;

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

    return Ok(list!(successes = success_files, errors = error_files))
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


#[derive(Debug, IntoDataFrameRow, Clone)]
pub struct RFileInfo {
    pub path: String,
    pub owner_id: Option<u32>,
    pub owner_name: Option<String>,
    pub group_id: Option<u32>,
    pub group_name: Option<String>,
    pub modification_time: Option<u64>,
    pub creation_time: Option<u64>,
    pub permissions: Option<String>,
    pub error: Option<String>,
}

#[extendr]
fn get_file_info_impl(paths: Vec<String>, df: bool) -> Robj {
    let file_info = info::info(&paths);
    let results = file_info
        .iter()
        .zip(&paths)
        .map(|(fi, path)| match fi {
            Ok(fi) => RFileInfo {
                path: fi.path.clone(),
                owner_id: Some(fi.owner_id),
                owner_name: Some(fi.owner_name.clone()),
                group_id: Some(fi.group_id),
                group_name: Some(fi.group_name.clone()),
                modification_time: Some(fi.modification_time),
                creation_time: Some(fi.creation_time),
                permissions: Some(fi.permissions.clone()),
                error: None,
            },
            Err(err) => RFileInfo {
                path: path.to_string(),
                owner_id: None,
                owner_name: None,
                group_id: None,
                group_name: None,
                modification_time: None,
                creation_time: None,
                permissions: None,
                error: Some(err.to_string()),
            },
        })
        .collect::<Vec<RFileInfo>>();
    if df {
        match results.into_dataframe() {
            Ok(dataframe) => dataframe.as_robj().clone(),
            Err(err) => Robj::from(format!("error converint to dataframe: {}", err)),
        }
    } else {
        let failures = results
            .clone()
            .into_iter()
            .filter(|res| res.error.is_some())
            .collect::<Vec<RFileInfo>>();
        let successes = results
            .into_iter()
            .filter(|res| res.error.is_none())
            .collect::<Vec<RFileInfo>>();
        let mut result = HashMap::new();
        if successes.len() > 0 {
            result.insert(
                "successes",
                successes.into_dataframe().unwrap().as_robj().clone(),
            );
        }
        if failures.len() > 0 {
            result.insert(
                "failures",
                failures.into_dataframe().unwrap().as_robj().clone(),
            );
        }
        let output = List::from_hashmap(result);
        output.into()
    }
}

extendr_module! {
    mod Rdevious;
    fn dvs_init_impl;
    fn dvs_add_impl;
    fn dvs_get_impl;
    fn dvs_status_impl;
    fn get_file_info_impl;
}
