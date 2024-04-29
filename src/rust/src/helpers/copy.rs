use std::{fs::{self, File}, path::PathBuf, os::unix::fs::PermissionsExt};
use crate::helpers::{error::{FileError, FileErrorType}, file::{get_absolute_path, get_relative_path_to_wd}};
use file_owner::{Group, PathExt};

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
                relative_path: get_absolute_path(local_path).ok(),
                absolute_path: get_relative_path_to_wd(local_path).ok(),
                error_type: FileErrorType::FileNotCopied,
                error_message: Some(e.to_string())
            }
        )?)
}

pub fn set_file_permissions(mode: &u32, dest_path: &PathBuf) -> std::result::Result<(), FileError> {
    let new_permissions = fs::Permissions::from_mode(*mode);
    fs::set_permissions(&dest_path, new_permissions).map_err(|e| {
        FileError {
            relative_path: get_absolute_path(dest_path).ok(),
            absolute_path: get_relative_path_to_wd(dest_path).ok(),
            error_type: FileErrorType::PermissionsNotSet,
            error_message: Some(format!("{mode} {e}")),
        }
    })?;
    Ok(())
}

pub fn set_group(group: &Option<Group>, storage_path: &PathBuf) -> std::result::Result<(), FileError> {
    if group.is_some() { 
        let group_name = group.unwrap(); // group.is_some() so can safely unwrap
        storage_path.set_group(group_name).map_err(|e|
            FileError{
                relative_path: get_absolute_path(storage_path).ok(),
                absolute_path: get_relative_path_to_wd(storage_path).ok(),
                error_type: FileErrorType::GroupNotSet,
                error_message: Some(format!("{group_name} {e}"))
            }
        )?;
    }
    Ok(())
}
