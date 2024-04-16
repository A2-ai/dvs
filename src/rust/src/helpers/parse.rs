use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;
use glob::glob;
use extendr_api::prelude::*;

pub fn parse_globs(globs: &Vec<String>) -> Vec<PathBuf> {
    let mut meta_files: Vec<PathBuf> = Vec::new();

    for glob in globs {
        // remove meta file extension
        let file_string = glob.replace(".dvsmeta", "");
        let file_path = PathBuf::from(file_string);

        // skip if already queued
        if meta_files.contains(&file_path) {continue}

        if file_path.is_dir() {
            continue
        }

        meta_files.push(file_path);
    }
    
    return meta_files;
}


pub fn get_all_meta_files(dir: &PathBuf) -> Vec<PathBuf> {
    //let mut meta_files: Vec<String> = Vec::new();
    WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "dvsmeta"))
        .map(|e| {
            let string = e.into_path().display().to_string().replace(".dvsmeta", "");
            PathBuf::from(string)
        })
        .collect()
}

pub fn parse_files_from_globs(globs: &Vec<String>) -> Result<Vec<PathBuf>> {
    let mut queued_paths: Vec<PathBuf> = Vec::new();

    for entry in globs {
        // need to have this if/else structure because glob parsing crate only works if a given file
        // exists, and often dvs_get is called with files that don't actually exist in the dir yet
        // tl;dr don't delete the if statement ->
        if PathBuf::from(entry).extension().and_then(OsStr::to_str).is_some() { // if individual file
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
        else { // else is a glob
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
        } // else, is a glob
    } // for entry in files

    Ok(queued_paths)
}
