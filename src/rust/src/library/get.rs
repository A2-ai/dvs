use std::{fmt, path::PathBuf};
use crate::helpers::{config, copy, hash, file, repo, parse};
// use extendr_api::{Dataframe, IntoDataFrameRow, prelude::*};

#[derive(PartialEq, Debug)]
pub enum Outcome {
    Copied,
    AlreadyPresent,
    Error,
}

#[derive(Debug)]
pub struct RetrievedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub size: u64,
    pub absolute_path: PathBuf,
    pub hash: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FileErrorType {
   PathIsDirectory,
   MetadataNotFound,
   RelativePathNotFound,
   FileNotCopied,
   AbsolutePathNotFound
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

#[derive(Clone, PartialEq, Debug)]
pub enum BatchErrorType {
    AnyMetaFilesDNE,
    GitRepoNotFound,
    ConfigNotFound,
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


pub fn get(globs: &Vec<String>) -> std::result::Result<Vec<std::result::Result<RetrievedFile, FileError>>, BatchError> {
    // get git root
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

    // collect queued paths
    let queued_paths = parse::parse_files_from_globs(&globs);

    // warn if no paths queued after sorting through input - likely not intentional by user
    if queued_paths.is_empty() {
        println!("warning: no files were queued")
     }

     // check that metadata file exists for all files
     check_meta_files_exist(&queued_paths)?;
    
     // get each file in queued_paths
    let retrieved_files = queued_paths.clone().into_iter().map(|file| {
        get_file(&file, &conf)
    }).collect::<Vec<std::result::Result<RetrievedFile, FileError>>>();

    Ok(retrieved_files)
}


// gets a file from storage
pub fn get_file(local_path: &PathBuf, conf: &config::Config) -> std::result::Result<RetrievedFile, FileError> {
    // get relative and absolute paths - probably don't exist, so make mutable options
    let mut relative_path = repo::get_relative_path(&PathBuf::from("."), &local_path).ok();
    let mut absolute_path = repo::absolutize_result(&local_path).ok();

    // return if is dir
    if local_path.is_dir() {
        return Err(FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::PathIsDirectory,
            error_message: None
        })
    }

    // get metadata
    let metadata = file::load(&local_path).map_err(|e| {
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::MetadataNotFound,
            error_message: Some(e.to_string())
        }
    })?;

    // get local hash 
    let local_hash = hash::get_file_hash(&local_path).clone().unwrap_or_default(); 

    // get storage data
    let storage_path = hash::get_storage_path(&conf.storage_dir, &metadata.hash);

    // check if up-to-date file is already present locally
    let outcome = 
        if !local_path.exists() || metadata.hash == String::from("") || local_hash == String::from("") || local_hash != metadata.hash {
            copy::copy(&storage_path, &local_path).map_err(|e| {
                FileError{
                    relative_path: relative_path.clone(),
                    absolute_path: absolute_path.clone(),
                    error_type: FileErrorType::FileNotCopied,
                    error_message: Some(e.to_string())
                }
            })?;
            Outcome::Copied
        }  // if file not present or not up-to-date
        else {
            Outcome::AlreadyPresent
        };

    if absolute_path.is_none() {
        // try to get relative and absolute paths again
        absolute_path = Some(local_path.canonicalize().map_err(|e| {
            FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::AbsolutePathNotFound,
                error_message: Some(e.to_string())
            }
        })?);
    }

    if relative_path.is_none() {
        relative_path = Some(repo::get_relative_path(&PathBuf::from("."), &local_path).map_err(|e| {
            FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::RelativePathNotFound,
                error_message: Some(e.to_string())
            }
        })?);
    }

    Ok(RetrievedFile {
            relative_path: relative_path.unwrap(),
            absolute_path: absolute_path.unwrap(),
            hash: metadata.hash,
            outcome: outcome,
            size: metadata.size
        }
    )
}

fn check_meta_files_exist(queued_paths: &Vec<PathBuf>) -> std::result::Result<(), BatchError> {
    // Find the first path that does not have a corresponding .dvsmeta file
    if let Some(path) = queued_paths
        .into_iter()
        .find(|p| !PathBuf::from(p.display().to_string() + ".dvsmeta").exists())
    {
        return Err(BatchError {
            error_type: BatchErrorType::AnyMetaFilesDNE,
            error_message: format!("missing for {}", path.display()),
        });
    }

    Ok(()) // If all .dvsmeta files found, return Ok
}
