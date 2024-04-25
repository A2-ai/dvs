use std::path::{PathBuf, Path};
use std::fs;
use path_absolutize::Absolutize;

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
    let abs_file_path = absolutize_result(&file_path)?;

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
                if is_git_repo(&directory.to_path_buf()) {
                    return Ok(directory.to_path_buf())
                }
                else {
                    directory
                    .parent()
                    .ok_or_else(|| format!("no nearby git repo"))?
                    .to_path_buf()
                }
            }
            None => directory,
        };
    }
    return Err(format!("no nearby git repo").into());
}

pub fn is_in_git_repo(path: &PathBuf, git_dir: &PathBuf) -> bool {
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