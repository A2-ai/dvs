use std::{fs::{self, File}, path::PathBuf};
use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub hash: String,
    pub size: u64,
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
    let contents = serde_json::to_string_pretty(&metadata)?;
    let _ = fs::write(&metadata_file_path, contents);
    Ok(())
}

pub fn load(path: &PathBuf) -> Result<Metadata> {
    let metadata_path_abs = PathBuf::from(path.display().to_string() + ".dvsmeta").canonicalize()?;
    let contents = fs::read_to_string(metadata_path_abs)?;
    let metadata: Metadata = serde_json::from_str(&contents)?;
    return Ok(metadata);
}

pub fn delete(path: &PathBuf) -> Result<()> {
    let metadata_path_abs = PathBuf::from(path.display().to_string() + ".dvsmeta").canonicalize()?;
    fs::remove_file(&metadata_path_abs)?;
    Ok(())
}