
use crate::helpers::{config, error::{BatchError, FileError, FileErrorType}, file, hash, outcome::Status, repo};
use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub struct FileStatus {
    pub relative_path: Option<PathBuf>,
    pub status: Status,
    pub size: u64,
    pub add_time: String,
    pub saved_by: String,
    pub message: String,
    pub absolute_path: Option<PathBuf>,
    pub blake3_checksum: String
}

pub fn status(files: &Vec<String>) -> std::result::Result<Vec<std::result::Result<FileStatus, FileError>>, BatchError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;

    // load the config
    config::read(&git_dir)?;
    
    // parsing before entering fxn now
    //let meta_paths: Vec<PathBuf> = parse::parse_files_from_globs_status(globs)?;

    // get the status of each file and collect
    Ok(files.into_iter().map(|path| {
        status_file(&PathBuf::from(path))
    }).collect::<Vec<std::result::Result<FileStatus, FileError>>>())
} 

fn status_file(local_path: &PathBuf) -> std::result::Result<FileStatus, FileError> {
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

    // assemble info into FileStatus
    Ok(FileStatus{
            relative_path,
            absolute_path,
            status,
            size: metadata.size,
            blake3_checksum: metadata.blake3_checksum,
            add_time: metadata.add_time,
            saved_by: metadata.saved_by,
            message: metadata.message
        })
}

