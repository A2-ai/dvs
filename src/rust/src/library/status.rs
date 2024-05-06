
use crate::helpers::{config, error::{BatchError, FileError}, file, hash, outcome::Status, parse, repo};
use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub struct FileStatus {
    pub relative_path: Option<PathBuf>,
    pub status: Status,
    pub size: u64,
    pub time_stamp: String,
    pub saved_by: String,
    pub message: String,
    pub absolute_path: Option<PathBuf>,
    pub hash: String,
    pub input: Option<PathBuf>
}

pub fn status(globs: &Vec<String>) -> std::result::Result<Vec<std::result::Result<FileStatus, FileError>>, BatchError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;

    // load the config
    config::read(&git_dir)?;

    let input_manually: bool;
    
    let meta_paths: Vec<PathBuf> = 
        if globs.contains(&String::from("")) && globs.len() == 1 { // if no arguments are provided, get the status of all files in the current git repository
            input_manually = false;
            parse::get_all_meta_files(&git_dir)
        } 
        else { // else, parse specifically inputted globs
            input_manually = true;
            parse::parse_meta_files_from_globs(&globs)
        };

    // get the status of each file and collect
    Ok(meta_paths.into_iter().map(|path| {
        status_file(&path, input_manually)
    }).collect::<Vec<std::result::Result<FileStatus, FileError>>>())
} // dvs_status

fn status_file(local_path: &PathBuf, input_manually: bool) -> std::result::Result<FileStatus, FileError> {
    let metadata_path_abs = file::metadata_path(local_path);

    // work around because while metadata file might exist, file itself may not
    let absolute_path = match file::get_absolute_path(&metadata_path_abs) {
        Ok(path) => Some(file::path_without_metadata(&path)),
        Err(_) => None,
    };

    let relative_path = match repo::get_relative_path_to_wd(&metadata_path_abs) {
        Ok(path) => Some(file::path_without_metadata(&path)),
        Err(_) => None,
    };

    file::check_if_dir(local_path)?;
    
    // get file info
    let metadata = file::load(local_path)?;
            
    // assign status
    let status = 
        if !local_path.exists() {
            Status::Absent
        }
        else {
            match hash::get_file_hash(&local_path) {
                Ok(current_hash) => {
                    if current_hash == metadata.hash {
                        Status::Current
                    }
                    else {Status::Unsynced}
                }
                Err(_) =>  Status::Unsynced,
            }
        };

    let input = 
        if input_manually {
            Some(local_path.clone())
        }
        else {
            None
        };

    // assemble info into FileStatus
    Ok(FileStatus{
            relative_path,
            absolute_path,
            status,
            size: metadata.size,
            hash: metadata.hash,
            time_stamp: metadata.time_stamp,
            saved_by: metadata.saved_by,
            message: metadata.message,
            input
        })
}

