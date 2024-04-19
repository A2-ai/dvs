use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;
use glob::glob;

// #[derive(Debug)]
// pub struct ParseError {
//     pub file: String,
//     pub error_message: String
// }

// impl fmt::Display for ParseError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f, "{}", self.error_message)
//     }
// }

// impl std::error::Error for ParseError {}

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

pub fn parse_files_from_globs(globs: &Vec<String>) -> Vec<PathBuf> {
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
        else if PathBuf::from(entry).is_dir() {
            queued_paths.push(PathBuf::from(entry));
        }

        // else, entry is a glob
        else { 
            let glob = match glob(&entry) {
                Ok(paths) => {paths},
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
        } // else, is a glob
    } // for entry in files

    queued_paths
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
