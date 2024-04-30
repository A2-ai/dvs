mod helpers;
mod library;
use helpers::outcome::Outcome;
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

// ADD
// one df
#[derive(Clone, PartialEq, Debug, IntoDataFrameRow)]
struct RFile {
    relative_path: Option<String>,
    outcome: String,
    size: Option<u64>,
    hash: Option<String>,
    absolute_path: Option<String>,
    input: String,
    error_type: Option<String>,
    error_message: Option<String>,
}

// success df
#[derive(Clone, PartialEq, IntoDataFrameRow)]
struct RFileSuccess {
    relative_path: String,
    outcome: String,
    size: u64,
    hash: String,
    absolute_path: String,
    input: String,
}

// error df
#[derive(Debug, IntoDataFrameRow)]
struct RFileError {
    relative_path: Option<String>,
    absolute_path: Option<String>,
    error_type: String,
    error_message: Option<String>,
    input: String,
}

#[extendr]
fn dvs_add_impl(globs: Vec<String>, message: &str, strict: bool, one_df: bool) -> Result<Robj> {
    let added_files = add::add(&globs, &String::from(message), strict).map_err(|e| {
        Error::Other(format!("{}: {}", e.error_type.batch_error_type_to_string(), e.error_message))
    })?;

    let results = added_files
        .iter()
        .zip(&globs)
        .map(|(fi, input)| match fi {
            Ok(fi) => RFile{
                relative_path: Some(fi.relative_path.display().to_string()),
                outcome: fi.outcome.outcome_to_string(),
                size: Some(fi.size),
                hash: Some(fi.hash.clone()),
                absolute_path: Some(fi.absolute_path.display().to_string()),
                input: input.clone(),
                error_type: None,
                error_message: None,
            },
            Err(e) => RFile{
                relative_path: e.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                outcome: Outcome::Error.outcome_to_string(),
                size: None,
                hash:  None,
                absolute_path: e.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                input: input.clone(),
                error_type: Some(e.error_type.file_error_type_to_string()),
                error_message: e.error_message.clone(),
            }
        })
        .collect::<Vec<RFile>>();

    if one_df {
        Ok(results
            .into_dataframe()
            .map_err(|e| Error::Other(format!("Error converting added files to data frame: {e}")))?
            .as_robj().clone())
    }
    else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error_type.is_some() {
                    Some(RFileError{
                        input: res.input.clone(),
                        relative_path: res.clone().relative_path,
                        absolute_path: res.clone().absolute_path,
                        error_type: res.clone().error_type.unwrap(),
                        error_message: res.clone().error_message
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error_type.is_none() {
                    Some(
                        RFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            outcome: res.outcome,
                            size: res.size.unwrap(),
                            hash: res.hash.unwrap(),
                            absolute_path: res.absolute_path.unwrap(),
                            input: res.input,
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RFileSuccess>>();

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
            
            Ok(List::from_hashmap(result).map_err(|e|Error::Other(format!("Error converting added files to data frame: {e}"))).into_robj())
    }
}

#[extendr]
fn dvs_get_impl(globs: Vec<String>, one_df: bool) -> Result<Robj> {
    let got_files = get::get(&globs).map_err(|e|
        Error::Other(format!("{}: {}", e.error_type.batch_error_type_to_string(), e.error_message))
    )?;

    let results = got_files
        .iter()
        .zip(&globs)
        .map(|(fi, input)| match fi {
            Ok(fi) => RFile{
                relative_path: Some(fi.relative_path.display().to_string()),
                outcome: fi.outcome.outcome_to_string(),
                size: Some(fi.size),
                absolute_path: Some(fi.absolute_path.display().to_string()),
                hash: Some(fi.hash.clone()),
                input: input.clone(),
                error_type: None,
                error_message: None
            },
            Err(e) => RFile{
                relative_path: e.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                outcome: Outcome::Error.outcome_to_string(),
                size: None,
                absolute_path: e.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                hash: None,
                input: input.clone(),
                error_type: Some(e.error_type.file_error_type_to_string()),
                error_message: e.error_message.clone()
            }

        })
        .collect::<Vec<RFile>>();

    if one_df {
        Ok(results
            .into_dataframe()
            .map_err(|e| Error::Other(format!("Error converting added files to data frame: {e}")))?
            .as_robj().clone())
    }
    else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error_type.is_some() {
                    Some(RFileError{
                        input: res.input.clone(),
                        relative_path: res.clone().relative_path,
                        absolute_path: res.clone().absolute_path,
                        error_type: res.clone().error_type.unwrap(),
                        error_message: res.clone().error_message
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error_type.is_none() {
                    Some(
                        RFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            outcome: res.outcome,
                            size: res.size.unwrap(),
                            hash: res.hash.unwrap(),
                            absolute_path: res.absolute_path.unwrap(),
                            input: res.input,
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RFileSuccess>>();

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
            
            Ok(List::from_hashmap(result).map_err(|e|Error::Other(format!("Error converting added files to data frame: {e}"))).into_robj())
            
    }
} // dvs_get_impl


#[derive(Clone, PartialEq, Debug, IntoDataFrameRow)]
struct RStatusFile {
    relative_path: Option<String>,
    status: String,
    size: Option<u64>,
    hash: Option<String>,
    absolute_path: Option<String>,
    error_type: Option<String>,
    error_message: Option<String>,
    input: String,
}

// success df
#[derive(Clone, PartialEq, IntoDataFrameRow)]
struct RStatusFileSuccess {
    relative_path: String,
    status: String,
    size: u64,
    hash: String,
    absolute_path: String,
}

// error df
#[derive(Debug, IntoDataFrameRow)]
struct RStatusFileError {
    input: String,
    relative_path: Option<String>,
    absolute_path: Option<String>,
    error_type: String,
    error_message: Option<String>,
}

#[extendr]
fn dvs_status_impl(globs: Vec<String>, one_df: bool) -> Result<Robj> {
    let status = status::status(&globs).map_err(|e|
        Error::Other(format!("{}: {}", e.error_type.batch_error_type_to_string(), e.error_message))
    )?;

    let results = status
        .iter()
        .map(|fi| match fi {
            Ok(fi) => RStatusFile{
                relative_path: fi.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                status: fi.outcome.outcome_to_string(),
                size: Some(fi.size),
                absolute_path: fi.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                hash: Some(fi.hash.clone()),
                error_type: None,
                error_message: None,
                input: fi.input.display().to_string(),
            },
            Err(e) => RStatusFile{
                relative_path: e.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                status: Outcome::Error.outcome_to_string(),
                size: None,
                absolute_path: e.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                hash: None,
                error_type: Some(e.error_type.file_error_type_to_string()),
                error_message: e.error_message.clone(),
                input: e.input.display().to_string(),
            }
        })
        .collect::<Vec<RStatusFile>>();

    if one_df {
        Ok(results
            .into_dataframe()
            .map_err(|e| Error::Other(format!("Error converting added files to data frame: {e}")))?
            .as_robj().clone())
    }
    else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error_type.is_some() {
                    Some(RStatusFileError{
                        relative_path: res.clone().relative_path,
                        absolute_path: res.clone().absolute_path,
                        error_type: res.clone().error_type.unwrap(),
                        error_message: res.clone().error_message,
                        input: res.clone().input
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RStatusFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error_type.is_none() {
                    Some(
                        RStatusFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            status: res.status,
                            size: res.size.unwrap(),
                            hash: res.hash.unwrap(),
                            absolute_path: res.absolute_path.unwrap(),
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RStatusFileSuccess>>();

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
            
            Ok(List::from_hashmap(result).map_err(|e|Error::Other(format!("Error converting added files to data frame: {e}"))).into_robj())
        }
}

// one df
#[derive(Debug, IntoDataFrameRow, Clone)]
struct RFileInfo {
    path: String,
    user_id: Option<u32>,
    user_name: Option<String>,
    group_id: Option<u32>,
    group_name: Option<String>,
    modification_time: Option<u64>,
    creation_time: Option<u64>,
    permissions: Option<String>,
    error: Option<String>,
}

// success df
#[derive(Debug, IntoDataFrameRow, Clone)]
struct RFileInfoSuccess {
    path: String,
    user_id: Option<u32>,
    user_name: Option<String>,
    group_id: Option<u32>,
    group_name: Option<String>,
    modification_time: Option<u64>,
    creation_time: Option<u64>,
    permissions: Option<String>
}

// error df
#[derive(Debug, IntoDataFrameRow, Clone)]
struct RInfoFileError {
    path: String,
    error: Option<String>,
}

#[extendr]
fn get_file_info_impl(paths: Vec<String>, one_df: bool) -> Robj {
    let file_info = info::info(&paths);
    let results = file_info
        .iter()
        .zip(&paths)
        .map(|(fi, path)| match fi {
            Ok(fi) => RFileInfo {
                path: fi.path.clone(),
                user_id: Some(fi.user_id.clone()),
                user_name: Some(fi.user_name.clone()),
                group_id: Some(fi.group_id.clone()),
                group_name: Some(fi.group_name.clone()),
                modification_time: Some(fi.modification_time),
                creation_time: Some(fi.creation_time),
                permissions: Some(fi.permissions.clone()),
                error: None,
            },
            Err(err) => RFileInfo {
                path: path.to_string(),
                user_id: None,
                user_name: None,
                group_id: None,
                group_name: None,
                modification_time: None,
                creation_time: None,
                permissions: None,
                error: Some(err.to_string()),
            },
        })
        .collect::<Vec<RFileInfo>>();
    if one_df {
        match results.into_dataframe() {
            Ok(dataframe) => dataframe.as_robj().clone(),
            Err(err) => Robj::from(format!("error converint to dataframe: {}", err)),
        }
    } else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error.is_some() {
                    Some(RInfoFileError {
                        path: res.path.clone(),
                        error: res.error.clone(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<RInfoFileError>>();
        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error.is_none() {
                    Some(RFileInfoSuccess{
                        path: res.path,
                        user_id: res.user_id,
                        user_name: res.user_name,
                        group_id: res.group_id,
                        group_name: res.group_name,
                        modification_time: res.modification_time,
                        creation_time: res.creation_time,
                        permissions: res.permissions
                    })
                }
                else {
                    None
                }
            })
            .collect::<Vec<RFileInfoSuccess>>();

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




 
