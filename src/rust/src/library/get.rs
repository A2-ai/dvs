use std::{fmt, path::PathBuf};
use crate::helpers::{config, copy, hash, file, repo, parse};
use extendr_api::{Dataframe, IntoDataFrameRow, prelude::*};

#[derive(PartialEq)]
enum Outcome {
    Copied,
    AlreadyPresent,
    Error,
}

impl Outcome {
    fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Copied => String::from("Copied"),
            Outcome::AlreadyPresent => String::from("Already Present"),
            Outcome::Error => String::from("Error"),
        }
    }
}

#[derive(IntoDataFrameRow)]
pub struct RetrievedFile {
    relative_path: String,
    outcome: String,
    error: Option<String>,
    size: Option<u64>,
    absolute_path: Option<String>,
    hash: Option<String>,
}



// #[derive(Clone, PartialEq)]
// enum GetFileErrorType {
//    PathIsDirectory,
//    MetadataNotFound,
//    RelativePathNotFound,
//    FileNotCopied,
//    AbsolutePathNotFound
// }

// impl GetFileErrorType {
//     fn get_file_error_type_to_string(&self) -> String {
//         match self {
//             GetFileErrorType::PathIsDirectory => String::from("path is a directory"),
//             GetFileErrorType::MetadataNotFound => String::from("metadata file not found"),
//             GetFileErrorType::RelativePathNotFound => String::from("relative path not found"),
//             GetFileErrorType::FileNotCopied => String::from("file not copied"),
//             GetFileErrorType::AbsolutePathNotFound => String::from("absolute path not found"),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct GetFileError {
//     pub error_type: String,
//     pub error_message:String,
// }

// impl fmt::Display for GetFileError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}: {}", self.error_type, self.error_message)
//     }
// }

// impl std::error::Error for GetFileError {}


#[derive(Clone, PartialEq)]
enum GetErrorType {
    AnyMetaFilesDNE,
    GitRepoNotFound,
    ConfigNotFound,
}

impl GetErrorType {
    fn get_error_type_to_string(&self) -> String {
        match self {
            GetErrorType::AnyMetaFilesDNE => String::from("metadata file not found for at least one file"),
            GetErrorType::GitRepoNotFound => String::from("git repository not found"),
            GetErrorType::ConfigNotFound => String::from("configuration file not found"),
        }
    }
}

#[derive(Debug)]
pub struct GetError {
    pub error_type: String,
    pub error_message:String,
}

impl fmt::Display for GetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.error_message)
    }
}

impl std::error::Error for GetError {}


pub fn dvs_get(globs: &Vec<String>) -> std::result::Result<Vec<RetrievedFile>, GetError> {
    // get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).map_err(|e|
        GetError{
            error_type: GetErrorType::GitRepoNotFound.get_error_type_to_string(), 
            error_message: format!("could not find git repo root; make sure you're in an active git repository: {e}")
        }
    )?;

    // load the config
    let conf = config::read(&git_dir).map_err(|e|
        GetError{
            error_type: GetErrorType::ConfigNotFound.get_error_type_to_string(), 
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
        get(&file, &conf)
    }).collect::<Vec<RetrievedFile>>();

    Ok(retrieved_files)
}


// gets a file from storage
pub fn get(local_path: &PathBuf, conf: &config::Config) -> RetrievedFile {
     // get local path relative to working directory
     // might not exist
     let unknown = "unknown".to_string();
     let mut relative_path = match repo::get_relative_path(&PathBuf::from("."), &local_path) {
        Ok(rel_path) => rel_path.display().to_string(),
        Err(_) => unknown.clone(),
    };

    // get absolute path
    // might not exist
    let mut absolute_path = match repo::absolutize_result(&local_path) {
            Ok(path) => Some(path.display().to_string()),
            Err(_) => None,
    };

    // return if is dir
    if local_path.is_dir() {
        return RetrievedFile{
            relative_path,
            absolute_path,
            hash: None,
            outcome: Outcome::Error.outcome_to_string(),
            error: Some(format!("path is a directory: {}", local_path.display())),
            size: None
        };
    }

    // get metadata
    let metadata = match file::load(&local_path) {
        Ok(data) => data,
        Err(_e) => {
            return RetrievedFile{
                relative_path,
                absolute_path,
                hash: None,
                outcome: Outcome::Error.outcome_to_string(),
                error: Some(format!("metadata file not found for {}", local_path.display())),
                size: None
            };
        }
    };

    // get local hash 
    let local_hash = hash::get_file_hash(&local_path).clone().unwrap_or_default(); 

    // get storage data
    let storage_path = hash::get_storage_path(&conf.storage_dir, &metadata.hash);

    // check if up-to-date file is already present locally
    let outcome = 
        if !local_path.exists() || metadata.hash == String::from("") || local_hash == String::from("") || local_hash != metadata.hash {
            if let Err(_e) = copy::copy(&storage_path, &local_path) {
                return RetrievedFile{
                    relative_path,
                    absolute_path,
                    hash: None,
                    outcome: Outcome::Error.outcome_to_string(),
                    error: Some(format!("file not copied: {}", local_path.display())),
                    size: None
                };
            } // if error
            Outcome::Copied
        }  // if file not present or not up-to-date
        else {
            Outcome::AlreadyPresent
        };

    if absolute_path.is_none() || relative_path == unknown {
        // try to get relative and absolute paths again
        absolute_path = match local_path.canonicalize() {
            Ok(path) => Some(path.display().to_string()),
            Err(_) => {
                return RetrievedFile{
                    relative_path,
                    absolute_path,
                    hash: None,
                    outcome: Outcome::Error.outcome_to_string(),
                    error: Some(format!("relative path not found")),
                    size: None
                }
            }
        };

        relative_path = match repo::get_relative_path(&PathBuf::from("."), &local_path) {
            Ok(rel_path) => rel_path,
            Err(_) => {
                return RetrievedFile{
                    relative_path: local_path.display().to_string(),
                    absolute_path,
                    hash: None,
                    outcome: Outcome::Error.outcome_to_string(),
                    error: Some(format!("relative path not found")),
                    size: None
                }
            },
        }.display().to_string();
    }
    

    RetrievedFile {
        relative_path,
        absolute_path,
        hash: Some(metadata.hash),
        outcome: outcome.outcome_to_string(),
        error: None,
        size: Some(metadata.size)
    }
}

fn check_meta_files_exist(queued_paths: &Vec<PathBuf>) -> std::result::Result<(), GetError> {
    // Find the first path that does not have a corresponding .dvsmeta file
    if let Some(path) = queued_paths
        .into_iter()
        .find(|p| !PathBuf::from(p.display().to_string() + ".dvsmeta").exists())
    {
        return Err(GetError {
            error_type: GetErrorType::AnyMetaFilesDNE.get_error_type_to_string(),
            error_message: format!("missing for {}", path.display()),
        });
    }

    Ok(()) // If all .dvsmeta files found, return Ok
}
