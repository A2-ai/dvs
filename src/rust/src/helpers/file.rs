use std::path::PathBuf;
use std::fs::File;
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use std::fs;
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
        Err(e) => return Err(anyhow!(format!("path {} not found: {e}", path.display())))
    };
    let contents = match fs::read_to_string(metafile_path_abs) {
        Ok(contents) => contents,
        Err(e) => return Err(anyhow!(format!("could not display contents of {}: {e}", path.display())))
    };
    let metadata: Metadata = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => return Err(anyhow!(format!("could not get metadata for {}: {e}", path.display())))
    };

    return Ok(metadata);
}