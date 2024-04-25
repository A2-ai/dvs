use crate::helpers::{config, copy, hash, file, repo, parse, ignore};

use extendr_api::{IntoDataFrameRow, Dataframe, eval_string, prelude::*};
use std::{fmt, fs, path::PathBuf, u32};
use file_owner::{Group, PathExt};
use serde::Serialize;

//pub type Result<T> = core::result::Result<T, Error>;

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
    input: String,
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
            AddErrorType::GitRepoNotFound => String::from("git repository not found"),
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
pub struct AddedFile {
    relative_path: String,
    outcome: String,
    size: u64,
    hash: String,
    absolute_path: String,
}

// #[derive(Clone, PartialEq, Serialize, IntoDataFrameRow)]
// pub struct ErrorFile {
//     input: String,
//     error_type: String,
//     error_message: Option<String>,
//     relative_path: Option<String>,
//     absolute_path: Option<String>,
// }

// pub struct AddedFileAttempts {
//     pub success_files: Vec<SuccessFile>,
//     pub error_files: Vec<ErrorFile>
// }



pub fn add(globs: &Vec<String>, message: &String, strict: bool) -> std::result::Result<Vec<std::result::Result<AddedFile, AddFileError>>, AddError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).map_err(|e| 
        AddError{ 
            error_type: AddErrorType::GitRepoNotFound.add_error_type_to_string(),
            error_message: format!("could not find git repo root; make sure you're in an active git repository: {e}")
        }
    )?;

    // load the config
    let conf = config::read(&git_dir).map_err(|e| 
        AddError{
            error_type: AddErrorType::ConfigNotFound.add_error_type_to_string(),
            error_message: format!("could not load configuration file, i.e. no dvs.yaml in directory; be sure to initiate devious: {e}")
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
                    error_message: format!("change group: {} in dvs.yaml, {e}", conf.group)
                }
            )?)
        };
        
    // check storage directory exists
    let storage_dir = conf.storage_dir.canonicalize().map_err(|e|
        AddError{
            error_type: AddErrorType::StorageDirNotFound.add_error_type_to_string(),
            error_message: format!("change storage_dir: {} in dvs.yaml, {e}", conf.storage_dir.display())
        }
    )?;

    // get file permissions
    let permissions = config::get_mode_u32(&conf.permissions).map_err(|e|
        AddError{
            error_type: AddErrorType::PermissionsInvalid.add_error_type_to_string(),
            error_message: format!("change permissions: {} in dvs.yaml, {e}", conf.permissions)
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

    let vec_of_added_files = queued_paths.into_iter().map(|file| {
        add_file(&file, &git_dir, &group, &storage_dir, &permissions, &message, strict)
    }).collect::<Vec<std::result::Result<AddedFile, AddFileError>>>();

    Ok(vec_of_added_files)
}

fn add_file(local_path: &PathBuf, git_dir: &PathBuf, group: &Option<Group>, storage_dir: &PathBuf, permissions: &u32, message: &String, strict: bool) -> std::result::Result<AddedFile, AddFileError> {
    // get absolute path
    let absolute_path = Some(local_path.canonicalize().map_err(|e|
        AddFileError{ // this should never error because if any paths aren't canonicalizable in the batch add fn, the fn returns
            input: local_path.display().to_string(),
            relative_path: None,
            absolute_path: None,
            error_type: AddFileErrorType::AbsolutePathNotFound.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?.display().to_string());

    // get relative path
    let relative_path = Some(repo::get_relative_path(&PathBuf::from("."), &local_path).map_err(|e|
            AddFileError{
                input: local_path.display().to_string(),
                relative_path: None,
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::RelativePathNotFound.add_file_error_type_to_string(),
                error_message: Some(e.to_string())
            }
        )?.display().to_string());

    // check if file in git repo
    if !repo::is_in_git_repo(&local_path, &git_dir) {
        return Err(AddFileError{
            input: local_path.display().to_string(),
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
                input: local_path.display().to_string(),
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
            input: local_path.display().to_string(),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::HashNotFound.add_file_error_type_to_string(),
            error_message: None
        }
    )?;

    // get file size
    let size = local_path.metadata().map_err(|e|
        AddFileError{
            input: local_path.display().to_string(),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::SizeNotFound.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?.len();

    // get user name
    let user_name: String = file::get_user_name(&local_path).map_err(|e|
        AddFileError{
            input: local_path.display().to_string(),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::OwnerNotFound.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?;

    // create metadata
    let metadata = file::Metadata{
        hash: hash.clone(),
        size,
        time_stamp: chrono::Local::now().to_string(),
        //time_stamp: chrono::offset::Utc::now().to_string(),
        message: message.clone(),
        saved_by: user_name
    };

    // write metadata file
    file::save(&metadata, &local_path).map_err(|e|
        AddFileError{
            input: local_path.display().to_string(),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::MetadataNotSaved.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?;

    // Add file to gitignore
    ignore::add_gitignore_entry(local_path).map_err(|e|
        AddFileError{
            input: local_path.display().to_string(),
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
        }.outcome_to_string();

    return Ok(
        AddedFile{
            relative_path: relative_path.unwrap(),
            absolute_path: absolute_path.unwrap(),
            outcome,
            size,
            hash
        }
    )
}




fn copy_file_to_storage_directory(local_path: &PathBuf, storage_path: &PathBuf, relative_path: &Option<String>, absolute_path: &Option<String>, permissions: &u32, group: &Option<Group>) -> std::result::Result<(), AddFileError> {
   // copy
    copy::copy(&local_path, &storage_path).map_err(|e|
        AddFileError{
            input: local_path.display().to_string(),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::FileNotCopied.add_file_error_type_to_string(),
            error_message: Some(e.to_string())
        }
    )?;

    // set file permissions
    copy::set_file_permissions(&permissions, &storage_path).map_err(|e|
        AddFileError {
            input: local_path.display().to_string(),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: AddFileErrorType::PermissionsNotSet.add_file_error_type_to_string(),
            error_message: Some(format!("{permissions} {e}")),
        }
    )?;

    // set group (if specified)
    if group.is_some() { 
        let group_name = group.unwrap(); // group.is_some() so can safely unwrap
        storage_path.set_group(group_name).map_err(|e|
            AddFileError{
                input: local_path.display().to_string(),
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: AddFileErrorType::GroupNotSet.add_file_error_type_to_string(),
                error_message: Some(format!("{group_name} {e}"))
            }
        )?;
    }
    return Ok(())
}