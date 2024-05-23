use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;
use glob::glob;
use crate::helpers::{file, repo, error::{BatchError, BatchErrorType}};


pub fn get_all_meta_files() -> Result<Vec<PathBuf>, BatchError> {
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from("."))?;
    Ok(WalkDir::new(&git_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "dvs"))
        .map(|e| {
            let string = file::path_without_metadata(&e.into_path());
            PathBuf::from(string)
        })
        .collect()
    )
}

pub fn parse_files_from_globs_add(globs: &Vec<String>) -> Vec<PathBuf> {
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
        return None
    }
    
    if queued_paths.contains(&path_clean) {
        return None
    }
        
    Some(path_clean)
}


pub fn parse_files_from_globs_status(globs: &Vec<String>) -> Result<Vec<PathBuf>, BatchError> {
    if globs.contains(&String::from("")) && globs.len() == 1 {
        return get_all_meta_files();
    }
    let mut queued_paths: Vec<PathBuf> = Vec::new();

    for entry in globs {
        // if entry is a file that explicitly exists
        if is_explicit_path(entry) {
            let clean_path = file::path_without_metadata(&PathBuf::from(entry));
            queued_paths.push(PathBuf::from(clean_path));
            continue;
        }

        // else, entry is: a file that doesn't exist, a file glob, or a dir
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
                    match filter_meta_path(&path, &queued_paths) {
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

        // if no files parsed from glob, then entry is a file that dne or a dir 
        if !entered_loop {
            queued_paths.push(PathBuf::from(entry)); // add to queued_paths anyway
        }
    } // for entry in files

    Ok(queued_paths)
}

fn filter_meta_path(path: &PathBuf, queued_paths: &Vec<PathBuf>) -> Option<PathBuf> {
    let path_clean = file::path_without_metadata(path);

    // if metadata path doesn't exist
    if !file::metadata_path(&path_clean).exists() {
        return None
    }
    
    if queued_paths.contains(&path_clean) {
        return None
    }
        
    Some(path_clean)
}

pub fn parse_files_from_globs_get(globs: &Vec<String>) -> Result<Vec<PathBuf>, BatchError> {
    // first check explicit paths for metadata files
    check_metafiles_for_explicit_paths(globs)?;

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
                    match filter_meta_path(&path, &queued_paths) {
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

    Ok(queued_paths)
}

// returns true if input is an explicit path to a file or directory (that may or may not exist) whose metafile should be checked
// in other words: is_not_a_valid_file_glob
pub fn is_explicit_path(entry: &String) -> bool {
    let path = PathBuf::from(entry);
    // if it's an explicitly inputted file or directory that exists, return true
    if path.is_file() || path.is_dir() {
        return true
    }
    // else, is a 
    // - file that dne, 
    // - dir that dne, or 
    // - file glob

    if let Ok(mut matches) = glob(entry) {
        // if glob pattern catches anything, it must be a file glob
        // it couldn't be that the glob pattern was actually an explicit file path because that only catches it if the file exists
        // the file wouldn't exist because that was checked earlier
        // e.g. suppose entry is "test.txt" and the file exists - this explicit path is considered a valid file glob, 
        // and matches.next() would be "test.txt"
        // e.g. suppose entry is "test.txt" and the file does NOT exist - this explicit path is considered a valid file glob, 
        // but matches.next() would be None
        if matches.next().is_some() { // if matches at least one thing
            return false; // is a file glob
        }
        else { // if it catches nothing, it could either be a file that dne, a dir that dne, or a valid file glob that caught nothing
            if entry.contains("*") || entry.contains("?") || entry.contains("[") || entry.contains("]") || entry.contains("{") || entry.contains("}") {
                // if it's a valid glob and contains a glob symbol, just suppose it was a file glob that caught nothing. 
                // (as opposed to a file/dir path which are also considered valid file globs)
                // example: *.txt could be a file glob or explicit path to a file named "*.txt"
                // glob symbols in file names is rare enough, so a suppose it was a file glob
                return false
            }
            else { // if it doesn't contain these symbols, it was probably a file or dir path that dne
                return true;
            }
        }
    }
    else { 
        // else, it's not a valid file glob or file/dir that exists, 
        // so return true for explicit entry because it should be checked
        return true
    }
}

pub fn check_metafiles_for_explicit_paths(files: &Vec<String>) -> Result<(), BatchError> {
    files
        .iter()
        // filter explict paths i.e. paths that are not valid file globs
        .filter(|file| is_explicit_path(file))
        // search for an explicit file without a metadata file
        .find(|file| !file::metadata_path(&PathBuf::from(file)).exists())
        // if one is found, return error
        .map_or(Ok(()), |missing_file| {
            Err(BatchError {
                error: BatchErrorType::AnyMetaFilesDNE,
                error_message: format!("missing for {}", missing_file),
            })
        })
}