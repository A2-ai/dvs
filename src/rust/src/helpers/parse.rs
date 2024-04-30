use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;
use glob::glob;

use super::file;

pub fn get_all_meta_files(dir: &PathBuf) -> Vec<PathBuf> {
    //let mut meta_files: Vec<String> = Vec::new();
    WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "dvsmeta"))
        .map(|e| {
            let string = file::path_without_metadata(&e.into_path());
            PathBuf::from(string)
        })
        .collect()
}

pub fn parse_files_from_globs(globs: &Vec<String>) -> Vec<PathBuf> {
    let mut queued_paths: Vec<PathBuf> = Vec::new();

    for entry in globs {
        let glob = match glob(&entry) {
            Ok(paths) => paths,
            Err(_) => {
                queued_paths.push(PathBuf::from(entry));
                continue;
            }
        };
        
        let mut entered_loop: bool = false;
        for file in glob {
            entered_loop = true;
            match file {
                Ok(path) => {
                    match filter_path(&path, &queued_paths) {
                        Some(clean_path) => queued_paths.push(clean_path),
                        None => continue
                    }
                }
                Err(_) => {
                    queued_paths.push(PathBuf::from(entry));
                    continue;
                }
            } // match file in glob
        } // for file in glob

        // if no files parsed from glob, add to queued_paths anyway
        if !entered_loop {
            queued_paths.push(PathBuf::from(entry));
        }
    } // for entry in files

    queued_paths
}

fn filter_path(path: &PathBuf, queued_paths: &Vec<PathBuf>) -> Option<PathBuf> {
    let path_clean = file::path_without_metadata(path);

    if path_clean.file_name().and_then(OsStr::to_str) == Some(".gitignore") {
        println!("skipping .gitignore file {}", path.display());
        return None
    }
    
    if queued_paths.contains(&path_clean) {
        //println!("skipping repeated path: {}", path_clean.display());
        return None
    }
        
    Some(path_clean)
}
