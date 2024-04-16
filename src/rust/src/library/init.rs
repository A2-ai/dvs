use crate::helpers::{repo, config};
use std::{ffi::OsStr, fs::create_dir, path::PathBuf};
use path_absolutize::Absolutize;
use file_owner::Group;
use anyhow::{anyhow, Context, Result};

pub fn dvs_init(storage_dir: &PathBuf, octal_permissions: &i32, group_name: &str) -> Result<()> { 
    // Get git root
   let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).with_context(|| "could not find git repo root - make sure you're in an active git repository")?;

    // get absolute path, but don't check if it exists yet
    let storage_dir_abs = PathBuf::from(storage_dir.absolutize().unwrap());
    
    if storage_dir_abs.extension().and_then(OsStr::to_str).is_some() {
        println!("warning: file name inputted as storage directory. Is this intentional?")
    }
    
    // create storage directory if it doesn't exist
    if !storage_dir_abs.exists() { 
        println!("storage directory doesn't exist\ncreating storage directory...");
        // create storage dir
        create_dir(&storage_dir_abs).with_context(|| format!("failed to create storage directory: {}", storage_dir.display()))?;
    } 
    else { // else, storage directory exists
        if !storage_dir_abs.is_dir() {
            return Err(anyhow!("{} is not a directory", storage_dir.display()))
        }

        println!("storage directory already exists");

        //  Warn if storage dir is not empty
        match repo::is_directory_empty(&storage_dir_abs) {
            Ok(empty) => {
                if !empty {
                    println!("warning: storage directory is not empty")
                }
            }
            Err(e) => {
                return Err(anyhow!("unable to check if directory is empty: {}", e))
            }
        }
    } // else

    

    // warn if storage directory is in git repo
    match repo::get_relative_path(&git_dir, &storage_dir_abs) {
        // if getting relative path between git_dir and storage_dir was successful, 
        // the storage dir is in the repo => sensitive files will be uploaded to git
        Ok(_) => {println!("warning: the storage directory is located in the git repo directory.\nFiles added to the storage directory will be uploaded directly to git, subverting the purpose of devious.")}
        Err(_) => {}
    }

    // check group exists
    if group_name != "" {
        Group::from_name(group_name).with_context(|| format!("group not found: {group_name}"))?;
    }

    // check permissions are convertible to u32
    match u32::from_str_radix(&octal_permissions.to_string(), 8) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!("could not convert permissions to unsigned integer \n{e}"))
    };

    // write config
    config::write(
        &config::Config{
            storage_dir: storage_dir_abs.clone(), 
            permissions: octal_permissions.clone(),
            group: group_name.to_string()
        }, 
        &git_dir)
        .with_context(|| "unable to write configuration file")?;
    
    println!("initialized storage directory: {}", storage_dir.display());
    Ok(())
}
