use std::{fs::{self, File}, path::PathBuf};
use file_owner::PathExt;
use serde::{Deserialize, Serialize};
use crate::helpers::{repo, error::{FileError, FileErrorType}};


pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub blake3_checksum: String,
    pub file_size_bytes: u64,
    pub time_stamp: String,
    pub message: String,
    pub saved_by: String
}

pub fn save(metadata: &Metadata, local_path: &PathBuf) -> std::result::Result<(), FileError> {
    // compose path file/to/file.ext.dvsmeta
    let metadata_file_path = metadata_path(local_path);
    // create file
    File::create(&metadata_file_path).map_err(|e| {
        FileError{
            relative_path: get_relative_path_to_wd(local_path).ok(),
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    })?;
    let contents = serde_json::to_string_pretty(&metadata).map_err(|e| {
        FileError{
            relative_path: get_relative_path_to_wd(local_path).ok(),
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    })?;
    fs::write(&metadata_file_path, contents).map_err(|e| {
        FileError{
            relative_path: get_relative_path_to_wd(local_path).ok(),
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::MetadataNotSaved,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    })?;
    Ok(())
}

pub fn load_helper(path: &PathBuf) -> Result<Metadata> {
    let metadata_path_abs = metadata_path(path).canonicalize()?;
    let contents = fs::read_to_string(metadata_path_abs)?;
    let metadata: Metadata = serde_json::from_str(&contents)?;
    return Ok(metadata);
}

pub fn load(local_path: &PathBuf) -> std::result::Result<Metadata, FileError> {
    Ok(load_helper(local_path).map_err(|e| {
            FileError{
                relative_path: get_relative_path_to_wd(local_path).ok(),
                absolute_path: get_absolute_path(local_path).ok(),
                error: FileErrorType::MetadataNotFound,
                error_message: Some(e.to_string()),
                input: local_path.clone()
            }
        })?)
}

pub fn metadata_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(path.display().to_string() + ".dvsmeta")
}

pub fn path_without_metadata(path: &PathBuf) -> PathBuf {
    PathBuf::from(path.display().to_string().replace(".dvsmeta", ""))
}

pub fn get_user_helper(path: &PathBuf) -> Result<String> {
    Ok(path
        .owner()?
        .name()?
        .ok_or_else(|| format!("owner not found"))?
    )
}

pub fn get_user_name(local_path: &PathBuf) -> std::result::Result<String, FileError> {
    Ok(get_user_helper(local_path).map_err(|e| {
        FileError{
            relative_path: get_relative_path_to_wd(local_path).ok(),
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::OwnerNotFound,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    })?)
}


pub fn get_absolute_path(local_path: &PathBuf) -> std::result::Result<PathBuf, FileError> {
    Ok(local_path.canonicalize().map_err(|e|
            FileError{ // this should never error because if any paths aren't canonicalizable in the batch add fn, the fn returns
                relative_path: None,
                absolute_path: None,
                error: FileErrorType::AbsolutePathNotFound,
                error_message: Some(e.to_string()),
                input: local_path.clone()
            }
        )?)
}

pub fn get_relative_path_to_wd(local_path: &PathBuf) -> std::result::Result<PathBuf, FileError> {
    Ok(repo::get_relative_path(&PathBuf::from("."), &local_path).map_err(|e|
        FileError{
            relative_path: None,
            absolute_path: get_absolute_path(local_path).ok(),
            error: FileErrorType::RelativePathNotFound,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    )?)
}



pub fn check_if_dir(local_path: &PathBuf) -> std::result::Result<(), FileError> {
    if local_path.is_dir() {
        Err(FileError{
                relative_path: repo::get_relative_path_to_wd(local_path).ok(),
                absolute_path: get_absolute_path(local_path).ok(),
                error: FileErrorType::PathIsDirectory,
                error_message: None,
                input: local_path.clone()
            }
        )
    }
    else {
        Ok(())
    }
}

pub fn get_file_size(local_path: &PathBuf) -> std::result::Result<u64, FileError> {
    Ok(local_path.metadata().map_err(|e|
            FileError{
                relative_path: repo::get_relative_path_to_wd(local_path).ok(), 
                absolute_path: get_absolute_path(local_path).ok(),
                error: FileErrorType::SizeNotFound,
                error_message: Some(e.to_string()),
                input: local_path.clone()
            }
        )?.len())
}

// pub fn check_meta_files_exist(queued_paths: &Vec<PathBuf>) -> std::result::Result<(), BatchError> {
//     // Find the first path that does not have a corresponding .dvsmeta file
//     if let Some(path) = queued_paths
//         .into_iter()
//         .find(|dvs_path| !metadata_path(dvs_path).exists())
//     {
//         return Err(BatchError {
//             error: BatchErrorType::AnyMetaFilesDNE,
//             error_message: format!("missing for {}", path.display()),
//         });
//     }

//     Ok(()) // If all .dvsmeta files found, return Ok
// }



