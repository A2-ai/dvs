use crate::helpers::{config, copy, error::{BatchError, BatchErrorType, FileError}, file, hash, ignore, outcome::Outcome, repo};
use std::{fs, path::PathBuf, u32};
use chrono:: Utc;
use file_owner::Group;

#[derive(Clone, Debug, PartialEq)]
pub struct AddedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub file_size_bytes: u64,
    pub blake3_checksum: String,
    pub absolute_path: PathBuf,
}

pub fn add(files: &Vec<PathBuf>, message: &String, strict: bool) -> std::result::Result<Vec<std::result::Result<AddedFile, FileError>>, BatchError> {
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;

    // load the config
    let conf = config::read(&git_dir)?;

    // get group, check if specified
    let group = config::get_group(&conf.group)?;
        
    // check storage directory exists
    let storage_dir = config::get_storage_dir(&conf.storage_dir)?;

    // get file permissions
    let permissions = config::get_mode_u32(&conf.permissions)?;

    // collect paths out of input - sort through globs/explicitly-named files
    //let queued_paths = parse::parse_files_from_globs_add(&globs);

    // warn if no paths queued after sorting through input - likely not intentional by user
    if files.is_empty() {
        println!("warning: no paths queued to add to dvs")
    }

    // return error if any files don't exist
    files.iter().map(|file| {
       file.canonicalize().map_err(|e|
            BatchError{
                error: BatchErrorType::AnyFilesDNE,
                error_message: format!("{} not found: {e}", file.display())
            })
    }).collect::<std::result:: Result<Vec<PathBuf>, BatchError>>()?;

    Ok(files.into_iter().map(|file| {
        add_file(&file, &git_dir, &group, &storage_dir, &permissions, &message, strict)
    }).collect::<Vec<std::result::Result<AddedFile, FileError>>>())
}

fn add_file(local_path: &PathBuf, git_dir: &PathBuf, group: &Option<Group>, storage_dir: &PathBuf, permissions: &u32, message: &String, strict: bool) -> std::result::Result<AddedFile, FileError> {
    // get absolute path
    let absolute_path = file::get_absolute_path(local_path)?;

    // get relative path
    let relative_path = repo::get_relative_path_to_wd(local_path)?;

    // error if file is a directory
    file::check_if_dir(local_path)?;

    // get file hash
    let blake3_checksum = hash::get_file_hash(local_path)?;

    // if file already added and current, no-op
    if let Ok(metadata) = file::load(local_path) { // check if already added
        if blake3_checksum == metadata.blake3_checksum { // check if current
            return Ok(AddedFile { // no-op
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                outcome: Outcome::Present,
                file_size_bytes: metadata.file_size_bytes,
                blake3_checksum: metadata.blake3_checksum,
            });
        }
    }
    // else, file not added already

    // check if file in git repo
    repo::check_file_in_git_repo(local_path, git_dir)?;

    // get file size
    let file_size_bytes = file::get_file_size(local_path)?;

    // get user name
    let user_name: String = file::get_user_name(local_path)?;

    // create metadata
    let metadata = file::Metadata{
        blake3_checksum: blake3_checksum.clone(),
        file_size_bytes,
        time_stamp: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        message: message.clone(),
        saved_by: user_name
    };

    // write metadata file
    file::save(&metadata, local_path)?;

    // Add file to gitignore
    ignore::add_gitignore_entry(local_path)?;
    
    // get storage path
    let storage_path = hash::get_storage_path(storage_dir, &blake3_checksum);
    
    // copy
    let outcome = 
        if !storage_path.exists() { // if not already copied
            if let Err(e) = copy::copy_file_to_storage_directory(local_path, &storage_path, permissions, group) {
                if strict {
                    // remove metadata file
                    let _ = fs::remove_file(file::metadata_path(local_path));
                    // remove copied file from storage directory
                    let _ = fs::remove_file(storage_path);
                }
                return Err(e)
            };
            Outcome::Copied
        }
        else {
            Outcome::Present
        };

    Ok(AddedFile{
            relative_path,
            absolute_path,
            outcome,
            file_size_bytes,
            blake3_checksum
        }
    )
}


