use std::path::PathBuf;
use crate::helpers::{config, copy, hash, file, repo, parse, outcome::Outcome, error::{BatchError, FileError, BatchErrorType}};

#[derive(Debug)]
pub struct RetrievedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub size: u64,
    pub absolute_path: PathBuf,
    pub hash: String,
}

pub fn get(globs: &Vec<String>) -> std::result::Result<Vec<std::result::Result<RetrievedFile, FileError>>, BatchError> {
    // get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;

    // load the config
    let conf = config::read(&git_dir)?;

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
    // get temporary relative and absolute paths because they probably don't exist
    let relative_path_temp: Option<PathBuf> = match repo::get_relative_path_to_wd(&PathBuf::from("."), &local_path) {
        Ok(path) => Some(path),
        Err(_) => None,
    };

    let absolute_path_temp = match repo::absolutize_result(&local_path) {
        Ok(path) => Some(path),
        Err(_) => None
    };

    // return if is dir
    file::check_if_dir(local_path, &relative_path_temp, &absolute_path_temp);

    // get metadata
    let metadata = file::load(&local_path, &relative_path_temp, &absolute_path_temp)?;

    // get local hash 
    let local_hash = hash::get_file_hash(local_path, &relative_path_temp, &absolute_path_temp)?;

    // get storage data
    let storage_path = hash::get_storage_path(&conf.storage_dir, &metadata.hash);

    // check if up-to-date file is already present locally
    let outcome = 
        if !local_path.exists() || metadata.hash == String::from("") || local_hash == String::from("") || local_hash != metadata.hash {
            copy::copy(&storage_path, &local_path, &relative_path_temp, &absolute_path_temp)?;
            Outcome::Success
        }  // if file not present or not up-to-date
        else {
            Outcome::AlreadyPresent
        };

    let absolute_path = 
        if absolute_path_temp.is_none() {
            file::get_absolute_path(local_path)?
        }
        else {
            absolute_path_temp.unwrap()
        };
   
    let relative_path = 
        if relative_path_temp.is_none() {
            repo::get_relative_path_to_wd(local_path, &absolute_path)?
        }
        else {
            relative_path_temp.unwrap()
        };

    Ok(RetrievedFile {
            relative_path,
            absolute_path,
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
        .find(|dvs_path| !PathBuf::from(dvs_path.display().to_string() + ".dvsmeta").exists())
    {
        return Err(BatchError {
            error_type: BatchErrorType::AnyMetaFilesDNE,
            error_message: format!("missing for {}", path.display()),
        });
    }

    Ok(()) // If all .dvsmeta files found, return Ok
}
