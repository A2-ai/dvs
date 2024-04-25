use std::{fs::{self, File}, path::PathBuf, time::SystemTime};
use serde::{Deserialize, Serialize};
use crate::helpers::repo;
use xdg;


pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize)]
pub struct CacheData {
    hash: String,
    modification_time: Option<SystemTime>,
}

pub fn get_cached_hash(path: &PathBuf) -> Result<String> {
    // get absolute path
    let abs_path = path.canonicalize()?;
    
    // open the cache
    let cache_path = get_cache_path(&abs_path)?;

    let contents = fs::read_to_string(&cache_path)?;

    let cache_data: CacheData = serde_json::from_str(&contents)?;

    let current_modification_time = Some(abs_path.metadata()?.modified()?);

    if current_modification_time != cache_data.modification_time {
        fs::remove_file(cache_path)?;
        // "file modification time does not match cache (invalidating)"
        return Err("file modification time does not match cache (invalidating)".into());
    }

    return Ok(cache_data.hash);
}

pub fn write_hash_to_cache(path: &PathBuf, hash: &String) -> Result<()> {
    let abs_path = path.canonicalize()?;

    // cache_path = $HOME/.cache/dvs/<project_name>/<relative path between file and git directory>
    let cache_path = get_cache_path(&abs_path)?;

    // create directories in path
    let parent_path = cache_path.parent().ok_or_else(||
        format!("Provided path has no parent: {}", cache_path.display())
    )?;
    fs::create_dir_all(&parent_path)?;

    // create file
    File::create(&cache_path)?;

    let modification_time = Some(abs_path.metadata()?.modified()?);

    // put modification_time and hash into struct
    let cached_data = CacheData{
        modification_time,
        hash: hash.clone()
    };

    // serialize file contents
    let contents = serde_json::to_string_pretty(&cached_data)?;

    // write contents to file
    fs::write(&cache_path, contents)?;

    Ok(())
}

fn get_cache_path(abs_path: &PathBuf) -> Result<PathBuf> {
    let git_dir = repo::get_nearest_repo_dir(&abs_path)?;
    let rel_path = repo::get_relative_path(&git_dir, &abs_path)?;
    
    let project_name = PathBuf::from(git_dir
        .file_name()
        .ok_or_else(|| format!("project name not found"))?
        .to_str()
        .ok_or_else(|| format!("project name not found"))?
    );

    // partial_cache_path = project_name/<relative path between file and git directory>
    let partial_cache_path = project_name.join(&rel_path);

    let xdg_dirs = xdg::BaseDirectories::with_prefix("dvs")?;

    // cache_path = $HOME/.cache/dvs/<project_name>/<relative path between file and git directory>
    let cache_path = xdg_dirs.place_cache_file(&partial_cache_path)?;
    Ok(cache_path)
}