pub mod helpers;
pub mod library;
use library::add_new::{AddErrorType, AddFileErrorType, Outcome};
use library::{init, add_new, get, status, info};
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
#[derive(Clone, PartialEq, IntoDataFrameRow)]
pub struct RAddedFile {
    relative_path: Option<String>,
    outcome: Option<String>,
    size: Option<u64>,
    hash: Option<String>,
    absolute_path: Option<String>,
    input: String,
    error_type: Option<String>,
    error_message: Option<String>,
}

// success df
#[derive(Clone, PartialEq, IntoDataFrameRow)]
pub struct RAddFileSuccess {
    pub relative_path: String,
    pub outcome: String,
    pub size: u64,
    pub hash: String,
    pub absolute_path: String,
    pub input: String,
}

// error df
#[derive(Debug, IntoDataFrameRow)]
pub struct RAddFileError {
    pub relative_path: Option<String>,
    pub absolute_path: Option<String>,
    pub error_type: String,
    pub error_message: Option<String>,
    pub input: String,
}

impl AddFileErrorType {
    pub fn add_file_error_type_to_string(&self) -> String {
        match self {
            AddFileErrorType::RelativePathNotFound => String::from("relative path not found"),
            AddFileErrorType::FileNotInGitRepo => String::from("file not in git repo"),
            AddFileErrorType::AbsolutePathNotFound => String::from("file not found"),
            AddFileErrorType::PathIsDirectory => String::from("path is a directory"),
            AddFileErrorType::HashNotFound => String::from("hash not found"),
            AddFileErrorType::SizeNotFound => String::from("size not found"),
            AddFileErrorType::OwnerNotFound => String::from("owner not found"),
            AddFileErrorType::GroupNotSet => String::from("group not set"),
            AddFileErrorType::PermissionsNotSet => String::from("group not set"),
            AddFileErrorType::MetadataNotSaved => String::from("metadata file not saved"),
            AddFileErrorType::GitIgnoreNotAdded => String::from("gitignore entry not added"),
            AddFileErrorType::FileNotCopied => String::from("file not copied"),
        }
    }
}

impl AddErrorType {
    pub fn add_error_type_to_string(&self) -> String {
        match self {
            AddErrorType::AnyFilesDNE => String::from("at least one inputted file not found"),
            AddErrorType::GitRepoNotFound => String::from("git repository not found"),
            AddErrorType::ConfigNotFound => String::from("configuration file not found"),
            AddErrorType::GroupNotFound => String::from("linux primary group not found"),
            AddErrorType::StorageDirNotFound => String::from("storage directory not found"),
            AddErrorType::PermissionsInvalid => String::from("linux file permissions invalid"),
        }
    }
}

impl Outcome {
    pub fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Success => String::from("Success"),
            Outcome::AlreadyPresent => String::from("Already Present")
        }
    }
}

#[extendr]
// std::result::Result<Robj, String> 
fn dvs_add_impl(globs: Vec<String>, message: &str, strict: bool, one_df: bool) -> Result<Robj> {
    let added_files = add_new::add(&globs, &String::from(message), strict).map_err(|e| {
        Error::Other(format!("{}: {}", e.error_type.add_error_type_to_string(), e.error_message))
    })?;

    let results = added_files
        .iter()
        .zip(&globs)
        .map(|(fi, input)| match fi {
            Ok(fi) => RAddedFile{
                relative_path: Some(fi.relative_path.clone()),
                outcome: Some(fi.outcome.outcome_to_string()),
                size: Some(fi.size),
                hash: Some(fi.hash.clone()),
                absolute_path: Some(fi.absolute_path.clone()),
                input: input.clone(),
                error_type: None,
                error_message: None,
            },
            Err(e) => RAddedFile{
                relative_path: e.relative_path.clone(),
                outcome: None,
                size: None,
                hash:  None,
                absolute_path: e.absolute_path.clone(),
                input: input.clone(),
                error_type: Some(e.error_type.add_file_error_type_to_string()),
                error_message: e.error_message.clone(),
            }
        })
        .collect::<Vec<RAddedFile>>();

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
                    Some(RAddFileError{
                        input: res.input.clone(),
                        relative_path: res.clone().relative_path,
                        absolute_path: res.clone().absolute_path,
                        error_type: res.clone().error_type.unwrap(),
                        error_message: res.clone().error_message
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RAddFileError>>();

        let successes = results
            .into_iter()
            .filter_map(|res| {
                if res.error_type.is_none() {
                    Some(
                        RAddFileSuccess{
                            relative_path: res.relative_path.unwrap(),
                            outcome: res.outcome.unwrap(),
                            size: res.size.unwrap(),
                            hash: res.hash.unwrap(),
                            absolute_path: res.absolute_path.unwrap(),
                            input: res.input,
                        }
                    )
                }
                else {None}
            }).collect::<Vec<RAddFileSuccess>>();

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


// one df
#[derive(Debug, IntoDataFrameRow, Clone)]
pub struct RFileInfo {
    pub path: String,
    pub user_id: Option<u32>,
    pub user_name: Option<String>,
    pub group_id: Option<u32>,
    pub group_name: Option<String>,
    pub modification_time: Option<u64>,
    pub creation_time: Option<u64>,
    pub permissions: Option<String>,
    pub error: Option<String>,
}

// success df
#[derive(Debug, IntoDataFrameRow, Clone)]
pub struct RFileInfoSuccess {
    pub path: String,
    pub user_id: Option<u32>,
    pub user_name: Option<String>,
    pub group_id: Option<u32>,
    pub group_name: Option<String>,
    pub modification_time: Option<u64>,
    pub creation_time: Option<u64>,
    pub permissions: Option<String>
}

// error df
#[derive(Debug, IntoDataFrameRow, Clone)]
pub struct RFileError {
    pub path: String,
    pub error: Option<String>,
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
                    Some(RFileError {
                        path: res.path.clone(),
                        error: res.error.clone(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<RFileError>>();
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
