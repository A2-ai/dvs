use std::{fs::{File, OpenOptions}, path::PathBuf};
use crate::helpers::repo;
use std::io::prelude::*;
use crate::helpers::{error::{FileError, FileErrorType}, file};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn add_gitignore_entry_helper(path: &PathBuf) -> Result<()> {
    let abs_path = path.canonicalize()?;

    let dir = abs_path
        .parent()
        .ok_or_else(|| format!("could not get parent of {}: ", abs_path.display()))?
        .to_path_buf();

    // get relative path with leading slash
    let ignore_entry1 = format!("/{}", repo::get_relative_path(&dir, path)?.display());
    let ignore_entry2 = format!("!/{}.dvs", repo::get_relative_path(&dir, path)?.display());

    // open the gitignore file, creating one if it doesn't exist
    let ignore_file = dir.join(".gitignore");
    if !ignore_file.exists() {
       File::create(&ignore_file)?;
    }

    let contents = std::fs::read_to_string(&ignore_file)?;
    
     // add ignore entry if not already present
    if !contents.contains(&ignore_entry1) || !contents.contains(&ignore_entry2) {
        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(ignore_file)?;

        writeln!(file, "\n\n# dvs entry\n{ignore_entry1}\n{ignore_entry2}")?;
    }
    Ok(())
}

pub fn add_gitignore_entry(local_path: &PathBuf) -> std::result::Result<(), FileError> {
    
    add_gitignore_entry_helper(local_path).map_err(|e| {
        let err_mess = match local_path.parent() {
            Some(parent) => format!("could not create entry for {}/.gitignore", parent.display()),
            None => format!("could not create entry for .gitignore"),
        };
        FileError{
            relative_path: file::try_to_get_rel_path(local_path),
            absolute_path: file::try_to_get_abs_path(local_path),
            error: FileErrorType::GitIgnoreNotAdded,
            error_message: Some(format!("{err_mess}: {e}")),
            input: local_path.clone()
        }
    })
}