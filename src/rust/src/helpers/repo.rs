use std::path::{PathBuf, Path};
use std::fs;
use path_absolutize::*;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn get_relative_path(root_dir: &PathBuf, file_path: &PathBuf) -> Result<PathBuf> {
    let abs_file_path = PathBuf::from(file_path
        .absolutize()?
        .to_str()
        .ok_or_else(||format!("could not get absolute path for {}", file_path.display()))?
        .to_string()
    );

    let abs_root_dir = root_dir.canonicalize()?;

    Ok(abs_file_path.strip_prefix(abs_root_dir)?.to_path_buf())
}

fn is_git_repo(dir: &PathBuf) -> bool {
    dir.join(".git").is_dir()
}

pub fn is_directory_empty(directory: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(directory)?;
    Ok(entries.next().is_none())
}

pub fn get_nearest_repo_dir(dir: &PathBuf) -> Result<PathBuf> {
    let mut directory = dir.canonicalize()?;

    if is_git_repo(&dir) {return Ok(directory)}

    while directory != PathBuf::from("/") {
        directory = match directory.parent() {
            Some(_) => {
                if is_git_repo(&directory.to_path_buf()) {return Ok(directory.to_path_buf())}
                else {directory.parent().unwrap().to_path_buf()}
            }
            None => directory,
        };
    }
    return Err(format!("no nearby git repo").into());
}

pub fn is_in_git_repo(path: &PathBuf, git_dir: &PathBuf) -> bool {
    match path.canonicalize() {
        Ok(path) => {
            return path.strip_prefix(&git_dir).unwrap() != path
        }
        Err(_) => return false,
    }
}