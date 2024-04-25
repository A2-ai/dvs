use std::{fs::{File, OpenOptions}, path::PathBuf};
use crate::helpers::repo;
use std::io::prelude::*;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn add_gitignore_entry(path: &PathBuf) -> Result<()> {
    let abs_path = path.canonicalize()?;

    let dir = abs_path
        .parent()
        .ok_or_else(|| format!("could not get parent of {}: ", abs_path.display()))?
        .to_path_buf();

    // get relative path
    let ignore_entry_temp = repo::get_relative_path(&dir, path)?;
    
    // Add leading slash
    let ignore_entry = ignore_entry_temp.display().to_string();

    // open the gitignore file, creating one if it doesn't exist
    let ignore_file = dir.join(".gitignore");
    if !ignore_file.exists() {
       File::create(&ignore_file)?;
    }

    let contents = std::fs::read_to_string(&ignore_file)?;
    if !contents.contains(&ignore_entry) {
        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(ignore_file)?;

        writeln!(file, "\n\n# Devious entry\n{ignore_entry}")?;

    } // add ignore entry
    Ok(())
}