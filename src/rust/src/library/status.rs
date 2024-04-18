use extendr_api::{prelude::*, Dataframe, IntoDataFrameRow, eval_string, Pairlist};
use crate::helpers::{config, hash, repo, file, parse};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, IntoDataFrameRow)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
    pub file_size: u64,
    pub file_hash: String,
    pub time_stamp: String,
    pub saved_by: String,
    pub message: String
}

pub fn dvs_status(globs: &Vec<String>) -> Result<Vec<FileStatus>> {
    let start_time = std::time::Instant::now();
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not find git repo root - make sure you're in an active git repository: \n{e}"))),
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
    // get local path relative to working directory
    let local_path_display = match repo::get_relative_path(&PathBuf::from("."), &path) {
        Ok(rel_path) => rel_path.display().to_string(),
        Err(_) => path.display().to_string(),
    };
    
    // get file info
    let metadata = file::load(&path).expect("couldn't get metadata");
    
    // assign status: not-present by default
    let mut status = String::from("out-of-sync");

    // if the file path doesn't exist, assign status to "not-present"
    if !path.exists() {status = String::from("not-present")}
    // else, the file path exists; check if up-to-date
    else {
        // get whether file was hashable and file hash
        match hash::get_file_hash(&path) {
            Some(file_hash) => {
                if file_hash == metadata.file_hash {
                    status = String::from("up-to-date")
                }
            }
            None => (),
        }; 
    }

    // assemble info intoFileStatus
    FileStatus{
        path: local_path_display,
        status,
        file_size: metadata.file_size,
        file_hash: metadata.file_hash,
        time_stamp: metadata.time_stamp,
        saved_by: metadata.saved_by,
        message: metadata.message
    }
}

