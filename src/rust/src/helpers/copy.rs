use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::fs::{create_dir_all, File};
use std::fs;
use anyhow::{anyhow, Context, Result};
use file_owner::{Group, PathExt};

pub fn copy(src_path: &PathBuf, dest_path: &PathBuf) -> Result<()> {
    // Ignore .. and . paths
    if *src_path == PathBuf::from(r"..") || *src_path == PathBuf::from(r".") {
        return Err(anyhow!("copy failed: \"..\" and \".\" paths not parsable"));
    }

    // Open source file
    let src_file = match File::open(src_path) {
        Ok(file) => {
            // json
            file
        }
        Err(e) => {
            // json
            return Err(anyhow!("could not open source file: {}\n{e}", src_path.display()));
        }
    };

    // Get file size
    let src_file_data = match src_file.metadata() {
        Ok(data) => data,
        Err(e) => return Err(anyhow!("could not get file size: {}\n{e}", src_path.display()))
    };
    let _src_file_size = src_file_data.len();

    // ensure destination exists
    match create_dir_all(dest_path.parent().unwrap()) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!("could not access copy destination: {} \n{e}", dest_path.display())),
    }

    // create destination file
    match File::create(dest_path) {
        Ok(file) => file,
        Err(e) => return Err(anyhow!("could not create copy of file at {} \n{e}", dest_path.display())),
    };

    // Copy the file
    fs::copy(src_path, dest_path).with_context(|| format!("could not copy file {} to {}", src_path.display(), dest_path.display()))?;

    Ok(())
}

pub fn set_file_permissions(mode: &u32, dest_path: &PathBuf) -> Result<()> {
    dest_path.metadata().unwrap().permissions().set_mode(*mode);
    let _file_mode = dest_path.metadata().unwrap().permissions().mode();
    let new_permissions = fs::Permissions::from_mode(*mode);
    fs::set_permissions(&dest_path, new_permissions).with_context(|| format!("unable to set permissions: {}", mode)).unwrap();
    Ok(())
}


pub fn set_file_group(group_name: &String, dest_path: &PathBuf) -> Result<()> {
    if group_name == "" {return Ok(())}

    // check if group exists
    let group = match Group::from_name(group_name) {
        Ok(group) => {group}
        Err(e) => {
            return Err(anyhow!(e))
        }
    };

    // set group
    match dest_path.set_group(group.clone()) {
        Ok(_) => {},
        Err(e) => {
            // set error
            return Err(anyhow!(e));
            // delete copied file
            // fs::remove_file(&dest_path)
            // .expect("could not set group.");
        }
    };

    Ok(())
}