use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use extendr_api::IntoDataFrameRow;
use extendr_api::Dataframe;
use extendr_api::eval_string;
use extendr_api::prelude::*;
use serde::Serialize;
use file_owner::{Group, PathExt};
use std::{fs, u32};
use anyhow::Context;
use crate::helpers::hash;
use crate::helpers::copy;
use crate::helpers::file;
use crate::helpers::ignore;
use crate::helpers::config;

#[derive(Clone, PartialEq, Serialize)]
enum Outcome {
    Success,
    AlreadyPresent,
    Error
}

impl Outcome {
    fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Success => String::from("Success"),
            Outcome::Error => String::from("Error"),
            Outcome::AlreadyPresent => String::from("Already Present")
        }
    }
}



#[derive(Clone, PartialEq, Serialize, IntoDataFrameRow)]
pub struct AddedFile {
    path: String,
    hash: Option<String>,
    outcome: String,
    error: Option<String>,
    size: Option<u64>,
}

pub fn dvs_add(files: &Vec<String>, git_dir: &PathBuf, conf: &config::Config, message: &String) -> Result<Vec<AddedFile>> {
    let mut queued_paths: Vec<PathBuf> = Vec::new();

    for file_in in files {
        let file_without_meta = file_in.replace(".dvsmeta", "");
        let file = PathBuf::from(file_without_meta);

        if queued_paths.contains(&file) {
            println!("skipping repeated path: {}", file.display());
            continue
        }

        queued_paths.push(file);
    } // for
    
    // add each file in queued_paths to storage
    let added_files = queued_paths.into_iter().map(|file| {
        add(&file, &git_dir, &conf, &message)
    }).collect::<Vec<AddedFile>>();

    return Ok(added_files)
} // run_add_cmd

fn add(local_path: &PathBuf, git_dir: &PathBuf, conf: &config::Config, message: &String) -> AddedFile {
    // set error to None by default
    let mut error: Option<String> = None;

    if error.is_none() {error = get_preliminary_errors(&local_path, &git_dir)}

    // get file hash
    let file_hash = hash::get_file_hash(&local_path);
    if file_hash.is_none() && error.is_none() {
        error = Some(String::from("hash not found"));
        println!("error: hash not found for {}", local_path.display());
    }

    // get file size
    let file_size: Option<u64> = match local_path.metadata() {
        Ok(data) => Some(data.len()),
        Err(e) => {
            if error.is_none() {
                error = Some(String::from("size not found"));
                println!("error: file size not found for {}\n{e}", local_path.display());
            }
            None
        }
    };

    // get user name
    let user_name: Option<String> = match local_path.owner().with_context(|| format!("owner not found")) {
        Ok(owner) => {
            let owner_name = match owner.name() {
                Ok(name) => Some(name.unwrap()),
                Err(e) => {
                    error = Some(String::from("owner name not found"));
                    println!("error: owner name not found for {}\n{e}", local_path.display());
                    None
                }
            };
            owner_name
        }
        Err(e) => {
            if error.is_none() {
                error = Some(String::from("owner not found"));
                println!("error: owner not found for {}\n{e}", local_path.display());
            }
            None
        }
    };

    // check group if group was specified
    let group_name = &conf.group;
    if group_name != "" {
        match Group::from_name(group_name) {
            Ok(_) => {}
            Err(e) => {
                if error.is_none() {error = Some(String::from("group not found"))}
                println!("group {group_name} not found for {}\n{e}", local_path.display());
            }
        };
    }

    // now see if file can be added
    let storage_dir_abs: Option<PathBuf> = match conf.storage_dir.canonicalize() {
        Ok(path) => Some(path),
        Err(e) => {
            if error.is_none() {error = Some(String::from("storage directory not found"))}
            println!("storage directory {} not found\n{e}", conf.storage_dir.display());
            None
        }
    };

    if error.is_some() {
        return AddedFile{
            path: local_path.display().to_string(), 
            hash: file_hash,
            outcome: Outcome::Error.outcome_to_string(),
            error: error,
            size: file_size
        };
    }

    // can safely unwrap storage_dir_abs and file_hash 
    let storage_dir_abs_value = storage_dir_abs.unwrap();
    let file_hash_value = file_hash.clone().unwrap();
    
    // get storage path
    let dest_path = hash::get_storage_path(&storage_dir_abs_value, &file_hash_value);

    // Copy the file to the storage directory if it's not already there
    let mut outcome: Outcome = Outcome::Success;
    if !dest_path.exists() {
        // copy
        copy_file_to_storage_directory(local_path, &dest_path, &conf.permissions, &group_name);
    }
    else {
        outcome = Outcome::AlreadyPresent;
    }

    // create metadata
    let metadata = file::Metadata{
        file_hash: file_hash_value,
        file_size: file_size.unwrap(),
        time_stamp: chrono::offset::Local::now().to_string(),
        message: message.clone(),
        saved_by: user_name.unwrap()
    };

    // write metadata file
    match file::save(&metadata, &local_path) {
        Ok(_) => {},
        Err(e) => if error.is_none() {
            error = Some(String::from("could not save metadata file"));
            println!("could not save metadata file for {}\n{e}", local_path.display());
        }
    };

    // Add file to gitignore
    match ignore::add_gitignore_entry(local_path) {
        Ok(_) => {},
        Err(e) => {
            if error.is_none() {
                error = Some(String::from("could not add .gitignore entry"));
                println!("could not save metadata file for {}\n{e}", local_path.display());
            }
        }
    };
    
    if error.is_some() {outcome = Outcome::Error}

    return AddedFile {
        path: local_path.display().to_string(),
        hash: file_hash.clone(),
        outcome: outcome.outcome_to_string(),
        error,
        size: file_size
    }
}

fn get_preliminary_errors(local_path: &PathBuf, git_dir: &PathBuf) -> Option<String> {
    // check if file exists
    match local_path.canonicalize() {
        Ok(local_path) => { // file exists
            // if file is outside of git repository
            if local_path.strip_prefix(&git_dir).unwrap() == local_path {
                println!("error: file {} not in git repository", local_path.display());
                return Some(String::from("file not in git repository"));
            }
        }
        Err(e) => { 
            println!("error: file {} not found\n{e}",local_path.display());
            return Some(String::from("file not found"));
        }
    };

    if local_path.is_dir() {
        println!("error: path {} is a directory", local_path.display());
        return Some(String::from("path is a directory"))
    }

    None
}


fn copy_file_to_storage_directory(local_path: &PathBuf, dest_path: &PathBuf, mode: &u32, group_name: &String) -> Option<String> {
    let mut error = None;
    match copy::copy(&local_path, &dest_path) {
        Ok(_) => {
            // set permissions
            match set_permissions(&mode, &dest_path) {
                Ok(_) => {},
                Err(e) => {
                    // set error
                    if error.is_none() {error = Some(String::from("could not set permissions"))}
                    println!("error: could not set permissions for {} in storage directory\n{e}", local_path.display());
                    // delete copied file
                    fs::remove_file(&dest_path)
                    .expect(format!("could not set permissions after copying {} to {}: error deleting copied file. Delete {} manually.", local_path.display(), dest_path.display(), dest_path.display()).as_str());
                }
            };

            if group_name != "" {
                // set group ownership
                let group = Group::from_name(group_name).with_context(|| format!("group not found: {group_name}")).unwrap();
                match dest_path.set_group(group.clone()) {
                    Ok(_) => {},
                    Err(e) => {
                        // set error
                        if error.is_none() {error = Some(String::from("could not set group"))}
                        println!("error: could not set group for {} in storage directory\n{e}", local_path.display());
                        // delete copied file
                        fs::remove_file(&dest_path)
                        .expect(format!("could not set group after copying {} to {}: error deleting copied file. Delete {} manually.", local_path.display(), dest_path.display(), dest_path.display()).as_str());

                    }
                };
            }
          
        } // Ok, could copy
        Err(e) => {
            println!("error: could copy {} to storage directory\n{e}", local_path.display());
            if error.is_none() {error = Some(String::from("could not copy file to storage directory"))}
        }
    };
    return error
}

fn set_permissions(mode: &u32, dest_path: &PathBuf) -> Result<()> {
    dest_path.metadata().unwrap().permissions().set_mode(*mode);
    let _file_mode = dest_path.metadata().unwrap().permissions().mode();
    let new_permissions = fs::Permissions::from_mode(*mode);
    fs::set_permissions(&dest_path, new_permissions).with_context(|| format!("unable to set permissions: {}", mode)).unwrap();
    Ok(())
}