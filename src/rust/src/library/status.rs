
use crate::helpers::{config, error::{BatchError, FileError}, file, hash, outcome::Outcome, parse, repo};
use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub struct FileStatus {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub size: u64,
    pub time_stamp: String,
    pub saved_by: String,
    pub message:String,
    pub absolute_path: PathBuf,
    pub hash: String,
}

pub fn dvs_status(globs: &Vec<String>) -> std::result::Result<Vec<std::result::Result<FileStatus, FileError>>, BatchError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;

    // load the config
    config::read(&git_dir)?;
    
    let meta_paths: Vec<PathBuf> = 
        if globs.len() == 0 { // if no arguments are provided, get the status of all files in the current git repository
            parse::get_all_meta_files(&git_dir)
        } 
        else { // else, parse specifically inputted globs
            parse::parse_files_from_globs(&globs)
        };

    // get the status of each file and collect
    Ok(meta_paths.into_iter().map(|path| {
            status(&path)
        }).collect::<Vec<std::result::Result<FileStatus, FileError>>>())
} // dvs_status

fn status(local_path: &PathBuf) -> std::result::Result<FileStatus, FileError> {
    let absolute_path = file::get_absolute_path(local_path)?;

    let relative_path = repo::get_relative_path_to_wd(local_path, &absolute_path)?;    

    file::check_if_dir(local_path, &Some(relative_path), &Some(absolute_path));
    
    // get file info
    let metadata = file::load(local_path, &Some(relative_path), &Some(absolute_path))?;
            
    // assign status: not-present by default
    let outcome = 
        if !local_path.exists() {
            Outcome::NotPresent
        }
        else {
            match hash::get_file_hash(&local_path, &Some(relative_path), &Some(absolute_path)) {
                Ok(current_hash) => {
                    if current_hash == metadata.hash {
                        Outcome::UpToDate
                    }
                    else {Outcome::OutOfSync}
                }
                Err(_) => {
                    Outcome::OutOfSync
                }
            }
        };

    // assemble info into FileStatus
    Ok(FileStatus{
            relative_path,
            absolute_path,
            outcome,
            size: metadata.size,
            hash: metadata.hash,
            time_stamp: metadata.time_stamp,
            saved_by: metadata.saved_by,
            message: metadata.message,
        })
}

