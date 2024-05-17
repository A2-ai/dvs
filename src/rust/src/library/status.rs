
use crate::helpers::{config, error::{BatchError, FileError, FileErrorType}, file, hash, outcome::Status, parse, repo};
use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub struct FileStatus {
    pub relative_path: Option<PathBuf>,
    pub status: Status,
    pub file_size_bytes: u64,
    pub time_stamp: String,
    pub saved_by: String,
    pub message: String,
    pub absolute_path: Option<PathBuf>,
    pub blake3_checksum: String,
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
            parse::parse_meta_files_from_globs_status(globs)
        };

    // get the status of each file and collect
    Ok(meta_paths.into_iter().map(|path| {
        status_file(&path, input_manually)
    }).collect::<Vec<std::result::Result<FileStatus, FileError>>>())
} 

fn status_file(local_path: &PathBuf, input_manually: bool) -> std::result::Result<FileStatus, FileError> {
    // info function, so just try to get abs path
    let absolute_path = file::try_to_get_abs_path(local_path);

    // info fn, so just try to get rel path
    let relative_path = file::try_to_get_rel_path(local_path);

    file::check_if_dir(local_path)?;

    // check if metadata file exists
    if !file::metadata_path(local_path).exists() {
        return Err(FileError{
            relative_path,
            absolute_path,
            error: FileErrorType::FileNotAdded,
            error_message: Some(format!("metadata file not found - add the file to dvs to get status")),
            input: local_path.clone()
        })
    }
    
    // load metadata
    let metadata = file::load(local_path)?;
            
    // assign status
    let status = 
        if !local_path.exists() {
            Status::Absent
        }
        else {
            let current_hash = hash::get_file_hash(local_path)?;

            if current_hash == metadata.blake3_checksum {Status::Current}
            else {Status::Unsynced}
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
            file_size_bytes: metadata.file_size_bytes,
            blake3_checksum: metadata.blake3_checksum,
            time_stamp: metadata.time_stamp,
            saved_by: metadata.saved_by,
            message: metadata.message,
            input
        })
}

