use std::path::{PathBuf, Path};
use std::fs;
use path_absolutize::Absolutize;
use crate::helpers::{file, error::{BatchError, BatchErrorType, FileError, FileErrorType}};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn absolutize_result(path: &PathBuf) -> Result<PathBuf> {
    Ok(PathBuf::from(path
        .absolutize()?
        .to_str()
        .ok_or_else(||format!("could not get absolute path for {}", path.display()))?
        .to_string()
    ))
}

pub fn get_relative_path(root_dir: &PathBuf, file_path: &PathBuf) -> Result<PathBuf> {
    let abs_root_dir = root_dir.canonicalize()?;
    let abs_file_path = absolutize_result(&file_path)?;

    pathdiff::diff_paths(abs_file_path, abs_root_dir).ok_or("relative path not found".into())
}

pub fn get_relative_path_to_wd(local_path: &PathBuf) -> std::result::Result<PathBuf, FileError> {
    Ok(get_relative_path(&PathBuf::from("."), &local_path).map_err(|e|
        FileError{
            relative_path: None,
            absolute_path: file::try_to_get_abs_path(local_path),
            error: FileErrorType::RelativePathNotFound,
            error_message: Some(e.to_string()),
            input: local_path.clone()
        }
    )?)
}

fn is_git_repo(dir: &PathBuf) -> bool {
    dir.join(".git").is_dir()
}

pub fn is_directory_empty(directory: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(directory)?;
    Ok(entries.next().is_none())
}

fn nearest_repo_error(dir: &PathBuf) -> BatchError {
    BatchError{ 
        error: BatchErrorType::GitRepoNotFound,
        error_message: format!("could not find git repo root; make sure you're in an active git repository: {}", dir.display())
    }
}

pub fn get_nearest_repo_dir(dir: &PathBuf) -> std::result::Result<PathBuf, BatchError> {
    let mut directory = dir.canonicalize().map_err(|_e| nearest_repo_error(dir))?;

    if is_git_repo(&dir) {return Ok(directory)}

    while directory != PathBuf::from("/") {
        directory = match directory.parent() {
            Some(_) => {
                if is_git_repo(&directory.to_path_buf()) {
                    return Ok(directory.to_path_buf())
                }
                else {
                    directory
                    .parent()
                    .ok_or_else(|| nearest_repo_error(dir))?
                    .to_path_buf()
                }
            }
            None => directory,
        };
    }
    return Err(nearest_repo_error(dir));
}

// LOOKUP relative_path: get_relative_path_to_wd(local_path).ok(),
fn in_repo_error(local_path: &PathBuf, e: Option<String>) -> FileError {
    FileError {
        relative_path: get_relative_path_to_wd(local_path).ok(),
        absolute_path: file::get_absolute_path(local_path).ok(),
        error: FileErrorType::FileNotInGitRepo,
        error_message: e,
        input: local_path.clone(),
    }
}

pub fn check_file_in_git_repo(local_path: &PathBuf, git_dir: &PathBuf) -> std::result::Result<(), FileError> {
    let canonical_path = local_path.canonicalize().map_err(|e| in_repo_error(local_path, Some(e.to_string())))?;

    let stripped = canonical_path.strip_prefix(git_dir).map_err(|e| in_repo_error(local_path, Some(e.to_string())))?;

    // if the stripped prefix is different from the original, it's inside the repo
    if stripped != canonical_path {
        return Ok(())
    }
    else {
        return Err(in_repo_error(local_path, None));
    }
}

pub fn dir_in_git_repo(path: &PathBuf, git_dir: &PathBuf) -> bool {
    let canonical_path = 
        if let Ok(path) = path.canonicalize() {
            path
        } 
        else {
            return false;
        };

    if let Ok(stripped) = canonical_path.strip_prefix(git_dir) {
        // if the stripped prefix is different from the original, it's inside the repo
        stripped != canonical_path
    } else {
        false
    }
}