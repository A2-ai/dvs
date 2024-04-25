use extendr_api::{prelude::*, Dataframe, IntoDataFrameRow, eval_string, Pairlist};
use path_absolutize::Absolutize;
use crate::helpers::{config, hash, repo, file, parse};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, IntoDataFrameRow)]
pub struct FileStatus {
    pub relative_path: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
    pub file_size: Option<u64>,
    pub time_stamp: Option<String>,
    pub saved_by: Option<String>,
    pub message: Option<String>,
    pub absolute_path: String,
    pub file_hash: Option<String>,
}

pub fn dvs_status(globs: &Vec<String>) -> Result<Vec<FileStatus>> {
    let start_time = std::time::Instant::now();
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not load configuration file, i.e. no dvs.yaml in directory; be sure to initiate devious: {e}"))),
    };

    // load the config
    match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}"))),
    };

    // initialize struct for each file's status and such
    let mut status_log: Vec<FileStatus> = Vec::new();

    // vector of files
    let mut meta_paths: Vec<PathBuf> = Vec::new();

    // if no arguments are provided, get the status of all files in the current git repository
    if globs.len() == 0 {
        // get meta files
       meta_paths = [meta_paths, parse::get_all_meta_files(&git_dir)].concat();
    } // if doing all files

    else { // else, parse specifically inputted globs
        meta_paths = parse::parse_files_from_globs(&globs);
    } // else specific globs

    // warn if no paths queued - likely not intentional by user
    if meta_paths.is_empty() {return Ok(status_log)}

    // get the status of each file and collect
    status_log = meta_paths.into_iter().map(|path| {
        status(&path)
    }).collect::<Vec<FileStatus>>();

    println!("Time elapsed: {:?}", start_time.elapsed());
    Ok(status_log)
} // dvs_status

fn status(path: &PathBuf) -> FileStatus {
    let absolute_path = path.absolutize().unwrap().to_str().unwrap().to_string();
    // get local path relative to working directory
    let relative_path = match repo::get_relative_path(&PathBuf::from("."), &path) {
        Ok(rel_path) => Some(rel_path.display().to_string()),
        Err(_) => {
            return FileStatus{
                relative_path: None,
                absolute_path,
                status: None,
                file_size: None,
                file_hash: None,
                time_stamp: None,
                saved_by: None,
                message: None,
                error: Some(format!("relative path not found for {}", path.display()))
            }
        },
    };

    if path.is_dir() {
        return FileStatus{
            relative_path,
            absolute_path,
            status: None,
            file_size: None,
            file_hash: None,
            time_stamp: None,
            saved_by: None,
            message: None,
            error: Some(format!("path is a directory"))
        }
    }
    
    // get file info
    let metadata = match file::load(&path) {
        Ok(data) => data,
        Err(_) => {
            return FileStatus{
                relative_path,
                absolute_path,
                status: None,
                file_size: None,
                file_hash: None,
                time_stamp: None,
                saved_by: None,
                message: None,
                error: Some(format!("metadata file not found"))
            }
        }
    };
            
    // assign status: not-present by default
    let status = 
        if !path.exists() {
            String::from("not-present")
        }
        else {
            match hash::get_file_hash(&path) {
                Some(current_hash) => {
                    if current_hash == metadata.hash {
                        String::from("up-to-date")
                    }
                    else {String::from("out-of-sync")}
                }
                None => {
                    String::from("out-of-sync")
                }
            }
        };

    // assemble info into FileStatus
    FileStatus{
        relative_path,
        absolute_path,
        status: Some(status),
        file_size: Some(metadata.size),
        file_hash: Some(metadata.hash),
        time_stamp: Some(metadata.time_stamp),
        saved_by: Some(metadata.saved_by),
        message: Some(metadata.message),
        error: None,
    }
}

