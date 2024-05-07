use std::{fs::{File, OpenOptions}, path::PathBuf};
use crate::helpers::repo;
use std::io::prelude::*;
use crate::helpers::{error::{FileError, FileErrorType}, file::{get_absolute_path, get_relative_path_to_wd}};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn add_gitignore_dir_level(path: &PathBuf) -> Result<()> {
    let abs_path = path.canonicalize()?;

    let dir = abs_path
        .parent()
        .ok_or_else(|| format!("could not get parent of {}: ", abs_path.display()))?
        .to_path_buf();

    // get relative path with leading slash
    let ignore_entry = format!("/{}", repo::get_relative_path(&dir, path)?.display());

    // open the gitignore file, creating one if it doesn't exist
    let ignore_file = dir.join(".gitignore");
    if !ignore_file.exists() {
       File::create(&ignore_file)?;
    }

    let contents = std::fs::read_to_string(&ignore_file)?;
    
     // add ignore entry if not already present
    if !contents.contains(&ignore_entry) {
        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(ignore_file)?;

        writeln!(file, "\n\n# Devious entry\n{ignore_entry}")?;
    }
    Ok(())
}

fn add_gitignore_proj_level(git_dir: &PathBuf) -> Result<()> {
    let ignore_entry = format!("*\n!.gitignore\n!*/.gitignore\n!*.dvsmeta");

    // open the gitignore file, erroring if it doesn't exist
    let ignore_file = git_dir.join(".gitignore");
    if !ignore_file.exists() {
        return Err("project level .gitignore does not exist".into())
    }

    let contents = std::fs::read_to_string(&ignore_file)?;

    // add ignore entry if not already present
    if !contents.contains(&ignore_entry) {
        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(ignore_file)?;

        writeln!(file, "\n{ignore_entry}")?;
    } 
    Ok(())
}

pub fn add_gitignore_entries(local_path: &PathBuf, git_dir: &PathBuf) -> std::result::Result<(), FileError> {
    add_gitignore_dir_level(local_path).map_err(|e| {
        FileError{
            relative_path: get_relative_path_to_wd(local_path).ok(),
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::DirGitIgnoreNotAdded,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    })?;

    add_gitignore_proj_level(git_dir).map_err(|e| {
        FileError{
            relative_path: get_relative_path_to_wd(local_path).ok(),
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::ProjectGitIgnoreNotAdded,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    }) 
}