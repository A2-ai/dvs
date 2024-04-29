use crate::helpers::{config, copy, error::{BatchError, BatchErrorType, FileError}, file, hash, ignore, outcome::Outcome, parse, repo};
use std::{fs, path::PathBuf, u32};
use file_owner::Group;

#[derive(Clone, PartialEq)]
pub struct AddedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub size: u64,
    pub hash: String,
    pub absolute_path: PathBuf,
}

pub fn add(globs: &Vec<String>, message: &String, strict: bool) -> std::result::Result<Vec<std::result::Result<AddedFile, FileError>>, BatchError> {
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
    let queued_paths = parse::parse_files_from_globs(&globs);

    // warn if no paths queued after sorting through input - likely not intentional by user
    if queued_paths.is_empty() {
        println!("warning: no paths queued to add to devious")
    }

    // return error if any files don't exist
    queued_paths.iter().map(|file| {
       file.canonicalize().map_err(|e|
            BatchError{
                error_type: BatchErrorType::AnyFilesDNE,
                error_message: format!("{} not found: {e}", file.display())
            })
    }).collect::<std::result:: Result<Vec<PathBuf>, BatchError>>()?;

    Ok(queued_paths.into_iter().map(|file| {
        add_file(&file, &git_dir, &group, &storage_dir, &permissions, &message, strict)
    }).collect::<Vec<std::result::Result<AddedFile, FileError>>>())
}

fn add_file(local_path: &PathBuf, git_dir: &PathBuf, group: &Option<Group>, storage_dir: &PathBuf, permissions: &u32, message: &String, strict: bool) -> std::result::Result<AddedFile, FileError> {
    // get absolute path
    let absolute_path = file::get_absolute_path(local_path)?;

    // get relative path
    let relative_path = repo::get_relative_path_to_wd(local_path)?;

    // check if file in git repo
    repo::check_file_in_git_repo(local_path, &git_dir, &relative_path, &absolute_path)?;

    // error if file is a directory
    file::check_if_dir(local_path)?;

    // get file hash
    let hash = hash::get_file_hash(local_path)?;

    // get file size
    let size = file::get_file_size(local_path)?;

    // get user name
    let user_name: String = file::get_user_name(&local_path)?;

    // create metadata
    let metadata = file::Metadata{
        hash: hash.clone(),
        size,
        time_stamp: chrono::Local::now().to_string(),
        //time_stamp: chrono::offset::Utc::now().to_string(),
        message: message.clone(),
        saved_by: user_name
    };

    // write metadata file
    file::save(&metadata, &local_path)?;

    // Add file to gitignore
    ignore::add_gitignore_entry(local_path)?;
    
    // get storage path
    let storage_path = hash::get_storage_path(&storage_dir, &hash);
    
    // copy
    let outcome = 
        if !storage_path.exists() { // if not already copied
            if let Err(e) = copy_file_to_storage_directory(&local_path, &storage_path, &permissions, &group) {
                if strict {
                    // remove metadata file
                    let _ = fs::remove_file(PathBuf::from(local_path.display().to_string() + ".dvsmeta"));
                    // remove copied file from storage directory
                    let _ = fs::remove_file(storage_path);
                }
                return Err(e)
            };
            Outcome::Success
        }
        else {
            Outcome::AlreadyPresent
        };

    Ok(AddedFile{
            relative_path,
            absolute_path,
            outcome,
            size,
            hash
        }
    )
}

fn copy_file_to_storage_directory(local_path: &PathBuf, storage_path: &PathBuf, permissions: &u32, group: &Option<Group>) -> std::result::Result<(), FileError> {
    // copy
    copy::copy(local_path, storage_path)?;

    // set file permissions
    copy::set_file_permissions(permissions, storage_path)?;

    // set group (if specified)
    Ok(copy::set_group(group, storage_path)?)
}