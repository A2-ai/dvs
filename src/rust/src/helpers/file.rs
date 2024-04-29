use std::{fs::{self, File}, path::PathBuf};
use file_owner::PathExt;
use serde::{Deserialize, Serialize};
use crate::helpers::error::{FileError, FileErrorType};

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

pub fn save(metadata: &Metadata, path: &PathBuf, relative_path: &Option<PathBuf>, absolute_path: &Option<PathBuf>) -> std::result::Result<(), FileError> {
    // compose path file/to/file.ext.dvsmeta
    let metadata_file_path = PathBuf::from(path.display().to_string() + ".dvsmeta");

    // create file
    File::create(&metadata_file_path).map_err(|e| {
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string())
        }
    })?;
    let contents = serde_json::to_string_pretty(&metadata).map_err(|e| {
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string())
        }
    })?;
    fs::write(&metadata_file_path, contents).map_err(|e| {
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string())
        }
    })?;
    Ok(())
}

pub fn load_helper(path: &PathBuf) -> Result<Metadata> {
    let metadata_path_abs = PathBuf::from(path.display().to_string() + ".dvsmeta").canonicalize()?;
    let contents = fs::read_to_string(metadata_path_abs)?;
    let metadata: Metadata = serde_json::from_str(&contents)?;
    return Ok(metadata);
}

pub fn load(path: &PathBuf, relative_path: &Option<PathBuf>, absolute_path: &Option<PathBuf>) -> std::result::Result<Metadata, FileError> {
    Ok(load_helper(path).map_err(|e| {
            FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::MetadataNotFound,
                error_message: Some(e.to_string())
            }
        })?)
    
}

pub fn delete(path: &PathBuf) -> Result<()> {
    let metadata_path_abs = PathBuf::from(path.display().to_string() + ".dvsmeta").canonicalize()?;
    fs::remove_file(&metadata_path_abs)?;
    Ok(())
}

pub fn get_user_helper(path: &PathBuf) -> Result<String> {
    Ok(path
        .owner()?
        .name()?
        .ok_or_else(|| format!("owner not found"))?
    )
}

pub fn get_user_name(local_path: &PathBuf, relative_path: &Option<PathBuf>, absolute_path: &Option<PathBuf>) -> std::result::Result<String, FileError> {
    Ok(get_user_helper(local_path).map_err(|e| {
        FileError{
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            error_type: FileErrorType::OwnerNotFound,
            error_message: Some(e.to_string())
        }
    })?)
}



pub fn get_absolute_path(local_path: &PathBuf) -> std::result::Result<PathBuf, FileError> {
    Ok(local_path.canonicalize().map_err(|e|
            FileError{ // this should never error because if any paths aren't canonicalizable in the batch add fn, the fn returns
                relative_path: None,
                absolute_path: None,
                error_type: FileErrorType::AbsolutePathNotFound,
                error_message: Some(e.to_string())
            }
        )?)
}

pub fn check_if_dir(local_path: &PathBuf, relative_path: &Option<PathBuf>, absolute_path: &Option<PathBuf>) -> std::result::Result<(), FileError> {
    if local_path.is_dir() {
        Err(FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::PathIsDirectory,
                error_message: None
            }
        )
    }
    else {
        Ok(())
    }
}

pub fn get_file_size(local_path: &PathBuf, relative_path: &Option<PathBuf>, absolute_path: &Option<PathBuf>) -> std::result::Result<u64, FileError> {
    Ok(local_path.metadata().map_err(|e|
            FileError{
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                error_type: FileErrorType::SizeNotFound,
                error_message: Some(e.to_string())
            }
        )?.len())
}



