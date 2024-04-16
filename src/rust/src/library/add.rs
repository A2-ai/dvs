use std::ffi::OsStr;
use std::path::PathBuf;
use extendr_api::{IntoDataFrameRow, Dataframe, eval_string, prelude::*};
use serde::Serialize;
use file_owner::{Group, PathExt};
use std::{fs, u32};
use anyhow::Context;
use crate::helpers::file::delete;
use crate::helpers::hash;
use crate::helpers::copy;
use crate::helpers::file;
use crate::helpers::ignore;
use crate::helpers::config;
use crate::helpers::repo;
use glob::glob;

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

pub fn dvs_add(files: &Vec<String>, message: &String, strict: bool) -> Result<Vec<AddedFile>> {
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not find git repo root - make sure you're in an active git repository: \n{e}"))),
    };

    // load the config
    let conf = match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}"))),
    };

    let mut queued_paths: Vec<PathBuf> = Vec::new();

    for entry in files {
        let glob = match glob(&entry) {
            Ok(paths) => paths,
            Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
        };

        for file in glob {
            match file {
                Ok(path) => {
                    match path.extension().and_then(OsStr::to_str) {
                        Some(ext) => {
                            if ext == "dvsmeta" { // avoid dvs files and .gitignore
                                println!("skipping .dvsmeta file {}", path.display());
                                continue
                            }
                        }
                        None => {} 
                    }

                    if path.file_name().and_then(OsStr::to_str) == Some(".gitignore") {
                        println!("skipping .gitignore file {}", path.display());
                        continue
                    }
                    
                    if queued_paths.contains(&path) {
                        println!("skipping repeated path: {}", path.display());
                        continue
                    }
                    queued_paths.push(path);
                },
                Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
            }
        } // for files in glob
    } // for entry in files

    if queued_paths.is_empty() {
        println!("warning: no paths queued to add to devious")
    }

    let mut added_files: Vec<AddedFile> = Vec::new();
    for file in queued_paths { // had to use for loop because add returns a result
        match add(&file, &git_dir, &conf, &message, strict) {
            Ok(file) => {
                added_files.push(file);
            }
            Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
        };
    }

    return Ok(added_files)
} // run_add_cmd

fn add(local_path: &PathBuf, git_dir: &PathBuf, conf: &config::Config, message: &String, strict: bool) -> Result<AddedFile> {
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

    // get file permissions
    let conf_mode_option: Option<u32> = match config::get_mode_u32(&conf.permissions) {
        Ok(mode) => Some(mode),
        Err(e) => {
            if error.is_none() {error = Some(format!("permissions not parsed"))}
            println!("unable to parse file permissions {} for {}\n{e}", &conf.permissions, local_path.display());
            None
        }
    };

    // get relative local path to display in struct
    let local_path_display = match repo::get_relative_path(&git_dir, &local_path) {
        Ok(rel_path) => rel_path.display().to_string(),
        Err(_) => local_path.display().to_string(),
    };

    if error.is_some() {
        return Ok(AddedFile{
            path: local_path_display, 
            hash: file_hash,
            outcome: Outcome::Error.outcome_to_string(),
            error: error,
            size: file_size
        });
    }

    // can safely unwrap storage_dir_abs and file_hash 
    let storage_dir_abs_value = storage_dir_abs.unwrap();
    let file_hash_value = file_hash.clone().unwrap();

    let conf_mode = conf_mode_option.unwrap();

    // create metadata
    let metadata = file::Metadata{
        file_hash: file_hash_value.clone(),
        file_size: file_size.unwrap(),
        time_stamp: chrono::Local::now().to_string(),
        //time_stamp: chrono::offset::Utc::now().to_string(),
        message: message.clone(),
        saved_by: user_name.unwrap()
    };

    // write metadata file
    match file::save(&metadata, &local_path) {
        Ok(_) => {},
        Err(e) => if error.is_none() {
            if strict { // return error
                return Err(extendr_api::error::Error::Other(format!("could not save metadata file for {}\n{e}", local_path.display())));
            }
            else { // print warning and put in data frame
                error = Some(String::from("could not save metadata file"));
                println!("could not save metadata file for {}\n{e}", local_path.display());
            }
        }
    };

    // Add file to gitignore
    match ignore::add_gitignore_entry(local_path) {
        Ok(_) => {},
        Err(e) => {
            if error.is_none() {
                if strict { // return error
                    return Err(extendr_api::error::Error::Other(format!("could not add .gitignore entry for {}\n{e}", local_path.display())));
                }
                else { // print warning and put in data frame
                    error = Some(String::from("could not add .gitignore entry"));
                    println!("could not add .gitignore entry for {}\n{e}", local_path.display());
                }
            }
        }
    };
    
    // get storage path
    let dest_path = hash::get_storage_path(&storage_dir_abs_value, &file_hash_value);

    // Copy the file to the storage directory if it's not already there
    let mut outcome: Outcome = Outcome::Success;
    if error.is_some() {
        outcome = Outcome::Error
    }
    else if !dest_path.exists() { // if not already copied
        // copy and get error
        error = match copy_file_to_storage_directory(local_path, &dest_path, &conf_mode, &group_name, strict) {
            Ok(error) => error,
            Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
        };
    }
    else {
        outcome = Outcome::AlreadyPresent;
    }

    return Ok(AddedFile {
        path: local_path_display,
        hash: file_hash.clone(),
        outcome: outcome.outcome_to_string(),
        error,
        size: file_size
    })
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


fn copy_file_to_storage_directory(local_path: &PathBuf, dest_path: &PathBuf, mode: &u32, group_name: &String, strict: bool) -> Result<Option<String>> {
    let mut error = None;
    match copy::copy(&local_path, &dest_path) {
        Ok(_) => {
            // set permissions
            match copy::set_file_permissions(&mode, &dest_path) {
                Ok(_) => {},
                Err(e) => {
                    if strict {
                        // remove copied file
                        fs::remove_file(&dest_path)
                        .expect(format!("could not set permissions after copying {} to {}: error deleting copied file. Delete {} manually.", 
                        local_path.display(), dest_path.display(), dest_path.display()).as_str());
                        return Err(extendr_api::error::Error::Other(format!("could not set permissions after copying {}\n{e}", local_path.display())));
                        // TODO: delete metadata file
                    }
                    else {
                        // set error
                        if error.is_none() {error = Some(String::from("could not set permissions"))}
                        println!("warning: could not set permissions for {} in storage directory\n{e}", local_path.display());
                    }
                }
            };

            if group_name != "" {
                // set group ownership
                let group = Group::from_name(group_name).with_context(|| format!("group not found: {group_name}")).unwrap();
                match dest_path.set_group(group.clone()) {
                    Ok(_) => {},
                    Err(e) => {
                        if strict {
                            // delete copied file
                            fs::remove_file(&dest_path)
                            .expect(format!("could not set group after copying {} to {}: error deleting copied file. Delete {} manually.", 
                            local_path.display(), dest_path.display(), dest_path.display()).as_str());
                            return Err(extendr_api::error::Error::Other(format!("could not set group after copying {}\n{e}", local_path.display())));
                        }
                        else {
                            // set error
                            if error.is_none() {error = Some(String::from("could not set group"))}
                            println!("warning: could not set group for {} in storage directory\n{e}", local_path.display());
                        }
                    }
                };
            }
          
        } // Ok, could copy
        Err(copy_e) => { // could not copy
            if strict {
                // delete metadata
                match file::delete(&local_path) {
                    Ok(_) => {
                        println!("deleting metadata file for {}", local_path.display());
                        return Err(extendr_api::error::Error::Other(format!("could not copy {} to storage directory\n{copy_e}", local_path.display())));
                    }
                    Err(delete_e) => {
                        return Err(extendr_api::error::Error::Other(format!("could not copy {} to storage directory\n{copy_e}\n could not delete metadatafile\n{delete_e}", local_path.display())));
                    }
                };
            } // strict
            else { // non-strict
                println!("error: could not copy {} to storage directory\n{copy_e}", local_path.display());
                if error.is_none() {error = Some(String::from("could not copy file to storage directory"))}
            }
        }
    };
    return Ok(error)
}