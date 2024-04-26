use crate::helpers::{config, copy, hash, file, repo, parse, ignore};
use std::{fmt, fs, path::PathBuf, u32};
use file_owner::{Group, PathExt};

// Outcome enum
#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Success,
    AlreadyPresent,
    Error,
}

// Custom error individual files
#[derive(Clone, PartialEq, Debug)]
pub enum FileErrorType {
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

#[derive(Debug)]
pub struct FileError {
    pub relative_path: Option<PathBuf>,
    pub absolute_path: Option<PathBuf>,
    pub error_type: FileErrorType,
    pub error_message: Option<String>,
}

impl fmt::Display for FileError {
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

impl std::error::Error for FileError {}


// custom error for add function (not file-specific errors)
#[derive(Clone, PartialEq, Debug)]
pub enum BatchErrorType {
    AnyFilesDNE,
    GitRepoNotFound,
    ConfigNotFound,
    GroupNotFound,
    StorageDirNotFound,
    PermissionsInvalid,
}


#[derive(Debug)]
pub struct BatchError {
    pub error_type: BatchErrorType,
    pub error_message: String,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl std::error::Error for BatchError {}

#[derive(Clone, PartialEq)]
pub struct AddedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub size: u64,
    pub hash: String,
    pub absolute_path: PathBuf,
}

pub fn add(globs: &Vec<String>, message: &String, strict: bool) -> std::result::Result<Vec<std::result::Result<AddedFile, FileError>>, BatchError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).map_err(|e| 
        BatchError{ 
            error_type: BatchErrorType::GitRepoNotFound,
            error_message: format!("could not find git repo root; make sure you're in an active git repository: {e}")
        }
    )?;

    // load the config
    let conf = config::read(&git_dir).map_err(|e| 
        BatchError{
            error_type: BatchErrorType::ConfigNotFound,
            error_message: format!("could not load configuration file, i.e. no dvs.yaml in directory; be sure to initiate devious: {e}")
        }
    )?;

    // get group, check if specified
    let group = 
        if conf.group == "" {None}
        else {
            Some(Group::from_name(&conf.group.as_str()).map_err(|e|
                BatchError{
                    error_type: BatchErrorType::GroupNotFound,
                    error_message: format!("change group: {} in dvs.yaml, {e}", conf.group)
                }
            )?)
        };
        
    // check storage directory exists
    let storage_dir = conf.storage_dir.canonicalize().map_err(|e|
        BatchError{
            error_type: BatchErrorType::StorageDirNotFound,
            error_message: format!("change storage_dir: {} in dvs.yaml, {e}", conf.storage_dir.display())
        }
    )?;

    // get file permissions
    let permissions = config::get_mode_u32(&conf.permissions).map_err(|e|
        BatchError{
            error_type: BatchErrorType::PermissionsInvalid,
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
            BatchError{
                error_type: BatchErrorType::AnyFilesDNE,
                error_message: format!("{} not found: {e}", file.display())
            })
    }).collect::<std::result:: Result<Vec<PathBuf>, BatchError>>()?;

    Ok(queued_paths.into_iter().map(|file| {
        add_file(&file, &git_dir, &group, &storage_dir, &permissions, &message, strict)
    }).collect::<Vec<std::result::Result<AddedFile, FileError>>>())
}

fn add_file(local_path: &PathBuf, git_dir: &PathBuf, group: &Option<Group>, storage_dir: &PathBuf, permissions: &u32, message: &String, strict: bool) -> std::result::Result<AddedFile, FileError> {
    // get absolute path
    let absolute_path = Some(local_path.canonicalize().map_err(|e|
        FileError{ // this should never error because if any paths aren't canonicalizable in the batch add fn, the fn returns
            relative_path: None,
            absolute_path: None,
            error_type: FileErrorType::AbsolutePathNotFound,
            error_message: Some(e.to_string())
        }
    )?);

    // get relative path
    let relative_path = Some(repo::get_relative_path(&PathBuf::from("."), &local_path).map_err(|e|
            FileError{
                relative_path: None,
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::RelativePathNotFound,
                error_message: Some(e.to_string())
            }
        )?);

    // check if file in git repo
    if !repo::is_in_git_repo(&local_path, &git_dir) {
        return Err(FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::FileNotInGitRepo,
            error_message: None
        })
    }

    // error if file is a directory
    if local_path.is_dir() {
        return Err(
            FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::PathIsDirectory,
                error_message: None
            }
        )
    }

    // get file hash
    let hash = hash::get_file_hash(&local_path).ok_or(
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::HashNotFound,
            error_message: None
        }
    )?;

    // get file size
    let size = local_path.metadata().map_err(|e|
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::SizeNotFound,
            error_message: Some(e.to_string())
        }
    )?.len();

    // get user name
    let user_name: String = file::get_user_name(&local_path).map_err(|e|
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::OwnerNotFound,
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
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string())
        }
    )?;

    // Add file to gitignore
    ignore::add_gitignore_entry(local_path).map_err(|e|
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::GitIgnoreNotAdded,
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

    Ok(
        AddedFile{
            relative_path: relative_path.unwrap(),
            absolute_path: absolute_path.unwrap(),
            outcome,
            size,
            hash
        }
    )
}




fn copy_file_to_storage_directory(local_path: &PathBuf, storage_path: &PathBuf, relative_path: &Option<PathBuf>, absolute_path: &Option<PathBuf>, permissions: &u32, group: &Option<Group>) -> std::result::Result<(), FileError> {
   // copy
    copy::copy(&local_path, &storage_path).map_err(|e|
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::FileNotCopied,
            error_message: Some(e.to_string())
        }
    )?;

    // set file permissions
    copy::set_file_permissions(&permissions, &storage_path).map_err(|e|
        FileError {
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::PermissionsNotSet,
            error_message: Some(format!("{permissions} {e}")),
        }
    )?;

    // set group (if specified)
    if group.is_some() { 
        let group_name = group.unwrap(); // group.is_some() so can safely unwrap
        storage_path.set_group(group_name).map_err(|e|
            FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::GroupNotSet,
                error_message: Some(format!("{group_name} {e}"))
            }
        )?;
    }
    return Ok(())
}