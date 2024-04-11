use std::ffi::OsStr;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use extendr_api::prelude::*;
use extendr_api::IntoDataFrameRow;
use extendr_api::eval_string;
use extendr_api::Pairlist;
use extendr_api::Dataframe;
use crate::helpers::config;
use crate::helpers::hash;
use crate::helpers::repo;
use crate::helpers::file;
use crate::helpers::parse;
use glob::glob;


#[derive(Serialize, Deserialize, PartialEq, Debug, IntoDataFrameRow)]
pub struct JsonFileResult {
    pub path: String,
    pub status: String,
    pub file_size: u64,
    pub file_hash: String,
    pub time_stamp: String,
    pub saved_by: String,
    pub message: String
}

pub fn dvs_status(files: &Vec<String>) -> Result<Vec<JsonFileResult>> {
    let start_time = std::time::Instant::now();
    // Get git root
    let git_dir = match repo::get_nearest_repo_dir(&PathBuf::from(".")) {
        Ok(git_dir) => git_dir,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not find git repo root - make sure you're in an active git repository: \n{e}"))),
    };

    // load the config
    match config::read(&git_dir) {
        Ok(conf) => conf,
        Err(e) => return Err(extendr_api::error::Error::Other(format!("could not load configuration file - no dvs.yaml in directory - be sure to initiate devious: \n{e}"))),
    };

    // struct for each file's status and such
    let mut json_logger: Vec<JsonFileResult> = Vec::new();

    // vector of files
    let mut meta_paths: Vec<PathBuf> = Vec::new();

    // if no arguments are provided, get the status of all files in the current git repository
    if files.len() == 0 {
        // get meta files
       meta_paths = [meta_paths, parse::get_all_meta_files(&git_dir)].concat();
    } // if doing all files

    else {
        //meta_paths = [meta_paths, parse::parse_globs(files)].concat();

        for entry in files {
            let glob = match glob(&entry) {
                Ok(paths) => {paths},
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
                        
                        if meta_paths.contains(&path_clean) {
                            println!("skipping repeated path: {}", path_clean.display());
                            continue
                        }
                        
                        meta_paths.push(path_clean);
                    }
                    Err(e) => {
                        return Err(extendr_api::error::Error::Other(e.to_string()));
                    }
                } // match file
            } // for file in glob
    } // for entry in files
    } // else specific files

    if meta_paths.is_empty() {return Ok(json_logger)}

    json_logger  = meta_paths.into_iter().map(|path| {
        // get relative path
        // get relative local path to display in struct
        let rel_path = repo::get_relative_path(&PathBuf::from("."), &path).expect("couldn't get relative path");
        
        // get file info
        let metadata = file::load(&path).expect("couldn't get metadata");
        
        // assign status: not-present by default
        let mut status = String::from("out-of-date");
        // if the file path doesn't exist assign status to "not-present"
        if !path.exists() {status = String::from("not-present")}
        else {
            // get whether file was hashable and file hash
            match hash::get_file_hash(&path) {
                Some(file_hash) => {
                    if file_hash == metadata.file_hash {
                        status = String::from("up-to-date")
                    }
                }
                None => (),
            }; 
        }
       
        // assemble info into JsonFileResult
        JsonFileResult{
            path: rel_path.display().to_string(),
            status: status,
            file_size: metadata.file_size,
            file_hash: metadata.file_hash,
            time_stamp: metadata.time_stamp,
            saved_by: metadata.saved_by,
            message: metadata.message
        }
    }).collect::<Vec<JsonFileResult>>();

    println!("Time elapsed: {:?}", start_time.elapsed());
    Ok(json_logger)
} // run_status_cmd