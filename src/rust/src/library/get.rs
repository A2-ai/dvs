use std::path::PathBuf;
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
    pub path: String,
    pub hash: Option<String>,
    pub outcome: String,
    pub error: Option<String>,
    pub size: Option<u64>
}



pub fn dvs_get(globs: &Vec<String>) -> Result<Vec<RetrievedFile>> {
    // get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not find git repo root - make sure you're in an active git repository: \n{e}"))),
    };

    // load the config
    let conf = match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}"))),
    };

    // collect queued paths
    let queued_paths = match parse::parse_files_from_globs(&globs) {
        Ok(paths) => paths,
        Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
    };

    // warn if no paths queued after sorting through input - likely not intentional by user
    if queued_paths.is_empty() {
        println!("warning: no files were queued")
     }

     // get each file in queued_paths
    let retrieved_files = queued_paths.clone().into_iter().map(|file| {
        get(&file, &conf)
    }).collect::<Vec<RetrievedFile>>();

    Ok(retrieved_files)
}


// gets a file from storage
pub fn get(local_path: &PathBuf, conf: &config::Config) -> RetrievedFile {
    // set error to None initially - if an error emerges, update
    let mut error: Option<String> = None;

    if local_path.is_dir() && error.is_none() {
        error = Some(format!("path is a directory"))
    }

    // get metadata
    let metadata: Option<file::Metadata> = match file::load(&local_path) {
        Ok(data) => Some(data),
        Err(e) => {
            if error.is_none() {error = Some(format!("dvs metadata file not found"))}
            println!("unable to find dvs metadata file for {}\n{e}", local_path.display());
            None
        }
    };

    // get local path relative to working directory
    let local_path_display = match repo::get_relative_path(&PathBuf::from("."), &local_path) {
        Ok(rel_path) => rel_path.display().to_string(),
        Err(_) => local_path.display().to_string(),
    };

    if error.is_some() {
        return RetrievedFile{
            path: local_path_display,
            hash: None,
            outcome: Outcome::Error.outcome_to_string(),
            error,
            size: None
        };
    }

    // get local hash 
    let local_hash_result = hash::get_file_hash(&local_path);
    let local_hash: String = match local_hash_result.clone() {
        Some(hash) => hash,
        None => String::from(""),
    }; 
    
    // get hashes to compare - can safely unwrap
    let metadata_unwrapped = metadata.unwrap();
    let metadata_hash = metadata_unwrapped.file_hash;
    let file_size = metadata_unwrapped.file_size;

    // get storage data
    let storage_path = hash::get_storage_path(&conf.storage_dir, &metadata_hash);

    // set outcome to already present by default
    let mut outcome = Outcome::AlreadyPresent;

    // check if up-to-date file is already present locally
    if !local_path.exists() || metadata_hash == String::from("") || local_hash == String::from("") || local_hash != metadata_hash {
        match copy::copy(&storage_path, &local_path) {
            Ok(_) => {
                outcome = Outcome::Copied;
            } // ok copy
            Err(e) => {
                outcome = Outcome::Error;
                error = Some(format!("file not copied"));
                println!("unable to copy file to {}\n{e}", local_path.display());
            }
        }; // match copy
    }  // if file not present or not up-to-date

    RetrievedFile {
        path: local_path_display,
        hash: Some(metadata_hash),
        outcome: outcome.outcome_to_string(),
        error,
        size: Some(file_size)
    }
} // get


