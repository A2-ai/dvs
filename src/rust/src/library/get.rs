
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::ffi::OsStr;
use crate::helpers::config::Config;
use crate::helpers::copy::set_file_permissions;
use crate::helpers::file::Metadata;
use crate::helpers::hash;
use crate::helpers::copy;
use crate::helpers::file;
use extendr_api::IntoDataFrameRow;
use extendr_api::Dataframe;
use extendr_api::prelude::*;
use glob::glob;
use std::fmt::Octal;

enum Outcome {
    Copied,
    AlreadyPresent,
    Error,
    PermissionsUpdated
}

impl Outcome {
    fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Copied => String::from("Copied"),
            Outcome::AlreadyPresent => String::from("Already Present"),
            Outcome::Error => String::from("Error"),
            Outcome::PermissionsUpdated => String::from("Permissions Updated")
        }
    }
}

#[derive(IntoDataFrameRow)]
pub struct RetrievedFile {
    pub path: String,
    pub hash: Option<String>,
    pub outcome: String,
    pub error: Option<String>,
    pub size: Option<u64>

}

pub fn dvs_get(files: &Vec<String>, conf: &Config) -> Result<Vec<RetrievedFile>> {
    // parse each glob
    //let queued_paths = parse::parse_globs(globs);
    let mut queued_paths: Vec<PathBuf> = Vec::new();

    for entry in files {
        if PathBuf::from(entry).extension().and_then(OsStr::to_str).is_some() {
            let path_clean = PathBuf::from(entry.replace(".dvsmeta", ""));

            if path_clean.file_name().and_then(OsStr::to_str) == Some(".gitignore") {
                println!("skipping .gitignore file {}", path_clean.display());
                continue
            }
            
            if queued_paths.contains(&path_clean) {
                println!("skipping repeated path: {}", path_clean.display());
                continue
            }
            
            queued_paths.push(path_clean);
        }
        else {
            let glob = match glob(&entry) {
                Ok(paths) => paths,
                Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
            };
    
            for file in glob {
                match file {
                    Ok(path) => {
                        let path_clean = PathBuf::from(path.display().to_string().replace(".dvsmeta", ""));
    
                        if path_clean.file_name().and_then(OsStr::to_str) == Some(".gitignore") {
                            println!("skipping .gitignore file {}", path.display());
                            continue
                        }
                        
                        if queued_paths.contains(&path_clean) {
                            println!("skipping repeated path: {}", path_clean.display());
                            continue
                        }
                        
                        queued_paths.push(path_clean);
                    }
                    Err(e) => {
                        return Err(extendr_api::error::Error::Other(e.to_string()));
                    }
    
                } // match file
            } // for file in glob
        }
    } // for entry in files

    if queued_paths.is_empty() {
        println!("warning: no files were queued")
     }

    let retrieved_files = queued_paths.clone().into_iter().map(|file| {
        get(&file, &conf)
    }).collect::<Vec<RetrievedFile>>();

    Ok(retrieved_files)
}


// gets a file from storage
pub fn get(local_path: &PathBuf, conf: &Config) -> RetrievedFile {
    
    let mut error: Option<String> = None;

    if local_path.is_dir() && error.is_none() {
        error = Some(format!("path is a directory"))
    }

    // get metadata
    let metadata: Option<Metadata> = match file::load(&local_path) {
        Ok(data) => Some(data),
        Err(e) => {
            if error.is_none() {error = Some(format!("dvs metadata file not found"))}
            println!("unable to find dvs metadata file for {}\n{e}", local_path.display());
            None
        }
    };

    if error.is_some() {
        return RetrievedFile{
            path: local_path.display().to_string(),
            hash: None,
            outcome: Outcome::Error.outcome_to_string(),
            error,
            size: None
        };
    }

    // get local hash 
    let local_hash_result = hash::get_file_hash(&local_path);
    let local_hash: String = match local_hash_result.clone() {
        Some(hash) => hash,
        None => String::from(""),
    }; 
    
    let mut outcome = Outcome::AlreadyPresent;

    // get hashes to compare - can safely unwrap
    let metadata_unwrapped = metadata.unwrap();
    let metadata_hash = metadata_unwrapped.file_hash;
    let file_size = metadata_unwrapped.file_size;


    // get storage data
    let storage_path = hash::get_storage_path(&conf.storage_dir, &metadata_hash);

    // check if up-to-date file is already present locally
    if !local_path.exists() || metadata_hash == String::from("") || local_hash == String::from("") || local_hash != metadata_hash {
        match copy::copy(&storage_path, &local_path) {
            Ok(_) => {
                match set_file_permissions(&conf.mode, &local_path) {
                    Ok(_) => {
                        outcome = Outcome::Copied;
                    }
                    Err(e) => {
                        // TODO: delete file
                        outcome = Outcome::Error;
                        error = Some(format!("permissions not set"));
                        println!("unable to set permissions for  {}\n{e}", local_path.display());
                    }
                }; // match set_file_permissions(
            } // ok copy
            Err(e) => {
                outcome = Outcome::Error;
                error = Some(format!("file not copied"));
                println!("unable to copy file to {}\n{e}", local_path.display());
            }
        };
    } // match copy
    else { // else already present and up to date
        // if permissions don't match, update them
        match local_path.metadata() {
            Ok(metadata) => {
                if metadata.permissions().mode() & 0o777 != conf.mode { // need to do bitwise & for mysterious reasons
                    println!("original permissions: {:o}, new permissions: {:o}", metadata.permissions().mode() & 0o777, conf.mode);
                    match set_file_permissions(&conf.mode, &local_path) {
                        Ok(_) => {
                            outcome = Outcome::PermissionsUpdated;
                        }
                        Err(e) => {
                            // TODO: delete file
                            if error.is_none() {
                                outcome = Outcome::Error;
                                error = Some(format!("permissions not set"));
                            }
                            println!("unable to set permissions for {}\n{e}", local_path.display());
                        }
                    }; // match set_file_permissions
                }
            }
            Err(e) => {
                if error.is_none() {
                    outcome = Outcome::Error;
                    error = Some(format!("metadata inaccessible"));
                }
                println!("metadata inaccessible for {}\n{e}", local_path.display());
            }
        };
        
       
    }

    RetrievedFile {
        path: local_path.display().to_string(),
        hash: Some(metadata_hash),
        outcome: outcome.outcome_to_string(),
        error,
        size: Some(file_size)
    }
} // get