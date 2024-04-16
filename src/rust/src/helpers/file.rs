use std::{fs::{self, File}, path::PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub file_hash: String,
    pub file_size: u64,
    pub time_stamp: String,
    pub message: String,
    pub saved_by: String
}

pub fn save(metadata: &Metadata, path: &PathBuf) -> Result<()> {
    // compose path file/to/file.ext.dvsmeta
    let metadata_file_path = PathBuf::from(path.display().to_string() + ".dvsmeta");

    // create file
    let _ = File::create(&metadata_file_path);
    // write to json
    let contents = serde_json::to_string_pretty(&metadata).unwrap();
    let _ = fs::write(&metadata_file_path, contents);
    Ok(())
}

pub fn load(path: &PathBuf) -> Result<Metadata> {
    let metadata_path = PathBuf::from(path.display().to_string() + ".dvsmeta");
    let metafile_path_abs = match metadata_path.canonicalize() {
        Ok(path) => path,
        Err(e) => return Err(anyhow!(format!("{} not found: be sure to add this file to devious \n{e} ", metadata_path.display())))
    };
    let contents = match fs::read_to_string(metafile_path_abs) {
        Ok(contents) => contents,
        Err(e) => return Err(anyhow!(format!("could not display contents of {}: \n{e}", path.display())))
    };
    let metadata: Metadata = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => return Err(anyhow!(format!("could not get metadata for {}: \n{e}", path.display())))
    };

    return Ok(metadata);
}

pub fn delete(path: &PathBuf) -> Result<()> {
    let metadata_path = PathBuf::from(path.display().to_string() + ".dvsmeta");
    let metafile_path_abs = match metadata_path.canonicalize() {
        Ok(path) => path,
        Err(e) => return Err(anyhow!(format!("{} not found\n{e} ", metadata_path.display())))
    };
    match fs::remove_file(&metafile_path_abs) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!(format!("could not remove metadata file: {}\n{e} ", metadata_path.display())))
    };
    Ok(())
}