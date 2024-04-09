use crate::helpers::repo;
use std::path::PathBuf;
use crate::helpers::config;
use std::fs::create_dir;
use path_absolutize::Absolutize;
use file_owner::Group;
use anyhow::{anyhow, Context, Result};

pub fn dvs_init(storage_dir: &PathBuf, octal_permissions: &i32, group_name: &str) -> Result<()> { 
    // Get git root
   let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).with_context(|| "could not find git repo root - make sure you're in an active git repository")?;

    // get absolute path, but don't check if it exists yet
    let storage_dir_abs = PathBuf::from(storage_dir.absolutize().unwrap());
    
    // check if directory exists
    if !storage_dir_abs.exists() { // if storage directory doesn't exist
        println!("storage directory doesn't exist\ncreating storage directory...");
        // create storage dir
        create_dir(&storage_dir_abs).with_context(|| format!("failed to create storage directory: {}", storage_dir.display()))?;
    } // if
    else { // else, storage directory exists
        if !storage_dir.is_dir() {
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
                //json
                return Err(anyhow!("unable to check if directory is empty: {}", e))
            }
        }
    } // else

    let mut group_name_for_config = String::from(group_name);

    if group_name != "" {
        Group::from_name(group_name).with_context(|| format!("group not found: {group_name}"))?;
    }
    else {
        group_name_for_config = String::from("");
    }

    let mode = match u32::from_str_radix(&octal_permissions.to_string(), 8) {
        Ok(mode) => mode,
        Err(e) => return Err(anyhow!("could not convert permissions to unsigned integer \n{e}"))
    };

    // write config
    config::write(
        &config::Config{
            storage_dir: storage_dir_abs.clone(), 
            permissions: octal_permissions.clone(),
            group: group_name_for_config
        }, 
        &git_dir)
        .with_context(|| "unable to write configuration file")?;
    
    println!("initialized storage directory: {}", storage_dir.display());
    Ok(())
    // json: success
}
