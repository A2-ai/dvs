use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::helpers::file;
use file_owner::PathExt;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub user_id: u32,
    pub user_name: String,
    pub group_id: u32,
    pub group_name: String,
    pub creation_time: u64,
    pub modification_time: u64,
    pub permissions: String,
}

pub fn info(paths: &Vec<String>) -> Vec<Result<FileInfo>> {
    paths
        .iter()
        .map(|path| {
            let path_buf = PathBuf::from(path);
            let metadata = fs::metadata(Path::new(path))?;
            let user_id = metadata.uid();
            let user_name = file::get_user_name(&path_buf, &None, &None)?; // TODO
            let group_id = metadata.gid();
            let group_name = path_buf.group()?.name()?.unwrap_or_default();
            let modification_time = metadata
                .modified()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            let permissions = format!("{:o}", metadata.permissions().mode());
            let creation_time = metadata
                .created()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();

            Ok(FileInfo {
                path: path.clone(),
                user_id,
                user_name,
                group_id,
                group_name,
                creation_time,
                modification_time,
                permissions,
            })
        })
        .collect()
}
