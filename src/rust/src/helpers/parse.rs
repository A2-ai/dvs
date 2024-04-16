use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;
use glob::glob;
use extendr_api::prelude::*;

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

        // if entry is a file
        if PathBuf::from(entry).extension().and_then(OsStr::to_str).is_some() { 
            match filter_path(&PathBuf::from(entry), &queued_paths) {
                Some(clean_path) => queued_paths.push(clean_path),
                None => continue
            }
        }

        // else, entry is a glob
        else { 
            let glob = match glob(&entry) {
                Ok(paths) => {paths},
                Err(e) => return Err(extendr_api::error::Error::Other(e.to_string())),
            };
            
            for file in glob {
                match file {
                    Ok(path) => {
                        match filter_path(&path, &queued_paths) {
                            Some(clean_path) => queued_paths.push(clean_path),
                            None => continue
                        }
                    }
                    Err(e) => {
                        return Err(extendr_api::error::Error::Other(e.to_string()));
                    }
                } // match file in glob
            } // for file in glob
        } // else, is a glob
    } // for entry in files

    Ok(queued_paths)
}

fn filter_path(path: &PathBuf, queued_paths: &Vec<PathBuf>) -> Option<PathBuf> {
    let path_clean = PathBuf::from(path.display().to_string().replace(".dvsmeta", ""));

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
