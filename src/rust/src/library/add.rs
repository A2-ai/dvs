use crate::helpers::{config, copy, hash, file, repo, parse, ignore};
use extendr_api::{IntoDataFrameRow, Dataframe, eval_string, prelude::*};
use std::{u32, fmt, path::PathBuf};
use file_owner::{Group, PathExt};
use serde::Serialize;
use anyhow::Context;

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
                write!(f, "{}", message)
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
            AddErrorType::AnyFilesDNE => String::from("a least one inputted file not found"),
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
    pub error_message: Option<String>,
}

impl fmt::Display for AddError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error_message.clone() {
            Some(message) => {
                write!(f, "{}", message)
            }
            None => {
                write!(f, "NA")
            }
        }
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




pub fn add(globs: &Vec<String>, message: &String, strict: bool) -> std::result::Result<(Vec<SuccessFile>, Vec<ErrorFile>), AddError> {
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => {
            return Err(
                AddError{ 
                    error_type: AddErrorType::GitRepoNotFound.add_error_type_to_string(),
                    error_message: Some(format!("could not find git repo root - make sure you're in an active git repository: \n{e}"))
                }
            )
        }
    };

    // load the config
    let conf = match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => {
            return Err(
                AddError{
                error_type: AddErrorType::ConfigNotFound.add_error_type_to_string(),
                error_message: Some(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}"))
                }
            )
        }
    };

    // check group if group was specified
    let group_name = conf.group;
    if group_name != "" {
        match Group::from_name(group_name.as_str()) {
            Ok(_) => {}
            Err(e) => {
                return Err(
                    AddError{
                        error_type: AddErrorType::GroupNotFound.add_error_type_to_string(),
                        error_message: Some(e.to_string())
                    }
                )
            }
        };
    }

    // check storage directory exists
    let storage_dir: PathBuf = match conf.storage_dir.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            return Err(
                AddError{
                    error_type: AddErrorType::StorageDirNotFound.add_error_type_to_string(),
                    error_message: Some(e.to_string())
                }
            )
        }
    };

    // get file permissions
    let permissions: u32 = match config::get_mode_u32(&conf.permissions) {
        Ok(mode) => mode,
        Err(e) => {
            return Err(
                AddError{
                    error_type: AddErrorType::PermissionsInvalid.add_error_type_to_string(),
                    error_message: Some(e.to_string())
                }
            )
        }
    };

    // collect paths out of input - sort through globs/explicitly-named files
    let queued_paths = parse::parse_files_from_globs(&globs);

    // warn if no paths queued after sorting through input - likely not intentional by user
    if queued_paths.is_empty() {
        println!("warning: no paths queued to add to devious")
    }

    // check if any files don't exist first
    // std::result:: Result<Vec<PathBuf>, AddError>
    // let _queued_abs_paths_result: std::result:: Result<Vec<PathBuf>, AddError> = 
    queued_paths.iter().map(|file| {
        match file.canonicalize() {
            Ok(path) => Ok(path),
            Err(e) => {
                return Err(
                    AddError{
                        error_type: AddErrorType::AnyFilesDNE.add_error_type_to_string(),
                        error_message: Some(format!("{} not found: {e}", file.display()))
                    }
                )
            }
        }
    }).collect::<std::result:: Result<Vec<PathBuf>, AddError>>()?;

    //let queued_abs_paths = queued_abs_paths_result?;
    
    // add each file to the storage directory
    let mut success_files: Vec<SuccessFile> = Vec::new();
    let mut error_files: Vec<ErrorFile> = Vec::new();
    for file in queued_paths { // had to use for loop instead of map because add returns a result
        match add_file(&file, &git_dir, &group_name, &storage_dir, &permissions, &message, strict) {
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

    return Ok((success_files, error_files))
} // run_add_cmd

fn add_file(local_path: &PathBuf, git_dir: &PathBuf, group_name: &String, storage_dir: &PathBuf, permissions: &u32, message: &String, strict: bool) -> std::result::Result<SuccessFile, AddFileError> {
    let mut error_type: Option<AddFileErrorType> = None;
    let mut error_message: Option<String> = None;
    // get absolute path
    let absolute_path: Option<String> = match local_path.canonicalize() {
        Ok(absolute) => { // file exists
            // error if file is outside of git repository
            if absolute.strip_prefix(&git_dir).unwrap() == absolute {
                error_type = Some(AddFileErrorType::FileNotInGitRepo);
            }
            Some(absolute.display().to_string())
        }
        // error if file not canonicalizable
        Err(e) => { 
            error_type = Some(AddFileErrorType::AbsolutePathNotFound);
            error_message = Some(e.to_string());
            None
        }
    };
    
    // get local path relative to working directory
    let relative_path: Option<String> = match repo::get_relative_path(&PathBuf::from("."), &local_path) {
        Ok(rel_path) => Some(rel_path.display().to_string()),
        Err(e) => {
            error_type = Some(AddFileErrorType::RelativePathNotFound);
            error_message = Some(e.to_string());
            None
        }
    };

    if error_type.is_some() {
        return Err(
            AddFileError{
                relative_path,
                absolute_path,
                error_type: error_type.unwrap().add_file_error_type_to_string(),
                error_message
            }
        )
    }
     

    // error if file is a directory
    if local_path.is_dir() {
        return Err(
            AddFileError{
                relative_path,
                absolute_path,
                error_type: AddFileErrorType::PathIsDirectory.add_file_error_type_to_string(),
                error_message: None
            }
        )
    }

    // get file hash
    let hash = match hash::get_file_hash(&local_path) {
        Some(hash) => hash,
        None => return Err(
            AddFileError{
                relative_path,
                absolute_path,
                error_type: AddFileErrorType::HashNotFound.add_file_error_type_to_string(),
                error_message: None
            }
        )
    };
    
    // get file size
    let size: u64 = match local_path.metadata() {
        Ok(data) => data.len(),
        Err(e) => {
            return Err(
                AddFileError{
                    relative_path,
                    absolute_path,
                    error_type: AddFileErrorType::SizeNotFound.add_file_error_type_to_string(),
                    error_message: Some(e.to_string())
                }
            )
        }
    };

    // get user name
    let user_name: String = match local_path.owner().with_context(|| format!("owner not found")) {
        Ok(owner) => {
            let owner_name = match owner.name() {
                Ok(name) => {
                    match name {
                        Some(name) => name,
                        None => {
                            return Err(
                                AddFileError{
                                    relative_path,
                                    absolute_path,
                                    error_type: AddFileErrorType::OwnerNameNotFound.add_file_error_type_to_string(),
                                    error_message: None
                                }
                            )
                        }
                    }
                }
                Err(e) => {
                    return Err(
                        AddFileError{
                            relative_path,
                            absolute_path,
                            error_type: AddFileErrorType::OwnerNameNotFound.add_file_error_type_to_string(),
                            error_message: Some(e.to_string())
                        }
                    )
                }
            };
            owner_name
        }
        Err(e) => {
            return Err(
                AddFileError{
                    relative_path,
                    absolute_path,
                    error_type: AddFileErrorType::OwnerNotFound.add_file_error_type_to_string(),
                    error_message: Some(e.to_string())
                }
            )
        }
    };

    // create metadata
    let metadata = file::Metadata{
        file_hash: hash.clone(),
        file_size: size,
        //time_stamp: chrono::Local::now().to_string(),
        time_stamp: chrono::offset::Utc::now().to_string(),
        message: message.clone(),
        saved_by: user_name
    };

    // write metadata file
    match file::save(&metadata, &local_path) {
        Ok(_) => {},
        Err(e) => {
            return Err(
                AddFileError{
                    relative_path,
                    absolute_path,
                    error_type: AddFileErrorType::MetadataNotSaved.add_file_error_type_to_string(),
                    error_message: Some(e.to_string())
                }
            )
        }
    };

    // Add file to gitignore
    match ignore::add_gitignore_entry(local_path) {
        Ok(_) => {},
        Err(e) => {
            return Err(
                AddFileError{
                    relative_path,
                    absolute_path,
                    error_type: AddFileErrorType::GitIgnoreNotAdded.add_file_error_type_to_string(),
                    error_message: Some(e.to_string())
                }
            )
        }
    };
    
    // get storage path
    let storage_path = hash::get_storage_path(&storage_dir, &hash);
    
    let mut outcome: Outcome = Outcome::AlreadyPresent;
   
    // copy the file to the storage directory if it's not already there and the metadata was successfully saved
    if !storage_path.exists() { // if not already copied
        // copy and get error
        match copy_file_to_storage_directory(local_path, &storage_path, relative_path.clone(), absolute_path.clone(), &permissions, &group_name, strict) {
            Ok(_) => outcome = Outcome::Success,
            Err(e) => return Err(e)
        };
    }

    return Ok(
        SuccessFile{relative_path: relative_path.unwrap(), absolute_path: absolute_path.unwrap(), hash, outcome: outcome.outcome_to_string(), size}
    )
}




fn copy_file_to_storage_directory(local_path: &PathBuf, dest_path: &PathBuf, relative_path: Option<String>, absolute_path: Option<String>, mode: &u32, group_name: &String, strict: bool) -> std::result::Result<(), AddFileError> {
    match copy::copy(&local_path, &dest_path) {
        Ok(_) => {
            // set permissions
            match copy::set_file_permissions(&mode, &dest_path) {
                Ok(_) => {},
                Err(e) => {
                    if strict {
                        // TODO: delete copied file
                        // TODO: delete metadata file
                    }
                    else {
                        return Err(AddFileError{
                            relative_path,
                            absolute_path,
                            error_type: AddFileErrorType::PermissionsNotSet.add_file_error_type_to_string(),
                            error_message: Some(e.to_string())
                        })
                    }
                }
            };

            if group_name != "" {
                // set group ownership
                let group = Group::from_name(group_name).with_context(|| format!("group not found: {group_name}")).unwrap();
                match dest_path.set_group(group.clone()) {
                    Ok(_) => {},
                    Err(e) => {
                        if strict {
                            // TODO: delete copied file
                            // TODO: delete metadata file
                        }
                        else {
                            return Err(AddFileError{
                                relative_path,
                                absolute_path,
                                error_type: AddFileErrorType::GroupNotSet.add_file_error_type_to_string(),
                                error_message: Some(e.to_string())
                            })
                        }
                    }
                };
            }
          
        } // Ok, could copy
        Err(copy_e) => { // could not copy
            if strict {
                // TODO: delete copied file
                // TODO: delete metadata file
            } // strict
            else { // non-strict
                return Err(
                    AddFileError{
                        relative_path,
                        absolute_path,
                        error_type: AddFileErrorType::FileNotCopied.add_file_error_type_to_string(),
                        error_message: Some(copy_e.to_string())
                    }
                )
            }
        }
    };
    return Ok(())
}