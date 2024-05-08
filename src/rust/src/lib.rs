mod helpers;
mod library;
use helpers::outcome::{Outcome, Status};
use library::{init, add, get, status, info};
use extendr_api::{prelude::*, robj::Robj};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, IntoDataFrameRow)]
struct RFile {
    relative_path: Option<String>,
    outcome: String,
    file_size_bytes: Option<u64>,
    blake3_checksum: Option<String>,
    absolute_path: Option<String>,
    input: Option<String>,
    error: Option<String>,
    error_message: Option<String>,
}

// success df
#[derive(Clone, PartialEq, IntoDataFrameRow)]
struct RFileSuccess {
    relative_path: String,
    outcome: String,
    file_size_bytes: u64,
    blake3_checksum: String,
    absolute_path: String,
}

// error df
#[derive(Debug, IntoDataFrameRow)]
struct RFileError {
    input: String,
    relative_path: Option<String>,
    absolute_path: Option<String>,
    error: String,
    error_message: Option<String>,
}

#[derive(Debug, IntoDataFrameRow)]
struct RInit {
    storage_directory: String,
    file_permissions: i32,
    group: String,
}

#[extendr]
fn dvs_init_impl(storage_dir: &str, mode: i32, group: &str) -> Result<Robj> {
    let init = init::dvs_init(&PathBuf::from(storage_dir), &mode, group).map_err(|e|
        Error::Other(format!("{}: {}", e.error.init_error_to_string(), e.error_message))
    )?;

    let init_df = RInit{
        storage_directory: init.storage_directory.display().to_string(),
        group: init.group,
        file_permissions: init.file_permissions,
    };

    Ok(vec![init_df]
        .into_dataframe()
        .map_err(|e|Error::Other(format!("Error converting initialization information to data frame: {e}")))?
        .as_robj()
        .clone())
} 


#[extendr]
fn dvs_add_impl(globs: Vec<String>, message: &str, strict: bool, split_output: bool) -> Result<Robj> {
    let added_files = add::add(&globs, &String::from(message), strict).map_err(|e| {
        Error::Other(format!("{}: {}", e.error.batch_error_to_string(), e.error_message))
    })?;

    let results = added_files
        .iter()
        .map(|fi| match fi {
            Ok(fi) => RFile{
                relative_path: Some(fi.relative_path.display().to_string()),
                outcome: fi.outcome.outcome_to_string(),
                file_size_bytes: Some(fi.file_size_bytes),
                blake3_checksum: Some(fi.blake3_checksum.clone()),
                absolute_path: Some(fi.absolute_path.display().to_string()),
                input: None,
                error: None,
                error_message: None,
            },
            Err(e) => RFile{
                relative_path: e.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                outcome: Outcome::Error.outcome_to_string(),
                file_size_bytes: None,
                blake3_checksum:  None,
                absolute_path: e.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                input: Some(e.input.display().to_string()),
                error: Some(e.error.file_error_to_string()),
                error_message: e.error_message.clone(),
            }
        })
        .collect::<Vec<RFile>>();

    if !split_output {
        Ok(results
            .into_dataframe()
            .map_err(|e| Error::Other(format!("Error converting added files to data frame: {e}")))?
            .as_robj().clone())
    }
    else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error.is_some() {
                    Some(RFileError{
                        input: res.input.clone().unwrap(),
                        relative_path: res.relative_path.clone(),
                        absolute_path: res.absolute_path.clone(),
                        error: res.error.clone().unwrap(),
                        error_message: res.error_message.clone()
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error.is_none() {
                    Some(
                        RFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            outcome: res.outcome,
                            file_size_bytes: res.file_size_bytes.unwrap(),
                            blake3_checksum: res.blake3_checksum.unwrap(),
                            absolute_path: res.absolute_path.unwrap(),
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
fn dvs_get_impl(globs: Vec<String>, split_output: bool) -> Result<Robj> {
    let got_files = get::get(&globs).map_err(|e|
        Error::Other(format!("{}: {}", e.error.batch_error_to_string(), e.error_message))
    )?;

    let results = got_files
        .iter()
        .map(|fi| match fi {
            Ok(fi) => RFile{
                relative_path: Some(fi.relative_path.display().to_string()),
                outcome: fi.outcome.outcome_to_string(),
                file_size_bytes: Some(fi.file_size_bytes),
                absolute_path: Some(fi.absolute_path.display().to_string()),
                blake3_checksum: Some(fi.blake3_checksum.clone()),
                input: None,
                error: None,
                error_message: None
            },
            Err(e) => RFile{
                relative_path: e.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                outcome: Outcome::Error.outcome_to_string(),
                file_size_bytes: None,
                absolute_path: e.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                blake3_checksum: None,
                input: Some(e.input.display().to_string()),
                error: Some(e.error.file_error_to_string()),
                error_message: e.error_message.clone()
            }
        })
        .collect::<Vec<RFile>>();

    if !split_output {
        Ok(results
            .into_dataframe()
            .map_err(|e| Error::Other(format!("Error converting added files to data frame: {e}")))?
            .as_robj().clone())
    }
    else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error.is_some() {
                    Some(RFileError{
                        input: res.input.clone().unwrap(),
                        relative_path: res.relative_path.clone(),
                        absolute_path: res.absolute_path.clone(),
                        error: res.error.clone().unwrap(),
                        error_message: res.error_message.clone(),
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error.is_none() {
                    Some(
                        RFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            outcome: res.outcome,
                            file_size_bytes: res.file_size_bytes.unwrap(),
                            blake3_checksum: res.blake3_checksum.unwrap(),
                            absolute_path: res.absolute_path.unwrap(),
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
    file_size_bytes: Option<u64>,
    blake3_checksum: Option<String>,
    time_stamp: Option<String>,
    saved_by: Option<String>,
    message: Option<String>,
    absolute_path: Option<String>,
    error: Option<String>,
    error_message: Option<String>,
    input: Option<String>,
}

// success df
#[derive(Clone, PartialEq, IntoDataFrameRow)]
struct RStatusFileSuccess {
    relative_path: String,
    status: String,
    file_size_bytes: u64,
    time_stamp: String,
    saved_by: String,
    message: String,
    blake3_checksum: String,
    absolute_path: String,
}

// error df
#[derive(Debug, IntoDataFrameRow)]
struct RStatusFileError {
    input: String,
    relative_path: Option<String>,
    absolute_path: Option<String>,
    error: String,
    error_message: Option<String>,
}

#[extendr]
fn dvs_status_impl(globs: Vec<String>, split_output: bool) -> Result<Robj> {
    let status = status::status(&globs).map_err(|e|
        Error::Other(format!("{}: {}", e.error.batch_error_to_string(), e.error_message))
    )?;

    let results = status
        .iter()
        .map(|fi| match fi {
            Ok(fi) => RStatusFile{
                relative_path: fi.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                status: fi.status.outcome_to_string(),
                file_size_bytes: Some(fi.file_size_bytes.clone()),
                time_stamp: Some(fi.time_stamp.clone()),
                saved_by: Some(fi.saved_by.clone()),
                message: Some(fi.message.clone()),
                absolute_path: fi.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                blake3_checksum: Some(fi.blake3_checksum.clone()),
                error: None,
                error_message: None,
                input: fi.input.clone().map(|p| p.to_string_lossy().to_string()),
            },
            Err(e) => RStatusFile{
                relative_path: e.relative_path.clone().map(|p| p.to_string_lossy().to_string()),
                status: Status::Error.outcome_to_string(),
                file_size_bytes: None,
                message: None,
                time_stamp: None,
                saved_by: None,
                absolute_path: e.absolute_path.clone().map(|p| p.to_string_lossy().to_string()),
                blake3_checksum: None,
                error: Some(e.error.file_error_to_string()),
                error_message: e.error_message.clone(),
                input: Some(e.input.display().to_string())
            }
        })
        .collect::<Vec<RStatusFile>>();

    if !split_output {
        Ok(results
            .into_dataframe()
            .map_err(|e| Error::Other(format!("Error converting added files to data frame: {e}")))?
            .as_robj().clone())
    }
    else {
        let failures = results
            .iter()
            .filter_map(|res| {
                if res.error.is_some() {
                    Some(RStatusFileError{
                        relative_path: res.relative_path.clone(),
                        absolute_path: res.absolute_path.clone(),
                        error: res.error.clone().unwrap(),
                        error_message: res.error_message.clone(),
                        input: res.input.clone().unwrap()
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RStatusFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error.is_none() {
                    Some(
                        RStatusFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            status: res.status,
                            time_stamp: res.time_stamp.unwrap(),
                            saved_by: res.saved_by.unwrap(),
                            message: res.message.unwrap(),
                            file_size_bytes: res.file_size_bytes.unwrap(),
                            blake3_checksum: res.blake3_checksum.unwrap(),
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
fn get_file_info_impl(paths: Vec<String>, split_output: bool) -> Robj {
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
    if !split_output {
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




 
