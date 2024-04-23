use crate::helpers::{config, copy, hash, file, repo, parse, ignore};

use extendr_api::{IntoDataFrameRow, Dataframe, eval_string, prelude::*};
use std::{fmt, fs, path::PathBuf, u32};
use file_owner::{Group, PathExt};
use serde::Serialize;

// Outcome enum
#[derive(Clone, PartialEq, Serialize)]
enum Outcome {
    Success,
    AlreadyPresent,
}

impl Outcome {
    fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Success => String::from("Success"),
            Outcome::AlreadyPresent => String::from("Already Present")
        }
    }
}

// Custom error individual files
#[derive(Clone, PartialEq, Serialize)]
enum AddFileErrorType {
    RelativePathNotFound,
    FileNotInGitRepo,
    AbsolutePathNotFound,
    PathIsDirectory,
    HashNotFound,
    SizeNotFound,
    OwnerNotFound,
    OwnerNameNotFound,
    GroupNotSet,
    PermissionsNotSet,
    MetadataNotSaved,
    GitIgnoreNotAdded,
    FileNotCopied,
}

impl AddFileErrorType {
    fn add_file_error_type_to_string(&self) -> String {
        match self {
            AddFileErrorType::RelativePathNotFound => String::from("relative path not found"),
            AddFileErrorType::FileNotInGitRepo => String::from("file not in git repo"),
            AddFileErrorType::AbsolutePathNotFound => String::from("file not found"),
            AddFileErrorType::PathIsDirectory => String::from("path is a directory"),
            AddFileErrorType::HashNotFound => String::from("hash not found"),
            AddFileErrorType::SizeNotFound => String::from("size not found"),
            AddFileErrorType::OwnerNotFound => String::from("owner not found"),
            AddFileErrorType::OwnerNameNotFound => String::from("owner name not found"),
            AddFileErrorType::GroupNotSet => String::from("group not set"),
            AddFileErrorType::PermissionsNotSet => String::from("group not set"),
            AddFileErrorType::MetadataNotSaved => String::from("metadata file not saved"),
            AddFileErrorType::GitIgnoreNotAdded => String::from("gitignore entry not added"),
            AddFileErrorType::FileNotCopied => String::from("file not copied"),
        }
    }
}

#[derive(Debug)]
pub struct AddFileError {
    relative_path: Option<String>,
    absolute_path: Option<String>,
    error_type: String,
    error_message: Option<String>,
}

impl fmt::Display for AddFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error_message.clone() {
            Some(message) => {
                write!(f, "{message}")
            }
            None => {
                write!(f, "NA")
            }
        }
    }
}

impl std::error::Error for AddFileError {}


// custom error for add function (not file-specific errors)
#[derive(Clone, PartialEq, Serialize)]
enum AddErrorType {
    AnyFilesDNE,
    GitRepoNotFound,
    ConfigNotFound,
    GroupNotFound,
    StorageDirNotFound,
    PermissionsInvalid,
}

impl AddErrorType {
    fn add_error_type_to_string(&self) -> String {
        match self {
            AddErrorType::AnyFilesDNE => String::from("at least one inputted file not found"),
            AddErrorType::GitRepoNotFound => String::from("git repo not found"),
            AddErrorType::ConfigNotFound => String::from("configuration file not found"),
            AddErrorType::GroupNotFound => String::from("linux primary group not found"),
            AddErrorType::StorageDirNotFound => String::from("storage directory not found"),
            AddErrorType::PermissionsInvalid => String::from("linux file permissions invalid"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AddError {
    pub error_type: String,
    pub error_message:String,
}

impl fmt::Display for AddError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.error_message)
    }
}

impl std::error::Error for AddError {}

#[derive(Clone, PartialEq, Serialize, IntoDataFrameRow)]
pub struct SuccessFile {
    relative_path: String,
    absolute_path: String,
    hash: String,
    outcome: String,
    size: u64,
}

#[derive(Clone, PartialEq, Serialize, IntoDataFrameRow)]
pub struct ErrorFile {
    input: String,
    relative_path: Option<String>,
    absolute_path: Option<String>,
    error_type: String,
    error_message: Option<String>,
}

pub struct AddedFileAttempts {
    pub success_files: Vec<SuccessFile>,
    pub error_files: Vec<ErrorFile>
}




pub fn add(globs: &Vec<String>, message: &String, strict: bool) -> std::result::Result<AddedFileAttempts, AddError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).map_err(|e| 
        AddError{ 
            error_type: AddErrorType::GitRepoNotFound.add_error_type_to_string(),
            error_message: format!("could not find git repo root - make sure you're in an active git repository: \n{e}")
        }
    )?;

    // load the config
    let conf = config::read(&git_dir).map_err(|e| 
        AddError{
            error_type: AddErrorType::ConfigNotFound.add_error_type_to_string(),
            error_message: format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}")
        }
    )?;

    // get group, check if specified
    let group = 
        if conf.group == "" {
            None
        }
        else {
            Some(Group::from_name(&conf.group.as_str()).map_err(|e|
                AddError{
                    error_type: AddErrorType::GroupNotFound.add_error_type_to_string(),
                    error_message: e.to_string()
                }
            )?)
        };
        
    // check storage directory exists
    let storage_dir = conf.storage_dir.canonicalize().map_err(|e|
        AddError{
            error_type: AddErrorType::StorageDirNotFound.add_error_type_to_string(),
            error_message: e.to_string()
        }
    )?;

    // get file permissions
    let permissions = config::get_mode_u32(&conf.permissions).map_err(|e|
        AddError{
            error_type: AddErrorType::PermissionsInvalid.add_error_type_to_string(),
            error_message: e.to_string()
        }
    )?;

    // collect paths out of input - sort through globs/explicitly-named files
    let queued_paths = parse::parse_files_from_globs(&globs);

    // warn if no paths queued after sorting through input - likely not intentional by user
    if queued_paths.is_empty() {
        println!("warning: no paths queued to add to devious")
    }

    // return error if any files don't exist
    queued_paths.iter().map(|file| {
       file.canonicalize().map_err(|e|
            AddError{
                error_type: AddErrorType::AnyFilesDNE.add_error_type_to_string(),
                error_message: format!("{} not found: {e}", file.display())
            })
    }).collect::<std::result:: Result<Vec<PathBuf>, AddError>>()?;

    // add each file to the storage directory
    let mut success_files: Vec<SuccessFile> = Vec::new();
    let mut error_files: Vec<ErrorFile> = Vec::new();
    for file in queued_paths { // had to use for loop instead of map because add returns a result
        match add_file(&file, &git_dir, &group, &storage_dir, &permissions, &message, strict) {
            Ok(file) => {
                success_files.push(file);
            }
            Err(e) => {
                let error_file = ErrorFile {
                    input: file.display().to_string(),
                    relative_path: e.relative_path,
                    absolute_path: e.absolute_path,
                    error_type: e.error_type,
                    error_message: e.error_message
                };
                error_files.push(error_file)
            }
        };
    }

    return Ok(AddedFileAttempts{success_files, error_files})
} // run_add_cmd

fn add_file(local_path: &PathBuf, git_dir: &PathBuf, group: &Option<Group>, storage_dir: &PathBuf, permissions: &u32, message: &String, strict: bool) -> std::result::Result<SuccessFile, AddFileError> {
    // get absolute path
    let absolute_path = Some(local_path.canonicalize().map_err(|e|
        AddFileError{ // this should never error because if any paths aren't canonicalizable in the batch add fn, the fn returns
            relative_path: None,
            absolute_path: None,
            error_type: AddFileErrorType::AbsolutePathNotFound.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?.display().to_string());

    // get relative path
    let relative_path = Some(repo::get_relative_path(&PathBuf::from("."), &local_path).map_err(|e|
            AddFileError{
                relative_path: None,
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::RelativePathNotFound.add_file_error_type_to_string(),
                error_message: Some(e.to_string())
            }
        )?.display().to_string());

    // check if file in git repo
    if !repo::is_in_git_repo(&local_path, &git_dir) {
        return Err(AddFileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::FileNotInGitRepo.add_file_error_type_to_string(),
            error_message: None
        })
    }

    // error if file is a directory
    if local_path.is_dir() {
        return Err(
            AddFileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::PathIsDirectory.add_file_error_type_to_string(),
                error_message: None
            }
        )
    }

    // get file hash
    let hash = hash::get_file_hash(&local_path).ok_or(
        AddFileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::HashNotFound.add_file_error_type_to_string(),
            error_message: None
        }
    )?;

    // get file size
    let size = local_path.metadata().map_err(|e|
        AddFileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::SizeNotFound.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?.len();

    // get user name
    let user_name: String = local_path.owner()
        .map_err(|e|
            AddFileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::OwnerNotFound.add_file_error_type_to_string(),
                error_message: Some(e.to_string())
            }
        )?
        .name()
        .map_err(|e|
            AddFileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::OwnerNameNotFound.add_file_error_type_to_string(),
                error_message: Some(e.to_string())
            }
        )?.unwrap_or_default();

    // create metadata
    let metadata = file::Metadata{
        file_hash: hash.clone(),
        file_size: size,
        time_stamp: chrono::Local::now().to_string(),
        //time_stamp: chrono::offset::Utc::now().to_string(),
        message: message.clone(),
        saved_by: user_name
    };

    // write metadata file
    file::save(&metadata, &local_path).map_err(|e|
        AddFileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::MetadataNotSaved.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?;

    // Add file to gitignore
    ignore::add_gitignore_entry(local_path).map_err(|e|
        AddFileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::GitIgnoreNotAdded.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?;
    
    // get storage path
    let storage_path = hash::get_storage_path(&storage_dir, &hash);
    
    // copy
    let outcome = 
        if !storage_path.exists() { // if not already copied
            if let Err(e) = copy_file_to_storage_directory(&local_path, &storage_path, &relative_path, &absolute_path, &permissions, &group) {
                    if strict {
                        // remove metadata file
                        let _ = fs::remove_file(PathBuf::from(local_path.display().to_string() + ".dvsmeta"));
                        // remove copied file from storage directory
                        let _ = fs::remove_file(storage_path);
                    }
                    return Err(e)
            };
            Outcome::Success
        }
        else {
            Outcome::AlreadyPresent
        };


    return Ok(
        SuccessFile{relative_path: relative_path.unwrap(), absolute_path: absolute_path.unwrap(), hash, outcome: outcome.outcome_to_string(), size}
    )
}




fn copy_file_to_storage_directory(local_path: &PathBuf, storage_path: &PathBuf, relative_path: &Option<String>, absolute_path: &Option<String>, permissions: &u32, group: &Option<Group>) -> std::result::Result<(), AddFileError> {
   // copy
    copy::copy(&local_path, &storage_path).map_err(|e|
        AddFileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::FileNotCopied.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?;

    // set file permissions
    copy::set_file_permissions(&permissions, &storage_path).map_err(|e|
        AddFileError {
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::PermissionsNotSet.add_file_error_type_to_string(),
            error_message: Some(e.to_string()),
        }
    )?;

    // set group (if specified)
    if group.is_some() {
        storage_path.set_group(group.unwrap()).map_err(|e|
            AddFileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::GroupNotSet.add_file_error_type_to_string(),
                error_message: Some(e.to_string())
            }
        )?;
    }
    return Ok(())
}