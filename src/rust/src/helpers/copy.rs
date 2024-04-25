use std::{fs::{self, File}, path::PathBuf, os::unix::fs::PermissionsExt};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn copy(src_path: &PathBuf, dest_path: &PathBuf) -> Result<()> {
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

pub fn set_file_permissions(mode: &u32, dest_path: &PathBuf) -> Result<()> {
    let new_permissions = fs::Permissions::from_mode(*mode);
    fs::set_permissions(&dest_path, new_permissions)?;
    Ok(())
}
