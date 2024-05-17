use std::{fs::{self, File}, path::PathBuf, os::unix::fs::PermissionsExt};
use crate::helpers::{error::{FileError, FileErrorType}, file::{get_absolute_path, get_relative_path_to_wd}};
use file_owner::{Group, PathExt};

use super::file;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn copy_impl(src_path: &PathBuf, dest_path: &PathBuf) -> Result<()> {
    
    // Ignore .. and . paths
    if *src_path == PathBuf::from(r"..") || *src_path == PathBuf::from(r".") {
        return Err(format!("copy failed: \"..\" and \".\" paths not parsable").into());
    }

    // Open source file
    let src_file = File::open(src_path).map_err(|e|
        format!("could not open source file: {}\n{e}", src_path.display())
    )?;

    // Get file size
    let _ = src_file.metadata().map_err(|e|
        format!("could not get file size: {}\n{e}", src_path.display())
    )?.len();
   

    // ensure destination exists
    fs::create_dir_all(dest_path
        .parent()
        .ok_or_else(|| format!("could not access parent of copy destination: {}", dest_path.display()))?)
        .map_err(|e| format!("could not access copy destination: {} \n{e}", dest_path.display())
    )?;

    // create destination file
    File::create(dest_path).map_err(|e|
        format!("could not create file at {} \n{e}", dest_path.display())
    )?;

    // Copy the file
    fs::copy(src_path, dest_path).map_err(|e|
        format!("could not copy file {} to {}: {e}", src_path.display(), dest_path.display())
    )?;

    Ok(())
}

pub fn copy(local_path: &PathBuf, storage_path: &PathBuf) -> std::result::Result<(), FileError> {
    Ok(copy_impl(local_path, storage_path).map_err(|e|
            FileError{
                relative_path: file::try_to_get_rel_path(local_path),
                absolute_path: file::try_to_get_abs_path(local_path),
                error: FileErrorType::FileNotCopied,
                error_message: Some(e.to_string()),
                input: local_path.clone()
            }
        )?)
}

pub fn set_file_permissions(mode: &u32, local_path: &PathBuf) -> std::result::Result<(), FileError> {
    let new_permissions = fs::Permissions::from_mode(*mode);
    fs::set_permissions(&local_path, new_permissions).map_err(|e| {
        FileError {
            relative_path: get_absolute_path(local_path).ok(),
            absolute_path: get_relative_path_to_wd(local_path).ok(),
            error: FileErrorType::PermissionsNotSet,
            error_message: Some(format!("{mode} {e}")),
            input: local_path.clone()
        }
    })?;
    Ok(())
}

pub fn set_group(group: &Option<Group>, local_path: &PathBuf) -> std::result::Result<(), FileError> {
    if group.is_some() { 
        let group_name = group.unwrap(); // group.is_some() so can safely unwrap
        local_path.set_group(group_name).map_err(|e|
            FileError{
                relative_path: file::try_to_get_rel_path(local_path),
                absolute_path: file::try_to_get_abs_path(local_path),
                error: FileErrorType::GroupNotSet,
                error_message: Some(format!("{group_name} {e}")),
                input: local_path.clone()
            }
        )?;
    }
    Ok(())
}

pub fn copy_file_to_storage_directory(local_path: &PathBuf, storage_path: &PathBuf, permissions: &u32, group: &Option<Group>) -> std::result::Result<(), FileError> {
    // copy
    copy(local_path, storage_path)?;

    // set file permissions
    set_file_permissions(permissions, storage_path)?;

    // set group (if specified)
    Ok(set_group(group, storage_path)?)
}
