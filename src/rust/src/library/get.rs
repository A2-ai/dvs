use std::path::PathBuf;
use crate::helpers::{config, copy, error::{BatchError, BatchErrorType, FileError}, file, hash, outcome::Outcome, parse, repo};

#[derive(Debug)]
pub struct RetrievedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub file_size_bytes: u64,
    pub absolute_path: PathBuf,
    pub blake3_checksum: String,
}

pub fn get(globs: &Vec<String>) -> std::result::Result<Vec<std::result::Result<RetrievedFile, FileError>>, BatchError> {
    // get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;

    // load the config
    let conf = config::read(&git_dir)?;

    // check storage directory exists
    let storage_dir = config::get_storage_dir(&conf.storage_dir)?;

    for glob in globs { // for each input in globs
        let file_path = PathBuf::from(glob);
        if file_path.extension().is_some() { // if the input is an explicit file path
            if !file::metadata_path(&file_path).exists() { // and that file path doesn't have a corresponding metadata file
                return Err(BatchError { // return error
                    error: BatchErrorType::AnyMetaFilesDNE,
                    error_message: format!("missing for {}", file_path.display()),
                })
            }
        }
    }
        
    // collect queued paths
    let queued_paths = parse::parse_meta_files_from_globs_get(globs);

    // warn if no paths queued after sorting through input - likely not intentional by user
    if queued_paths.is_empty() {
        println!("warning: no files were queued")
    }

    // check that metadata file exists for all files
    file::check_meta_files_exist(&queued_paths)?;
    
    // get each file in queued_paths
    let retrieved_files = queued_paths.clone().into_iter().map(|file| {
        get_file(&file, &storage_dir, &git_dir)
    }).collect::<Vec<std::result::Result<RetrievedFile, FileError>>>();

    Ok(retrieved_files)
}


// gets a file from storage
pub fn get_file(local_path: &PathBuf, storage_dir: &PathBuf, git_dir: &PathBuf) -> std::result::Result<RetrievedFile, FileError> {
    // get temporary relative and absolute paths because they probably don't exist
    let relative_path_temp = match repo::get_relative_path_to_wd(local_path) {
        Ok(path) => Some(path),
        Err(_) => None,
    };

    let absolute_path_temp = match repo::absolutize_result(local_path) {
        Ok(path) => Some(path),
        Err(_) => None
    };

    // return if is dir
    // shouldn't error because metadata file exists has already been confirmed in batch function
    // file::check_if_dir(local_path)?; 

    // check if file in git repo
    repo::check_file_in_git_repo(local_path, git_dir)?;

    // get metadata
    let metadata = file::load(local_path)?;

    // get local hash 
    let local_hash = hash::get_file_hash(local_path).unwrap_or_default();
    // get hash from metadata
    let meta_hash = metadata.blake3_checksum;

    // get storage data
    let storage_path = hash::get_storage_path(storage_dir, &meta_hash);

    // check if most current file is already present locally
    let outcome = 
        if !local_path.exists() || meta_hash == String::from("") || local_hash == String::from("") || local_hash != meta_hash {
            copy::copy(&storage_path, local_path)?;
            Outcome::Copied
        }  // if file not present or not current
        else {
            Outcome::Present
        };

    // now that the file exists again, get info for data frame

    let absolute_path = match absolute_path_temp {
        Some(path) => path,
        None => file::get_absolute_path(local_path)?
    };

    let relative_path = match relative_path_temp {
        Some(path) => path,
        None => repo::get_relative_path_to_wd(local_path)?
    };
   
    let blake3_checksum = hash::get_file_hash(local_path)?;

    let file_size_bytes = file::get_file_size(local_path)?;

    Ok(RetrievedFile {
            relative_path,
            absolute_path,
            blake3_checksum,
            outcome,
            file_size_bytes
        }
    )
}


